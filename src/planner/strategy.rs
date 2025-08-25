/*!
Strategy selection and complexity heuristics for the scheduler.

This module isolates:
- Distribution strategy selection
- Course complexity analysis
- User experience heuristics
- Session capacity estimation (videos per session)

By keeping these here, the core scheduler focuses on packing and date assignment.
*/

use crate::PlanError;
use crate::planner::PlanningDefaults;
use crate::types::{Course, DifficultyLevel, DistributionStrategy, PlanSettings};
use std::time::Duration;

/// Heuristic keywords that tend to increase/decrease cognitive load.
/// Positive values increase load, negative values decrease load slightly.
const COGNITIVE_LOAD_FACTORS: &[(f32, &str)] = &[
    (0.30, "advanced"),
    (0.25, "expert"),
    (0.20, "theory"),
    (0.20, "derivation"),
    (0.20, "proof"),
    (0.20, "project"),
    (0.15, "assignment"),
    (0.15, "exam"),
    (0.15, "test"),
    (0.12, "exercise"),
    (0.10, "practice"),
    (0.10, "lab"),
    (-0.10, "introduction"),
    (-0.08, "intro"),
    (-0.08, "overview"),
    (-0.08, "basics"),
];

/// Select a distribution strategy for the given course and settings.
///
/// This function uses a blend of heuristics:
/// - Course complexity inferred from titles and durations
/// - Estimated session capacity (videos per session)
/// - Module structure properties
/// - User experience inferred from schedule intensity
pub fn choose_distribution_strategy(
    course: &Course,
    settings: &PlanSettings,
) -> Result<DistributionStrategy, PlanError> {
    let structure = course
        .structure
        .as_ref()
        .ok_or(PlanError::CourseNotStructured)?;

    // Analyze course characteristics
    let total_videos = total_video_count(course);
    let module_count = structure.modules.len();
    let average_module_size = if module_count > 0 {
        total_videos / module_count
    } else {
        total_videos
    };

    // Capacity and heuristics
    let estimated_videos_per_session = estimate_videos_per_session(course, settings);
    let course_complexity = analyze_course_complexity(course);
    let user_experience_level = infer_user_experience_level(settings);

    // Opinionated strategy selection
    let strategy = match (course_complexity, user_experience_level, module_count) {
        // High complexity courses need adaptive scheduling
        (complexity, _, _) if complexity > 0.8 => DistributionStrategy::Adaptive,

        // Beginner users benefit from spaced repetition
        (_, DifficultyLevel::Beginner, _) => DistributionStrategy::SpacedRepetition,

        // Well-structured courses with clear modules -> module-based flow works great
        (_, _, modules)
            if modules > 3
                && average_module_size <= estimated_videos_per_session.saturating_mul(2) =>
        {
            DistributionStrategy::ModuleBased
        }

        // Very large courses: emphasize difficulty progression
        (_, _, _) if total_videos > estimated_videos_per_session.saturating_mul(15) => {
            DistributionStrategy::DifficultyBased
        }

        // Large courses: emphasize time-based distribution
        (_, _, _) if total_videos > estimated_videos_per_session.saturating_mul(10) => {
            DistributionStrategy::TimeBased
        }

        // Default: hybrid strikes a balance
        _ => DistributionStrategy::Hybrid,
    };

    Ok(strategy)
}

/// Estimate videos per session based on the course metadata and user settings.
///
/// Uses the average video duration if available; falls back to a reasonable default.
/// Adds a small buffer per video (notes, cognitive switching) using PlanningDefaults.
pub fn estimate_videos_per_session(course: &Course, settings: &PlanSettings) -> usize {
    let structure_opt = course.structure.as_ref();

    // Average duration across all sections with known duration
    let (sum_secs, count) = if let Some(structure) = structure_opt {
        let mut sum = 0u64;
        let mut cnt = 0usize;
        for module in &structure.modules {
            for section in &module.sections {
                let secs = section.duration.as_secs();
                if secs > 0 {
                    sum += secs;
                    cnt += 1;
                }
            }
        }
        (sum, cnt)
    } else {
        (0, 0)
    };

    let avg_secs = if count > 0 {
        (sum_secs / count as u64).max(60) // minimum 1 minute to avoid divide by zero
    } else {
        10 * 60 // fallback: 10 minutes per video
    };

    let buffer_secs_per_video = (PlanningDefaults::BUFFER_TIME_MINUTES as u64) * 60 / 2; // half of buffer assumed per video
    let per_video_effective = avg_secs.saturating_add(buffer_secs_per_video);

    let session_secs = (settings.session_length_minutes as u64).saturating_mul(60);
    let est = (session_secs / per_video_effective).max(1);

    est as usize
}

/// Compute a 0.0–1.0 complexity score for the course content.
///
/// Signals:
/// - Longer videos are typically more complex
/// - Presence of certain keywords correlates with higher cognitive load
/// - Normalized across all sections, defaults to a moderate 0.5 if no data
pub fn analyze_course_complexity(course: &Course) -> f32 {
    let structure = match &course.structure {
        Some(s) => s,
        None => return 0.5, // default moderate complexity if no structure
    };

    let mut complexity_score = 0.0f32;
    let mut total_sections = 0usize;

    for module in &structure.modules {
        for section in &module.sections {
            total_sections += 1;

            // Keyword-based cognitive load
            let title_lower = section.title.to_lowercase();
            for (load_factor, keyword) in COGNITIVE_LOAD_FACTORS {
                if title_lower.contains(keyword) {
                    complexity_score += *load_factor;
                    break;
                }
            }

            // Duration-based weight: >30 min significantly increases perceived complexity
            let duration_minutes = section.duration.as_secs() / 60;
            if duration_minutes > 30 {
                complexity_score += 0.30;
            } else if duration_minutes > 15 {
                complexity_score += 0.10;
            }
        }
    }

    if total_sections > 0 {
        // Normalize to [0, 1], clamp to be safe
        (complexity_score / total_sections as f32).clamp(0.0, 1.0)
    } else {
        0.5
    }
}

