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
/// - Strips BOM and format headers
/// - Removes timestamp lines and cue indices
/// - Drops inline formatting tags
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

            if is_vtt_header(line) {
                continue;
            }

            if is_cue_index(line) {
                continue;
            }

            if is_timestamp_line(line) {
                continue;
            }

            let cleaned = strip_inline_tags(line);
            let cleaned = normalize_whitespace(&cleaned);

            if cleaned.is_empty() {
                continue;
            }

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
}

fn is_cue_index(line: &str) -> bool {
    line.chars().all(|ch| ch.is_ascii_digit())
}

fn is_timestamp_line(line: &str) -> bool {
    // Matches patterns like:
    // 00:00:01.000 --> 00:00:03.000
    // 00:00:01,000 --> 00:00:03,000
    // 00:01.000 --> 00:02.000
    let mut parts = line.split("-->");
    let start = parts.next().map(str::trim);
    let end = parts.next().map(str::trim);

    match (start, end) {
        (Some(s), Some(e)) => is_timecode(s) && is_timecode(e),
        _ => false,
    }
}

fn is_timecode(value: &str) -> bool {
    // Accept hh:mm:ss.mmm or mm:ss.mmm or hh:mm:ss,mmm
    let mut v = value.split_whitespace().next().unwrap_or(value);

    // Remove trailing cues like "position:0%" in VTT
    if let Some((time, _rest)) = v.split_once(' ') {
        v = time;
    }

    let v = v.trim();

    // Normalize separators
    let v = v.replace(',', ".");
    let parts: Vec<&str> = v.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    let (sec_part, min_part, hour_part) = match parts.len() {
        2 => (parts[1], parts[0], None),
        3 => (parts[2], parts[1], Some(parts[0])),
        _ => return false,
    };

    if let Some(hour) = hour_part {
        if !is_number(hour) {
            return false;
        }
    }

    if !is_number(min_part) {
        return false;
    }

    let (sec, millis) = match sec_part.split_once('.') {
        Some((s, ms)) => (s, Some(ms)),
        None => (sec_part, None),
    };

    if !is_number(sec) {
        return false;
    }

    if let Some(ms) = millis {
        if !is_number(ms) {
            return false;
        }
    }

    true
}

fn is_number(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_digit())
}

fn strip_inline_tags(line: &str) -> Cow<'_, str> {
    // Remove simple tags like <i>, </i>, <b>, </b>, <u>, </u>, and VTT cues like <c>
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
    fn cleans_srt() {
        let input = "\u{feff}1\n00:00:01,000 --> 00:00:02,000\nHello\n\n2\n00:00:02,500 --> 00:00:03,000\nWorld\n";
        let cleaned = SubtitleCleaner::new().clean(input);
        assert_eq!(cleaned, "Hello World");
    }

    #[test]
    fn cleans_vtt_with_tags() {
        let input = "WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n<i>Hello</i>\n00:00:01.000 --> 00:00:02.000\n<c>World</c>";
        let cleaned = SubtitleCleaner::new().clean(input);
        assert_eq!(cleaned, "Hello World");
    }

    #[test]
    fn removes_duplicate_lines() {
        let input =
            "1\n00:00:01,000 --> 00:00:02,000\nHello\n\n2\n00:00:02,000 --> 00:00:03,000\nHello\n";
        let cleaned = SubtitleCleaner::new().clean(input);
        assert_eq!(cleaned, "Hello");
    }
}
