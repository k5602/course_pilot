//! Study Plan Generation Algorithm
//!
//! This module implements intelligent scheduling algorithms with advanced features:
//! - Adaptive difficulty-based pacing
//! - Spaced repetition integration
//! - Learning curve optimization
//! - Cognitive load balancing
//! - Prerequisite dependency tracking

use crate::PlanError;
use crate::planner::sequential::{generate_sequential_plan, should_use_sequential_planning};
use crate::types::{Course, Plan, PlanSettings};

mod analytics;
mod calendar;
mod capacity;
mod optimization;
mod packing;
mod recommendations;
mod sequential;
mod strategies;
mod strategy;
#[cfg(test)]
mod tests;

pub struct PlanningDefaults;
impl PlanningDefaults {
    pub const BUFFER_TIME_MINUTES: u32 = 5;
}

// Study recommendations structure

// Enhanced calculation of videos per session using actual video durations

// Calculate session capacity based on actual video duration and settings

// Fallback calculation when course structure is not available

// Calculate optimal session frequency based on course characteristics

/// Validate plan settings. Returns Ok(()) when valid, or Err(String) with a message.
///
/// Rules:
/// - sessions_per_week: 1..=7
/// - session_length_minutes: >= 15
/// - start_date can be in the past (we'll adjust dates when generating), so no constraint here.
pub fn validate_plan_settings(
    sessions_per_week: u8,
    session_length_minutes: u32,
    _start_date: chrono::DateTime<chrono::Utc>,
) -> Result<(), String> {
    if sessions_per_week == 0 || sessions_per_week > 7 {
        return Err("sessions_per_week must be between 1 and 7".to_string());
    }
    if session_length_minutes < 15 {
        return Err("session_length_minutes must be at least 15".to_string());
    }
    Ok(())
}

/// Compute the next session date based on the current date and schedule policy.
/// Spacing heuristic:
/// - If include_weekends is true: advance by ceil(7 / sessions_per_week) days
/// - If weekends are excluded: advance by 1 day repeatedly until landing on a weekday,
///   ensuring at least 1 day passes between sessions.
pub fn get_next_session_date(
    current: chrono::DateTime<chrono::Utc>,
    sessions_per_week: u8,
    include_weekends: bool,
) -> chrono::DateTime<chrono::Utc> {
    use chrono::Datelike;
    use chrono::Duration as ChronoDuration;

    let mut next = if sessions_per_week > 0 {
        let step_days = (7.0f32 / sessions_per_week as f32).ceil() as i64;
        current + ChronoDuration::days(step_days.max(1))
    } else {
        current + ChronoDuration::days(1)
    };

    if !include_weekends {
        while matches!(next.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
            next += ChronoDuration::days(1);
        }
    }

    next
}

/// Calculate the number of calendar weeks needed to complete a number of sessions,
/// given sessions_per_week. Returns 0 when sessions_per_week is 0 or total_sessions is 0.
pub fn calculate_course_duration_weeks(total_sessions: usize, sessions_per_week: u8) -> usize {
    if total_sessions == 0 || sessions_per_week == 0 {
        return 0;
    }
    ((total_sessions as f32) / (sessions_per_week as f32)).ceil() as usize
}

/// Compute total expected study time:
/// - Sum of video durations = video_count * average_video_duration
/// - Plus a 5-minute buffer per video (notes, transitions)
pub fn calculate_total_study_time(
    video_count: usize,
    average_video_duration: std::time::Duration,
) -> std::time::Duration {
    let base = average_video_duration
        .checked_mul(video_count as u32)
        .unwrap_or_else(|| std::time::Duration::from_secs(0));
    // 5 minutes buffer per video
    let buffer_per_video = std::time::Duration::from_secs(5 * 60);
    let buffer_total = buffer_per_video
        .checked_mul(video_count as u32)
        .unwrap_or_else(|| std::time::Duration::from_secs(0));
    base + buffer_total
}

// Re-exports for external consumers (no name collisions with existing items)
pub use analytics::{
    LearningVelocityAnalysis, LoadDistribution, PlanAnalysis, TemporalDistribution,
    VelocityCategory, analyze_learning_velocity, analyze_plan_effectiveness,
};
pub use calendar::{generate_session_dates, total_study_time_estimate};
pub use optimization::optimize_plan;
pub use packing::pack_videos_into_session;
pub use recommendations::{
    DifficultyProgression, StudyRecommendations, generate_study_recommendations,
};
pub use strategy::choose_distribution_strategy;
// Import adaptive helpers (public in strategies::adaptive) for use within this module.

