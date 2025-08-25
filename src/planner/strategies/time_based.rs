use crate::PlanError;
use crate::types::{Course, PlanItem, PlanSettings};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::time::Duration;

// Use packing helpers from the scheduler's packing module
use crate::planner::packing::{VideoItem, pack_videos_into_session};

/// Generate a time-based study plan with duration-aware session grouping
///
/// Strategy:
/// - Flatten all course sections into a queue of VideoItem entries
/// - Iteratively pack videos into sessions using bin-packing constrained by
///   the user's effective session length (with buffer)
/// - Create PlanItem entries per packed session and advance dates accordingly
pub fn generate_time_based_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course
        .structure
        .as_ref()
        .expect("Course must be structured for time-based plan");

    // Flatten all sections into a queue
    let mut video_queue: VecDeque<VideoItem> = VecDeque::new();
    for module in &structure.modules {
        for section in &module.sections {
            video_queue.push_back(VideoItem {
                module_title: module.title.clone(),
                section_title: section.title.clone(),
                video_index: section.video_index,
                duration: section.duration,
            });
        }
    }

    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Apply bin-packing logic to optimize session utilization
    while !video_queue.is_empty() {
        let session_videos = pack_videos_into_session(&mut video_queue, settings)?;

        if !session_videos.is_empty() {
            plan_items.push(create_plan_item_from_videos(session_videos, current_date));

            current_date = crate::planner::get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
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
