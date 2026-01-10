//! YouTube API adapter using google-youtube3.

use crate::domain::ports::{FetchError, PlaylistFetcher, RawVideoMetadata};
use crate::domain::value_objects::PlaylistUrl;

/// YouTube API adapter using google-youtube3.
pub struct YouTubeApiAdapter {
    api_key: String,
}

impl YouTubeApiAdapter {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl PlaylistFetcher for YouTubeApiAdapter {
    async fn fetch_playlist(&self, url: &PlaylistUrl) -> Result<Vec<RawVideoMetadata>, FetchError> {
        // TODO: Implement with google-youtube3
        // 1. Create YouTube hub
        // 2. Call playlist_items().list()
        // 3. Paginate through all results
        // 4. For each video, fetch contentDetails for duration

        let _ = (&self.api_key, url);
        todo!("Implement with google-youtube3 crate")
    }
}
