//! Repository ports for persistence.

use crate::domain::entities::{Course, Exam, Module, Video};
use crate::domain::value_objects::{CourseId, ExamId, ModuleId, VideoId};

/// Error type for repository operations.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Constraint violation: {0}")]
    Constraint(String),
}

/// Repository for Course entities.
pub trait CourseRepository: Send + Sync {
    fn save(&self, course: &Course) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &CourseId) -> Result<Option<Course>, RepositoryError>;
    fn find_all(&self) -> Result<Vec<Course>, RepositoryError>;
    fn delete(&self, id: &CourseId) -> Result<(), RepositoryError>;
}

/// Repository for Module entities.
pub trait ModuleRepository: Send + Sync {
    fn save(&self, module: &Module) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &ModuleId) -> Result<Option<Module>, RepositoryError>;
    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Module>, RepositoryError>;
    fn delete(&self, id: &ModuleId) -> Result<(), RepositoryError>;
}

/// Repository for Video entities.
pub trait VideoRepository: Send + Sync {
    fn save(&self, video: &Video) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &VideoId) -> Result<Option<Video>, RepositoryError>;
    fn find_by_module(&self, module_id: &ModuleId) -> Result<Vec<Video>, RepositoryError>;
    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Video>, RepositoryError>;
    fn update_completion(&self, id: &VideoId, completed: bool) -> Result<(), RepositoryError>;
    fn delete(&self, id: &VideoId) -> Result<(), RepositoryError>;
}

/// Repository for Exam entities.
pub trait ExamRepository: Send + Sync {
    fn save(&self, exam: &Exam) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &ExamId) -> Result<Option<Exam>, RepositoryError>;
    fn find_all(&self) -> Result<Vec<Exam>, RepositoryError>;
    fn find_by_video(&self, video_id: &VideoId) -> Result<Vec<Exam>, RepositoryError>;
    fn update_result(
        &self,
        id: &ExamId,
        score: f32,
        passed: bool,
        user_answers_json: Option<String>,
    ) -> Result<(), RepositoryError>;
}
