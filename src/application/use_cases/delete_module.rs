//! Delete Module Use Case
//!
//! Deletes a module only if it has no videos.

use std::sync::Arc;

use crate::domain::{
    ports::{ModuleRepository, RepositoryError, VideoRepository},
    value_objects::ModuleId,
};

/// Error type for module deletion.
#[derive(Debug, thiserror::Error)]
pub enum DeleteModuleError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("Module has videos: remove or move them first")]
    HasVideos,
}

/// Input for deleting a module.
pub struct DeleteModuleInput {
    pub module_id: ModuleId,
    /// When true, skip the has-videos safety check and delete even if videos exist.
    pub force: bool,
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
        if !input.force {
            let videos = self.video_repo.find_by_module(&input.module_id)?;
            if !videos.is_empty() {
                return Err(DeleteModuleError::HasVideos);
            }
        }
        self.module_repo.delete(&input.module_id)?;
        Ok(())
    }
}
