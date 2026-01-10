//! YouTube playlist fetcher port.

use crate::domain::value_objects::PlaylistUrl;

/// Raw video metadata from YouTube.
#[derive(Debug, Clone)]
pub struct RawVideoMetadata {
    pub youtube_id: String,
    pub title: String,
    pub description: Option<String>,
    pub duration_secs: u32,
    pub position: u32,
}

/// Error type for playlist fetching.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Playlist not found: {0}")]
    NotFound(String),
    #[error("Rate limited")]
    RateLimited,
}

/// Port for fetching YouTube playlist data.
#[allow(async_fn_in_trait)]
pub trait PlaylistFetcher: Send + Sync {
    /// Fetches all videos from a playlist.
    async fn fetch_playlist(&self, url: &PlaylistUrl) -> Result<Vec<RawVideoMetadata>, FetchError>;
}
