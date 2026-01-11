//! Note entity - User notes for a video.

use crate::domain::value_objects::VideoId;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A note associated with a video.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    id: NoteId,
    video_id: VideoId,
    content: String,
    updated_at: DateTime<Utc>,
}

/// Unique identifier for a Note.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteId(Uuid);

impl NoteId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for NoteId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for NoteId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

impl Note {
    /// Creates a new note.
    pub fn new(id: NoteId, video_id: VideoId, content: String) -> Self {
        Self { id, video_id, content, updated_at: Utc::now() }
    }

    /// Creates a new note for a video with empty content.
    pub fn empty_for_video(video_id: VideoId) -> Self {
        Self::new(NoteId::new(), video_id, String::new())
    }

    pub fn id(&self) -> &NoteId {
        &self.id
    }

    pub fn video_id(&self) -> &VideoId {
        &self.video_id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Updates the note content.
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}
