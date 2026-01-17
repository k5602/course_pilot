//! Search result entity and types.

use crate::domain::value_objects::CourseId;

/// Type of entity returned in search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchResultType {
    Course,
    Video,
    Note,
}

impl std::fmt::Display for SearchResultType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Course => write!(f, "Course"),
            Self::Video => write!(f, "Video"),
            Self::Note => write!(f, "Note"),
        }
    }
}

/// A search result from the full-text search index.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Type of entity (course, video, note)
    pub entity_type: SearchResultType,
    /// UUID of the entity
    pub entity_id: String,
    /// Title or name of the entity
    pub title: String,
    /// Snippet of matching content
    pub snippet: String,
    /// Course ID for navigation
    pub course_id: CourseId,
}

impl SearchResult {
    pub fn new(
        entity_type: SearchResultType,
        entity_id: String,
        title: String,
        snippet: String,
        course_id: CourseId,
    ) -> Self {
        Self { entity_type, entity_id, title, snippet, course_id }
    }
}
