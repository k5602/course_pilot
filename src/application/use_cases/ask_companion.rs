//! Ask Companion Use Case
//!
//! Handles Q&A with the AI companion in video context.

use std::sync::Arc;

use crate::domain::{
    ports::{CompanionAI, CompanionContext, CourseRepository, ModuleRepository, VideoRepository},
    value_objects::VideoId,
};

/// Error type for companion queries.
#[derive(Debug, thiserror::Error)]
pub enum CompanionError {
    #[error("Video not found")]
    VideoNotFound,
    #[error("Module not found")]
    ModuleNotFound,
    #[error("Course not found")]
    CourseNotFound,
    #[error("AI error: {0}")]
    AI(String),
    #[error("Repository error: {0}")]
    Repository(String),
}

/// Input for the ask companion use case.
pub struct AskCompanionInput {
    pub video_id: VideoId,
    pub question: String,
}

/// Use case for asking questions to the AI companion.
pub struct AskCompanionUseCase<AI, VR, MR, CR>
where
    AI: CompanionAI,
    VR: VideoRepository,
    MR: ModuleRepository,
    CR: CourseRepository,
{
    companion: Arc<AI>,
    video_repo: Arc<VR>,
    module_repo: Arc<MR>,
    course_repo: Arc<CR>,
}

impl<AI, VR, MR, CR> AskCompanionUseCase<AI, VR, MR, CR>
where
    AI: CompanionAI,
    VR: VideoRepository,
    MR: ModuleRepository,
    CR: CourseRepository,
{
    pub fn new(
        companion: Arc<AI>,
        video_repo: Arc<VR>,
        module_repo: Arc<MR>,
        course_repo: Arc<CR>,
    ) -> Self {
        Self { companion, video_repo, module_repo, course_repo }
    }

    /// Executes the Q&A request.
    pub async fn execute(&self, input: AskCompanionInput) -> Result<String, CompanionError> {
        // Get video
        let video = self
            .video_repo
            .find_by_id(&input.video_id)
            .map_err(|e| CompanionError::Repository(e.to_string()))?
            .ok_or(CompanionError::VideoNotFound)?;

        // Get module
        let module = self
            .module_repo
            .find_by_id(video.module_id())
            .map_err(|e| CompanionError::Repository(e.to_string()))?
            .ok_or(CompanionError::ModuleNotFound)?;

        // Get course
        let course = self
            .course_repo
            .find_by_id(module.course_id())
            .map_err(|e| CompanionError::Repository(e.to_string()))?
            .ok_or(CompanionError::CourseNotFound)?;

        // Build context
        let context = CompanionContext {
            video_title: video.title().to_string(),
            video_description: video.description().map(|s| s.to_string()),
            module_title: module.title().to_string(),
            course_name: course.name().to_string(),
        };

        // Ask the AI
        self.companion
            .ask(&input.question, &context)
            .await
            .map_err(|e| CompanionError::AI(e.to_string()))
    }
}
