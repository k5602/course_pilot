//! Note CRUD use case with tag and search integrations.
//!
//! Orchestrates NoteRepository with video/course context, tags, and search indexing.

use std::sync::Arc;

use crate::domain::{
    entities::{Note, Tag},
    ports::{
        CourseRepository, ModuleRepository, NoteRepository, SearchRepository, TagRepository,
        VideoRepository,
    },
    value_objects::{CourseId, VideoId},
};

/// Error type for note operations.
#[derive(Debug, thiserror::Error)]
pub enum NotesError {
    #[error("Video not found")]
    VideoNotFound,
    #[error("Module not found")]
    ModuleNotFound,
    #[error("Course not found")]
    CourseNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
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
pub struct NotesUseCase<NR, VR, MR, CR, TR, SR>
where
    NR: NoteRepository,
    VR: VideoRepository,
    MR: ModuleRepository,
    CR: CourseRepository,
    TR: TagRepository,
    SR: SearchRepository,
{
    note_repo: Arc<NR>,
    video_repo: Arc<VR>,
    module_repo: Arc<MR>,
    course_repo: Arc<CR>,
    tag_repo: Arc<TR>,
    search_repo: Arc<SR>,
}

impl<NR, VR, MR, CR, TR, SR> NotesUseCase<NR, VR, MR, CR, TR, SR>
where
    NR: NoteRepository,
    VR: VideoRepository,
    MR: ModuleRepository,
    CR: CourseRepository,
    TR: TagRepository,
    SR: SearchRepository,
{
    pub fn new(
        note_repo: Arc<NR>,
        video_repo: Arc<VR>,
        module_repo: Arc<MR>,
        course_repo: Arc<CR>,
        tag_repo: Arc<TR>,
        search_repo: Arc<SR>,
    ) -> Self {
        Self { note_repo, video_repo, module_repo, course_repo, tag_repo, search_repo }
    }

    /// Saves a note (create/update) and updates the search index.
    pub fn save_note(&self, input: SaveNoteInput) -> Result<NoteView, NotesError> {
        let (video_title, course_id, course_tags) = self.load_context(&input.video_id)?;

        let mut note = self
            .note_repo
            .find_by_video(&input.video_id)
            .map_err(|e| NotesError::Repository(e.to_string()))?
            .unwrap_or_else(|| Note::empty_for_video(input.video_id.clone()));

        note.update_content(input.content.clone());

        self.note_repo.save(&note).map_err(|e| NotesError::Repository(e.to_string()))?;

        // Index note for search.
        self.search_repo
            .index_note(&note.id().as_uuid().to_string(), &video_title, note.content(), &course_id)
            .map_err(|e| NotesError::Repository(e.to_string()))?;

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

        let note = self
            .note_repo
            .find_by_video(&input.video_id)
            .map_err(|e| NotesError::Repository(e.to_string()))?;

        let note = match note {
            Some(note) => note,
            None => return Ok(None),
        };

        // Ensure search index stays up-to-date.
        self.search_repo
            .index_note(&note.id().as_uuid().to_string(), &video_title, note.content(), &course_id)
            .map_err(|e| NotesError::Repository(e.to_string()))?;

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
        let note = self
            .note_repo
            .find_by_video(&input.video_id)
            .map_err(|e| NotesError::Repository(e.to_string()))?;

        if let Some(note) = note {
            self.note_repo
                .delete(&input.video_id)
                .map_err(|e| NotesError::Repository(e.to_string()))?;

            self.search_repo
                .remove_from_index(&note.id().as_uuid().to_string())
                .map_err(|e| NotesError::Repository(e.to_string()))?;
        }

        Ok(())
    }

    fn load_context(&self, video_id: &VideoId) -> Result<(String, CourseId, Vec<Tag>), NotesError> {
        let video = self
            .video_repo
            .find_by_id(video_id)
            .map_err(|e| NotesError::Repository(e.to_string()))?
            .ok_or(NotesError::VideoNotFound)?;

        let module = self
            .module_repo
            .find_by_id(video.module_id())
            .map_err(|e| NotesError::Repository(e.to_string()))?
            .ok_or(NotesError::ModuleNotFound)?;

        let course = self
            .course_repo
            .find_by_id(module.course_id())
            .map_err(|e| NotesError::Repository(e.to_string()))?
            .ok_or(NotesError::CourseNotFound)?;

        let tags = self
            .tag_repo
            .find_by_course(course.id())
            .map_err(|e| NotesError::Repository(e.to_string()))?;

        Ok((video.title().to_string(), course.id().clone(), tags))
    }
}
