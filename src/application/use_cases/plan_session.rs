//! Plan Session Use Case
//!
//! Plans daily study sessions based on user's cognitive limit.

use std::sync::Arc;

use crate::domain::{
    ports::VideoRepository,
    services::SessionPlanner,
    value_objects::{CognitiveLimit, CourseId, SessionPlan},
};

/// Error type for session planning.
#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("Course not found")]
    CourseNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
}

/// Input for the plan session use case.
pub struct PlanSessionInput {
    pub course_id: CourseId,
    pub cognitive_limit_minutes: u32,
}

/// Use case for planning study sessions.
pub struct PlanSessionUseCase<VR>
where
    VR: VideoRepository,
{
    video_repo: Arc<VR>,
}

impl<VR> PlanSessionUseCase<VR>
where
    VR: VideoRepository,
{
    pub fn new(video_repo: Arc<VR>) -> Self {
        Self { video_repo }
    }

    /// Executes the session planning.
    pub fn execute(&self, input: PlanSessionInput) -> Result<Vec<SessionPlan>, PlanError> {
        // Get all videos for the course
        let videos = self
            .video_repo
            .find_by_course(&input.course_id)
            .map_err(|e| PlanError::Repository(e.to_string()))?;

        if videos.is_empty() {
            return Err(PlanError::CourseNotFound);
        }

        // Extract durations
        let durations: Vec<u32> = videos.iter().map(|v| v.duration_secs()).collect();

        // Create session planner and plan
        let cognitive_limit = CognitiveLimit::new(input.cognitive_limit_minutes);
        let planner = SessionPlanner::new(cognitive_limit);

        Ok(planner.plan_sessions(&durations, None))
    }
}
