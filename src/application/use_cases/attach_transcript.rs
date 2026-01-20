//! Attach transcript use case.
//!
//! Orchestrates: Validate → Clean → Persist

use std::sync::Arc;

use crate::domain::{
    ports::{RepositoryError, VideoRepository},
    services::SubtitleCleaner,
    value_objects::VideoId,
};

/// Error type for transcript attachment.
#[derive(Debug, thiserror::Error)]
pub enum AttachTranscriptError {
    #[error("Video not found")]
    VideoNotFound,
    #[error("Transcript is empty after cleaning")]
    EmptyTranscript,
    #[error("Repository error: {0}")]
    Repository(String),
}

/// Input for attaching a transcript to a video.
#[derive(Debug, Clone)]
pub struct AttachTranscriptInput {
    pub video_id: VideoId,
    /// Raw subtitle or transcript text (SRT/VTT/plain).
    pub transcript_text: String,
}

/// Output after attaching a transcript.
#[derive(Debug, Clone)]
pub struct AttachTranscriptOutput {
    pub cleaned_length: usize,
}

/// Use case for attaching a transcript to a video.
pub struct AttachTranscriptUseCase<VR>
where
    VR: VideoRepository,
{
    video_repo: Arc<VR>,
    cleaner: SubtitleCleaner,
}

impl<VR> AttachTranscriptUseCase<VR>
where
    VR: VideoRepository,
{
    /// Creates a new use case instance.
    pub fn new(video_repo: Arc<VR>) -> Self {
        Self { video_repo, cleaner: SubtitleCleaner::new() }
    }

    /// Cleans and attaches the transcript to the video.
    pub fn execute(
        &self,
        input: AttachTranscriptInput,
    ) -> Result<AttachTranscriptOutput, AttachTranscriptError> {
        let video = self
            .video_repo
            .find_by_id(&input.video_id)
            .map_err(map_repo_err)?
            .ok_or(AttachTranscriptError::VideoNotFound)?;

        let cleaned = self.cleaner.clean(&input.transcript_text);
        if cleaned.trim().is_empty() {
            return Err(AttachTranscriptError::EmptyTranscript);
        }

        self.video_repo.update_transcript(video.id(), Some(&cleaned)).map_err(map_repo_err)?;

        Ok(AttachTranscriptOutput { cleaned_length: cleaned.len() })
    }
}

fn map_repo_err(err: RepositoryError) -> AttachTranscriptError {
    AttachTranscriptError::Repository(err.to_string())
}
