//! Reorder Video Use Case
//!
//! Updates a video's sort order within a module.

use std::sync::Arc;

use crate::domain::{
    ports::{RepositoryError, VideoRepository},
    value_objects::{ModuleId, VideoId},
};

/// Error type for video reordering.
#[derive(Debug, thiserror::Error)]
pub enum ReorderError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Input for reordering a video.
pub struct ReorderVideoInput {
    pub video_id: VideoId,
    pub module_id: ModuleId,
    pub new_sort_order: u32,
}

/// Use case for changing a video's sort order.
pub struct ReorderVideoUseCase<VR>
where
    VR: VideoRepository,
{
    video_repo: Arc<VR>,
}

impl<VR> ReorderVideoUseCase<VR>
where
    VR: VideoRepository,
{
    pub fn new(video_repo: Arc<VR>) -> Self {
        Self { video_repo }
    }

    pub fn execute(&self, input: ReorderVideoInput) -> Result<(), ReorderError> {
        self.video_repo.update_module(&input.video_id, &input.module_id, input.new_sort_order)?;
        Ok(())
    }
}
