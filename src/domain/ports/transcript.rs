//! Transcript provider port.
//!
//! Defines the contract for fetching video transcripts from external providers.

/// Error type for transcript operations.
#[derive(Debug, thiserror::Error)]
pub enum TranscriptError {
    #[error("No transcript available for this video")]
    NotAvailable,
    #[error("Provider error: {0}")]
    Provider(String),
}

/// Port for fetching transcripts for videos.
#[allow(async_fn_in_trait)]
pub trait TranscriptProvider: Send + Sync {
    /// Fetch the transcript for a given video ID.
    async fn fetch_transcript(&self, video_id: &str) -> Result<String, TranscriptError>;
}