use self::strategies::{
    generate_adaptive_plan as strat_generate_adaptive_plan,
    generate_difficulty_based_plan as strat_generate_difficulty_based_plan,
    generate_hybrid_plan as strat_generate_hybrid_plan,
    generate_module_based_plan as strat_generate_module_based_plan,
    generate_time_based_plan as strat_generate_time_based_plan,
};

// Use submodule APIs locally

/// Generate a study plan for a course based on user settings
///
/// # Arguments
/// * `course` - The course to create a plan for
/// * `settings` - User preferences for the study schedule
///
/// # Returns
/// * `Ok(Plan)` - Generated study plan with scheduled sessions
/// * `Err(PlanError)` - Error if plan generation fails
pub fn generate_plan(course: &Course, settings: &PlanSettings) -> Result<Plan, PlanError> {
    // Step 1: Check if content should be processed sequentially
    if should_use_sequential_planning(course) {
        log::info!("Using sequential planning to preserve original video order");
        return generate_sequential_plan(course, settings);
    }

    // Step 2: Generate basic plan (clustering metadata ignored for now )
    // Planned algorithm improvements in Phase 6
    generate_basic_plan(course, settings)
}

/// Generate basic study plan (fallback when no clustering data available)
pub fn generate_basic_plan(course: &Course, settings: &PlanSettings) -> Result<Plan, PlanError> {
    // Validate input parameters
    crate::planner::calendar::validate_settings(settings)?;

    // Check if course has structure
    let _structure = course
        .structure
        .as_ref()
        .ok_or(PlanError::CourseNotStructured)?;

    // Create session distribution strategy
    let strategy = choose_distribution_strategy(course, settings)?;

    // Generate plan items based on strategy
    let plan_items = match strategy {
        DistributionStrategy::ModuleBased => strat_generate_module_based_plan(course, settings)?,
        DistributionStrategy::TimeBased => strat_generate_time_based_plan(course, settings)?,
        DistributionStrategy::Hybrid => strat_generate_hybrid_plan(course, settings)?,
        DistributionStrategy::DifficultyBased => {
            strat_generate_difficulty_based_plan(course, settings)?
        }
        DistributionStrategy::SpacedRepetition => {
            // Prefer the extracted strategy implementation
            self::strategies::generate_spaced_repetition_plan(course, settings)?
        }
        DistributionStrategy::Adaptive => strat_generate_adaptive_plan(course, settings)?,
    };

    // Create and return the plan
    let mut plan = Plan::new(course.id, settings.clone());
    plan.items = plan_items;

    // Apply advanced optimization features
    optimize_plan(&mut plan)?;

    Ok(plan)
}

// Import distribution strategy and difficulty level from types
use crate::types::DistributionStrategy;

/// Generate a plan from precomputed session groups (indices into the course videos).
/// This stable API allows upstream NLP grouping to drive planning directly.
/// Group-aware behavior:
/// - Videos are ordered by their group position (groups[0], then groups[1], ...).
/// - Existing packed sessions (from the base strategy) are reordered to respect group order.
/// - Session dates are recomputed according to settings to keep a valid schedule.
///
/// Notes:
/// - This preserves the base packing (durations, session composition) while enforcing group order.
/// - Future enhancement can split/pack strictly within each group using `pack_videos_into_session`.
pub fn generate_plan_from_groups(
    course: &crate::types::Course,
    groups: Vec<Vec<usize>>,
    settings: &crate::types::PlanSettings,
) -> std::result::Result<crate::types::Plan, crate::PlanError> {
    // Start from a baseline plan (respects session length, durations, etc.)
    let mut plan = generate_plan(course, settings)?;

    // Build a mapping of video index -> group order (lower = earlier)
    let mut group_order: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (g_idx, group) in groups.iter().enumerate() {
        for &vid in group {
            group_order.insert(vid, g_idx);
        }
    }

    // Reorder existing plan items by the minimum group index of their contained videos.
    let mut reordered_items = plan.items.clone();
    reordered_items.sort_by_key(|item| {
        let mut min_g = usize::MAX;
        for &vid in item.video_indices.iter() {
            if let Some(&g) = group_order.get(&vid) {
                if g < min_g {
                    min_g = g;
                }
            }
        }
        min_g
    });

    // Recompute dates based on settings, preserving a consistent schedule.
    let mut current_date = if let Some(first) = plan.items.first() {
        first.date
    } else {
        chrono::Utc::now()
    };

    for it in reordered_items.iter_mut() {
        it.date = current_date;
        current_date = crate::planner::calendar::next_session_date(current_date, settings);
    }

    plan.items = reordered_items;
    Ok(plan)
}
