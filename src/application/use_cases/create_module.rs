//! Create Module Use Case
//!
//! Creates a new module in a course.

use std::sync::Arc;

use crate::domain::{
    entities::Module,
    ports::{ModuleRepository, RepositoryError},
    value_objects::{CourseId, ModuleId},
};

/// Error type for module creation.
#[derive(Debug, thiserror::Error)]
pub enum CreateModuleError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Input for creating a new module.
pub struct CreateModuleInput {
    pub course_id: CourseId,
    pub title: String,
    pub sort_order: u32,
}

/// Use case for creating a new module.
pub struct CreateModuleUseCase<MR>
where
    MR: ModuleRepository,
{
    module_repo: Arc<MR>,
}

impl<MR> CreateModuleUseCase<MR>
where
    MR: ModuleRepository,
{
    pub fn new(module_repo: Arc<MR>) -> Self {
        Self { module_repo }
    }

    pub fn execute(&self, input: CreateModuleInput) -> Result<ModuleId, CreateModuleError> {
        let id = ModuleId::new();
        let module = Module::new(id, input.course_id, input.title, input.sort_order);
        self.module_repo.save(&module)?;
        Ok(id)
    }
}
