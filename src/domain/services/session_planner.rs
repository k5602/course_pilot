//! Session Planner - Calculates daily study sessions based on cognitive limit.

use crate::domain::value_objects::{CognitiveLimit, SessionPlan};

/// Plans study sessions based on video durations and user's cognitive limit.
#[derive(Debug)]
pub struct SessionPlanner {
    cognitive_limit: CognitiveLimit,
}

impl SessionPlanner {
    /// Creates a new session planner with the given cognitive limit.
    pub fn new(cognitive_limit: CognitiveLimit) -> Self {
        Self { cognitive_limit }
    }

    /// Plans sessions for a list of video durations (in seconds).
    /// Respects module boundaries when provided.
    ///
    /// # Arguments
    /// * `durations` - Duration of each video in seconds
    /// * `module_boundaries` - Optional indices where module boundaries exist
    ///
    /// # Returns
    /// A list of session plans, each containing video indices for that day.
    pub fn plan_sessions(
        &self,
        durations: &[u32],
        module_boundaries: Option<&[usize]>,
    ) -> Vec<SessionPlan> {
        if durations.is_empty() {
            return vec![];
        }

        let limit_secs = self.cognitive_limit.seconds();
        let boundaries: Vec<usize> = module_boundaries.map(|b| b.to_vec()).unwrap_or_default();

        let mut sessions = Vec::new();
        let mut current_session_videos = Vec::new();
        let mut current_session_duration = 0u32;
        let mut day = 1u32;

        for (idx, &duration) in durations.iter().enumerate() {
            let is_boundary = boundaries.contains(&idx);

            // Check if adding this video would exceed the limit
            let would_exceed = current_session_duration + duration > limit_secs;

            // Start new session if:
            // 1. We would exceed the limit AND we have at least one video
            // 2. We're at a module boundary AND we have videos AND the current session is substantial
            let should_split = (would_exceed && !current_session_videos.is_empty())
                || (is_boundary
                    && !current_session_videos.is_empty()
                    && current_session_duration >= limit_secs / 2);

            if should_split {
                sessions.push(SessionPlan::new(
                    day,
                    current_session_videos.clone(),
                    current_session_duration,
                ));
                day += 1;
                current_session_videos.clear();
                current_session_duration = 0;
            }

            current_session_videos.push(idx);
            current_session_duration += duration;
        }

        // Don't forget the last session
        if !current_session_videos.is_empty() {
            sessions.push(SessionPlan::new(day, current_session_videos, current_session_duration));
        }

        sessions
    }

    /// Calculates the total number of days needed to complete the course.
    pub fn estimate_days(&self, durations: &[u32]) -> u32 {
        self.plan_sessions(durations, None).len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_session() {
        let planner = SessionPlanner::new(CognitiveLimit::new(60)); // 60 min
        let durations = vec![600, 600, 600]; // 3 videos, 10 min each = 30 min total
        let sessions = planner.plan_sessions(&durations, None);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].video_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_multiple_sessions() {
        let planner = SessionPlanner::new(CognitiveLimit::new(30)); // 30 min
        let durations = vec![900, 900, 900, 900]; // 4 videos, 15 min each
        let sessions = planner.plan_sessions(&durations, None);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].video_indices, vec![0, 1]); // 30 min
        assert_eq!(sessions[1].video_indices, vec![2, 3]); // 30 min
    }

    #[test]
    fn test_estimate_days() {
        let planner = SessionPlanner::new(CognitiveLimit::new(45)); // 45 min
        let durations = vec![900; 6]; // 6 videos, 15 min each = 90 min total
        let days = planner.estimate_days(&durations);

        assert_eq!(days, 2); // 45 min per day = 2 days
    }
}
