//! YouTube playlist import functionality
//!
//! This module provides functionality to extract video titles and durations from YouTube playlists
//! using the YouTube Data API v3 for reliable playlist metadata retrieval.

use crate::ImportError;
use serde::Deserialize;
use std::time::Duration;

/// Create a properly configured HTTP client for YouTube API requests
fn create_http_client() -> Result<reqwest::Client, ImportError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("CoursePilot/0.1.0")
        .danger_accept_invalid_certs(false)
        .use_rustls_tls()
        .build()
        .map_err(|e| ImportError::Network(format!("Failed to create HTTP client: {e}")))
}

/// Struct representing a YouTube video section with title and duration
#[derive(Debug, Clone)]
pub struct YoutubeSection {
    pub title: String,
    pub duration: Duration,
    pub video_id: Option<String>,
    pub url: Option<String>,
}

/// YouTube playlist metadata
#[derive(Debug, Clone)]
pub struct YoutubePlaylistMetadata {
    pub title: String,
    pub description: Option<String>,
    pub channel_title: Option<String>,
    pub video_count: usize,
}

/// Import video titles and durations from a YouTube playlist URL
///
/// # Arguments
/// * `url` - The YouTube playlist URL to import from
/// * `api_key` - The YouTube Data API v3 key
///
/// # Returns
/// * `Ok((Vec<YoutubeSection>, YoutubePlaylistMetadata))` - Vector of video sections and playlist metadata
/// * `Err(ImportError)` - Error if import fails
pub async fn import_from_youtube(
    url: &str,
    api_key: &str,
) -> Result<(Vec<YoutubeSection>, YoutubePlaylistMetadata), ImportError> {
    if !is_valid_youtube_playlist_url(url) {
        return Err(ImportError::InvalidUrl(format!(
            "Invalid YouTube playlist URL: {url}"
        )));
    }

    let playlist_id = extract_playlist_id(url).unwrap_or_default();
    if playlist_id.is_empty() {
        return Err(ImportError::InvalidUrl(
            "Could not extract playlist ID".to_string(),
        ));
    }

    // Create a configured HTTP client with proper TLS settings
    let client = create_http_client()?;

    // Step 1: Get all video IDs in the playlist
    let mut video_ids = Vec::new();
    let mut next_page_token = None;
    loop {
        let api_url = format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?part=contentDetails&maxResults=50&playlistId={playlist_id}&key={api_key}"
        );
        let url_with_page = if let Some(token) = &next_page_token {
            format!("{api_url}&pageToken={token}")
        } else {
            api_url.clone()
        };

        let resp =
            client.get(&url_with_page).send().await.map_err(|e| {
                ImportError::Network(format!("Failed to fetch playlist items: {e}"))
            })?;
        #[derive(Deserialize)]
        struct PlaylistItemsResponse {
            items: Vec<PlaylistItem>,
            #[serde(rename = "nextPageToken")]
            next_page_token: Option<String>,
        }
        #[derive(Deserialize)]
        struct PlaylistItem {
            #[serde(rename = "contentDetails")]
            content_details: ContentDetails,
        }
        #[derive(Deserialize)]
        struct ContentDetails {
            #[serde(rename = "videoId")]
            video_id: String,
        }
        let playlist_resp: PlaylistItemsResponse = resp.json().await.map_err(|e| {
            ImportError::Network(format!("Failed to parse playlist items response: {e}"))
        })?;
        for item in playlist_resp.items {
            video_ids.push(item.content_details.video_id);
        }
        if let Some(token) = playlist_resp.next_page_token {
            next_page_token = Some(token);
        } else {
            break;
        }
    }

    if video_ids.is_empty() {
        return Err(ImportError::NoContent);
    }

    // Step 1.5: Fetch playlist metadata
    let playlist_metadata = fetch_playlist_metadata(playlist_id, api_key, &client).await?;

    // Step 2: Fetch video details (title, duration) in batches of 50
    let mut sections = Vec::new();
    for chunk in video_ids.chunks(50) {
        let ids = chunk.join(",");
        let api_url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=contentDetails,snippet&id={ids}&key={api_key}"
        );
        let resp = client
            .get(&api_url)
            .send()
            .await
            .map_err(|e| ImportError::Network(format!("Failed to fetch video details: {e}")))?;
        #[derive(Deserialize)]
        struct VideosResponse {
            items: Vec<VideoItem>,
        }
        #[derive(Deserialize)]
        struct VideoItem {
            snippet: Snippet,
            #[serde(rename = "contentDetails")]
            content_details: VideoContentDetails,
        }
        #[derive(Deserialize)]
        struct Snippet {
            title: String,
        }
        #[derive(Deserialize)]
        struct VideoContentDetails {
            duration: String,
        }
        let videos_resp: VideosResponse = resp.json().await.map_err(|e| {
            ImportError::Network(format!("Failed to parse video details response: {e}"))
        })?;
        for (item, video_id) in videos_resp.items.iter().zip(chunk.iter()) {
            let title = clean_video_title(&item.snippet.title);
            let duration = parse_iso8601_duration(&item.content_details.duration)
                .unwrap_or_else(|| Duration::from_secs(0));
            let url = format!("https://www.youtube.com/watch?v={}", video_id);
            sections.push(YoutubeSection { 
                title, 
                duration, 
                video_id: Some(video_id.to_string()),
                url: Some(url),
            });
        }
    }

    Ok((sections, playlist_metadata))
}

