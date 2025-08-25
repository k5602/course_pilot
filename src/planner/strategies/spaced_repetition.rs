use crate::PlanError;
use crate::types::{Course, PlanItem, PlanSettings};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Duration;

/// Default spaced repetition intervals (in days)
const SPACED_REPETITION_INTERVALS: &[i64] = &[1, 3, 7, 14, 30, 90];

/// Generate a spaced repetition plan optimized for memory retention.
///
/// Strategy:
/// - First pass: schedule the initial learning sessions for each section.
/// - For each initial session, compute a vector of review dates using the
///   spaced repetition intervals and remember those per video index.
/// - Second pass: add review sessions on the computed dates, with reduced duration
///   relative to the original content.
/// - Finally, sort all items by date to form a coherent plan.
pub fn generate_spaced_repetition_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course
        .structure
        .as_ref()
        .expect("Course must be structured for spaced repetition plan");
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;
    let mut review_schedule: HashMap<usize, Vec<DateTime<Utc>>> = HashMap::new();

    // First pass: Create initial learning sessions
    for module in &structure.modules {
        for section in &module.sections {
            let estimated_completion_time =
                crate::types::duration_utils::calculate_completion_time_with_buffer(
                    section.duration,
                    0.25,
                );
            let overflow_warnings = if crate::types::duration_utils::is_duration_excessive(
                section.duration,
                settings.session_length_minutes,
            ) {
                vec![format!(
                    "Video '{}' ({}) exceeds session limit",
                    section.title,
                    crate::types::duration_utils::format_duration(section.duration)
                )]
            } else {
                Vec::new()
            };

            plan_items.push(PlanItem {
                date: current_date,
                module_title: module.title.clone(),
                section_title: section.title.clone(),
                video_indices: vec![section.video_index],
                completed: false,
                total_duration: section.duration,
                estimated_completion_time,
                overflow_warnings,
            });

            // Schedule spaced repetition reviews
            let mut review_dates = Vec::new();
            for &interval in SPACED_REPETITION_INTERVALS {
                let review_date = current_date + chrono::Duration::days(interval);
                review_dates.push(review_date);
            }
            review_schedule.insert(section.video_index, review_dates);

            current_date = crate::planner::get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    // Second pass: Add review sessions
    for (video_index, review_dates) in review_schedule {
        for (review_num, &review_date) in review_dates.iter().enumerate() {
            // Find the original section for context
            let mut section_title = "Review Session".to_string();
            let mut module_title = "Review".to_string();
            let mut section_duration = Duration::from_secs(15 * 60); // Default 15 minutes for review

            for module in &structure.modules {
                for section in &module.sections {
                    if section.video_index == video_index {
                        section_title = format!("Review: {}", section.title);
                        module_title = format!("Review: {}", module.title);
                        // Review sessions are typically shorter than original
                        section_duration =
                            Duration::from_secs((section.duration.as_secs() as f32 * 0.6) as u64);
                        break;
                    }
                }
            }

            let estimated_completion_time =
                crate::types::duration_utils::calculate_completion_time_with_buffer(
                    section_duration,
                    0.25,
                );

            plan_items.push(PlanItem {
                date: review_date,
                module_title,
                section_title: format!("{} (Review #{})", section_title, review_num + 1),
                video_indices: vec![video_index],
                completed: false,
                total_duration: section_duration,
                estimated_completion_time,
                overflow_warnings: Vec::new(),
            });
        }
    }

    // Sort all items by date
    plan_items.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(plan_items)
}
