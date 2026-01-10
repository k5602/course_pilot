//! Persistence - SQLite adapter using Diesel.

// TODO: Implement Diesel schema and repository adapters
// This will require:
// 1. diesel setup
// 2. diesel migration generate create_courses
// 3. Implement repository traits

use crate::domain::{
    entities::{Course, Exam, Module, Video},
    ports::{CourseRepository, ExamRepository, ModuleRepository, RepositoryError, VideoRepository},
    value_objects::{CourseId, ExamId, ModuleId, VideoId},
};

/// SQLite-backed course repository.
pub struct SqliteCourseRepository {
    // TODO: Add diesel connection pool
}

impl SqliteCourseRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SqliteCourseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl CourseRepository for SqliteCourseRepository {
    fn save(&self, _course: &Course) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_id(&self, _id: &CourseId) -> Result<Option<Course>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_all(&self) -> Result<Vec<Course>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn delete(&self, _id: &CourseId) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }
}

/// SQLite-backed module repository.
pub struct SqliteModuleRepository;

impl ModuleRepository for SqliteModuleRepository {
    fn save(&self, _module: &Module) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_id(&self, _id: &ModuleId) -> Result<Option<Module>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_course(&self, _course_id: &CourseId) -> Result<Vec<Module>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn delete(&self, _id: &ModuleId) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }
}

/// SQLite-backed video repository.
pub struct SqliteVideoRepository;

impl VideoRepository for SqliteVideoRepository {
    fn save(&self, _video: &Video) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_id(&self, _id: &VideoId) -> Result<Option<Video>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_module(&self, _module_id: &ModuleId) -> Result<Vec<Video>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_course(&self, _course_id: &CourseId) -> Result<Vec<Video>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn update_completion(&self, _id: &VideoId, _completed: bool) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn delete(&self, _id: &VideoId) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }
}

/// SQLite-backed exam repository.
pub struct SqliteExamRepository;

impl ExamRepository for SqliteExamRepository {
    fn save(&self, _exam: &Exam) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_id(&self, _id: &ExamId) -> Result<Option<Exam>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn find_by_video(&self, _video_id: &VideoId) -> Result<Vec<Exam>, RepositoryError> {
        todo!("Implement with Diesel")
    }

    fn update_result(
        &self,
        _id: &ExamId,
        _score: f32,
        _passed: bool,
    ) -> Result<(), RepositoryError> {
        todo!("Implement with Diesel")
    }
}
