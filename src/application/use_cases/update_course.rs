/*! Update Course Use Case
 *
 * Updates course metadata and keeps the search index in sync.
 */

use std::sync::Arc;

use crate::domain::{
    ports::{CourseRepository, SearchRepository},
    value_objects::CourseId,
};

/// Error type for course updates.
#[derive(Debug, thiserror::Error)]
pub enum UpdateCourseError {
    #[error("Course not found")]
    CourseNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
}

/// Input for updating course metadata.
#[derive(Debug, Clone)]
pub struct UpdateCourseInput {
    pub course_id: CourseId,
    pub name: String,
    pub description: Option<String>,
}

/// Output of course update.
#[derive(Debug, Clone)]
pub struct UpdateCourseOutput {
    pub course_id: CourseId,
}

/// Use case for updating course metadata and search index.
pub struct UpdateCourseUseCase<CR, SR>
where
    CR: CourseRepository,
    SR: SearchRepository,
{
    course_repo: Arc<CR>,
    search_repo: Arc<SR>,
}

impl<CR, SR> UpdateCourseUseCase<CR, SR>
where
    CR: CourseRepository,
    SR: SearchRepository,
{
    /// Creates a new use case with injected repositories.
    pub fn new(course_repo: Arc<CR>, search_repo: Arc<SR>) -> Self {
        Self { course_repo, search_repo }
    }

    /// Updates course name/description and re-indexes for search.
    pub fn execute(
        &self,
        input: UpdateCourseInput,
    ) -> Result<UpdateCourseOutput, UpdateCourseError> {
        let existing = self
            .course_repo
            .find_by_id(&input.course_id)
            .map_err(|e| UpdateCourseError::Repository(e.to_string()))?;

        let Some(course) = existing else {
            return Err(UpdateCourseError::CourseNotFound);
        };

        self.course_repo
            .update_metadata(&input.course_id, &input.name, input.description.as_deref())
            .map_err(|e| UpdateCourseError::Repository(e.to_string()))?;

        self.search_repo
            .index_course(course.id(), &input.name, input.description.as_deref())
            .map_err(|e| UpdateCourseError::Repository(e.to_string()))?;

        Ok(UpdateCourseOutput { course_id: input.course_id })
    }
}
