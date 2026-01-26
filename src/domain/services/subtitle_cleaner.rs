//! Subtitle cleaner domain service.
//!
//! Normalizes subtitle text from common formats (SRT, VTT) into plain transcript
//! suitable for downstream LLM use. This service is deterministic and free of
//! external dependencies.

use std::borrow::Cow;

/// Cleans subtitle text into a compact, readable transcript.
///
/// Supported formats:
/// - SRT
/// - WebVTT
///
/// The cleaner:
/// - Strips BOM and format headers (WEBVTT, KIND, etc.)
/// - Removes timestamp lines and cue indices
/// - Drops inline formatting tags (e.g., <i>, <c>, <u>)
/// - Removes speaker labels (e.g., [John]:, SPEAKER:, >>)
/// - Collapses whitespace
/// - Removes duplicate consecutive lines
#[derive(Debug, Default, Clone)]
pub struct SubtitleCleaner;

impl SubtitleCleaner {
    /// Creates a new `SubtitleCleaner`.
    pub fn new() -> Self {
        Self
    }

    /// Cleans the provided subtitle content and returns a normalized transcript.
    pub fn clean(&self, raw: &str) -> String {
        let normalized = strip_bom(raw);
        let mut out: Vec<String> = Vec::new();
        let mut prev_line: Option<String> = None;

        for line in normalized.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Skip format metadata
            if is_vtt_header(line) || is_cue_index(line) || is_timestamp_line(line) {
                continue;
            }

            // Strip inline tags like <i>...</i>
            let cleaned = strip_inline_tags(line);

            // Strip speaker indicators like "[Speaker]:" or ">>"
            let cleaned = strip_speaker_labels(&cleaned);

            // Normalize whitespace (internal and surrounding)
            let cleaned = normalize_whitespace(&cleaned);

            if cleaned.is_empty() {
                continue;
            }

            // Deduplicate consecutive identical lines
            if let Some(prev) = prev_line.as_ref() {
                if prev == &cleaned {
                    continue;
                }
            }

            prev_line = Some(cleaned.clone());
            out.push(cleaned);
        }

        out.join(" ")
    }
}

fn strip_bom(input: &str) -> &str {
    input.trim_start_matches('\u{feff}')
}

fn is_vtt_header(line: &str) -> bool {
    let upper = line.to_ascii_uppercase();
    upper.starts_with("WEBVTT")
        || upper.starts_with("KIND:")
        || upper.starts_with("LANGUAGE:")
        || upper.starts_with("STYLE")
        || upper.starts_with("NOTE")
}

fn is_cue_index(line: &str) -> bool {
    // Cue indices in SRT are just numbers on their own line
    line.chars().all(|ch| ch.is_ascii_digit())
}

fn is_timestamp_line(line: &str) -> bool {
    // Matches patterns containing the separator "-->"
    if !line.contains("-->") {
        return false;
    }

    let mut parts = line.split("-->");
    let start = parts.next().map(str::trim);
    let end = parts.next().map(str::trim);

    match (start, end) {
        (Some(s), Some(e)) => is_timecode(s) && is_timecode(e),
        _ => false,
    }
}

fn is_timecode(value: &str) -> bool {
    // Basic timecode check: should contain ':' and digits
    // Accepts hh:mm:ss.mmm, mm:ss.mmm, or variants with commas
    let v = value.split_whitespace().next().unwrap_or(value).trim();
    if v.is_empty() {
        return false;
    }

    let parts: Vec<&str> = v.split(':').collect();
    if parts.len() < 2 {
        return false;
    }

    // Every part should contain at least one digit
    parts.iter().all(|p| p.chars().any(|c| c.is_ascii_digit()))
}

fn strip_inline_tags(line: &str) -> Cow<'_, str> {
    if !line.contains('<') {
        return Cow::Borrowed(line);
    }

    let mut out = String::with_capacity(line.len());
    let mut in_tag = false;
    for ch in line.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {},
        }
    }

    Cow::Owned(out)
}

/// Removes speaker labels like "[John]:", "SPEAKER:", or ">>"
fn strip_speaker_labels(line: &str) -> Cow<'_, str> {
    let trimmed = line.trim();

    // Remove ">>" prefix (common in news/multi-speaker transcripts)
    if trimmed.starts_with(">>") {
        let after = trimmed.trim_start_matches('>').trim_start_matches(' ');
        return Cow::Owned(after.to_string());
    }

    // Detect labels like "NAME:" or "[NAME]:"
    if let Some(colon_pos) = trimmed.find(':') {
        let prefix = trimmed[..colon_pos].trim();

        if prefix.is_empty() {
            return Cow::Borrowed(line);
        }

        let is_bracketed = prefix.starts_with('[') && prefix.ends_with(']');
        let is_all_caps = prefix.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());

        // Speakers are typically short, all caps, or bracketed
        // We limit to 25 chars to avoid catching long sentences that happen to have a colon
        if (is_bracketed || is_all_caps) && prefix.len() < 25 {
            let after = &trimmed[colon_pos + 1..].trim();
            return Cow::Owned(after.to_string());
        }
    }

    Cow::Borrowed(line)
}

fn normalize_whitespace(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut prev_space = false;

    for ch in line.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            prev_space = false;
            out.push(ch);
        }
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_srt_standard() {
        let input =
            "1\n00:00:01,000 --> 00:00:02,000\nHello\n\n2\n00:00:02,500 --> 00:00:03,000\nWorld\n";
        let cleaned = SubtitleCleaner::new().clean(input);
        assert_eq!(cleaned, "Hello World");
    }

    #[test]
    fn cleans_vtt_with_headers_and_tags() {
        let input = "WEBVTT\nKIND: captions\n\n00:00:00.000 --> 00:00:01.000\n<i>Hello</i>\n00:00:01.000 --> 00:00:02.000\n<c.yellow>World</c>";
        let cleaned = SubtitleCleaner::new().clean(input);
        assert_eq!(cleaned, "Hello World");
    }

    #[test]
    fn handles_speaker_labels() {
        let cleaner = SubtitleCleaner::new();

        assert_eq!(cleaner.clean(">> This is a new speaker"), "This is a new speaker");
        assert_eq!(cleaner.clean("[JOHN]: Good morning"), "Good morning");
        assert_eq!(cleaner.clean("SPEAKER 1: Test message"), "Test message");
        assert_eq!(cleaner.clean("Normal sentence: with a colon"), "Normal sentence: with a colon");
    }

    #[test]
    fn removes_duplicate_consecutive_lines() {
        let input = "1\n00:01:00,000 --> 00:01:02,000\nRepeat this\n\n2\n00:01:02,000 --> 00:01:04,000\nRepeat this\n";
        let cleaned = SubtitleCleaner::new().clean(input);
        assert_eq!(cleaned, "Repeat this");
    }

    #[test]
    fn strips_bom_correctly() {
        let input = "\u{feff}WEBVTT\n\n00:01.000 --> 00:02.000\nBOM Test";
        let cleaned = SubtitleCleaner::new().clean(input);
        assert_eq!(cleaned, "BOM Test");
    }
}