/// Parse ISO 8601 duration string (e.g., PT1H2M3S) to std::time::Duration
fn parse_iso8601_duration(s: &str) -> Option<Duration> {
    let mut secs = 0u64;
    let mut num = String::new();
    let chars = s.chars().peekable();
    for c in chars {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            match c {
                'H' => {
                    secs += num.parse::<u64>().ok()? * 3600;
                    num.clear();
                }
                'M' => {
                    secs += num.parse::<u64>().ok()? * 60;
                    num.clear();
                }
                'S' => {
                    secs += num.parse::<u64>().ok()?;
                    num.clear();
                }
                _ => {
                    num.clear();
                }
            }
        }
    }
    Some(Duration::from_secs(secs))
}

/// Validate that a URL is a YouTube playlist URL
fn is_valid_youtube_playlist_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Check for YouTube domain
    if !url_lower.contains("youtube.com") && !url_lower.contains("youtu.be") {
        return false;
    }

    // Check for playlist indicators
    url_lower.contains("playlist") || url_lower.contains("list=")
}

/// Fetch playlist metadata (title, description, etc.)
async fn fetch_playlist_metadata(
    playlist_id: String,
    api_key: &str,
    client: &reqwest::Client,
) -> Result<YoutubePlaylistMetadata, ImportError> {
    let api_url = format!(
        "https://www.googleapis.com/youtube/v3/playlists?part=snippet,contentDetails&id={playlist_id}&key={api_key}"
    );

    let resp = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| ImportError::Network(format!("Failed to fetch playlist metadata: {e}")))?;

    #[derive(Deserialize)]
    struct PlaylistResponse {
        items: Vec<PlaylistItem>,
    }

    #[derive(Deserialize)]
    struct PlaylistItem {
        snippet: PlaylistSnippet,
        #[serde(rename = "contentDetails")]
        content_details: PlaylistContentDetails,
    }

    #[derive(Deserialize)]
    struct PlaylistSnippet {
        title: String,
        description: String,
        #[serde(rename = "channelTitle")]
        channel_title: String,
    }

    #[derive(Deserialize)]
    struct PlaylistContentDetails {
        #[serde(rename = "itemCount")]
        item_count: usize,
    }

    let playlist_resp: PlaylistResponse = resp.json().await.map_err(|e| {
        ImportError::Network(format!("Failed to parse playlist metadata response: {e}"))
    })?;

    if let Some(item) = playlist_resp.items.first() {
        Ok(YoutubePlaylistMetadata {
            title: item.snippet.title.clone(),
            description: Some(item.snippet.description.clone()),
            channel_title: Some(item.snippet.channel_title.clone()),
            video_count: item.content_details.item_count,
        })
    } else {
        Ok(YoutubePlaylistMetadata {
            title: format!(
                "YouTube Playlist {}",
                &playlist_id[..8.min(playlist_id.len())]
            ),
            description: None,
            channel_title: None,
            video_count: 0,
        })
    }
}

