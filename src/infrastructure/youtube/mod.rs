//! YouTube adapter using yt-dlp CLI subprocess.

use std::env;
use std::time::Duration;

use serde::Deserialize;
use tokio::process::Command;

use crate::domain::ports::{FetchError, PlaylistFetcher, RawVideoMetadata, StreamResolver};
use crate::domain::value_objects::{PlaylistUrl, VideoQuality};

const RETRY_DELAYS_MS: [u64; 3] = [500, 1000, 2000];

#[derive(Debug, Deserialize)]
struct YtDlpEntry {
    id: String,
    title: Option<String>,
    description: Option<String>,
    duration: Option<f64>,
    playlist_index: Option<u32>,
}

/// YouTube adapter using yt-dlp CLI.
pub struct RustyYtdlAdapter {
    cookies: Option<String>,
}

impl RustyYtdlAdapter {
    pub fn new() -> Self {
        Self::with_cookies(None)
    }

    pub fn with_cookies(cookie_path: Option<String>) -> Self {
        let cookies = cookie_path.or_else(|| {
            env::var("YOUTUBE_COOKIES")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        });
        Self { cookies }
    }
}

impl Default for RustyYtdlAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaylistFetcher for RustyYtdlAdapter {
    async fn fetch_playlist(&self, url: &PlaylistUrl) -> Result<Vec<RawVideoMetadata>, FetchError> {
        let url_str = url.raw().to_string();
        let cookies = self.cookies.clone();
        let is_playlist = url_str.contains("list=");

        let result = fetch_with_retry(url_str.clone(), cookies.clone(), is_playlist).await;

        match result {
            Ok(videos) if !videos.is_empty() => Ok(videos),
            Ok(_) | Err(_) if is_playlist && url.video_id().is_some() => {
                // Playlist empty or fetch failed; fall back to single video
                let single_url =
                    format!("https://www.youtube.com/watch?v={}", url.video_id().unwrap());
                fetch_with_retry(single_url, cookies, false).await
            },
            Ok(_) => Err(FetchError::NotFound(url.playlist_id().to_string())),
            Err(e) => Err(e),
        }
    }
}

async fn fetch_with_retry(
    url: String,
    cookies: Option<String>,
    is_playlist: bool,
) -> Result<Vec<RawVideoMetadata>, FetchError> {
    let mut last_err = None;

    for (i, delay_ms) in RETRY_DELAYS_MS.iter().enumerate() {
        match run_yt_dlp(&url, cookies.as_deref(), is_playlist).await {
            Ok(videos) => return Ok(videos),
            Err(FetchError::Network(msg)) if i < RETRY_DELAYS_MS.len() - 1 => {
                last_err = Some(FetchError::Network(msg));
                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
            },
            Err(e) => return Err(e),
        }
    }

    Err(last_err.unwrap_or_else(|| FetchError::Api("unknown error".to_string())))
}

async fn run_yt_dlp(
    url: &str,
    cookies: Option<&str>,
    is_playlist: bool,
) -> Result<Vec<RawVideoMetadata>, FetchError> {
    let mut cmd = Command::new("yt-dlp");

    cmd.arg("--dump-json");
    cmd.arg("--no-warnings");

    if is_playlist {
        cmd.arg("--flat-playlist");
        cmd.arg("--no-download");
        cmd.arg("--ignore-errors");
    }

    if let Some(cookie_path) = cookies {
        cmd.arg("--cookies");
        cmd.arg(cookie_path);
    }

    cmd.arg(url);

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            FetchError::NotAvailable(
                "yt-dlp not found. Install with: pip install yt-dlp".to_string(),
            )
        } else {
            FetchError::Api(format!("failed to execute yt-dlp: {}", e))
        }
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return classify_error(&stderr, &stdout);
    }

    if stdout.trim().is_empty() {
        return Err(FetchError::NotFound("no videos found".to_string()));
    }

    parse_output(&stdout)
}

fn classify_error(stderr: &str, stdout: &str) -> Result<Vec<RawVideoMetadata>, FetchError> {
    let combined = format!("{} {}", stderr, stdout);
    let lower = combined.to_lowercase();

    if lower.contains("not found")
        || lower.contains("does not exist")
        || lower.contains("video unavailable")
        || lower.contains("private")
        || lower.contains("no longer")
    {
        return Err(FetchError::NotFound(combined));
    }

    if lower.contains("network")
        || lower.contains("connection")
        || lower.contains("timed out")
        || lower.contains("http error")
        || lower.contains("unable to download")
    {
        return Err(FetchError::Network(combined));
    }

    Err(FetchError::Api(combined))
}

