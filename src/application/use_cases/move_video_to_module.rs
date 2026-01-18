//! Move Video To Module Use Case
//!
//! Reassigns a video to a different module and updates its sort order.

use std::sync::Arc;

use crate::domain::{
    ports::{RepositoryError, VideoRepository},
    value_objects::{ModuleId, VideoId},
};

/// Error type for moving a video between modules.
#[derive(Debug, thiserror::Error)]
pub enum MoveVideoError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Failed to move video: {0}")]
    PersistFailed(String),
}

/// Input for moving a video.
pub struct MoveVideoInput {
    pub video_id: VideoId,
    pub target_module_id: ModuleId,
    /// Optional explicit sort order. Use 0 to append to the end.
    pub sort_order: u32,
}

/// Use case for moving a video to a different module.
pub struct MoveVideoToModuleUseCase<VR>
where
    VR: VideoRepository,
{
    video_repo: Arc<VR>,
}

impl<VR> MoveVideoToModuleUseCase<VR>
where
    VR: VideoRepository,
{
    /// Creates a new use case with the given repository.
    pub fn new(video_repo: Arc<VR>) -> Self {
        Self { video_repo }
    }

    /// Executes the move operation.
    pub fn execute(&self, input: MoveVideoInput) -> Result<(), MoveVideoError> {
        let sort_order = if input.sort_order == 0 {
            let existing = self
                .video_repo
                .find_by_module(&input.target_module_id)
                .map_err(|e| MoveVideoError::PersistFailed(format!("{e}")))?;
            existing.iter().map(|video| video.sort_order()).max().unwrap_or(0).saturating_add(1)
        } else {
            input.sort_order
        };

        self.video_repo
            .update_module(&input.video_id, &input.target_module_id, sort_order)
            .map_err(|e| MoveVideoError::PersistFailed(format!("{e}")))?;

        Ok(())
    }
}

impl From<RepositoryError> for MoveVideoError {
    fn from(err: RepositoryError) -> Self {
        MoveVideoError::PersistFailed(err.to_string())
    }
}
