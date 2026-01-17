//! YouTube transcript fetcher using yt-transcript-rs.

use crate::domain::ports::{TranscriptError as PortTranscriptError, TranscriptProvider};
use yt_transcript_rs::YouTubeTranscriptApi;

/// Error type for transcript operations.
#[derive(Debug, thiserror::Error)]
pub enum TranscriptError {
    #[error("No captions available for this video")]
    NoCaptions,
    #[error("Failed to fetch transcript: {0}")]
    FetchError(String),
}

/// Fetches transcripts from YouTube videos.
pub struct TranscriptAdapter {
    api: YouTubeTranscriptApi,
}

impl TranscriptAdapter {
    pub fn new() -> Result<Self, TranscriptError> {
        let api = YouTubeTranscriptApi::new(None, None, None)
            .map_err(|e| TranscriptError::FetchError(e.to_string()))?;
        Ok(Self { api })
    }

    /// Fetches the transcript for a YouTube video.
    /// Returns the full transcript text or an error if unavailable.
    pub async fn fetch_transcript(&self, video_id: &str) -> Result<String, TranscriptError> {
        // Try English first, then any available language
        let transcript = self
            .api
            .fetch_transcript(video_id, &["en", "en-US", "en-GB"], false)
            .await
            .map_err(|e| TranscriptError::FetchError(e.to_string()))?;

        let text = transcript.text();

        if text.is_empty() { Err(TranscriptError::NoCaptions) } else { Ok(text) }
    }
}

#[allow(async_fn_in_trait)]
impl TranscriptProvider for TranscriptAdapter {
    async fn fetch_transcript(&self, video_id: &str) -> Result<String, PortTranscriptError> {
        self.fetch_transcript(video_id).await.map_err(|e| match e {
            TranscriptError::NoCaptions => PortTranscriptError::NotAvailable,
            TranscriptError::FetchError(msg) => PortTranscriptError::Provider(msg),
        })
    }
}

impl Default for TranscriptAdapter {
    fn default() -> Self {
        Self::new().expect("Failed to create TranscriptAdapter")
    }
}
