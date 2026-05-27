//! YouTube transcript fetcher using yt-dlp directly.

use crate::domain::ports::{TranscriptError as PortTranscriptError, TranscriptProvider};

/// Error type for transcript operations.
#[derive(Debug, thiserror::Error)]
pub enum TranscriptError {
    #[error("No captions available for this video")]
    NoCaptions,
    #[error("Failed to fetch transcript: {0}")]
    FetchError(String),
}

/// Fetches transcripts from YouTube videos.
pub struct TranscriptAdapter;

impl TranscriptAdapter {
    pub fn new() -> Result<Self, TranscriptError> {
        Ok(Self)
    }

    /// Fetches the transcript for a YouTube video using yt-dlp.
    pub async fn fetch_transcript(&self, video_id: &str) -> Result<String, TranscriptError> {
        let url = format!("https://www.youtube.com/watch?v={video_id}");
        let output_template = format!("/tmp/cpilot_{video_id}");

        let mut cmd = tokio::process::Command::new("yt-dlp");
        cmd.args([
            "--write-subs",
            "--write-auto-subs",
            "--sub-langs",
            "en,en-*",
            "--skip-download",
            "--sub-format",
            "vtt",
            "-o",
            &output_template,
            &url,
        ]);
        cmd.kill_on_drop(true);

        let status_res =
            tokio::time::timeout(std::time::Duration::from_secs(60), cmd.output()).await;

        let status = match status_res {
            Ok(res) => {
                res.map_err(|e| TranscriptError::FetchError(format!("yt-dlp spawn failed: {e}")))?
            },
            Err(_) => {
                return Err(TranscriptError::FetchError(
                    "yt-dlp transcript fetch timed out after 60 seconds".to_string(),
                ));
            },
        };

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            return Err(TranscriptError::FetchError(format!("yt-dlp exited with error: {stderr}")));
        }

        // Dynamic VTT file scanning in /tmp
        let mut vtt_content: Option<String> = None;
        if let Ok(mut entries) = tokio::fs::read_dir("/tmp").await {
            let prefix = format!("cpilot_{video_id}.");
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(filename) = entry.file_name().to_str()
                    && filename.starts_with(&prefix)
                    && filename.ends_with(".vtt")
                {
                    let path = entry.path();
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        vtt_content = Some(content);
                        // Best-effort cleanup
                        let _ = tokio::fs::remove_file(&path).await;
                        break;
                    }
                }
            }
        }

        let raw = vtt_content.ok_or(TranscriptError::NoCaptions)?;

        let text = parse_vtt(&raw);
        if text.is_empty() { Err(TranscriptError::NoCaptions) } else { Ok(text) }
    }
}

/// Parses a WebVTT string into plain text.
///
/// Strips the `WEBVTT` header, timestamp lines, cue settings, `<c>` / `<timestamp>` inline
/// tags, and deduplicates consecutive identical lines before joining with spaces.
pub fn parse_vtt(vtt: &str) -> String {
    let mut out = String::with_capacity(vtt.len());
    let mut last: Option<String> = None;

    for raw_line in vtt.lines() {
        let line = raw_line.trim();

        // Skip WEBVTT header and NOTE blocks
        if line.starts_with("WEBVTT") || line.starts_with("NOTE") || line.starts_with("STYLE") {
            continue;
        }

        // Skip timestamp lines: "00:00:00.000 --> 00:00:05.000 ..."
        if line.contains("-->") {
            continue;
        }

        // Skip numeric cue identifiers (pure digit strings or UUID-like)
        if line.chars().all(|c| c.is_ascii_digit() || c == '-') && !line.is_empty() {
            continue;
        }

        if line.is_empty() {
            continue;
        }

        // Strip inline VTT tags: <c>, </c>, <00:00:00.000>, <v Speaker>, <b>, <i>, etc.
        let cleaned = strip_vtt_tags(line);
        let cleaned = cleaned.trim().to_string();

        if cleaned.is_empty() {
            continue;
        }

        // Deduplicate consecutive identical lines (yt-dlp auto-subs often repeats)
        if last.as_deref() == Some(cleaned.as_str()) {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&cleaned);
        last = Some(cleaned);
    }

    out
}