fn parse_output(stdout: &str) -> Result<Vec<RawVideoMetadata>, FetchError> {
    let mut videos: Vec<RawVideoMetadata> = Vec::new();

    for (position, line) in stdout.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry: YtDlpEntry = serde_json::from_str(line)
            .map_err(|e| FetchError::Api(format!("failed to parse yt-dlp output: {}", e)))?;

        let duration_secs = entry.duration.map(|d| d as u32).unwrap_or(0);
        let title = entry.title.unwrap_or_else(|| "untitled".to_string());

        let pos = entry.playlist_index.unwrap_or(position as u32);
        videos.push(RawVideoMetadata {
            youtube_id: entry.id,
            title,
            description: entry.description,
            duration_secs,
            position: pos,
        });
    }

    Ok(videos)
}

impl StreamResolver for RustyYtdlAdapter {
    async fn resolve_youtube_stream(
        &self,
        youtube_id: &str,
        quality: VideoQuality,
    ) -> Result<String, FetchError> {
        resolve_youtube_stream_inner(youtube_id, quality).await
    }
}

pub async fn resolve_stream_url(youtube_id: &str) -> Result<String, FetchError> {
    resolve_youtube_stream_inner(youtube_id, VideoQuality::Best).await
}

pub(crate) async fn resolve_youtube_stream_inner(
    youtube_id: &str,
    quality: VideoQuality,
) -> Result<String, FetchError> {
    let url = format!("https://www.youtube.com/watch?v={youtube_id}");
    let format_str = quality.ytdlp_format();
    let output = Command::new("yt-dlp")
        .arg("-g")
        .arg("--no-warnings")
        .arg("-f")
        .arg(format_str)
        .arg(&url)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FetchError::NotAvailable(
                    "yt-dlp not found. Install with: pip install yt-dlp".to_string(),
                )
            } else {
                FetchError::Api(format!("failed to execute yt-dlp: {e}"))
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let err = match classify_error(&stderr, &stdout) {
            Err(e) => e,
            _ => FetchError::Api(stderr.trim().to_string()),
        };
        return Err(err);
    }

    stdout
        .lines()
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("http"))
        .ok_or_else(|| FetchError::Api("no stream URL found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_output_handles_valid_json_lines() {
        let input = r#"{"id":"abc123","title":"Video 1","duration":120.5,"playlist_index":1}
{"id":"def456","title":"Video 2","duration":300.0,"playlist_index":2,"description":"A description"}"#;
        let result = parse_output(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].youtube_id, "abc123");
        assert_eq!(result[0].position, 1);
        assert_eq!(result[1].position, 2);
        assert_eq!(result[1].description.as_deref(), Some("A description"));
    }

    #[test]
    fn parse_output_falls_back_to_enumerate_without_playlist_index() {
        let input = r#"{"id":"abc123","title":"Video 1","duration":120.0}
{"id":"def456","title":"Video 2","duration":300.0}"#;
        let result = parse_output(input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].position, 0);
        assert_eq!(result[1].position, 1);
    }

    #[test]
    fn parse_output_handles_empty_lines() {
        let input = r#"{"id":"a","title":"V1","duration":10.0}

{"id":"b","title":"V2","duration":20.0}"#;
        let result = parse_output(input).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_output_handles_missing_title() {
        let input = r#"{"id":"a","duration":10.0}"#;
        let result = parse_output(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "untitled");
    }

    #[test]
    fn classify_error_detects_not_found() {
        let err = classify_error("ERROR: Video not found", "");
        assert!(matches!(err, Err(FetchError::NotFound(_))));
    }

    #[test]
    fn classify_error_detects_network_error() {
        let err = classify_error("HTTP Error 503: Service Unavailable", "");
        assert!(matches!(err, Err(FetchError::Network(_))));
    }

    #[test]
    fn classify_error_detects_generic_api_error() {
        let err = classify_error("Some unknown error happened", "");
        assert!(matches!(err, Err(FetchError::Api(_))));
    }
}
