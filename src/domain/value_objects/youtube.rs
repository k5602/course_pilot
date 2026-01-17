//! YouTube-related value objects.

use rusty_ytdl::get_video_id;
use rusty_ytdl::search::Playlist;
use thiserror::Error;

/// Error when parsing YouTube URLs or IDs.
#[derive(Debug, Error)]
pub enum YouTubeError {
    #[error("Invalid playlist URL: {0}")]
    InvalidPlaylistUrl(String),
    #[error("Invalid video ID: {0}")]
    InvalidVideoId(String),
}

/// A validated YouTube playlist URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistUrl {
    raw: String,
    playlist_id: String,
    video_id: Option<String>,
}

impl PlaylistUrl {
    /// Creates a new PlaylistUrl from a raw URL string.
    /// Extracts and validates the playlist ID.
    pub fn new(url: &str) -> Result<Self, YouTubeError> {
        if let Some(playlist_id) = Self::extract_playlist_id(url) {
            let video_id = Self::extract_video_id(url);
            let raw = Playlist::get_playlist_url(url).unwrap_or_else(|| url.to_string());
            return Ok(Self { raw, playlist_id, video_id });
        }

        if let Some(video_id) = Self::extract_video_id(url) {
            let raw = format!("https://www.youtube.com/watch?v={}", video_id);
            return Ok(Self { raw, playlist_id: video_id.clone(), video_id: Some(video_id) });
        }

        Err(YouTubeError::InvalidPlaylistUrl(url.to_string()))
    }

    fn extract_playlist_id(url: &str) -> Option<String> {
        // Try finding list= in the raw URL first (handles watch?v=...&list=...)
        if let Some(id) = Self::parse_list_param(url) {
            return Some(id);
        }

        // Fallback to checking what rusty_ytdl considers a playlist URL
        Playlist::get_playlist_url(url).and_then(|p_url| Self::parse_list_param(&p_url))
    }

    fn parse_list_param(url: &str) -> Option<String> {
        if let Some(start) = url.find("list=") {
            let id_start = start + 5;
            let id_end = url[id_start..].find('&').map(|i| id_start + i).unwrap_or(url.len());
            let id = &url[id_start..id_end];
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
        None
    }

    fn extract_video_id(url: &str) -> Option<String> {
        get_video_id(url)
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn playlist_id(&self) -> &str {
        &self.playlist_id
    }

    pub fn video_id(&self) -> Option<&str> {
        self.video_id.as_deref()
    }
}

/// A validated YouTube video ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YouTubeVideoId(String);

impl YouTubeVideoId {
    /// Creates a new YouTubeVideoId.
    /// YouTube video IDs are typically 11 characters.
    pub fn new(id: &str) -> Result<Self, YouTubeError> {
        // Basic validation: YouTube IDs are typically 11 chars, alphanumeric with - and _
        if id.len() >= 10
            && id.len() <= 12
            && id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            Ok(Self(id.to_string()))
        } else {
            Err(YouTubeError::InvalidVideoId(id.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_url_valid() {
        let url = PlaylistUrl::new(
            "https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf",
        )
        .unwrap();
        assert_eq!(url.playlist_id(), "PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf");
    }

    #[test]
    fn test_playlist_url_with_extra_params() {
        let url =
            PlaylistUrl::new("https://www.youtube.com/watch?v=abc&list=PLtest123&index=1").unwrap();
        assert_eq!(url.playlist_id(), "PLtest123");
    }

    #[test]
    fn test_video_id_valid() {
        let id = YouTubeVideoId::new("dQw4w9WgXcQ").unwrap();
        assert_eq!(id.as_str(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_playlist_url_with_video_only() {
        let url = PlaylistUrl::new("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(url.video_id(), Some("dQw4w9WgXcQ"));
    }
}
