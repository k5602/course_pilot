//! Study plan generation algorithm
//!
//! This module implements intelligent scheduling algorithms to distribute
//! course content across time based on user preferences and course structure.

use crate::PlanError;
use crate::planner::{get_next_session_date, validate_plan_settings};
use crate::types::{Course, Plan, PlanItem, PlanSettings};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::time::Duration;

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
    // Validate input parameters
    validate_plan_settings(
        settings.sessions_per_week,
        settings.session_length_minutes,
        settings.start_date,
    )
    .map_err(PlanError::InvalidSettings)?;

    // Check if course has structure
    let _structure = course
        .structure
        .as_ref()
        .ok_or(PlanError::CourseNotStructured)?;

    // Create session distribution strategy
    let strategy = choose_distribution_strategy(course, settings)?;

    // Generate plan items based on strategy
    let plan_items = match strategy {
        DistributionStrategy::ModuleBased => generate_module_based_plan(course, settings)?,
        DistributionStrategy::TimeBased => generate_time_based_plan(course, settings)?,
        DistributionStrategy::Hybrid => generate_hybrid_plan(course, settings)?,
    };

    // Create and return the plan
    let mut plan = Plan::new(course.id, settings.clone());
    plan.items = plan_items;

    Ok(plan)
}

/// Different strategies for distributing course content
#[derive(Debug, Clone)]
enum DistributionStrategy {
    ModuleBased, // Respect module boundaries
    TimeBased,   // Focus on even time distribution
    Hybrid,      // Balance both approaches
}

/// Choose the best distribution strategy for the course
fn choose_distribution_strategy(
    course: &Course,
    settings: &PlanSettings,
) -> Result<DistributionStrategy, PlanError> {
    let structure = course.structure.as_ref().unwrap();

    // Analyze course characteristics
    let total_videos = course.video_count();
    let module_count = structure.modules.len();
    let average_module_size = if module_count > 0 {
        total_videos / module_count
    } else {
        total_videos
    };

    // Calculate session capacity
    let estimated_videos_per_session = calculate_videos_per_session(settings);

    // Choose strategy based on analysis
    if module_count > 1 && average_module_size <= estimated_videos_per_session * 2 {
        Ok(DistributionStrategy::ModuleBased)
    } else if total_videos > estimated_videos_per_session * 10 {
        Ok(DistributionStrategy::TimeBased)
    } else {
        Ok(DistributionStrategy::Hybrid)
    }
}

