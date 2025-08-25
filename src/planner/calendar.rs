/*!
Calendar utilities for the scheduler.

This module provides:
- Date progression helpers for sessions
- Thin wrappers for plan settings validation and coarse-grained estimates

It intentionally keeps pure time/date logic separate from packing or strategy concerns.
*/

use crate::types::PlanSettings;
use chrono::{DateTime, Datelike, Utc, Weekday};
use std::time::Duration;

/// Validate plan settings and map errors to `PlanError`.
///
/// This is a thin wrapper around the planner's canonical validator.
pub fn validate_settings(settings: &PlanSettings) -> Result<(), crate::PlanError> {
    crate::planner::validate_plan_settings(
        settings.sessions_per_week,
        settings.session_length_minutes,
        settings.start_date,
    )
    .map_err(crate::PlanError::InvalidSettings)
}

/// Compute the next session date given current date and settings.
///
/// Delegates to the top-level planner logic to ensure centralized rules.
/// Use this when incrementally assigning dates while generating a plan.
pub fn next_session_date(current: DateTime<Utc>, settings: &PlanSettings) -> DateTime<Utc> {
    crate::planner::get_next_session_date(
        current,
        settings.sessions_per_week,
        settings.include_weekends,
    )
}

/// Generate a sequence of session dates starting from `start_date`.
///
/// - The first item will always be `start_date`.
/// - Subsequent items are computed using `next_session_date`.
pub fn generate_session_dates(
    start_date: DateTime<Utc>,
    total_sessions: usize,
    settings: &PlanSettings,
) -> Vec<DateTime<Utc>> {
    if total_sessions == 0 {
        return Vec::new();
    }

    let mut dates = Vec::with_capacity(total_sessions);
    let mut current = start_date;

    dates.push(current);
    for _ in 1..total_sessions {
        current = next_session_date(current, settings);
        dates.push(current);
    }

    dates
}

/// Convenience wrapper: estimate number of calendar weeks for a given count of sessions.
#[allow(dead_code)]
pub fn course_duration_weeks(total_sessions: usize, sessions_per_week: u8) -> usize {
    crate::planner::calculate_course_duration_weeks(total_sessions, sessions_per_week)
}

/// Convenience wrapper: compute total study time from video count and average duration,
/// including buffer time policy defined by the planner.
pub fn total_study_time_estimate(video_count: usize, average_video_duration: Duration) -> Duration {
    crate::planner::calculate_total_study_time(video_count, average_video_duration)
}

/// Helper to list available study days based on `include_weekends`.
#[allow(dead_code)]
#[inline]
pub fn available_weekdays(include_weekends: bool) -> &'static [Weekday] {
    if include_weekends {
        const ALL: [Weekday; 7] = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];
        &ALL
    } else {
        const WEEKDAYS: [Weekday; 5] =
            [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri];
        &WEEKDAYS
    }
}

/// Validate that all generated dates respect the weekend policy.
/// This is intended for internal assertions/tests.
#[allow(dead_code)]
pub fn validate_weekend_policy(
    dates: &[DateTime<Utc>],
    include_weekends: bool,
) -> Result<(), String> {
    if include_weekends {
        return Ok(());
    }

    for (i, d) in dates.iter().enumerate() {
        match d.weekday() {
            Weekday::Sat | Weekday::Sun => {
                return Err(format!(
                    "Date at index {} falls on weekend ({:?}) while weekends are excluded",
                    i,
                    d.weekday()
                ));
            },
            _ => {},
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn settings(include_weekends: bool, sessions_per_week: u8, minutes: u32) -> PlanSettings {
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week,
            session_length_minutes: minutes,
            include_weekends,
            advanced_settings: None,
        }
    }

    #[test]
    fn test_validate_settings_ok() {
        let s = settings(false, 3, 60);
        assert!(validate_settings(&s).is_ok());
    }

    #[test]
    fn test_validate_settings_err() {
        let mut s = settings(false, 0, 60);
        // sessions_per_week = 0 should be invalid
        assert!(validate_settings(&s).is_err());

        s = settings(false, 3, 5);
        // session_length too small
        assert!(validate_settings(&s).is_err());
    }

    #[test]
    fn test_generate_session_dates_monotonic() {
        let s = settings(false, 3, 60);
        let start = Utc::now();
        let dates = generate_session_dates(start, 6, &s);
        assert_eq!(dates.len(), 6);
        for w in dates.windows(2) {
            assert!(w[1] >= w[0], "dates must be non-decreasing");
        }
    }

    #[test]
    fn test_weekend_policy_respected_when_excluded() {
        let s = settings(false, 3, 60);
        let start = Utc::now();
        let dates = generate_session_dates(start, 10, &s);
        validate_weekend_policy(&dates, s.include_weekends).expect("no weekend dates expected");
    }

    #[test]
    fn test_course_duration_weeks_wrapper() {
        assert_eq!(course_duration_weeks(10, 3), 4);
        assert_eq!(course_duration_weeks(9, 3), 3);
        assert_eq!(course_duration_weeks(0, 3), 0);
    }

    #[test]
    fn test_total_study_time_estimate_wrapper() {
        // average 10 minutes, 5 videos -> 50 minutes + buffers (5 * 5 min) = 75 min
        let total = total_study_time_estimate(5, Duration::from_secs(600));
        assert_eq!(total.as_secs(), 75 * 60);
    }
}
