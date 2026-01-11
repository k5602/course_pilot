//! Course entity - The aggregate root for a learning course.

use crate::domain::value_objects::{CourseId, PlaylistUrl};

/// A course represents a structured learning path derived from a YouTube playlist.
#[derive(Debug, Clone, PartialEq)]
pub struct Course {
    id: CourseId,
    name: String,
    source_url: PlaylistUrl,
    playlist_id: String,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Course {
    /// Creates a new course.
    pub fn new(
        id: CourseId,
        name: String,
        source_url: PlaylistUrl,
        playlist_id: String,
        description: Option<String>,
    ) -> Self {
        Self { id, name, source_url, playlist_id, description, created_at: chrono::Utc::now() }
    }

    pub fn id(&self) -> &CourseId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_url(&self) -> &PlaylistUrl {
        &self.source_url
    }

    pub fn playlist_id(&self) -> &str {
        &self.playlist_id
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
}
