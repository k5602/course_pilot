//! Note CRUD use case with tag and search integrations.
//!
//! Orchestrates NoteRepository with video/course context, tags, and search indexing.

use std::sync::Arc;

use crate::domain::{
    entities::{Note, Tag},
    ports::{
        CourseRepository, DomainEvent, EventBus, ModuleRepository, NoteRepository, RepositoryError,
        SearchRepository, TagRepository, VideoRepository,
    },
    value_objects::{CourseId, VideoId},
};

/// Error type for note operations.
#[derive(Debug, thiserror::Error)]
pub enum NotesError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Input for saving (create/update) a note.
pub struct SaveNoteInput {
    pub video_id: VideoId,
    pub content: String,
}

/// Output for note operations, enriched with course tags.
#[derive(Debug, Clone)]
pub struct NoteView {
    pub note_id: String,
    pub video_id: VideoId,
    pub content: String,
    pub course_id: CourseId,
    pub course_tags: Vec<Tag>,
}

/// Input for loading a note by video.
pub struct LoadNoteInput {
    pub video_id: VideoId,
}

/// Input for deleting a note by video.
pub struct DeleteNoteInput {
    pub video_id: VideoId,
}

/// Use case for note CRUD with tag and search integrations.
pub struct NotesUseCase {
    note_repo: Arc<dyn NoteRepository>,
    video_repo: Arc<dyn VideoRepository>,
    module_repo: Arc<dyn ModuleRepository>,
    course_repo: Arc<dyn CourseRepository>,
    tag_repo: Arc<dyn TagRepository>,
    search_repo: Arc<dyn SearchRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl NotesUseCase {
    pub fn new(
        note_repo: Arc<dyn NoteRepository>,
        video_repo: Arc<dyn VideoRepository>,
        module_repo: Arc<dyn ModuleRepository>,
        course_repo: Arc<dyn CourseRepository>,
        tag_repo: Arc<dyn TagRepository>,
        search_repo: Arc<dyn SearchRepository>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self { note_repo, video_repo, module_repo, course_repo, tag_repo, search_repo, event_bus }
    }

    /// Saves a note (create/update) and updates the search index.
    pub fn save_note(&self, input: SaveNoteInput) -> Result<NoteView, NotesError> {
        let (video_title, course_id, course_tags) = self.load_context(&input.video_id)?;

        let mut note = self
            .note_repo
            .find_by_video(&input.video_id)?
            .unwrap_or_else(|| Note::empty_for_video(input.video_id));

        note.update_content(input.content.clone());

        self.note_repo.save(&note)?;

        // Index note for search.
        self.search_repo.index_note(
            &note.id().as_uuid().to_string(),
            &video_title,
            note.content(),
            &course_id,
        )?;

        self.event_bus.publish(DomainEvent::NotesUpdated(input.video_id));

        Ok(NoteView {
            note_id: note.id().as_uuid().to_string(),
            video_id: input.video_id,
            content: note.content().to_string(),
            course_id,
            course_tags,
        })
    }

    /// Loads a note by video ID, along with course tags.
    pub fn load_note(&self, input: LoadNoteInput) -> Result<Option<NoteView>, NotesError> {
        let (video_title, course_id, course_tags) = self.load_context(&input.video_id)?;

        let note = self.note_repo.find_by_video(&input.video_id)?;

        let note = match note {
            Some(note) => note,
            None => return Ok(None),
        };

        // Ensure search index stays up-to-date.
        self.search_repo.index_note(
            &note.id().as_uuid().to_string(),
            &video_title,
            note.content(),
            &course_id,
        )?;

        Ok(Some(NoteView {
            note_id: note.id().as_uuid().to_string(),
            video_id: input.video_id,
            content: note.content().to_string(),
            course_id,
            course_tags,
        }))
    }

    /// Deletes a note by video ID and removes it from the search index.
    pub fn delete_note(&self, input: DeleteNoteInput) -> Result<(), NotesError> {
        let note = self.note_repo.find_by_video(&input.video_id)?;

        if let Some(note) = note {
            self.note_repo.delete(&input.video_id)?;

            self.search_repo.remove_from_index(&note.id().as_uuid().to_string())?;

            self.event_bus.publish(DomainEvent::NotesUpdated(input.video_id));
        }

        Ok(())
    }

    fn load_context(&self, video_id: &VideoId) -> Result<(String, CourseId, Vec<Tag>), NotesError> {
        let video = self.video_repo.find_by_id(video_id)?.ok_or_else(|| {
            RepositoryError::NotFound { entity: "Video", id: video_id.to_string() }
        })?;

        let module = self.module_repo.find_by_id(video.module_id())?.ok_or_else(|| {
            RepositoryError::NotFound { entity: "Module", id: video.module_id().to_string() }
        })?;

        let course = self.course_repo.find_by_id(module.course_id())?.ok_or_else(|| {
            RepositoryError::NotFound { entity: "Course", id: module.course_id().to_string() }
        })?;

        let tags = self.tag_repo.find_by_course(course.id())?;

        Ok((video.title().to_string(), *course.id(), tags))
    }
}
