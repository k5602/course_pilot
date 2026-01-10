//! YouTube API adapter using google-youtube3.

use google_youtube3::{YouTube, hyper_rustls, hyper_util};
use hyper_util::client::legacy::connect::HttpConnector;
use std::sync::Arc;

use crate::domain::ports::{FetchError, PlaylistFetcher, RawVideoMetadata};
use crate::domain::value_objects::PlaylistUrl;

type YouTubeHub = YouTube<hyper_rustls::HttpsConnector<HttpConnector>>;

/// YouTube API adapter for fetching playlist data.
pub struct YouTubeApiAdapter {
    hub: Arc<YouTubeHub>,
}

impl YouTubeApiAdapter {
    /// Creates a new YouTube API adapter with the given API key.
    pub fn new(api_key: String) -> Self {
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .expect("Failed to load native TLS roots")
                        .https_or_http()
                        .enable_http2()
                        .build(),
                );

        let hub = YouTube::new(client, api_key);
        Self { hub: Arc::new(hub) }
    }

    /// Parses ISO 8601 duration (e.g., "PT1H2M3S") to seconds.
    fn parse_duration(duration: &str) -> u32 {
        // ISO 8601 duration format: PT#H#M#S
        let duration = duration.trim_start_matches("PT");
        let mut seconds = 0u32;
        let mut current_num = String::new();

        for c in duration.chars() {
            if c.is_ascii_digit() {
                current_num.push(c);
            } else {
                let value: u32 = current_num.parse().unwrap_or(0);
                match c {
                    'H' => seconds += value * 3600,
                    'M' => seconds += value * 60,
                    'S' => seconds += value,
                    _ => {},
                }
                current_num.clear();
            }
        }

        seconds
    }
}

impl PlaylistFetcher for YouTubeApiAdapter {
    async fn fetch_playlist(&self, url: &PlaylistUrl) -> Result<Vec<RawVideoMetadata>, FetchError> {
        let playlist_id = url.playlist_id();
        let mut all_videos = Vec::new();
        let mut page_token: Option<String> = None;

        // Paginate through all playlist items
        loop {
            let mut request = self
                .hub
                .playlist_items()
                .list(&vec!["snippet".into(), "contentDetails".into()])
                .playlist_id(playlist_id)
                .max_results(50);

            if let Some(ref token) = page_token {
                request = request.page_token(token);
            }

            let (_, response) = request.doit().await.map_err(|e| FetchError::Api(e.to_string()))?;

            if let Some(items) = response.items {
                for (position, item) in items.into_iter().enumerate() {
                    let snippet = item.snippet.ok_or_else(|| {
                        FetchError::Api("Missing snippet in playlist item".to_string())
                    })?;

                    let content_details = item
                        .content_details
                        .ok_or_else(|| FetchError::Api("Missing content details".to_string()))?;

                    let video_id = content_details
                        .video_id
                        .ok_or_else(|| FetchError::Api("Missing video ID".to_string()))?;

                    // Fetch video duration separately
                    let duration_secs = self.fetch_video_duration(&video_id).await?;

                    all_videos.push(RawVideoMetadata {
                        youtube_id: video_id,
                        title: snippet.title.unwrap_or_default(),
                        description: snippet.description,
                        duration_secs,
                        position: (all_videos.len() + position) as u32,
                    });
                }
            }

            // Check for next page
            page_token = response.next_page_token;
            if page_token.is_none() {
                break;
            }
        }

        if all_videos.is_empty() {
            return Err(FetchError::NotFound(playlist_id.to_string()));
        }

        Ok(all_videos)
    }
}

impl YouTubeApiAdapter {
    /// Fetches the duration of a video by its ID.
    async fn fetch_video_duration(&self, video_id: &str) -> Result<u32, FetchError> {
        let (_, response) = self
            .hub
            .videos()
            .list(&vec!["contentDetails".into()])
            .add_id(video_id)
            .doit()
            .await
            .map_err(|e| FetchError::Api(e.to_string()))?;

        if let Some(items) = response.items {
            if let Some(video) = items.into_iter().next() {
                if let Some(content_details) = video.content_details {
                    if let Some(duration) = content_details.duration {
                        return Ok(Self::parse_duration(&duration));
                    }
                }
            }
        }

        // Default to 0 if duration not found
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_minutes_seconds() {
        assert_eq!(YouTubeApiAdapter::parse_duration("PT10M30S"), 630);
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(YouTubeApiAdapter::parse_duration("PT1H2M3S"), 3723);
    }

    #[test]
    fn test_parse_duration_seconds_only() {
        assert_eq!(YouTubeApiAdapter::parse_duration("PT45S"), 45);
    }
}
