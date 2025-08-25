use crate::PlanError;
use crate::types::{Course, PlanItem, PlanSettings};
use std::time::Duration;

/// Hybrid strategy:
/// - Respects module boundaries
/// - Packs sections into duration-aware sessions per module
/// - Creates PlanItems per session and advances dates according to settings
pub fn generate_hybrid_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course
        .structure
        .as_ref()
        .expect("Course must be structured for hybrid plan");
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Try to respect module boundaries while considering time constraints
    for module in &structure.modules {
        let module_sessions = plan_module_sessions(module, settings)?;

        for session in module_sessions {
            let estimated_completion_time =
                crate::types::duration_utils::calculate_completion_time_with_buffer(
                    session.total_duration,
                    0.25,
                );

            plan_items.push(PlanItem {
                date: current_date,
                module_title: module.title.clone(),
                section_title: session.title,
                video_indices: session.video_indices,
                completed: false,
                total_duration: session.total_duration,
                estimated_completion_time,
                overflow_warnings: session.overflow_warnings,
            });

            current_date = crate::planner::get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
}

#[derive(Debug, Clone)]
struct SessionPlan {
    title: String,
    video_indices: Vec<usize>,
    total_duration: Duration,
    overflow_warnings: Vec<String>,
}

/// Plan sessions for a specific module with duration-aware grouping
fn plan_module_sessions(
    module: &crate::types::Module,
    settings: &PlanSettings,
) -> Result<Vec<SessionPlan>, PlanError> {
    let effective_session_limit = crate::planner::capacity::effective_session_limit(settings);

    let mut sessions = Vec::new();
    let mut current_session_videos = Vec::new();
    let mut current_session_duration = Duration::from_secs(0);

    for section in &module.sections {
        let section_duration = section.duration;

        // Handle videos that exceed session time limits
        if video_exceeds_session_limit(section_duration, settings) {
            if !current_session_videos.is_empty() {
                // Finalize current session before adding the oversized video
                let session_title = create_module_session_title(&current_session_videos, &sessions);
                sessions.push(SessionPlan {
                    title: session_title,
                    video_indices: std::mem::take(&mut current_session_videos),
                    total_duration: current_session_duration,
                    overflow_warnings: Vec::new(),
                });
                current_session_duration = Duration::from_secs(0);
            }

            // Add the oversized video in its own session
            let overflow_warnings = vec![format!(
                "Video '{}' ({}) exceeds session limit",
                section.title,
                crate::types::duration_utils::format_duration(section.duration)
            )];
            sessions.push(SessionPlan {
                title: format!("{} (Extended Session)", section.title),
                video_indices: vec![section.video_index],
                total_duration: section.duration,
                overflow_warnings,
            });
            continue;
        }

        // Check if adding this section would exceed effective session limit
        if current_session_duration + section_duration > effective_session_limit
            && !current_session_videos.is_empty()
        {
            // Finalize current session
            let session_title = create_module_session_title(&current_session_videos, &sessions);
            sessions.push(SessionPlan {
                title: session_title,
                video_indices: std::mem::take(&mut current_session_videos),
                total_duration: current_session_duration,
                overflow_warnings: Vec::new(),
            });
            current_session_duration = Duration::from_secs(0);
        }

        // Add current section to session
        current_session_videos.push(section.video_index);
        current_session_duration += section_duration;
    }

    // Add remaining videos as final session
    if !current_session_videos.is_empty() {
        let session_title = create_module_session_title(&current_session_videos, &sessions);
        sessions.push(SessionPlan {
            title: session_title,
            video_indices: current_session_videos,
            total_duration: current_session_duration,
            overflow_warnings: Vec::new(),
        });
    }

    Ok(sessions)
}

/// Create an appropriate session title for module-based sessions
fn create_module_session_title(
    video_indices: &[usize],
    existing_sessions: &[SessionPlan],
) -> String {
    if video_indices.len() == 1 {
        format!("Section {}", existing_sessions.len() + 1)
    } else if existing_sessions.is_empty() {
        "Complete Module".to_string()
    } else {
        format!(
            "Sections {}-{}",
            existing_sessions.len() + 1,
            existing_sessions.len() + video_indices.len()
        )
    }
}

/// Check if a video exceeds the session time limit (with 20% buffer)
fn video_exceeds_session_limit(video_duration: Duration, settings: &PlanSettings) -> bool {
    crate::planner::capacity::video_exceeds_effective_limit(video_duration, settings)
}
