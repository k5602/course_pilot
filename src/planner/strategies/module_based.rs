use crate::PlanError;
use crate::types::{Course, PlanItem, PlanSettings};
use std::time::Duration;

/// Generate a module-based study plan
///
/// Splits each module into duration-aware session groups that do not exceed the
/// effective session length (session_length_minutes with a 20% buffer for breaks/notes).
///
/// Notes:
/// - Oversized videos (exceeding the effective session time) are placed in their own session
///   and produce overflow warnings upstream via validation utilities when rendered.
/// - Session dates are advanced using `crate::planner::get_next_session_date`.
pub fn generate_module_based_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure =
        course.structure.as_ref().expect("Course must be structured for module-based plan");

    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    for module in &structure.modules {
        // Group sections within the module by session capacity
        let session_groups = group_sections_by_capacity(module, course, settings);

        for group in session_groups {
            let video_indices: Vec<usize> = group.iter().map(|s| s.video_index).collect();
            let total_duration: Duration = group.iter().map(|s| s.duration).sum();
            let estimated_completion_time =
                crate::types::duration_utils::calculate_completion_time_with_buffer(
                    total_duration,
                    0.25,
                );
            let overflow_warnings =
                crate::types::duration_utils::validate_session_duration(&group, settings);

            plan_items.push(PlanItem {
                date: current_date,
                module_title: module.title.clone(),
                section_title: create_session_title(&group),
                video_indices,
                completed: false,
                total_duration,
                estimated_completion_time,
                overflow_warnings,
            });

            // Calculate next session date
            current_date = crate::planner::get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
}

/// Group sections within a module by session capacity using actual durations
fn group_sections_by_capacity<'a>(
    module: &'a crate::types::Module,
    _course: &Course,
    settings: &PlanSettings,
) -> Vec<Vec<&'a crate::types::Section>> {
    let effective_session_limit = crate::planner::capacity::effective_session_limit(settings);

    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    let mut current_group_duration = Duration::from_secs(0);

    for section in &module.sections {
        let section_duration = section.duration;

        // Handle videos that exceed session time limits
        if video_exceeds_session_limit(section_duration, settings) {
            if !current_group.is_empty() {
                // Finalize current group before adding the oversized video
                groups.push(std::mem::take(&mut current_group));
                current_group_duration = Duration::from_secs(0);
            }

            // Add the oversized video in its own group
            groups.push(vec![section]);
            continue;
        }

        // Check if adding this section would exceed the session limit
        if current_group_duration + section_duration > effective_session_limit
            && !current_group.is_empty()
        {
            // Finalize current group
            groups.push(std::mem::take(&mut current_group));
            current_group_duration = Duration::from_secs(0);
        }

        // Add section to current group
        current_group.push(section);
        current_group_duration += section_duration;
    }

    // Add remaining sections as final group
    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

/// Check if a video exceeds the session time limit (with 20% buffer)
fn video_exceeds_session_limit(video_duration: Duration, settings: &PlanSettings) -> bool {
    crate::planner::capacity::video_exceeds_effective_limit(video_duration, settings)
}

/// Create a session title from a group of sections
fn create_session_title(sections: &[&crate::types::Section]) -> String {
    if sections.len() == 1 {
        sections[0].title.clone()
    } else {
        format!("{} (+{} more)", sections[0].title, sections.len() - 1)
    }
}
