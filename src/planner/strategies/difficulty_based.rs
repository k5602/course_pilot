use crate::PlanError;
use crate::types::{Course, DifficultyLevel, PlanItem, PlanSettings};
use chrono::{DateTime, Utc};
use std::time::Duration;

// Reuse the internal representation for packing/grouping videos into sessions.
use crate::planner::capacity::{strict_session_limit, video_exceeds_effective_limit};
use crate::planner::packing::VideoItem;

/// Generate a difficulty-based study plan that adapts pacing to content complexity.
///
/// Strategy:
/// - Analyze each section to estimate difficulty using title keywords and duration heuristics.
/// - Partition content into phases (Beginner -> Intermediate -> Advanced -> Expert).
/// - Within each phase, group content into sessions constrained by the user's effective session
///   length (with phase-specific buffer factors).
/// - Add extra spacing between harder phases to reduce cognitive overload.
pub fn generate_difficulty_based_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course
        .structure
        .as_ref()
        .expect("Course must be structured for difficulty-based plan");
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Analyze and sort content by difficulty
    let mut content_items: Vec<(DifficultyLevel, VideoItem)> = Vec::new();

    for module in &structure.modules {
        for section in &module.sections {
            let difficulty = analyze_section_difficulty(&section.title, section.duration);
            content_items.push((
                difficulty,
                VideoItem {
                    module_title: module.title.clone(),
                    section_title: section.title.clone(),
                    video_index: section.video_index,
                    duration: section.duration,
                },
            ));
        }
    }

    // Group content by difficulty and create progressive sessions
    let mut beginner_content = Vec::new();
    let mut intermediate_content = Vec::new();
    let mut advanced_content = Vec::new();
    let mut expert_content = Vec::new();

    for (difficulty, item) in content_items {
        match difficulty {
            DifficultyLevel::Beginner => beginner_content.push(item),
            DifficultyLevel::Intermediate => intermediate_content.push(item),
            DifficultyLevel::Advanced => advanced_content.push(item),
            DifficultyLevel::Expert => expert_content.push(item),
        }
    }

    // Create sessions with progressive difficulty using duration-based grouping
    let all_content = [
        beginner_content,
        intermediate_content,
        advanced_content,
        expert_content,
    ];

    for (phase, content) in all_content.iter().enumerate() {
        if content.is_empty() {
            continue;
        }

        // Create duration-aware sessions for each difficulty phase
        let phase_sessions = create_difficulty_phase_sessions(content, phase, settings)?;

        for session_videos in phase_sessions {
            if !session_videos.is_empty() {
                plan_items.push(create_plan_item_from_videos(session_videos, current_date));

                // Add extra time between difficult sessions
                let days_to_add = if phase >= 2 { 2 } else { 1 };
                current_date = crate::planner::get_next_session_date(
                    current_date + chrono::Duration::days(days_to_add - 1),
                    settings.sessions_per_week,
                    settings.include_weekends,
                );
            }
        }
    }

    Ok(plan_items)
}

/// Create duration-aware sessions for a difficulty phase
fn create_difficulty_phase_sessions(
    content: &[VideoItem],
    phase: usize,
    settings: &PlanSettings,
) -> Result<Vec<Vec<VideoItem>>, PlanError> {
    let session_limit = strict_session_limit(settings);
    // Apply phase-specific buffer factor
    let buffer_factor = match phase {
        0 => 0.7,  // Beginner: more buffer time for processing
        1 => 0.8,  // Intermediate: standard buffer
        2 => 0.85, // Advanced: less buffer, more focused
        3 => 0.9,  // Expert: minimal buffer, intensive sessions
        _ => 0.8,
    };
    let effective_session_limit =
        Duration::from_secs((session_limit.as_secs() as f32 * buffer_factor) as u64);

    let mut sessions = Vec::new();
    let mut current_session = Vec::new();
    let mut current_duration = Duration::from_secs(0);

    for video in content {
        // Handle videos that exceed session time limits
        if video_exceeds_session_limit(video.duration, settings) {
            if !current_session.is_empty() {
                sessions.push(std::mem::take(&mut current_session));
                current_duration = Duration::from_secs(0);
            }

            // Add oversized video in its own session
            sessions.push(vec![video.clone()]);
            continue;
        }

        // Check if adding this video would exceed the session limit
        if current_duration + video.duration > effective_session_limit
            && !current_session.is_empty()
        {
            sessions.push(std::mem::take(&mut current_session));
            current_duration = Duration::from_secs(0);
        }

        // Add video to current session
        current_session.push(video.clone());
        current_duration += video.duration;

        // For expert content (phase 3), limit to one video per session
        if phase == 3 {
            sessions.push(std::mem::take(&mut current_session));
            current_duration = Duration::from_secs(0);
        }
    }

    // Add remaining videos as final session
    if !current_session.is_empty() {
        sessions.push(current_session);
    }

    Ok(sessions)
}

/// Heuristic difficulty analysis based on title keywords and duration.
/// Falls back to duration when keywords are absent.
fn analyze_section_difficulty(title: &str, duration: Duration) -> DifficultyLevel {
    let title_lower = title.to_lowercase();
    let duration_minutes = duration.as_secs() / 60;

    // Keywords indicating difficulty levels
    let expert_keywords = ["advanced", "expert", "complex", "algorithm", "optimization"];
    let advanced_keywords = ["intermediate", "deep", "detailed", "implementation"];
    let beginner_keywords = ["introduction", "basic", "getting started", "overview"];

    // Check for explicit difficulty indicators
    for keyword in expert_keywords {
        if title_lower.contains(keyword) {
            return DifficultyLevel::Expert;
        }
    }

    for keyword in advanced_keywords {
        if title_lower.contains(keyword) {
            return DifficultyLevel::Advanced;
        }
    }

    for keyword in beginner_keywords {
        if title_lower.contains(keyword) {
            return DifficultyLevel::Beginner;
        }
    }

    // Use duration as a heuristic
    match duration_minutes {
        0..=10 => DifficultyLevel::Beginner,
        11..=25 => DifficultyLevel::Intermediate,
        26..=45 => DifficultyLevel::Advanced,
        _ => DifficultyLevel::Expert,
    }
}

/// Check if a video exceeds the session time limit (with 20% buffer)
fn video_exceeds_session_limit(video_duration: Duration, settings: &PlanSettings) -> bool {
    video_exceeds_effective_limit(video_duration, settings)
}

/// Create a plan item from a collection of video items
fn create_plan_item_from_videos(videos: Vec<VideoItem>, date: DateTime<Utc>) -> PlanItem {
    let module_title = videos[0].module_title.clone();
    let section_title = if videos.len() == 1 {
        videos[0].section_title.clone()
    } else {
        format!("Mixed Content ({} videos)", videos.len())
    };

    let total_duration: Duration = videos.iter().map(|v| v.duration).sum();
    let estimated_completion_time =
        crate::types::duration_utils::calculate_completion_time_with_buffer(total_duration, 0.25);
    let video_indices = videos.into_iter().map(|v| v.video_index).collect();

    // Generate basic overflow warnings for mixed content
    let mut overflow_warnings = Vec::new();
    if total_duration.as_secs() > 90 * 60 {
        // More than 90 minutes
        overflow_warnings.push(format!(
            "Session duration ({}) is quite long",
            crate::types::duration_utils::format_duration(total_duration)
        ));
    }

    PlanItem {
        date,
        module_title,
        section_title,
        video_indices,
        completed: false,
        total_duration,
        estimated_completion_time,
        overflow_warnings,
    }
}
