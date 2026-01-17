//! YouTube adapter.

use std::env;

use rusty_ytdl::Video;
use rusty_ytdl::search::{Playlist, PlaylistSearchOptions, RequestOptions};

use crate::domain::ports::{FetchError, PlaylistFetcher, RawVideoMetadata};
use crate::domain::value_objects::PlaylistUrl;

/// YouTube adapter for fetching playlist data.
pub struct RustyYtdlAdapter {
    cookies: Option<String>,
}

impl RustyYtdlAdapter {
    /// Creates a new adapter.
    pub fn new() -> Self {
        let cookies = env::var("YOUTUBE_COOKIES")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

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
        let playlist_url = url.raw();

        // Configure to fetch all videos in one pass when possible.
        let mut options =
            PlaylistSearchOptions { limit: 500, fetch_all: true, ..Default::default() };

        if let Some(ref cookie) = self.cookies {
            options.request_options =
                Some(RequestOptions { cookies: Some(cookie.clone()), ..Default::default() });
        }

        // Fetch playlist (fallback to single video on failure).
        let mut playlist: Playlist = match Playlist::get(playlist_url, Some(&options)).await {
            Ok(playlist) => playlist,
            Err(e) => {
                if url.video_id().is_some() {
                    return fetch_single_video_or_err(playlist_url, e.to_string()).await;
                }
                return Err(FetchError::Api(format!("Failed to fetch playlist: {}", e)));
            },
        };

        // Fetch all videos if the initial request did not include them.
        playlist.fetch(None).await;

        if playlist.videos.is_empty() {
            if url.video_id().is_some() {
                return fetch_single_video_or_err(playlist_url, "Playlist is empty".to_string())
                    .await;
            }
            return Err(FetchError::NotFound(url.playlist_id().to_string()));
        }

        // Convert rusty_ytdl videos to domain RawVideoMetadata.
        let videos: Vec<RawVideoMetadata> = playlist
            .videos
            .into_iter()
            .enumerate()
            .map(|(position, video)| {
                // Duration is in milliseconds, convert to seconds.
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

async fn fetch_single_video_or_err(
    url_or_id: &str,
    reason: String,
) -> Result<Vec<RawVideoMetadata>, FetchError> {
    let video = Video::new(url_or_id)
        .map_err(|_| FetchError::Api(format!("Failed to fetch playlist: {}", reason)))?;

    let info = video
        .get_info()
        .await
        .map_err(|_| FetchError::Api(format!("Failed to fetch playlist: {}", reason)))?;

    let details = info.video_details;
    let duration_secs = parse_duration_secs(&details.length_seconds);

    Ok(vec![RawVideoMetadata {
        youtube_id: details.video_id,
        title: details.title,
        description: Some(details.description),
        duration_secs,
        position: 0,
    }])
}

fn parse_duration_secs(raw: &str) -> u32 {
    raw.parse::<u32>().unwrap_or(0)
}
