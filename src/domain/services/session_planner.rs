//! Session Planner - Calculates daily study sessions based on cognitive limit.

use std::collections::HashSet;

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

    /// Converts a 1-indexed session number to a 1-indexed calendar day,
    /// inserting weekend-style gaps when `week_study_days < 7`.
    fn session_to_calendar_day(session_num: u32, week_study_days: u32) -> u32 {
        let n = session_num - 1;
        let weeks = n / week_study_days;
        let day_in_week = n % week_study_days;
        weeks * 7 + day_in_week + 1
    }

    /// Plans sessions for a list of video durations (in seconds).
    /// Respects module boundaries when provided.
    ///
    /// # Arguments
    /// * `durations` - Duration of each video in seconds
    /// * `module_boundaries` - Optional indices where module boundaries exist
    /// * `week_study_days` - How many days per week are study days (e.g. 5 = weekdays only, 7 = every day)
    ///
    /// # Returns
    /// A list of session plans, each containing video indices for that day.
    pub fn plan_sessions(
        &self,
        durations: &[u32],
        module_boundaries: Option<&[usize]>,
        week_study_days: u32,
    ) -> Vec<SessionPlan> {
        if durations.is_empty() {
            return vec![];
        }

        let week_study_days = week_study_days.max(1);
        let limit_secs = self.cognitive_limit.seconds();
        let boundaries: HashSet<usize> =
            module_boundaries.map(|b| b.iter().copied().collect()).unwrap_or_default();

        let mut sessions = Vec::new();
        let mut current_session_videos = Vec::new();
        let mut current_session_duration = 0u32;
        let mut session_num = 0u32;

        for (idx, &duration) in durations.iter().enumerate() {
            let is_boundary = boundaries.contains(&idx);

            let would_exceed = current_session_duration + duration > limit_secs;

            let should_split = (would_exceed && !current_session_videos.is_empty())
                || (is_boundary
                    && !current_session_videos.is_empty()
                    && current_session_duration >= limit_secs / 2);

            if should_split {
                session_num += 1;
                let videos = std::mem::take(&mut current_session_videos);
                sessions.push(SessionPlan::new(
                    Self::session_to_calendar_day(session_num, week_study_days),
                    videos,
                    current_session_duration,
                ));
                current_session_duration = 0;
            }

            current_session_videos.push(idx);
            current_session_duration += duration;
        }

        if !current_session_videos.is_empty() {
            session_num += 1;
            sessions.push(SessionPlan::new(
                Self::session_to_calendar_day(session_num, week_study_days),
                current_session_videos,
                current_session_duration,
            ));
        }

        sessions
    }

    /// Calculates the total number of calendar days needed to complete the course.
    pub fn estimate_days(&self, durations: &[u32], week_study_days: u32) -> u32 {
        let sessions = self.plan_sessions(durations, None, week_study_days);
        sessions.last().map(|s| s.day).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_session() {
        let planner = SessionPlanner::new(CognitiveLimit::new(60));
        let durations = vec![600, 600, 600];
        let sessions = planner.plan_sessions(&durations, None, 7);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].video_indices, vec![0, 1, 2]);
        assert_eq!(sessions[0].day, 1);
    }

    #[test]
    fn test_multiple_sessions() {
        let planner = SessionPlanner::new(CognitiveLimit::new(30));
        let durations = vec![900, 900, 900, 900];
        let sessions = planner.plan_sessions(&durations, None, 7);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].video_indices, vec![0, 1]);
        assert_eq!(sessions[1].video_indices, vec![2, 3]);
        assert_eq!(sessions[0].day, 1);
        assert_eq!(sessions[1].day, 2);
    }

    #[test]
    fn test_estimate_days() {
        let planner = SessionPlanner::new(CognitiveLimit::new(45));
        let durations = vec![900; 6];
        let days = planner.estimate_days(&durations, 7);
        assert_eq!(days, 2);
    }

    #[test]
    fn test_week_study_days_weekday_schedule() {
        let planner = SessionPlanner::new(CognitiveLimit::new(30));
        let durations = vec![900; 14];
        let sessions = planner.plan_sessions(&durations, None, 5);

        assert_eq!(sessions.len(), 7);
        assert_eq!(sessions[0].day, 1);
        assert_eq!(sessions[1].day, 2);
        assert_eq!(sessions[2].day, 3);
        assert_eq!(sessions[3].day, 4);
        assert_eq!(sessions[4].day, 5);
        assert_eq!(sessions[5].day, 8);
        assert_eq!(sessions[6].day, 9);
    }

    #[test]
    fn test_week_study_days_midweek_gap() {
        let planner = SessionPlanner::new(CognitiveLimit::new(30));
        let durations = vec![900; 6];
        let sessions = planner.plan_sessions(&durations, None, 2);

        assert_eq!(sessions.len(), 3);
        assert_eq!(sessions[0].day, 1);
        assert_eq!(sessions[1].day, 2);
        assert_eq!(sessions[2].day, 8);
    }

    #[test]
    fn test_empty_durations() {
        let planner = SessionPlanner::new(CognitiveLimit::new(30));
        let sessions = planner.plan_sessions(&[], None, 5);
        assert!(sessions.is_empty());
    }

    #[test]
    fn respects_week_study_days() {
        let planner = SessionPlanner::new(CognitiveLimit::new(60));
        let durations = vec![600; 10];
        let sessions = planner.plan_sessions(&durations, None, 5);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].day, 1);
        assert_eq!(sessions[1].day, 2);
    }

    #[test]
    fn week_boundary_crossing() {
        let planner = SessionPlanner::new(CognitiveLimit::new(10));
        let durations = vec![600; 12];
        let sessions = planner.plan_sessions(&durations, None, 5);

        if sessions.len() >= 6 {
            assert!(sessions[5].day >= 7);
        }
    }

    #[test]
    fn test_respects_module_boundaries_under_half_limit() {
        let planner = SessionPlanner::new(CognitiveLimit::new(60)); // limit_secs = 3600, half = 1800
        let durations = vec![1000, 1000, 1000];
        let boundaries = vec![1];
        let sessions = planner.plan_sessions(&durations, Some(&boundaries), 7);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].video_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_respects_module_boundaries_over_half_limit() {
        let planner = SessionPlanner::new(CognitiveLimit::new(60)); // limit_secs = 3600, half = 1800
        let durations = vec![2000, 1000, 1000];
        let boundaries = vec![1];
        let sessions = planner.plan_sessions(&durations, Some(&boundaries), 7);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].video_indices, vec![0]);
        assert_eq!(sessions[1].video_indices, vec![1, 2]);
    }

    #[test]
    fn test_single_huge_video_exceeding_limit() {
        let planner = SessionPlanner::new(CognitiveLimit::new(60)); // limit_secs = 3600
        let durations = vec![5000, 1000];
        let sessions = planner.plan_sessions(&durations, None, 7);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].video_indices, vec![0]);
        assert_eq!(sessions[0].total_duration_secs, 5000);
        assert_eq!(sessions[1].video_indices, vec![1]);
        assert_eq!(sessions[1].total_duration_secs, 1000);
    }

    #[test]
    fn test_weekly_study_days_extreme_values() {
        let planner = SessionPlanner::new(CognitiveLimit::new(30)); // 1800s limit
        let durations = vec![1000, 1000, 1000, 1000];

        let sessions_1 = planner.plan_sessions(&durations, None, 1);
        assert_eq!(sessions_1.len(), 4);
        assert_eq!(sessions_1[0].day, 1);
        assert_eq!(sessions_1[1].day, 8);
        assert_eq!(sessions_1[2].day, 15);
        assert_eq!(sessions_1[3].day, 22);

        let sessions_0 = planner.plan_sessions(&durations, None, 0);
        assert_eq!(sessions_0.len(), 4);
        assert_eq!(sessions_0[0].day, 1);
        assert_eq!(sessions_0[1].day, 8);
        assert_eq!(sessions_0[2].day, 15);
        assert_eq!(sessions_0[3].day, 22);
    }
}
