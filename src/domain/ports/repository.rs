//! Repository ports for persistence.

use crate::domain::entities::{Course, Exam, Module, Note, Tag, Video};
use crate::domain::value_objects::{CourseId, ExamId, ModuleId, TagId, VideoId};

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

/// Repository for Note entities.
pub trait NoteRepository: Send + Sync {
    fn save(&self, note: &Note) -> Result<(), RepositoryError>;
    fn find_by_video(&self, video_id: &VideoId) -> Result<Option<Note>, RepositoryError>;
    fn delete(&self, video_id: &VideoId) -> Result<(), RepositoryError>;
}

/// Repository for Tag entities (course categorization).
pub trait TagRepository: Send + Sync {
    /// Saves a new tag or updates an existing one.
    fn save(&self, tag: &Tag) -> Result<(), RepositoryError>;

    /// Finds all tags.
    fn find_all(&self) -> Result<Vec<Tag>, RepositoryError>;

    /// Finds tags associated with a course.
    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Tag>, RepositoryError>;

    /// Associates a tag with a course.
    fn add_to_course(&self, course_id: &CourseId, tag_id: &TagId) -> Result<(), RepositoryError>;

    /// Removes a tag association from a course.
    fn remove_from_course(
        &self,
        course_id: &CourseId,
        tag_id: &TagId,
    ) -> Result<(), RepositoryError>;

    /// Deletes a tag (and all its course associations).
    fn delete(&self, tag_id: &TagId) -> Result<(), RepositoryError>;
}

/// Repository for full-text search.
pub trait SearchRepository: Send + Sync {
    /// Searches across courses, videos, and notes.
    /// Returns results ordered by relevance.
    fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::domain::entities::SearchResult>, RepositoryError>;

    /// Indexes a course for search.
    fn index_course(
        &self,
        course_id: &CourseId,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), RepositoryError>;

    /// Indexes a video for search.
    fn index_video(
        &self,
        video_id: &str,
        title: &str,
        description: Option<&str>,
        course_id: &CourseId,
    ) -> Result<(), RepositoryError>;

    /// Indexes a note for search.
    fn index_note(
        &self,
        note_id: &str,
        video_title: &str,
        content: &str,
        course_id: &CourseId,
    ) -> Result<(), RepositoryError>;

    /// Removes an entity from the search index.
    fn remove_from_index(&self, entity_id: &str) -> Result<(), RepositoryError>;
}
