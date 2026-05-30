//! Ask Companion Use Case
//!
//! Handles Q&A with the AI companion in video context.

use std::sync::Arc;

use crate::domain::{
    ports::{
        CompanionAI, CompanionContext, CourseRepository, LLMError, ModuleRepository,
        NoteRepository, RepositoryError, VideoRepository,
    },
    value_objects::VideoId,
};

/// Error type for companion queries.
#[derive(Debug, thiserror::Error)]
pub enum CompanionError {
    #[error(transparent)]
    AI(#[from] LLMError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Input for the ask companion use case.
#[derive(Debug, Clone)]
pub struct AskCompanionInput {
    pub video_id: VideoId,
    pub question: String,
    pub local_context: Option<String>,
}

/// Use case for asking questions to the AI companion.
pub struct AskCompanionUseCase {
    companion: Arc<dyn CompanionAI>,
    video_repo: Arc<dyn VideoRepository>,
    module_repo: Arc<dyn ModuleRepository>,
    course_repo: Arc<dyn CourseRepository>,
    note_repo: Arc<dyn NoteRepository>,
}

impl AskCompanionUseCase {
    pub fn new(
        companion: Arc<dyn CompanionAI>,
        video_repo: Arc<dyn VideoRepository>,
        module_repo: Arc<dyn ModuleRepository>,
        course_repo: Arc<dyn CourseRepository>,
        note_repo: Arc<dyn NoteRepository>,
    ) -> Self {
        Self { companion, video_repo, module_repo, course_repo, note_repo }
    }

    /// Executes the Q&A request.
    pub async fn execute(&self, input: AskCompanionInput) -> Result<String, CompanionError> {
        // Get video
        let video = self.video_repo.find_by_id(&input.video_id)?.ok_or_else(|| {
            RepositoryError::NotFound { entity: "Video", id: input.video_id.to_string() }
        })?;

        // Get module
        let module = self.module_repo.find_by_id(video.module_id())?.ok_or_else(|| {
            RepositoryError::NotFound { entity: "Module", id: video.module_id().to_string() }
        })?;

        // Get course
        let course = self.course_repo.find_by_id(module.course_id())?.ok_or_else(|| {
            RepositoryError::NotFound { entity: "Course", id: module.course_id().to_string() }
        })?;

        let notes =
            self.note_repo.find_by_video(&input.video_id)?.map(|note| note.content().to_string());

        // Build context
        let context = CompanionContext {
            video_title: video.title().to_string(),
            video_description: video.description().map(|s| s.to_string()),
            module_title: module.title().to_string(),
            course_name: course.name().to_string(),
            summary: video.summary().map(|s| s.to_string()),
            notes,
            local_context: input.local_context.clone(),
        };

        // Ask the AI
        self.companion.ask(&input.question, &context).await.map_err(CompanionError::from)
    }
}
