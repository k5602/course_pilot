//! YouTube adapter using yt-dlp CLI subprocess.

use std::env;
use std::process::Command;
use std::time::Duration;

use serde::Deserialize;
use tokio::task::spawn_blocking;

use crate::domain::ports::{FetchError, PlaylistFetcher, RawVideoMetadata};
use crate::domain::value_objects::PlaylistUrl;

const RETRY_DELAYS_MS: [u64; 3] = [500, 1000, 2000];

#[derive(Debug, Deserialize)]
struct YtDlpEntry {
    id: String,
    title: Option<String>,
    description: Option<String>,
    duration: Option<f64>,
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
    let url = url.to_string();
    let cookies = cookies.map(String::from);

    spawn_blocking(move || run_yt_dlp_sync(&url, cookies.as_deref(), is_playlist))
        .await
        .map_err(|e| FetchError::Api(format!("task join error: {}", e)))?
}

fn run_yt_dlp_sync(
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

    let output = cmd.output().map_err(|e| {
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
        || lower.contains("unavailable")
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

        videos.push(RawVideoMetadata {
            youtube_id: entry.id,
            title,
            description: entry.description,
            duration_secs,
            position: position as u32,
        });
    }

    Ok(videos)
}