/// Generate a module-based study plan
fn generate_module_based_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    for module in &structure.modules {
        // Group sections within the module by session capacity
        let session_groups = group_sections_by_capacity(module, settings);

        for group in session_groups {
            let video_indices: Vec<usize> = group.iter().map(|s| s.video_index).collect();

            plan_items.push(PlanItem {
                date: current_date,
                module_title: module.title.clone(),
                section_title: create_session_title(&group),
                video_indices,
                completed: false,
            });

            // Calculate next session date
            current_date = get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
}

/// Generate a time-based study plan
fn generate_time_based_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let _videos_per_session = calculate_videos_per_session(settings);

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

    while !video_queue.is_empty() {
        let mut session_videos = Vec::new();
        let mut session_duration = Duration::from_secs(0);
        let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);

        // Fill session up to time limit
        while let Some(video) = video_queue.front() {
            let video_duration = video.duration;

            if session_duration + video_duration <= session_limit || session_videos.is_empty() {
                let video = video_queue.pop_front().unwrap();
                session_duration += video_duration;
                session_videos.push(video);
            } else {
                break;
            }
        }

        if !session_videos.is_empty() {
            plan_items.push(create_plan_item_from_videos(session_videos, current_date));

            current_date = get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
}

/// Generate a hybrid study plan
fn generate_hybrid_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Try to respect module boundaries while considering time constraints
    for module in &structure.modules {
        let module_sessions = plan_module_sessions(module, settings)?;

        for session in module_sessions {
            plan_items.push(PlanItem {
                date: current_date,
                module_title: module.title.clone(),
                section_title: session.title,
                video_indices: session.video_indices,
                completed: false,
            });

            current_date = get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
}

/// Helper struct for video items in planning
#[derive(Debug, Clone)]
struct VideoItem {
    module_title: String,
    section_title: String,
    video_index: usize,
    duration: Duration,
}

/// Helper struct for session planning
#[derive(Debug, Clone)]
struct SessionPlan {
    title: String,
    video_indices: Vec<usize>,
}

/// Calculate how many videos can fit in a session on average
fn calculate_videos_per_session(settings: &PlanSettings) -> usize {
    let session_minutes = settings.session_length_minutes;
    let average_video_minutes = 12; // Estimated average including buffer time

    std::cmp::max(1, session_minutes as usize / average_video_minutes)
}

/// Group sections within a module by session capacity
fn group_sections_by_capacity<'a>(
    module: &'a crate::types::Module,
    settings: &PlanSettings,
) -> Vec<Vec<&'a crate::types::Section>> {
    let videos_per_session = calculate_videos_per_session(settings);
    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    let mut current_group_size = 0;

    for section in &module.sections {
        if current_group_size >= videos_per_session && !current_group.is_empty() {
            groups.push(std::mem::take(&mut current_group));
            current_group_size = 0;
        }

        current_group.push(section);
        current_group_size += 1;
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

/// Create a session title from a group of sections
fn create_session_title(sections: &[&crate::types::Section]) -> String {
    if sections.len() == 1 {
        sections[0].title.clone()
    } else {
        format!("{} (+{} more)", sections[0].title, sections.len() - 1)
    }
}

/// Create a plan item from a collection of video items
fn create_plan_item_from_videos(videos: Vec<VideoItem>, date: DateTime<Utc>) -> PlanItem {
    let module_title = videos[0].module_title.clone();
    let section_title = if videos.len() == 1 {
        videos[0].section_title.clone()
    } else {
        format!("Mixed Content ({} videos)", videos.len())
    };

    let video_indices = videos.into_iter().map(|v| v.video_index).collect();

    PlanItem {
        date,
        module_title,
        section_title,
        video_indices,
        completed: false,
    }
}

/// Plan sessions for a specific module
fn plan_module_sessions(
    module: &crate::types::Module,
    settings: &PlanSettings,
) -> Result<Vec<SessionPlan>, PlanError> {
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    let mut sessions = Vec::new();
    let mut current_session_videos = Vec::new();
    let mut current_session_duration = Duration::from_secs(0);

    for section in &module.sections {
        let section_duration = section.duration;

        // Check if adding this section would exceed session limit
        if current_session_duration + section_duration > session_limit
            && !current_session_videos.is_empty()
        {
            // Finalize current session
            let session_title = if current_session_videos.len() == 1 {
                format!("Section {}", current_session_videos.len())
            } else {
                format!("Sections 1-{}", current_session_videos.len())
            };

            sessions.push(SessionPlan {
                title: session_title,
                video_indices: std::mem::take(&mut current_session_videos),
            });

            current_session_duration = Duration::from_secs(0);
        }

        // Add current section to session
        current_session_videos.push(section.video_index);
        current_session_duration += section_duration;
    }

    // Add remaining videos as final session
    if !current_session_videos.is_empty() {
        let session_title = if sessions.is_empty() {
            "Complete Module".to_string()
        } else {
            "Remaining Sections".to_string()
        };

        sessions.push(SessionPlan {
            title: session_title,
            video_indices: current_session_videos,
        });
    }

    Ok(sessions)
}

/// Optimize plan for better learning outcomes
pub fn optimize_plan(plan: &mut Plan) -> Result<(), PlanError> {
    // Add review sessions
    add_review_sessions(plan)?;

    // Balance session workload
    balance_session_workload(plan)?;

    // Add buffer days for complex topics
    add_buffer_days(plan)?;

    Ok(())
}

/// Add review sessions to the plan
fn add_review_sessions(plan: &mut Plan) -> Result<(), PlanError> {
    let total_sessions = plan.items.len();
    let review_interval = std::cmp::max(5, total_sessions / 4); // Review every 5-25% of course

    let mut review_items = Vec::new();
    for (i, item) in plan.items.iter().enumerate() {
        if (i + 1) % review_interval == 0 && i < plan.items.len() - 1 {
            let review_date = get_next_session_date(
                item.date,
                plan.settings.sessions_per_week,
                plan.settings.include_weekends,
            );

            review_items.push(PlanItem {
                date: review_date,
                module_title: "Review".to_string(),
                section_title: format!("Review: Modules 1-{}", (i / review_interval) + 1),
                video_indices: vec![], // Review sessions don't have specific videos
                completed: false,
            });
        }
    }

    // Insert review items and re-sort by date
    plan.items.extend(review_items);
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(())
}

/// Balance workload across sessions
fn balance_session_workload(plan: &mut Plan) -> Result<(), PlanError> {
    // Find sessions that are too heavy or too light
    let average_videos = plan
        .items
        .iter()
        .map(|item| item.video_indices.len())
        .sum::<usize>()
        / plan.items.len();

    // Redistribute videos from heavy sessions to light ones
    let mut i = 0;
    while i < plan.items.len() {
        if plan.items[i].video_indices.len() > average_videos * 2 {
            // Find the next light session
            let mut light_session_index = None;
            for (j, item) in plan.items.iter().enumerate().skip(i + 1) {
                if item.video_indices.len() < average_videos / 2 {
                    light_session_index = Some(j);
                    break;
                }
            }

            if let Some(light_index) = light_session_index {
                // Move some videos to the lighter session
                let excess_videos = plan.items[i].video_indices.len() - average_videos;
                let videos_to_move = std::cmp::min(excess_videos / 2, 2);

                for _ in 0..videos_to_move {
                    if let Some(video_index) = plan.items[i].video_indices.pop() {
                        plan.items[light_index].video_indices.push(video_index);
                    }
                }
            }
        }
        i += 1;
    }

    Ok(())
}

/// Add buffer days for complex topics
fn add_buffer_days(plan: &mut Plan) -> Result<(), PlanError> {
    let buffer_threshold = 5; // Add buffer if session has more than 5 videos

    for item in plan.items.iter_mut() {
        if item.video_indices.len() > buffer_threshold {
            // Add extra day by moving date forward
            item.date += chrono::Duration::days(1);
        }
    }

    // Re-sort by date after modifications
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Course, CourseStructure, Module, Section, StructureMetadata};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_course() -> Course {
        let structure = CourseStructure {
            modules: vec![
                Module {
                    title: "Introduction".to_string(),
                    sections: vec![
                        Section {
                            title: "Welcome".to_string(),
                            video_index: 0,
                            duration: Duration::from_secs(600),
                        },
                        Section {
                            title: "Setup".to_string(),
                            video_index: 1,
                            duration: Duration::from_secs(900),
                        },
                    ],
                    total_duration: Duration::from_secs(600 + 900),
                },
                Module {
                    title: "Advanced Topics".to_string(),
                    sections: vec![Section {
                        title: "Complex Example".to_string(),
                        video_index: 2,
                        duration: Duration::from_secs(1800),
                    }],
                    total_duration: Duration::from_secs(1800),
                },
            ],
            metadata: StructureMetadata {
                total_videos: 3,
                total_duration: Duration::from_secs(600 + 900 + 1800),
                estimated_duration_hours: Some(1.0),
                difficulty_level: Some("Intermediate".to_string()),
            },
        };

        Course {
            id: Uuid::new_v4(),
            name: "Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec![
                "Welcome".to_string(),
                "Setup".to_string(),
                "Complex Example".to_string(),
            ],
            structure: Some(structure),
        }
    }

    fn create_test_settings() -> PlanSettings {
        PlanSettings {
            start_date: Utc::now() + chrono::Duration::days(1),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
        }
    }

    #[test]
    fn test_generate_plan_basic() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_plan(&course, &settings);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(!plan.items.is_empty());
        assert_eq!(plan.course_id, course.id);
    }

    #[test]
    fn test_generate_plan_without_structure() {
        let mut course = create_test_course();
        course.structure = None;
        let settings = create_test_settings();

        let result = generate_plan(&course, &settings);
        assert!(matches!(result, Err(PlanError::CourseNotStructured)));
    }

    #[test]
    fn test_videos_per_session_calculation() {
        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
        };

        let videos = calculate_videos_per_session(&settings);
        assert_eq!(videos, 5); // 60 minutes / 12 minutes per video
    }

    #[test]
    fn test_invalid_settings() {
        let course = create_test_course();
        let mut settings = create_test_settings();
        settings.sessions_per_week = 0;

        let result = generate_plan(&course, &settings);
        assert!(matches!(result, Err(PlanError::InvalidSettings(_))));
    }

    #[test]
    fn test_plan_optimization() {
        let course = create_test_course();
        let settings = create_test_settings();

        let mut plan = generate_plan(&course, &settings).unwrap();
        let original_length = plan.items.len();

        optimize_plan(&mut plan).unwrap();

        // Should have added review sessions
        assert!(plan.items.len() >= original_length);
    }
}
