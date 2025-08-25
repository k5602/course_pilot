/*!
Shared capacity and duration helpers for the planner.

This module centralizes logic for:
- Computing strict/effective session limits from `PlanSettings`
- Checking if a video exceeds session limits
- Estimating videos-per-session from average durations or sensible fallbacks
- Computing the average video duration from a course structure

By consolidating these helpers, other planner modules (e.g., strategies, packing,
sequential) can rely on consistent policies for buffer time and capacity.
*/

use std::time::Duration;

use crate::types::{Course, PlanSettings};

/// Fraction of the session time reserved as buffer (notes, breaks, transitions).
/// Applied to compute the effective session limit.
pub const EFFECTIVE_BUFFER_FRACTION: f32 = 0.8;

/// Compute the strict session time limit from settings (in wall-clock time).
#[inline]
pub fn strict_session_limit(settings: &PlanSettings) -> Duration {
    Duration::from_secs(settings.session_length_minutes as u64 * 60)
}

/// Compute the effective session limit by applying a buffer to the strict limit.
/// A minimum of 60 seconds is enforced to avoid degenerate cases for very small limits.
#[inline]
pub fn effective_session_limit(settings: &PlanSettings) -> Duration {
    let strict = strict_session_limit(settings);
    Duration::from_secs(((strict.as_secs() as f32) * EFFECTIVE_BUFFER_FRACTION).max(60.0) as u64)
}

/// Returns true if `video_duration` exceeds the strict session limit.
#[allow(dead_code)]
#[inline]
pub fn video_exceeds_strict_limit(video_duration: Duration, settings: &PlanSettings) -> bool {
    video_duration > strict_session_limit(settings)
}

/// Returns true if `video_duration` exceeds the effective session limit (with buffer).
#[inline]
pub fn video_exceeds_effective_limit(video_duration: Duration, settings: &PlanSettings) -> bool {
    video_duration > effective_session_limit(settings)
}

/// Compute session capacity given an average video duration and user settings.
/// Ensures at least 1 video per session, even if average exceeds the effective limit.
pub fn session_capacity_for_average(
    average_video_duration: Duration,
    settings: &PlanSettings,
) -> usize {
    let effective = effective_session_limit(settings);

    if average_video_duration >= effective {
        return 1;
    }

    let n = effective.as_secs() / average_video_duration.as_secs();
    std::cmp::max(1, n as usize)
}

/// Fallback estimation of videos per session when course durations are unavailable.
pub fn fallback_videos_per_session(settings: &PlanSettings) -> usize {
    let session_minutes = settings.session_length_minutes;

    // Adaptive average video duration estimate by session length
    let average_video_minutes = match session_minutes {
        0..=30 => 8,   // Short sessions => shorter videos
        31..=60 => 12, // Standard sessions
        61..=90 => 15, // Longer sessions => potentially longer videos
        _ => 18,       // Very long sessions
    };

    let effective_minutes = (session_minutes as f32 * EFFECTIVE_BUFFER_FRACTION) as u32;
    std::cmp::max(1, (effective_minutes / average_video_minutes) as usize)
}

/// Compute the average video duration from a course's structure, if available.
/// Guarantees a minimum return of 60 seconds when durations exist but are very small.
pub fn average_video_duration_from_course(course: &Course) -> Option<Duration> {
    let structure = course.structure.as_ref()?;

    let mut count = 0usize;
    let mut total_secs = 0u64;

    for module in &structure.modules {
        for section in &module.sections {
            let secs = section.duration.as_secs();
            if secs > 0 {
                count += 1;
                total_secs += secs;
            }
        }
    }

    if count == 0 { None } else { Some(Duration::from_secs((total_secs / count as u64).max(60))) }
}

/// Estimate videos per session for a course, using actual durations when available,
/// falling back to a sensible heuristic otherwise.
pub fn estimated_videos_per_session(course: &Course, settings: &PlanSettings) -> usize {
    match average_video_duration_from_course(course) {
        Some(avg) => session_capacity_for_average(avg, settings),
        None => fallback_videos_per_session(settings),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn settings(minutes: u32, sessions_per_week: u8, include_weekends: bool) -> PlanSettings {
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week,
            session_length_minutes: minutes,
            include_weekends,
            advanced_settings: None,
        }
    }

    #[test]
    fn test_limits_basic() {
        let s = settings(60, 3, false);
        let strict = strict_session_limit(&s);
        let eff = effective_session_limit(&s);

        assert_eq!(strict.as_secs(), 60 * 60);
        assert_eq!(eff.as_secs(), (60.0 * 60.0 * EFFECTIVE_BUFFER_FRACTION) as u64);
        assert!(eff < strict);
    }

    #[test]
    fn test_limits_minimum_effective() {
        let s = settings(1, 3, false);
        let strict = strict_session_limit(&s);
        let eff = effective_session_limit(&s);

        // strict = 60s, effective = max(60s * 0.8, 60s) = 60s
        assert_eq!(strict.as_secs(), 60);
        assert_eq!(eff.as_secs(), 60);
    }

    #[test]
    fn test_video_exceeds_checks() {
        let s = settings(60, 3, false);
        assert!(!video_exceeds_strict_limit(Duration::from_secs(50 * 60), &s));
        assert!(video_exceeds_strict_limit(Duration::from_secs(61 * 60), &s));

        // Effective is 48 minutes for 60-minute session
        assert!(!video_exceeds_effective_limit(Duration::from_secs(45 * 60), &s));
        assert!(video_exceeds_effective_limit(Duration::from_secs(49 * 60), &s));
    }

    #[test]
    fn test_session_capacity_for_average() {
        let s = settings(60, 3, false);
        // Effective = 48 min
        assert_eq!(session_capacity_for_average(Duration::from_secs(5 * 60), &s), 9);
        assert_eq!(session_capacity_for_average(Duration::from_secs(50 * 60), &s), 1);
    }

    #[test]
    fn test_fallback_videos_per_session() {
        let s60 = settings(60, 3, false);
        // 60 * 0.8 = 48; 48 / 12 = 4
        assert_eq!(fallback_videos_per_session(&s60), 4);

        let s30 = settings(30, 3, false);
        // 30 * 0.8 = 24; 24 / 8 = 3
        assert_eq!(fallback_videos_per_session(&s30), 3);

        let s120 = settings(120, 3, true);
        // 120 * 0.8 = 96; 96 / 18 = 5 (integer division)
        assert_eq!(fallback_videos_per_session(&s120), 5);
    }
}
