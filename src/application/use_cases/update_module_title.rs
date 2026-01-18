//! Update Module Title Use Case
//!
//! Renames an existing module and persists the change.

use std::sync::Arc;

use crate::domain::{
    ports::{ModuleRepository, RepositoryError},
    value_objects::ModuleId,
};

/// Error type for module title updates.
#[derive(Debug, thiserror::Error)]
pub enum UpdateModuleTitleError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Failed to update module: {0}")]
    PersistFailed(String),
}

/// Input for updating a module title.
pub struct UpdateModuleTitleInput {
    pub module_id: ModuleId,
    pub title: String,
}

/// Use case for renaming a module.
pub struct UpdateModuleTitleUseCase<MR>
where
    MR: ModuleRepository,
{
    module_repo: Arc<MR>,
}

impl<MR> UpdateModuleTitleUseCase<MR>
where
    MR: ModuleRepository,
{
    /// Creates a new use case with the given repository.
    pub fn new(module_repo: Arc<MR>) -> Self {
        Self { module_repo }
    }

    /// Executes the module title update.
    pub fn execute(&self, input: UpdateModuleTitleInput) -> Result<(), UpdateModuleTitleError> {
        let trimmed = input.title.trim();
        if trimmed.is_empty() {
            return Err(UpdateModuleTitleError::InvalidInput(
                "Module title cannot be empty.".to_string(),
            ));
        }

        self.module_repo
            .update_title(&input.module_id, trimmed)
            .map_err(|e| UpdateModuleTitleError::PersistFailed(format!("{e}")))?;

        Ok(())
    }
}

impl From<RepositoryError> for UpdateModuleTitleError {
    fn from(err: RepositoryError) -> Self {
        UpdateModuleTitleError::PersistFailed(err.to_string())
    }
}
