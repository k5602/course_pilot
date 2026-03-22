//! Delete Module Use Case
//!
//! Deletes a module only if it has no videos.

use std::sync::Arc;

use crate::domain::{
    ports::{ModuleRepository, VideoRepository},
    value_objects::ModuleId,
};

/// Error type for module deletion.
#[derive(Debug, thiserror::Error)]
pub enum DeleteModuleError {
    #[error("Repository error: {0}")]
    Repository(String),
    #[error("Module has videos: remove or move them first")]
    HasVideos,
}

/// Input for deleting a module.
pub struct DeleteModuleInput {
    pub module_id: ModuleId,
}

/// Use case for deleting an empty module.
pub struct DeleteModuleUseCase<MR, VR>
where
    MR: ModuleRepository,
    VR: VideoRepository,
{
    module_repo: Arc<MR>,
    video_repo: Arc<VR>,
}

impl<MR, VR> DeleteModuleUseCase<MR, VR>
where
    MR: ModuleRepository,
    VR: VideoRepository,
{
    pub fn new(module_repo: Arc<MR>, video_repo: Arc<VR>) -> Self {
        Self { module_repo, video_repo }
    }

    pub fn execute(&self, input: DeleteModuleInput) -> Result<(), DeleteModuleError> {
        let videos = self
            .video_repo
            .find_by_module(&input.module_id)
            .map_err(|e| DeleteModuleError::Repository(e.to_string()))?;
        if !videos.is_empty() {
            return Err(DeleteModuleError::HasVideos);
        }
        self.module_repo
            .delete(&input.module_id)
            .map_err(|e| DeleteModuleError::Repository(e.to_string()))
    }
}
