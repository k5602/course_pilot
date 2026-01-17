//! YouTube adapter.

use rusty_ytdl::search::{Playlist, PlaylistSearchOptions};

use crate::domain::ports::{FetchError, PlaylistFetcher, RawVideoMetadata};
use crate::domain::value_objects::PlaylistUrl;

/// YouTube adapter for fetching playlist data.
pub struct RustyYtdlAdapter;

impl RustyYtdlAdapter {
    /// Creates a new adapter (no configuration needed).
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustyYtdlAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaylistFetcher for RustyYtdlAdapter {
    async fn fetch_playlist(&self, url: &PlaylistUrl) -> Result<Vec<RawVideoMetadata>, FetchError> {
        let playlist_url = url.raw();

        // Configure to fetch all videos
        let options = PlaylistSearchOptions { limit: u64::MAX, ..Default::default() };

        // Fetch playlist
        let mut playlist: Playlist = Playlist::get(playlist_url, Some(&options))
            .await
            .map_err(|e| FetchError::Api(format!("Failed to fetch playlist: {}", e)))?;

        // Fetch any remaining videos if playlist is large
        while let Ok(more_videos) = playlist.next(Some(100)).await {
            if more_videos.is_empty() {
                break;
            }
        }

        if playlist.videos.is_empty() {
            return Err(FetchError::NotFound(url.playlist_id().to_string()));
        }

        // Convert rusty_ytdl videos to domain RawVideoMetadata
        let videos: Vec<RawVideoMetadata> = playlist
            .videos
            .into_iter()
            .enumerate()
            .map(|(position, video)| {
                // Duration is in milliseconds, convert to seconds
                let duration_secs = (video.duration / 1000) as u32;

                RawVideoMetadata {
                    youtube_id: video.id,
                    title: video.title,
                    description: Some(video.description),
                    duration_secs,
                    position: position as u32,
                }
            })
            .collect();

        Ok(videos)
    }
}
