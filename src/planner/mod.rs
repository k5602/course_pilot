//! Planning module for Course Pilot
//!
//! This module provides functionality for generating intelligent study schedules
//! from structured course content.

pub mod clustering_integration;
pub mod difficulty_progression;
pub mod scheduler;

// Re-export main planning function
pub use scheduler::{
    choose_distribution_strategy, generate_plan, generate_spaced_repetition_plan, optimize_plan,
    pack_videos_into_session,
};

// Re-export clustering-aware planning functions
pub use clustering_integration::{
    choose_clustering_aware_strategy, generate_clustering_adaptive_plan,
    generate_clustering_difficulty_plan, generate_clustering_hybrid_plan,
    generate_duration_optimized_plan, generate_topic_aware_module_plan,
    generate_topic_spaced_repetition_plan, optimize_clustering_aware_plan,
};

// Re-export difficulty progression functions
pub use difficulty_progression::{
    AdaptivePacingConfig, DifficultyProgressionPlanner, PacingAdjustment,
    ProgressionValidationResult, SessionReorderingResult,
};

// Re-export error types
pub use crate::PlanError;

// Common planning utilities
use chrono::{DateTime, Datelike, Utc, Weekday};
use std::time::Duration;

/// Default planning configuration
pub struct PlanningDefaults;

impl PlanningDefaults {
    /// Default session length in minutes
    pub const DEFAULT_SESSION_LENGTH: u32 = 60;

    /// Default sessions per week
    pub const DEFAULT_SESSIONS_PER_WEEK: u8 = 3;

    /// Minimum session length in minutes
    pub const MIN_SESSION_LENGTH: u32 = 15;

    /// Maximum session length in minutes
    pub const MAX_SESSION_LENGTH: u32 = 180;

    /// Maximum sessions per week
    pub const MAX_SESSIONS_PER_WEEK: u8 = 14;

    /// Buffer time between videos in minutes
    pub const BUFFER_TIME_MINUTES: u32 = 5;
}

/// Calculate the number of weeks needed for a course
pub fn calculate_course_duration_weeks(total_sessions: usize, sessions_per_week: u8) -> usize {
    if sessions_per_week == 0 {
        return 0;
    }

    total_sessions.div_ceil(sessions_per_week as usize)
}

/// Get next session date based on current date and schedule
pub fn get_next_session_date(
    current_date: DateTime<Utc>,
    sessions_per_week: u8,
    include_weekends: bool,
) -> DateTime<Utc> {
    let _weekday = current_date.weekday();

    // Define available days based on weekend preference
    let available_days: Vec<Weekday> = if include_weekends {
        vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ]
    } else {
        vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ]
    };

    // Calculate days between sessions
    let days_between = if sessions_per_week >= available_days.len() as u8 {
        1 // Daily sessions
    } else {
        available_days.len() / sessions_per_week as usize
    };

    // Find next available day
    let mut next_date = current_date;
    for _ in 0..7 {
        next_date += chrono::Duration::days(days_between as i64);
        if available_days.contains(&next_date.weekday()) {
            break;
        }
    }

    next_date
}

/// Validate planning settings
pub fn validate_plan_settings(
    sessions_per_week: u8,
    session_length_minutes: u32,
    start_date: DateTime<Utc>,
) -> Result<(), String> {
    if sessions_per_week == 0 {
        return Err("Sessions per week must be greater than 0".to_string());
    }

    if sessions_per_week > PlanningDefaults::MAX_SESSIONS_PER_WEEK {
        return Err(format!(
            "Sessions per week cannot exceed {}",
            PlanningDefaults::MAX_SESSIONS_PER_WEEK
        ));
    }

    if session_length_minutes < PlanningDefaults::MIN_SESSION_LENGTH {
        return Err(format!(
            "Session length must be at least {} minutes",
            PlanningDefaults::MIN_SESSION_LENGTH
        ));
    }

    if session_length_minutes > PlanningDefaults::MAX_SESSION_LENGTH {
        return Err(format!(
            "Session length cannot exceed {} minutes",
            PlanningDefaults::MAX_SESSION_LENGTH
        ));
    }

    if start_date < Utc::now() - chrono::Duration::days(1) {
        return Err("Start date cannot be more than 1 day in the past".to_string());
    }

    Ok(())
}

/// Calculate estimated total study time for a course
pub fn calculate_total_study_time(
    video_count: usize,
    average_video_duration: Duration,
) -> Duration {
    let total_video_time = average_video_duration * video_count as u32;
    let buffer_time = Duration::from_secs(
        (PlanningDefaults::BUFFER_TIME_MINUTES * video_count as u32 * 60) as u64,
    );

    total_video_time + buffer_time
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_duration_calculation() {
        assert_eq!(calculate_course_duration_weeks(10, 3), 4);
        assert_eq!(calculate_course_duration_weeks(9, 3), 3);
        assert_eq!(calculate_course_duration_weeks(0, 3), 0);
    }

    #[test]
    fn test_plan_settings_validation() {
        let start_date = Utc::now() + chrono::Duration::days(1);

        // Valid settings
        assert!(validate_plan_settings(3, 60, start_date).is_ok());

        // Invalid sessions per week
        assert!(validate_plan_settings(0, 60, start_date).is_err());
        assert!(validate_plan_settings(20, 60, start_date).is_err());

        // Invalid session length
        assert!(validate_plan_settings(3, 5, start_date).is_err());
        assert!(validate_plan_settings(3, 300, start_date).is_err());

        // Invalid start date
        let past_date = Utc::now() - chrono::Duration::days(2);
        assert!(validate_plan_settings(3, 60, past_date).is_err());
    }

    #[test]
    fn test_total_study_time_calculation() {
        let video_duration = Duration::from_secs(600); // 10 minutes
        let total_time = calculate_total_study_time(5, video_duration);

        // 5 videos * 10 minutes + 5 videos * 5 minutes buffer = 75 minutes
        assert_eq!(total_time.as_secs(), 75 * 60);
    }
}
