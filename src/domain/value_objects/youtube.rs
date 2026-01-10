//! YouTube-related value objects.

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
}

impl PlaylistUrl {
    /// Creates a new PlaylistUrl from a raw URL string.
    /// Extracts and validates the playlist ID.
    pub fn new(url: &str) -> Result<Self, YouTubeError> {
        let playlist_id = Self::extract_playlist_id(url)
            .ok_or_else(|| YouTubeError::InvalidPlaylistUrl(url.to_string()))?;

        Ok(Self { raw: url.to_string(), playlist_id })
    }

    fn extract_playlist_id(url: &str) -> Option<String> {
        // Handle both youtube.com and youtu.be URLs
        // Expected format: ?list=PLAYLIST_ID or &list=PLAYLIST_ID
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

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn playlist_id(&self) -> &str {
        &self.playlist_id
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
}