/// Infer user experience level from their schedule intensity.
pub fn infer_user_experience_level(settings: &PlanSettings) -> DifficultyLevel {
    match (settings.sessions_per_week, settings.session_length_minutes) {
        (sessions, duration) if sessions >= 5 && duration >= 90 => DifficultyLevel::Expert,
        (sessions, duration) if sessions >= 4 && duration >= 60 => DifficultyLevel::Advanced,
        (sessions, duration) if sessions >= 3 && duration >= 45 => DifficultyLevel::Intermediate,
        _ => DifficultyLevel::Beginner,
    }
}

/// Helper: compute total video count (from structure if available; else fallback to videos list).
fn total_video_count(course: &Course) -> usize {
    if let Some(structure) = &course.structure {
        structure.modules.iter().map(|m| m.sections.len()).sum()
    } else {
        course.videos.len()
    }
}

/// Utility: convert minutes to a Duration.
#[allow(dead_code)]
#[inline]
fn minutes(min: u32) -> Duration {
    Duration::from_secs(min as u64 * 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Course, CourseStructure, Module, Section, StructureMetadata};
    use chrono::Utc;

    fn make_structure(section_durations_min: &[u64]) -> CourseStructure {
        let sections: Vec<Section> = section_durations_min
            .iter()
            .enumerate()
            .map(|(i, m)| Section {
                title: format!("Video {}", i + 1),
                video_index: i,
                duration: Duration::from_secs(m * 60),
            })
            .collect();

        let modules = vec![Module::new_basic("Module 1".to_string(), sections)];
        let metadata = StructureMetadata {
            total_videos: section_durations_min.len(),
            total_duration: Duration::from_secs(section_durations_min.iter().sum::<u64>() * 60),
            estimated_duration_hours: None,
            difficulty_level: None,
            structure_quality_score: None,
            content_coherence_score: None,
            content_type_detected: None,
            original_order_preserved: None,
            processing_strategy_used: None,
        };

        CourseStructure::new_basic(modules, metadata)
    }

    fn make_course(section_durations_min: &[u64]) -> Course {
        Course {
            id: uuid::Uuid::new_v4(),
            name: "Test".into(),
            created_at: Utc::now(),
            raw_titles: vec![],
            videos: vec![],
            structure: Some(make_structure(section_durations_min)),
        }
    }

    fn settings(sessions_per_week: u8, session_len_min: u32) -> PlanSettings {
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week,
            session_length_minutes: session_len_min,
            include_weekends: false,
            advanced_settings: None,
        }
    }

    #[test]
    fn test_estimate_videos_per_session() {
        let c = make_course(&[10, 12, 8, 30, 5]); // avg ~13 min + buffer
        let s = settings(3, 60);
        let est = estimate_videos_per_session(&c, &s);

        assert!(est >= 2 && est <= 5, "unexpected estimate: {}", est);
    }

    #[test]
    fn test_complexity_increases_with_duration_and_keywords() {
        let c1 = make_course(&[5, 8, 10]); // short videos
        let c2 = make_course(&[35, 40, 25]); // long videos

        // Inject some keywords in module title for c2 to increase load
        let mut c2 = c2;
        if let Some(structure) = &mut c2.structure {
            structure.modules[0].sections[0].title = "Advanced Topic".into();
        }

        let comp1 = analyze_course_complexity(&c1);
        let comp2 = analyze_course_complexity(&c2);

        assert!(comp2 > comp1, "complexity should be higher for c2");
    }

    #[test]
    fn test_infer_user_experience_level() {
        assert_eq!(
            infer_user_experience_level(&settings(5, 90)),
            DifficultyLevel::Expert
        );
        assert_eq!(
            infer_user_experience_level(&settings(4, 60)),
            DifficultyLevel::Advanced
        );
        assert_eq!(
            infer_user_experience_level(&settings(3, 45)),
            DifficultyLevel::Intermediate
        );
        assert_eq!(
            infer_user_experience_level(&settings(2, 30)),
            DifficultyLevel::Beginner
        );
    }

    #[test]
    fn test_choose_distribution_strategy_sane_defaults() {
        let course = make_course(&[10, 12, 14, 16, 18, 20, 7, 9, 11, 13, 15, 17]); // 12 videos
        let s = settings(3, 60);
        let strategy = choose_distribution_strategy(&course, &s).expect("strategy");
        // Should be one of the known variants; hybrid is common default
        match strategy {
            DistributionStrategy::ModuleBased
            | DistributionStrategy::TimeBased
            | DistributionStrategy::Hybrid
            | DistributionStrategy::DifficultyBased
            | DistributionStrategy::SpacedRepetition
            | DistributionStrategy::Adaptive => {}
        }
    }
}