/// Validate playlist existence and accessibility using YouTube Data API v3
async fn validate_playlist_real(url: &str, api_key: &str) -> Result<bool, ImportError> {
    let client = create_http_client()?;
    let playlist_id = extract_playlist_id(url)
        .ok_or_else(|| ImportError::InvalidUrl("Could not extract playlist ID".to_string()))?;
    let api_url = format!(
        "https://www.googleapis.com/youtube/v3/playlists?part=status&id={playlist_id}&key={api_key}"
    );
    let resp = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| ImportError::Network(format!("Failed to fetch playlist: {e}")))?;
    #[derive(serde::Deserialize)]
    struct PlaylistStatusResponse {
        items: Vec<PlaylistStatusItem>,
    }
    #[derive(serde::Deserialize)]
    struct PlaylistStatusItem {
        status: PlaylistPrivacyStatus,
    }
    #[derive(serde::Deserialize)]
    struct PlaylistPrivacyStatus {
        #[serde(rename = "privacyStatus")]
        privacy_status: String,
    }
    let playlist_resp: PlaylistStatusResponse = resp.json().await.map_err(|e| {
        ImportError::Network(format!("Failed to parse playlist status response: {e}"))
    })?;
    if let Some(item) = playlist_resp.items.first() {
        Ok(item.status.privacy_status == "public" || item.status.privacy_status == "unlisted")
    } else {
        Ok(false)
    }
}

/// Clean and normalize video titles
fn clean_video_title(title: &str) -> String {
    title
        .trim()
        // Remove common patterns that don't add value
        .replace(" - YouTube", "")
        .replace(" | YouTube", "")
        // Normalize whitespace
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract playlist ID from various YouTube URL formats
pub fn extract_playlist_id(url: &str) -> Option<String> {
    if let Some(start) = url.find("list=") {
        let id_start = start + 5; // length of "list="
        if let Some(end) = url[id_start..].find('&') {
            Some(url[id_start..id_start + end].to_string())
        } else {
            Some(url[id_start..].to_string())
        }
    } else {
        None
    }
}

/// Get playlist metadata without downloading all videos (for quick validation)
/// Validate a YouTube playlist URL using the YouTube Data API v3.
/// Returns true if the playlist exists and is accessible (public or unlisted).
pub async fn validate_playlist_url(url: &str, api_key: &str) -> Result<bool, ImportError> {
    if !is_valid_youtube_playlist_url(url) {
        return Ok(false);
    }
    validate_playlist_real(url, api_key).await
}

/// Validate a YouTube API key by making a simple API request
/// Returns true if the API key is valid and has the necessary permissions
pub async fn validate_api_key(api_key: &str) -> Result<bool, ImportError> {
    let client = create_http_client()?;

    // Make a simple request to the channels endpoint to test the API key
    let api_url =
        format!("https://www.googleapis.com/youtube/v3/channels?part=id&mine=true&key={api_key}");

    let resp = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| ImportError::Network(format!("Failed to validate API key: {e}")))?;

    // Check if the response is successful
    if resp.status().is_success() {
        Ok(true)
    } else if resp.status() == 403 {
        // API key is invalid or doesn't have required permissions
        Ok(false)
    } else {
        // Other error - treat as network error
        Err(ImportError::Network(format!(
            "API validation failed with status: {}",
            resp.status()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        // Valid URLs
        assert!(is_valid_youtube_playlist_url(
            "https://www.youtube.com/playlist?list=PLrAXtmRdnEQy"
        ));
        assert!(is_valid_youtube_playlist_url(
            "https://youtube.com/watch?v=abc&list=PLrAXtmRdnEQy"
        ));

        // Invalid URLs
        assert!(!is_valid_youtube_playlist_url("https://example.com"));
        assert!(!is_valid_youtube_playlist_url(
            "https://youtube.com/watch?v=abc"
        ));
        assert!(!is_valid_youtube_playlist_url("not a url"));
    }

    #[test]
    fn test_title_cleaning() {
        assert_eq!(
            clean_video_title("  My Video Title - YouTube  "),
            "My Video Title"
        );
        assert_eq!(clean_video_title("Tutorial   Part   1"), "Tutorial Part 1");
    }

    #[test]
    fn test_playlist_id_extraction() {
        assert_eq!(
            extract_playlist_id(
                "https://youtube.com/playlist?list=PLJEZDlUEtOf5rZjVFnijy6wSW-laKiY0l&si=aLcYFs9uDCcfvNGd"
            ),
            Some("PLJEZDlUEtOf5rZjVFnijy6wSW-laKiY0l".to_string())
        );
        assert_eq!(
            extract_playlist_id("https://youtube.com/watch?v=abc&list=PLtest&other=param"),
            Some("PLtest".to_string())
        );
        assert_eq!(extract_playlist_id("https://example.com"), None);
    }

    #[tokio::test]
    async fn test_import_invalid_url() {
        let result = import_from_youtube("not a url", "dummy_api_key").await;
        assert!(matches!(result, Err(ImportError::InvalidUrl(_))));
    }
}