/// Strips inline VTT tags from a cue line.
/// Handles `<tag>`, `</tag>`, `<00:00:00.000>` timestamp tags.
fn strip_vtt_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if ch == '<' {
            // Scan forward in the original string for the closing '>'
            if let Some(close) = input[i..].find('>') {
                // Advance the char iterator past the tag
                let tag_end = i + close + 1; // byte index after '>'
                while chars.peek().map(|&(j, _)| j < tag_end).unwrap_or(false) {
                    chars.next();
                }
                continue;
            }
        }
        out.push(ch);
    }
    out
}

#[async_trait::async_trait]
impl TranscriptProvider for TranscriptAdapter {
    async fn fetch_transcript(&self, video_id: &str) -> Result<String, PortTranscriptError> {
        self.fetch_transcript(video_id).await.map_err(|e| match e {
            TranscriptError::NoCaptions => PortTranscriptError::NotAvailable,
            TranscriptError::FetchError(msg) => PortTranscriptError::Provider(msg),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_VTT: &str = r#"WEBVTT
Kind: captions
Language: en

00:00:00.000 --> 00:00:03.500 align:start position:0%

Hello<00:00:00.480><c> and</c><00:00:00.720><c> welcome</c>

00:00:03.500 --> 00:00:07.000 align:start position:0%
Hello and welcome
<00:00:03.700><c> to</c><00:00:03.900><c> this</c><00:00:04.200><c> course.</c>

00:00:07.000 --> 00:00:10.000 align:start position:0%

In this lesson we cover Rust.

00:00:10.000 --> 00:00:13.000 align:start position:0%
In this lesson we cover Rust.
<00:00:10.200><c> Enjoy!</c>
"#;

    #[test]
    fn parse_vtt_strips_header_and_timestamps() {
        let result = parse_vtt(SAMPLE_VTT);
        assert!(!result.contains("WEBVTT"), "should strip WEBVTT header");
        assert!(!result.contains("-->"), "should strip timestamp lines");
        assert!(!result.contains("<c>"), "should strip inline tags");
        assert!(!result.contains("<00:"), "should strip timestamp tags");
    }

    #[test]
    fn parse_vtt_deduplicates_consecutive_lines() {
        let result = parse_vtt(SAMPLE_VTT);
        // "Hello and welcome" appears in two consecutive cues but should appear only once
        let count = result.matches("Hello and welcome").count();
        assert_eq!(count, 1, "duplicate consecutive lines should be merged");
    }

    #[test]
    fn parse_vtt_preserves_unique_content() {
        let result = parse_vtt(SAMPLE_VTT);
        assert!(result.contains("Hello and welcome"), "should contain first cue text");
        assert!(result.contains("In this lesson we cover Rust"), "should contain unique cue text");
    }

    #[test]
    fn parse_vtt_empty_input() {
        assert_eq!(parse_vtt(""), "");
    }

    #[test]
    fn parse_vtt_only_header() {
        assert_eq!(parse_vtt("WEBVTT\n\n"), "");
    }

    #[test]
    fn strip_vtt_tags_removes_angle_bracket_tags() {
        let input = "Hello<00:00:00.480><c> and</c><00:00:00.720><c> welcome</c>";
        let result = strip_vtt_tags(input);
        assert_eq!(result, "Hello and welcome");
    }

    #[test]
    fn parse_vtt_real_world_auto_sub() {
        let vtt = "WEBVTT\n\n00:00:00.000 --> 00:00:02.000\n<v Speaker>Hello world</v>\n\n00:00:02.000 --> 00:00:04.000\nHello world\n\n00:00:04.000 --> 00:00:06.000\nThis is a test.\n";
        let result = parse_vtt(vtt);
        // "Hello world" should only appear once due to dedup
        assert_eq!(result.matches("Hello world").count(), 1);
        assert!(result.contains("This is a test"));
    }
}
