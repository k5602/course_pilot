//! Summarize video use case with caching for transcript and summary.
//!
//! This use case:
//! - Loads the video from the repository
//! - Uses cached summary/transcript when available
//! - Fetches transcript from a provider when missing or forced
//! - Generates summary with the LLM and persists it

use std::sync::Arc;

use crate::domain::{
    ports::{RepositoryError, SummarizerAI, TranscriptError, TranscriptProvider, VideoRepository},
    value_objects::VideoId,
};

/// Error type for summary generation.
#[derive(Debug, thiserror::Error)]
pub enum SummarizeVideoError {
    #[error("Video not found")]
    VideoNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
    #[error("Transcript error: {0}")]
    Transcript(String),
    #[error("AI error: {0}")]
    AI(String),
}

/// Input for the summarize video use case.
#[derive(Debug, Clone)]
pub struct SummarizeVideoInput {
    pub video_id: VideoId,
    /// When true, bypasses cached transcript/summary and regenerates.
    pub force_refresh: bool,
}

/// Result for summary generation.
#[derive(Debug, Clone)]
pub struct SummarizeVideoOutput {
    pub summary: String,
    pub transcript_used: String,
    pub cached: bool,
}

/// Use case for summarizing a video with caching.
pub struct SummarizeVideoUseCase<AI, TR, VR>
where
    AI: SummarizerAI,
    TR: TranscriptProvider,
    VR: VideoRepository,
{
    llm: Arc<AI>,
    transcript_provider: Arc<TR>,
    video_repo: Arc<VR>,
}

impl<AI, TR, VR> SummarizeVideoUseCase<AI, TR, VR>
where
    AI: SummarizerAI,
    TR: TranscriptProvider,
    VR: VideoRepository,
{
    pub fn new(llm: Arc<AI>, transcript_provider: Arc<TR>, video_repo: Arc<VR>) -> Self {
        Self { llm, transcript_provider, video_repo }
    }

    /// Generates a summary for the video with caching.
    pub async fn execute(
        &self,
        input: SummarizeVideoInput,
    ) -> Result<SummarizeVideoOutput, SummarizeVideoError> {
        let video = self
            .video_repo
            .find_by_id(&input.video_id)
            .map_err(map_repo_err)?
            .ok_or(SummarizeVideoError::VideoNotFound)?;

        if !input.force_refresh {
            if let Some(summary) = video.summary() {
                let transcript = video.transcript().unwrap_or_default().to_string();
                return Ok(SummarizeVideoOutput {
                    summary: summary.to_string(),
                    transcript_used: transcript,
                    cached: true,
                });
            }
        }

        let transcript =
            if !input.force_refresh { video.transcript().map(|t| t.to_string()) } else { None };

        let transcript = match transcript {
            Some(t) if !t.trim().is_empty() => t,
            _ => {
                let fetched = self
                    .transcript_provider
                    .fetch_transcript(video.youtube_id().as_str())
                    .await
                    .map_err(map_transcript_err)?;

                self.video_repo
                    .update_transcript(&input.video_id, Some(&fetched))
                    .map_err(map_repo_err)?;

                fetched
            },
        };

        let summary = self
            .llm
            .summarize_transcript(&transcript, video.title())
            .await
            .map_err(|e| SummarizeVideoError::AI(e.to_string()))?;

        self.video_repo.update_summary(&input.video_id, Some(&summary)).map_err(map_repo_err)?;

        Ok(SummarizeVideoOutput { summary, transcript_used: transcript, cached: false })
    }
}

fn map_repo_err(err: RepositoryError) -> SummarizeVideoError {
    SummarizeVideoError::Repository(err.to_string())
}

fn map_transcript_err(err: TranscriptError) -> SummarizeVideoError {
    SummarizeVideoError::Transcript(err.to_string())
}
