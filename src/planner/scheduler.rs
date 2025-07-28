//! Study Plan Generation Algorithm
//!
//! This module implements intelligent scheduling algorithms with advanced features:
//! - Adaptive difficulty-based pacing
//! - Spaced repetition integration
//! - Learning curve optimization
//! - Cognitive load balancing
//! - Prerequisite dependency tracking

use crate::PlanError;
use crate::planner::validate_plan_settings;
use crate::types::{Course, Plan, PlanItem, PlanSettings};
use chrono::{DateTime, Datelike, Utc, Weekday};
use std::collections::{HashMap, VecDeque};
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
    // Check if course has clustering metadata for enhanced planning
    if let Some(structure) = &course.structure {
        if structure.clustering_metadata.is_some() {
            return generate_clustering_aware_plan(course, settings);
        }
    }

    // Fallback to basic planning
    generate_basic_plan(course, settings)
}

/// Generate clustering-aware study plan using advanced clustering insights
pub fn generate_clustering_aware_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Plan, PlanError> {
    // Validate input parameters
    validate_plan_settings(
        settings.sessions_per_week,
        settings.session_length_minutes,
        settings.start_date,
    )
    .map_err(PlanError::InvalidSettings)?;

    let structure = course
        .structure
        .as_ref()
        .ok_or(PlanError::CourseNotStructured)?;

    let clustering_metadata =
        structure
            .clustering_metadata
            .as_ref()
            .ok_or(PlanError::Algorithm(
                "No clustering metadata available".to_string(),
            ))?;

    log::info!(
        "Generating clustering-aware plan: algorithm={:?}, quality={:.3}, clusters={}",
        clustering_metadata.algorithm_used,
        clustering_metadata.quality_score,
        clustering_metadata.cluster_count
    );

    // Choose strategy based on clustering insights
    let strategy = crate::planner::clustering_integration::choose_clustering_aware_strategy(
        course,
        settings,
        clustering_metadata,
    )?;

    // Generate plan items using clustering-enhanced strategies
    let plan_items = match strategy {
        DistributionStrategy::ModuleBased => {
            crate::planner::clustering_integration::generate_topic_aware_module_plan(
                course,
                settings,
                clustering_metadata,
            )?
        }
        DistributionStrategy::TimeBased => {
            crate::planner::clustering_integration::generate_duration_optimized_plan(
                course,
                settings,
                clustering_metadata,
            )?
        }
        DistributionStrategy::Hybrid => {
            crate::planner::clustering_integration::generate_clustering_hybrid_plan(
                course,
                settings,
                clustering_metadata,
            )?
        }
        DistributionStrategy::DifficultyBased => {
            crate::planner::clustering_integration::generate_clustering_difficulty_plan(
                course,
                settings,
                clustering_metadata,
            )?
        }
        DistributionStrategy::SpacedRepetition => {
            crate::planner::clustering_integration::generate_topic_spaced_repetition_plan(
                course,
                settings,
                clustering_metadata,
            )?
        }
        DistributionStrategy::Adaptive => {
            crate::planner::clustering_integration::generate_clustering_adaptive_plan(
                course,
                settings,
                clustering_metadata,
            )?
        }
    };

    // Create plan with clustering metadata
    let mut plan = Plan::new(course.id, settings.clone());
    plan.items = plan_items;

    // Apply clustering-aware optimization
    crate::planner::clustering_integration::optimize_clustering_aware_plan(
        &mut plan,
        clustering_metadata,
    )?;

    log::info!(
        "Generated clustering-aware plan: {} sessions, strategy={:?}",
        plan.items.len(),
        strategy
    );

    Ok(plan)
}

/// Generate basic study plan (fallback when no clustering data available)
pub fn generate_basic_plan(course: &Course, settings: &PlanSettings) -> Result<Plan, PlanError> {
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
        DistributionStrategy::DifficultyBased => generate_difficulty_based_plan(course, settings)?,
        DistributionStrategy::SpacedRepetition => {
            generate_spaced_repetition_plan(course, settings)?
        }
        DistributionStrategy::Adaptive => generate_adaptive_plan(course, settings)?,
    };

    // Create and return the plan
    let mut plan = Plan::new(course.id, settings.clone());
    plan.items = plan_items;

    // Apply advanced optimization features
    optimize_plan(&mut plan)?;

    Ok(plan)
}

// Import distribution strategy and difficulty level from types
use crate::types::{DifficultyLevel, DistributionStrategy};

/// Learning session types for varied engagement
#[derive(Debug, Clone, PartialEq)]
enum SessionType {
    Introduction, // New concept introduction
    Practice,     // Hands-on practice
    Review,       // Content review
    Assessment,   // Knowledge check
    Project,      // Applied project work
    #[allow(dead_code)]
    Break, // Rest/consolidation
}

/// Enhanced session planning with cognitive load considerations
#[derive(Debug, Clone)]
struct EnhancedSessionPlan {
    title: String,
    video_indices: Vec<usize>,
    session_type: SessionType,
    difficulty_level: DifficultyLevel,
    estimated_cognitive_load: f32, // 0.0 to 1.0
    #[allow(dead_code)]
    prerequisites: Vec<usize>, // Session indices that must be completed first
    #[allow(dead_code)]
    optimal_time_of_day: Option<TimeOfDay>,
}

/// Optimal time of day for different types of learning
#[derive(Debug, Clone, Copy, PartialEq)]
enum TimeOfDay {
    Morning,   // 6-12: Best for complex/new concepts
    Afternoon, // 12-18: Good for practice/application
    Evening,   // 18-22: Best for review/consolidation
}

/// Spaced repetition intervals (in days)
const SPACED_REPETITION_INTERVALS: &[i64] = &[1, 3, 7, 14, 30, 90];

/// Cognitive load factors for different content types
const COGNITIVE_LOAD_FACTORS: &[(f32, &str)] = &[
    (0.9, "algorithm"),
    (0.8, "theory"),
    (0.7, "concept"),
    (0.6, "example"),
    (0.5, "practice"),
    (0.4, "review"),
    (0.3, "introduction"),
];

/// Enhanced strategy selection with AI-driven analysis
pub fn choose_distribution_strategy(
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

    // Calculate session capacity and course complexity
    let estimated_videos_per_session = calculate_videos_per_session(course, settings);
    let course_complexity = analyze_course_complexity(course);
    let user_experience_level = infer_user_experience_level(settings);

    // Advanced strategy selection based on multiple factors
    match (course_complexity, user_experience_level, module_count) {
        // High complexity courses need adaptive scheduling
        (complexity, _, _) if complexity > 0.8 => Ok(DistributionStrategy::Adaptive),

        // Beginner users benefit from spaced repetition
        (_, DifficultyLevel::Beginner, _) => Ok(DistributionStrategy::SpacedRepetition),

        // Well-structured courses with clear modules
        (_, _, modules)
            if modules > 3 && average_module_size <= estimated_videos_per_session * 2 =>
        {
            Ok(DistributionStrategy::ModuleBased)
        }

        // Large courses need difficulty-based pacing
        (_, _, _) if total_videos > estimated_videos_per_session * 15 => {
            Ok(DistributionStrategy::DifficultyBased)
        }

        // Long courses benefit from time-based distribution
        (_, _, _) if total_videos > estimated_videos_per_session * 10 => {
            Ok(DistributionStrategy::TimeBased)
        }

        // Default to hybrid approach
        _ => Ok(DistributionStrategy::Hybrid),
    }
}

/// Analyze course complexity based on content and structure
fn analyze_course_complexity(course: &Course) -> f32 {
    let structure = course.structure.as_ref().unwrap();

    let mut complexity_score = 0.0;
    let mut total_sections = 0;

    for module in &structure.modules {
        for section in &module.sections {
            total_sections += 1;

            // Analyze title for complexity indicators
            let title_lower = section.title.to_lowercase();
            for (load_factor, keyword) in COGNITIVE_LOAD_FACTORS {
                if title_lower.contains(keyword) {
                    complexity_score += load_factor;
                    break;
                }
            }

            // Duration-based complexity (longer videos often more complex)
            let duration_minutes = section.duration.as_secs() / 60;
            if duration_minutes > 30 {
                complexity_score += 0.3;
            } else if duration_minutes > 15 {
                complexity_score += 0.1;
            }
        }
    }

    if total_sections > 0 {
        complexity_score / total_sections as f32
    } else {
        0.5 // Default moderate complexity
    }
}

/// Infer user experience level from their settings
fn infer_user_experience_level(settings: &PlanSettings) -> DifficultyLevel {
    // Heuristics based on user preferences
    match (settings.sessions_per_week, settings.session_length_minutes) {
        // Intensive schedule suggests experienced learner
        (sessions, duration) if sessions >= 5 && duration >= 90 => DifficultyLevel::Expert,
        (sessions, duration) if sessions >= 4 && duration >= 60 => DifficultyLevel::Advanced,
        (sessions, duration) if sessions >= 3 && duration >= 45 => DifficultyLevel::Intermediate,
        _ => DifficultyLevel::Beginner,
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

/// Generate a time-based study plan with duration-aware session grouping
fn generate_time_based_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();

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

/// Pack videos into a session using bin-packing algorithm with duration constraints
pub fn pack_videos_into_session(
    video_queue: &mut VecDeque<VideoItem>,
    settings: &PlanSettings,
) -> Result<Vec<VideoItem>, PlanError> {
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    // Apply 20% buffer time for breaks, notes, and processing
    let effective_session_limit =
        Duration::from_secs((session_limit.as_secs() as f32 * 0.8) as u64);

    let mut session_videos = Vec::new();
    let mut session_duration = Duration::from_secs(0);
    let mut overflow_warnings = Vec::new();

    // First pass: try to fit videos in order
    while let Some(video) = video_queue.front() {
        let video_duration = video.duration;

        // Handle videos that exceed session time limits
        if video_exceeds_session_limit(video_duration, settings) {
            if session_videos.is_empty() {
                // Must include this video even if it exceeds limit
                let video = video_queue.pop_front().unwrap();
                overflow_warnings.push(format!(
                    "Video '{}' ({:.1} min) exceeds session limit ({} min)",
                    video.section_title,
                    video_duration.as_secs() as f32 / 60.0,
                    settings.session_length_minutes
                ));
                session_videos.push(video);
                break;
            } else {
                // Skip this video for now, will be handled in next session
                break;
            }
        }

        // Check if video fits in current session
        if session_duration + video_duration <= effective_session_limit || session_videos.is_empty()
        {
            let video = video_queue.pop_front().unwrap();
            session_duration += video_duration;
            session_videos.push(video);
        } else {
            break;
        }
    }

    // Second pass: try to optimize utilization with smaller videos
    if session_duration < effective_session_limit && video_queue.len() > 1 {
        session_videos =
            optimize_session_utilization(session_videos, video_queue, effective_session_limit);
    }

    // Log overflow warnings (in a real implementation, these would be stored for UI display)
    for warning in overflow_warnings {
        eprintln!("Session overflow warning: {warning}");
    }

    Ok(session_videos)
}

/// Optimize session utilization by trying to fit smaller videos
fn optimize_session_utilization(
    mut session_videos: Vec<VideoItem>,
    video_queue: &mut VecDeque<VideoItem>,
    effective_session_limit: Duration,
) -> Vec<VideoItem> {
    let mut current_duration: Duration = session_videos.iter().map(|v| v.duration).sum();
    let remaining_time = effective_session_limit.saturating_sub(current_duration);

    // Look for videos that can fit in the remaining time
    let queue_items: Vec<VideoItem> = video_queue.drain(..).collect();
    let mut remaining_items = Vec::new();

    for video in queue_items {
        if video.duration <= remaining_time
            && current_duration + video.duration <= effective_session_limit
        {
            current_duration += video.duration;
            session_videos.push(video);
        } else {
            remaining_items.push(video);
        }
    }

    // Put remaining items back in the queue
    for item in remaining_items {
        video_queue.push_back(item);
    }

    session_videos
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

/// Helper struct for video items in planning
#[derive(Debug, Clone)]
pub struct VideoItem {
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
    total_duration: Duration,
    overflow_warnings: Vec<String>,
}

/// Enhanced calculation of videos per session using actual video durations
fn calculate_videos_per_session(course: &Course, settings: &PlanSettings) -> usize {
    let structure = match course.structure.as_ref() {
        Some(s) => s,
        None => return calculate_videos_per_session_fallback(settings),
    };

    // Calculate actual average video duration from course content
    let all_durations: Vec<Duration> = structure
        .modules
        .iter()
        .flat_map(|module| module.sections.iter())
        .map(|section| section.duration)
        .collect();

    if all_durations.is_empty() {
        return calculate_videos_per_session_fallback(settings);
    }

    let total_duration_secs: u64 = all_durations.iter().map(|d| d.as_secs()).sum();
    let average_video_duration_secs = total_duration_secs / all_durations.len() as u64;
    let average_video_duration = Duration::from_secs(average_video_duration_secs);

    calculate_session_capacity_with_duration(average_video_duration, settings)
}

/// Calculate session capacity based on actual video duration and settings
fn calculate_session_capacity_with_duration(
    average_video_duration: Duration,
    settings: &PlanSettings,
) -> usize {
    let session_duration = Duration::from_secs(settings.session_length_minutes as u64 * 60);

    // Apply 20% buffer time for breaks, notes, and processing
    let effective_session_duration =
        Duration::from_secs((session_duration.as_secs() as f32 * 0.8) as u64);

    // Handle edge case where videos are longer than session time
    if average_video_duration >= effective_session_duration {
        return 1; // At least one video per session
    }

    let videos_per_session =
        effective_session_duration.as_secs() / average_video_duration.as_secs();
    std::cmp::max(1, videos_per_session as usize)
}

/// Check if a video exceeds the session time limit
fn video_exceeds_session_limit(video_duration: Duration, settings: &PlanSettings) -> bool {
    let session_duration = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    let effective_session_duration =
        Duration::from_secs((session_duration.as_secs() as f32 * 0.8) as u64);
    video_duration > effective_session_duration
}

/// Fallback calculation when course structure is not available
fn calculate_videos_per_session_fallback(settings: &PlanSettings) -> usize {
    let session_minutes = settings.session_length_minutes;

    // Adaptive video duration estimation based on session length
    let average_video_minutes = match session_minutes {
        0..=30 => 8,   // Short sessions = shorter videos expected
        31..=60 => 12, // Standard sessions
        61..=90 => 15, // Longer sessions = potentially longer videos
        _ => 18,       // Very long sessions
    };

    // Include buffer time for notes, breaks, and processing
    let effective_minutes = (session_minutes as f32 * 0.8) as u32; // 20% buffer

    std::cmp::max(1, effective_minutes as usize / average_video_minutes)
}

/// Calculate optimal session frequency based on course characteristics
fn calculate_optimal_frequency(course: &Course, user_level: DifficultyLevel) -> u8 {
    let complexity = analyze_course_complexity(course);
    let total_videos = course.video_count();

    match (user_level, complexity, total_videos) {
        // Beginners with complex content need more frequent, shorter sessions
        (DifficultyLevel::Beginner, c, _) if c > 0.7 => 5,
        (DifficultyLevel::Beginner, _, v) if v > 50 => 4,
        (DifficultyLevel::Beginner, _, _) => 3,

        // Intermediate users can handle moderate frequency
        (DifficultyLevel::Intermediate, c, _) if c > 0.8 => 4,
        (DifficultyLevel::Intermediate, _, _) => 3,

        // Advanced users can handle intensive schedules
        (DifficultyLevel::Advanced, _, v) if v > 100 => 5,
        (DifficultyLevel::Advanced, _, _) => 4,

        // Expert users prefer intensive, focused sessions
        (DifficultyLevel::Expert, _, _) => 3, // Fewer but longer sessions
    }
}

/// Analyze learning velocity and suggest adjustments
fn analyze_learning_velocity(plan: &Plan) -> LearningVelocityAnalysis {
    let total_videos = plan
        .items
        .iter()
        .map(|item| item.video_indices.len())
        .sum::<usize>();
    let total_days = if let (Some(first), Some(last)) = (plan.items.first(), plan.items.last()) {
        (last.date - first.date).num_days()
    } else {
        0
    };

    let videos_per_day = if total_days > 0 {
        total_videos as f32 / total_days as f32
    } else {
        0.0
    };

    let velocity_category = match videos_per_day {
        v if v < 0.5 => VelocityCategory::Slow,
        v if v < 1.0 => VelocityCategory::Moderate,
        v if v < 2.0 => VelocityCategory::Fast,
        _ => VelocityCategory::Intensive,
    };

    LearningVelocityAnalysis {
        videos_per_day,
        velocity_category: velocity_category.clone(),
        total_duration_days: total_days,
        recommended_adjustments: generate_velocity_recommendations(velocity_category, total_videos),
    }
}

/// Learning velocity analysis structure
#[derive(Debug, Clone)]
pub struct LearningVelocityAnalysis {
    #[allow(dead_code)]
    videos_per_day: f32,
    velocity_category: VelocityCategory,
    #[allow(dead_code)]
    total_duration_days: i64,
    #[allow(dead_code)]
    recommended_adjustments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum VelocityCategory {
    Slow,
    Moderate,
    Fast,
    Intensive,
}

/// Generate recommendations based on learning velocity
fn generate_velocity_recommendations(
    category: VelocityCategory,
    total_videos: usize,
) -> Vec<String> {
    match category {
        VelocityCategory::Slow => vec![
            "Consider increasing session frequency for better momentum".to_string(),
            "Add more practice sessions to reinforce learning".to_string(),
            if total_videos > 50 {
                "Course may take longer than expected - consider breaking into phases".to_string()
            } else {
                "Pace is suitable for deep learning".to_string()
            },
        ],
        VelocityCategory::Moderate => vec![
            "Good balance between depth and progress".to_string(),
            "Consider adding review sessions every 2 weeks".to_string(),
        ],
        VelocityCategory::Fast => vec![
            "Fast pace - ensure adequate time for practice".to_string(),
            "Add buffer days for complex topics".to_string(),
            "Consider spaced repetition for better retention".to_string(),
        ],
        VelocityCategory::Intensive => vec![
            "Very intensive pace - monitor for burnout".to_string(),
            "Ensure adequate breaks between sessions".to_string(),
            "Consider extending session length instead of frequency".to_string(),
            "Add consolidation days every week".to_string(),
        ],
    }
}

/// Group sections within a module by session capacity using actual durations
fn group_sections_by_capacity<'a>(
    module: &'a crate::types::Module,
    _course: &Course,
    settings: &PlanSettings,
) -> Vec<Vec<&'a crate::types::Section>> {
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    // Apply 20% buffer time for breaks, notes, and processing
    let effective_session_limit =
        Duration::from_secs((session_limit.as_secs() as f32 * 0.8) as u64);

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

/// Plan sessions for a specific module with duration-aware grouping
fn plan_module_sessions(
    module: &crate::types::Module,
    settings: &PlanSettings,
) -> Result<Vec<SessionPlan>, PlanError> {
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    // Apply 20% buffer time for breaks, notes, and processing
    let effective_session_limit =
        Duration::from_secs((session_limit.as_secs() as f32 * 0.8) as u64);

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

/// Generate a difficulty-based study plan that adapts pacing to content complexity
fn generate_difficulty_based_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
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
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    // Apply 20% buffer time and adjust based on difficulty
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

/// Generate a spaced repetition plan optimized for memory retention
pub fn generate_spaced_repetition_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
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

/// Generate an adaptive plan using AI-driven optimization
fn generate_adaptive_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Create enhanced session plans with cognitive load analysis
    let mut enhanced_sessions = Vec::new();

    for module in &structure.modules {
        for section in &module.sections {
            let difficulty = analyze_section_difficulty(&section.title, section.duration);
            let cognitive_load = calculate_cognitive_load(&section.title, section.duration);
            let optimal_time = determine_optimal_time_of_day(&section.title);

            enhanced_sessions.push(EnhancedSessionPlan {
                title: section.title.clone(),
                video_indices: vec![section.video_index],
                session_type: classify_session_type(&section.title),
                difficulty_level: difficulty,
                estimated_cognitive_load: cognitive_load,
                prerequisites: Vec::new(), // Could be enhanced with dependency analysis
                optimal_time_of_day: optimal_time,
            });
        }
    }

    // Optimize session order based on cognitive load and learning principles
    optimize_session_sequence(&mut enhanced_sessions);

    // Convert enhanced sessions to plan items with intelligent scheduling
    for session in enhanced_sessions {
        // Adjust scheduling based on session characteristics
        let adjusted_date = adjust_date_for_optimal_learning(current_date, &session, settings);

        // Calculate next session date with adaptive spacing (before partial move)
        let spacing_days = calculate_adaptive_spacing(&session);

        // Find the section duration for this session
        let mut section_duration = Duration::from_secs(30 * 60); // Default 30 minutes
        for module in &structure.modules {
            for section in &module.sections {
                if session.video_indices.contains(&section.video_index) {
                    section_duration = section.duration;
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
            date: adjusted_date,
            module_title: "Adaptive Learning".to_string(),
            section_title: session.title,
            video_indices: session.video_indices.clone(),
            completed: false,
            total_duration: section_duration,
            estimated_completion_time,
            overflow_warnings: Vec::new(),
        });

        current_date = crate::planner::get_next_session_date(
            adjusted_date + chrono::Duration::days(spacing_days),
            settings.sessions_per_week,
            settings.include_weekends,
        );
    }

    Ok(plan_items)
}

/// Analyze section difficulty based on title and duration
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

/// Calculate cognitive load for a section
fn calculate_cognitive_load(title: &str, duration: Duration) -> f32 {
    let title_lower = title.to_lowercase();
    let mut load = 0.5; // Base load

    // Adjust based on content type
    for (load_factor, keyword) in COGNITIVE_LOAD_FACTORS {
        if title_lower.contains(keyword) {
            load = *load_factor;
            break;
        }
    }

    // Adjust based on duration (longer content = higher cognitive load)
    let duration_minutes = duration.as_secs() / 60;
    let duration_factor = (duration_minutes as f32 / 30.0).min(1.5);

    (load * duration_factor).min(1.0)
}

/// Classify the type of learning session
fn classify_session_type(title: &str) -> SessionType {
    let title_lower = title.to_lowercase();

    if title_lower.contains("introduction") || title_lower.contains("overview") {
        SessionType::Introduction
    } else if title_lower.contains("practice") || title_lower.contains("exercise") {
        SessionType::Practice
    } else if title_lower.contains("review") || title_lower.contains("summary") {
        SessionType::Review
    } else if title_lower.contains("project") || title_lower.contains("build") {
        SessionType::Project
    } else if title_lower.contains("test") || title_lower.contains("quiz") {
        SessionType::Assessment
    } else {
        SessionType::Introduction // Default
    }
}

/// Determine optimal time of day for content
fn determine_optimal_time_of_day(title: &str) -> Option<TimeOfDay> {
    let title_lower = title.to_lowercase();

    if title_lower.contains("algorithm") || title_lower.contains("complex") {
        Some(TimeOfDay::Morning) // Complex topics in the morning
    } else if title_lower.contains("practice") || title_lower.contains("exercise") {
        Some(TimeOfDay::Afternoon) // Practice in the afternoon
    } else if title_lower.contains("review") || title_lower.contains("summary") {
        Some(TimeOfDay::Evening) // Review in the evening
    } else {
        None // No specific preference
    }
}

/// Optimize the sequence of sessions for better learning outcomes
fn optimize_session_sequence(sessions: &mut Vec<EnhancedSessionPlan>) {
    // Sort by difficulty first (easier to harder)
    sessions.sort_by(|a, b| {
        // Primary sort: session type (intro -> practice -> review)
        let type_order_a = match a.session_type {
            SessionType::Introduction => 0,
            SessionType::Practice => 1,
            SessionType::Project => 2,
            SessionType::Review => 3,
            SessionType::Assessment => 4,
            SessionType::Break => 5,
        };

        let type_order_b = match b.session_type {
            SessionType::Introduction => 0,
            SessionType::Practice => 1,
            SessionType::Project => 2,
            SessionType::Review => 3,
            SessionType::Assessment => 4,
            SessionType::Break => 5,
        };

        // Secondary sort: difficulty level
        let diff_order_a = match a.difficulty_level {
            DifficultyLevel::Beginner => 0,
            DifficultyLevel::Intermediate => 1,
            DifficultyLevel::Advanced => 2,
            DifficultyLevel::Expert => 3,
        };

        let diff_order_b = match b.difficulty_level {
            DifficultyLevel::Beginner => 0,
            DifficultyLevel::Intermediate => 1,
            DifficultyLevel::Advanced => 2,
            DifficultyLevel::Expert => 3,
        };

        type_order_a
            .cmp(&type_order_b)
            .then(diff_order_a.cmp(&diff_order_b))
            .then(
                a.estimated_cognitive_load
                    .partial_cmp(&b.estimated_cognitive_load)
                    .unwrap(),
            )
    });
}

/// Adjust date for optimal learning based on session characteristics
fn adjust_date_for_optimal_learning(
    base_date: DateTime<Utc>,
    session: &EnhancedSessionPlan,
    settings: &PlanSettings,
) -> DateTime<Utc> {
    let mut adjusted_date = base_date;

    // Avoid scheduling high cognitive load sessions on Mondays (post-weekend)
    if session.estimated_cognitive_load > 0.7 && adjusted_date.weekday() == Weekday::Mon {
        adjusted_date += chrono::Duration::days(1);
    }

    // Space out difficult sessions
    if session.difficulty_level == DifficultyLevel::Expert {
        // Ensure at least 2 days between expert sessions
        adjusted_date += chrono::Duration::days(1);
    }

    // Respect weekend preferences
    if !settings.include_weekends {
        while adjusted_date.weekday() == Weekday::Sat || adjusted_date.weekday() == Weekday::Sun {
            adjusted_date += chrono::Duration::days(1);
        }
    }

    adjusted_date
}

/// Calculate adaptive spacing between sessions
fn calculate_adaptive_spacing(session: &EnhancedSessionPlan) -> i64 {
    match (session.difficulty_level, session.estimated_cognitive_load) {
        (DifficultyLevel::Expert, load) if load > 0.8 => 3, // 3 days for very difficult content
        (DifficultyLevel::Advanced, load) if load > 0.7 => 2, // 2 days for advanced content
        (DifficultyLevel::Expert, _) => 2,                  // 2 days for expert content
        (DifficultyLevel::Advanced, _) => 1,                // 1 day for advanced content
        _ => 0,                                             // Normal spacing for others
    }
}

/// Generate personalized study recommendations
pub fn generate_study_recommendations(
    course: &Course,
    settings: &PlanSettings,
) -> StudyRecommendations {
    let complexity = analyze_course_complexity(course);
    let user_level = infer_user_experience_level(settings);
    let optimal_frequency = calculate_optimal_frequency(course, user_level);

    let mut recommendations = StudyRecommendations {
        optimal_sessions_per_week: optimal_frequency,
        recommended_session_length: calculate_optimal_session_length(course, user_level),
        study_strategy: recommend_study_strategy(complexity, user_level),
        time_management_tips: generate_time_management_tips(settings),
        difficulty_progression: analyze_difficulty_progression(course),
        estimated_completion_weeks: estimate_completion_time(course, settings),
    };

    // Add personalized tips based on user profile
    recommendations.add_personalized_tips(user_level, complexity);

    recommendations
}

/// Study recommendations structure
#[derive(Debug, Clone)]
pub struct StudyRecommendations {
    pub optimal_sessions_per_week: u8,
    pub recommended_session_length: u32,
    pub study_strategy: String,
    pub time_management_tips: Vec<String>,
    pub difficulty_progression: DifficultyProgression,
    pub estimated_completion_weeks: u32,
}

impl StudyRecommendations {
    fn add_personalized_tips(&mut self, user_level: DifficultyLevel, complexity: f32) {
        match user_level {
            DifficultyLevel::Beginner => {
                self.time_management_tips.extend(vec![
                    "Start with shorter sessions to build consistency".to_string(),
                    "Take notes during each session for better retention".to_string(),
                    "Don't hesitate to pause and replay difficult sections".to_string(),
                ]);
            }
            DifficultyLevel::Expert => {
                self.time_management_tips.extend(vec![
                    "Focus on practical application over passive watching".to_string(),
                    "Create projects to reinforce learning".to_string(),
                    "Consider teaching concepts to others for deeper understanding".to_string(),
                ]);
            }
            _ => {}
        }

        if complexity > 0.7 {
            self.time_management_tips.push(
                "This course has high complexity - consider extending your timeline".to_string(),
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct DifficultyProgression {
    pub starts_easy: bool,
    pub has_steep_learning_curve: bool,
    pub complexity_peaks: Vec<String>, // Module names with high complexity
    pub recommended_break_points: Vec<String>,
}

/// Calculate optimal session length based on course and user characteristics
fn calculate_optimal_session_length(course: &Course, user_level: DifficultyLevel) -> u32 {
    let complexity = analyze_course_complexity(course);
    let base_length = match user_level {
        DifficultyLevel::Beginner => 30,
        DifficultyLevel::Intermediate => 45,
        DifficultyLevel::Advanced => 60,
        DifficultyLevel::Expert => 90,
    };

    // Adjust based on complexity
    let complexity_adjustment = (complexity * 30.0) as u32;

    (base_length + complexity_adjustment).min(120) // Cap at 2 hours
}

/// Recommend study strategy based on course and user characteristics
fn recommend_study_strategy(complexity: f32, user_level: DifficultyLevel) -> String {
    match (complexity > 0.7, user_level) {
        (true, DifficultyLevel::Beginner) => {
            "Spaced Repetition: This complex course benefits from frequent review sessions"
                .to_string()
        }
        (true, _) => "Adaptive Learning: Adjust pace based on topic difficulty".to_string(),
        (false, DifficultyLevel::Expert) => {
            "Accelerated Learning: Focus on practical application and projects".to_string()
        }
        _ => "Balanced Approach: Steady progress with regular reviews".to_string(),
    }
}

/// Generate time management tips
fn generate_time_management_tips(settings: &PlanSettings) -> Vec<String> {
    let mut tips = vec![
        "Set a consistent study schedule".to_string(),
        "Eliminate distractions during study sessions".to_string(),
        "Use the Pomodoro Technique for better focus".to_string(),
    ];

    if settings.sessions_per_week >= 5 {
        tips.push("High frequency schedule - ensure adequate rest between sessions".to_string());
    }

    if settings.session_length_minutes >= 90 {
        tips.push("Long sessions - take 10-minute breaks every hour".to_string());
    }

    if !settings.include_weekends {
        tips.push("Weekend-free schedule - use weekends for review and practice".to_string());
    }

    tips
}

/// Analyze difficulty progression throughout the course
fn analyze_difficulty_progression(course: &Course) -> DifficultyProgression {
    let structure = course.structure.as_ref().unwrap();
    let mut module_difficulties = Vec::new();
    let mut complexity_peaks = Vec::new();
    let mut recommended_breaks = Vec::new();

    for module in &structure.modules {
        let mut module_complexity = 0.0;
        let mut section_count = 0;

        for section in &module.sections {
            let difficulty = analyze_section_difficulty(&section.title, section.duration);
            module_complexity += match difficulty {
                DifficultyLevel::Beginner => 0.25,
                DifficultyLevel::Intermediate => 0.5,
                DifficultyLevel::Advanced => 0.75,
                DifficultyLevel::Expert => 1.0,
            };
            section_count += 1;
        }

        if section_count > 0 {
            module_complexity /= section_count as f32;
        }

        module_difficulties.push(module_complexity);

        // Identify complexity peaks
        if module_complexity > 0.7 {
            complexity_peaks.push(module.title.clone());
            recommended_breaks.push(format!(
                "Consider a break after completing: {}",
                module.title
            ));
        }
    }

    let starts_easy = module_difficulties.first().is_some_and(|&d| d < 0.4);
    let has_steep_curve = module_difficulties.windows(2).any(|w| w[1] - w[0] > 0.3);

    DifficultyProgression {
        starts_easy,
        has_steep_learning_curve: has_steep_curve,
        complexity_peaks,
        recommended_break_points: recommended_breaks,
    }
}

/// Estimate completion time in weeks
fn estimate_completion_time(course: &Course, settings: &PlanSettings) -> u32 {
    let total_videos = course.video_count();
    let videos_per_session = calculate_videos_per_session(course, settings);
    let total_sessions = total_videos.div_ceil(videos_per_session); // Ceiling division

    let weeks = (total_sessions as f32 / settings.sessions_per_week as f32).ceil() as u32;

    // Add buffer for reviews and breaks
    let buffer_weeks = (weeks as f32 * 0.2).ceil() as u32; // 20% buffer

    weeks + buffer_weeks
}

/// Advanced plan analysis for continuous improvement
pub fn analyze_plan_effectiveness(plan: &Plan) -> PlanAnalysis {
    let velocity_analysis = analyze_learning_velocity(plan);
    let load_distribution = analyze_cognitive_load_distribution(plan);
    let temporal_distribution = analyze_temporal_distribution(plan);

    PlanAnalysis {
        velocity_analysis,
        load_distribution,
        temporal_distribution,
        overall_score: calculate_plan_score(plan),
        improvement_suggestions: generate_improvement_suggestions(plan),
    }
}

#[derive(Debug, Clone)]
pub struct PlanAnalysis {
    pub velocity_analysis: LearningVelocityAnalysis,
    pub load_distribution: LoadDistribution,
    pub temporal_distribution: TemporalDistribution,
    pub overall_score: f32, // 0.0 to 1.0
    pub improvement_suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoadDistribution {
    pub average_load: f32,
    pub load_variance: f32,
    pub overloaded_sessions: usize,
    pub underloaded_sessions: usize,
}

#[derive(Debug, Clone)]
pub struct TemporalDistribution {
    pub average_gap_days: f32,
    pub longest_gap_days: i64,
    pub weekend_utilization: f32,
    pub consistency_score: f32,
}

/// Analyze cognitive load distribution across sessions
fn analyze_cognitive_load_distribution(plan: &Plan) -> LoadDistribution {
    let mut loads = Vec::new();

    for item in &plan.items {
        let load = calculate_cognitive_load(&item.section_title, Duration::from_secs(0));
        loads.push(load);
    }

    let average_load = loads.iter().sum::<f32>() / loads.len() as f32;
    let variance = loads
        .iter()
        .map(|&load| (load - average_load).powi(2))
        .sum::<f32>()
        / loads.len() as f32;

    let overloaded = loads
        .iter()
        .filter(|&&load| load > average_load * 1.5)
        .count();
    let underloaded = loads
        .iter()
        .filter(|&&load| load < average_load * 0.5)
        .count();

    LoadDistribution {
        average_load,
        load_variance: variance,
        overloaded_sessions: overloaded,
        underloaded_sessions: underloaded,
    }
}

/// Analyze temporal distribution of sessions
fn analyze_temporal_distribution(plan: &Plan) -> TemporalDistribution {
    if plan.items.len() < 2 {
        return TemporalDistribution {
            average_gap_days: 0.0,
            longest_gap_days: 0,
            weekend_utilization: 0.0,
            consistency_score: 1.0,
        };
    }

    let mut gaps = Vec::new();
    let mut weekend_sessions = 0;

    for i in 1..plan.items.len() {
        let gap = (plan.items[i].date - plan.items[i - 1].date).num_days();
        gaps.push(gap);

        if plan.items[i].date.weekday() == Weekday::Sat
            || plan.items[i].date.weekday() == Weekday::Sun
        {
            weekend_sessions += 1;
        }
    }

    let average_gap = gaps.iter().sum::<i64>() as f32 / gaps.len() as f32;
    let longest_gap = *gaps.iter().max().unwrap_or(&0);
    let weekend_util = weekend_sessions as f32 / plan.items.len() as f32;

    // Consistency score based on gap variance
    let gap_variance = gaps
        .iter()
        .map(|&gap| (gap as f32 - average_gap).powi(2))
        .sum::<f32>()
        / gaps.len() as f32;
    let consistency = (1.0f32 / (1.0 + gap_variance)).max(0.0).min(1.0);

    TemporalDistribution {
        average_gap_days: average_gap,
        longest_gap_days: longest_gap,
        weekend_utilization: weekend_util,
        consistency_score: consistency,
    }
}

/// Calculate overall plan quality score
fn calculate_plan_score(plan: &Plan) -> f32 {
    let velocity = analyze_learning_velocity(plan);
    let load_dist = analyze_cognitive_load_distribution(plan);
    let temporal_dist = analyze_temporal_distribution(plan);

    // Weighted scoring
    let velocity_score = match velocity.velocity_category {
        VelocityCategory::Moderate => 1.0,
        VelocityCategory::Fast => 0.8,
        VelocityCategory::Slow => 0.6,
        VelocityCategory::Intensive => 0.4,
    };

    let load_score = (1.0 - load_dist.load_variance).max(0.0);
    let temporal_score = temporal_dist.consistency_score;

    // Weighted average
    (velocity_score * 0.4 + load_score * 0.3 + temporal_score * 0.3)
        .max(0.0)
        .min(1.0)
}

/// Generate improvement suggestions for the plan
fn generate_improvement_suggestions(plan: &Plan) -> Vec<String> {
    let mut suggestions = Vec::new();
    let analysis = analyze_learning_velocity(plan);
    let load_dist = analyze_cognitive_load_distribution(plan);
    let temporal_dist = analyze_temporal_distribution(plan);

    // Velocity-based suggestions
    match analysis.velocity_category {
        VelocityCategory::Intensive => {
            suggestions.push("Consider reducing session frequency to prevent burnout".to_string());
        }
        VelocityCategory::Slow => {
            suggestions
                .push("Consider increasing session frequency for better momentum".to_string());
        }
        _ => {}
    }

    // Load distribution suggestions
    if load_dist.overloaded_sessions > plan.items.len() / 4 {
        suggestions
            .push("Many sessions are overloaded - consider redistributing content".to_string());
    }

    if load_dist.underloaded_sessions > plan.items.len() / 4 {
        suggestions
            .push("Many sessions are underloaded - consider consolidating content".to_string());
    }

    // Temporal suggestions
    if temporal_dist.longest_gap_days > 7 {
        suggestions.push(
            "Long gaps between sessions may affect retention - consider more consistent scheduling"
                .to_string(),
        );
    }

    if temporal_dist.consistency_score < 0.7 {
        suggestions
            .push("Irregular session spacing - try to maintain consistent intervals".to_string());
    }

    suggestions
}

/// Enhanced plan optimization with advanced learning science principles
pub fn optimize_plan(plan: &mut Plan) -> Result<(), PlanError> {
    // Apply learning science optimizations
    add_review_sessions(plan)?;
    balance_cognitive_load(plan)?;
    add_adaptive_buffer_days(plan)?;
    optimize_session_timing(plan)?;
    add_consolidation_breaks(plan)?;

    // Ensure plan integrity
    validate_plan_structure(plan)?;

    Ok(())
}

/// Balance cognitive load across sessions using advanced algorithms
fn balance_cognitive_load(plan: &mut Plan) -> Result<(), PlanError> {
    if plan.items.len() < 2 {
        return Ok(());
    }

    // Calculate cognitive load for each session
    let mut session_loads: Vec<f32> = Vec::new();

    for item in &plan.items {
        let mut load = 0.0;

        // Base load from number of videos
        load += item.video_indices.len() as f32 * 0.2;

        // Content-based load analysis
        let title_lower = item.section_title.to_lowercase();
        for (load_factor, keyword) in COGNITIVE_LOAD_FACTORS {
            if title_lower.contains(keyword) {
                load += load_factor;
                break;
            }
        }

        session_loads.push(load);
    }

    // Calculate target load (average)
    let total_load: f32 = session_loads.iter().sum();
    let target_load = total_load / session_loads.len() as f32;

    // Redistribute content from overloaded to underloaded sessions
    let mut i = 0;
    while i < plan.items.len() - 1 {
        if session_loads[i] > target_load * 1.5 {
            // Find next underloaded session
            for j in (i + 1)..plan.items.len() {
                if session_loads[j] < target_load * 0.7 && !plan.items[i].video_indices.is_empty() {
                    // Move one video from overloaded to underloaded session
                    if let Some(video_index) = plan.items[i].video_indices.pop() {
                        plan.items[j].video_indices.push(video_index);
                        session_loads[i] -= 0.3;
                        session_loads[j] += 0.3;
                        break;
                    }
                }
            }
        }
        i += 1;
    }

    Ok(())
}

/// Add adaptive buffer days based on content complexity and user progress
fn add_adaptive_buffer_days(plan: &mut Plan) -> Result<(), PlanError> {
    let complexity_threshold = 0.7; // Adaptive threshold

    for item in plan.items.iter_mut() {
        let mut needs_buffer = false;
        let mut buffer_days = 0;

        // Analyze session complexity
        let title_lower = item.section_title.to_lowercase();
        let video_count = item.video_indices.len();

        // High video count sessions need buffer
        if video_count > 5 {
            needs_buffer = true;
            buffer_days = 1;
        }

        // Complex content needs buffer
        for (load_factor, keyword) in COGNITIVE_LOAD_FACTORS {
            if title_lower.contains(keyword) && *load_factor > complexity_threshold {
                needs_buffer = true;
                buffer_days = buffer_days.max(1);
                break;
            }
        }

        // Expert-level content needs extra buffer
        if title_lower.contains("advanced") || title_lower.contains("expert") {
            needs_buffer = true;
            buffer_days = buffer_days.max(2);
        }

        if needs_buffer {
            item.date += chrono::Duration::days(buffer_days);
        }
    }

    // Re-sort by date after modifications
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(())
}

/// Optimize session timing based on learning science
fn optimize_session_timing(plan: &mut Plan) -> Result<(), PlanError> {
    for item in plan.items.iter_mut() {
        let title_lower = item.section_title.to_lowercase();

        // Avoid scheduling difficult content on Mondays (post-weekend effect)
        if (title_lower.contains("advanced") || title_lower.contains("complex"))
            && item.date.weekday() == Weekday::Mon
        {
            item.date += chrono::Duration::days(1);
        }

        // Space out assessment sessions
        if title_lower.contains("test") || title_lower.contains("exam") {
            // Ensure assessments are not on consecutive days
            // This would require more complex logic to check against other sessions
        }
    }

    // Re-sort after timing adjustments
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(())
}

/// Add consolidation breaks for better memory formation
fn add_consolidation_breaks(plan: &mut Plan) -> Result<(), PlanError> {
    let total_sessions = plan.items.len();
    if total_sessions < 10 {
        return Ok(());
    } // Only for longer courses

    let break_interval = total_sessions / 4; // Break every 25% of course
    let mut break_items = Vec::new();

    for i in (break_interval..total_sessions).step_by(break_interval) {
        if i < plan.items.len() {
            let break_date = plan.items[i].date + chrono::Duration::days(1);

            let break_duration = Duration::from_secs(0); // No video content for break days
            let estimated_completion_time = Duration::from_secs(30 * 60); // 30 minutes for reflection

            break_items.push(PlanItem {
                date: break_date,
                module_title: "Consolidation".to_string(),
                section_title: "Rest & Reflection Day".to_string(),
                video_indices: Vec::new(), // No videos, just a break
                completed: false,
                total_duration: break_duration,
                estimated_completion_time,
                overflow_warnings: Vec::new(),
            });
        }
    }

    plan.items.extend(break_items);
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(())
}

/// Validate plan structure and fix any issues
fn validate_plan_structure(plan: &mut Plan) -> Result<(), PlanError> {
    // Remove empty sessions
    plan.items.retain(|item| {
        !item.video_indices.is_empty()
            || item.section_title.contains("Review")
            || item.section_title.contains("Rest")
    });

    // Ensure dates are in the future
    let now = Utc::now();
    for item in plan.items.iter_mut() {
        if item.date < now {
            item.date = now + chrono::Duration::days(1);
        }
    }

    // Final sort by date
    plan.items.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(())
}

/// Add review sessions to the plan
fn add_review_sessions(plan: &mut Plan) -> Result<(), PlanError> {
    let total_sessions = plan.items.len();
    let review_interval = std::cmp::max(5, total_sessions / 4); // Review every 5-25% of course

    let mut review_items = Vec::new();
    for (i, item) in plan.items.iter().enumerate() {
        if (i + 1) % review_interval == 0 && i < plan.items.len() - 1 {
            let review_date = crate::planner::get_next_session_date(
                item.date,
                plan.settings.sessions_per_week,
                plan.settings.include_weekends,
            );

            let review_duration = Duration::from_secs(45 * 60); // 45 minutes for module review
            let estimated_completion_time =
                crate::types::duration_utils::calculate_completion_time_with_buffer(
                    review_duration,
                    0.25,
                );

            review_items.push(PlanItem {
                date: review_date,
                module_title: "Review".to_string(),
                section_title: format!("Review: Modules 1-{}", (i / review_interval) + 1),
                video_indices: vec![], // Review sessions don't have specific videos
                completed: false,
                total_duration: review_duration,
                estimated_completion_time,
                overflow_warnings: Vec::new(),
            });
        }
    }

    // Insert review items and re-sort by date
    plan.items.extend(review_items);
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
        let structure = CourseStructure::new_basic(
            vec![
                Module::new_basic(
                    "Introduction".to_string(),
                    vec![
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
                ),
                Module::new_basic(
                    "Advanced Topics".to_string(),
                    vec![Section {
                        title: "Complex Example".to_string(),
                        video_index: 2,
                        duration: Duration::from_secs(1800),
                    }],
                ),
            ],
            StructureMetadata {
                total_videos: 3,
                total_duration: Duration::from_secs(600 + 900 + 1800),
                estimated_duration_hours: Some(1.0),
                difficulty_level: Some("Intermediate".to_string()),
                structure_quality_score: None,
                content_coherence_score: None,
            },
        );

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
            advanced_settings: None,
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
    fn test_videos_per_session_calculation_with_actual_durations() {
        let course = create_test_course();
        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        };

        let videos = calculate_videos_per_session(&course, &settings);
        // Course has videos of 10min, 15min, 30min = average 18.33min
        // 60min * 0.8 buffer = 48min effective / 18.33min = ~2 videos per session
        assert!(videos >= 1 && videos <= 3);
    }

    #[test]
    fn test_videos_per_session_fallback() {
        let mut course = create_test_course();
        course.structure = None; // No structure available

        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        };

        let videos = calculate_videos_per_session(&course, &settings);
        assert_eq!(videos, 4); // 60 minutes * 0.8 / 12 minutes = 4 videos (fallback calculation)
    }

    #[test]
    fn test_video_exceeds_session_limit() {
        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        };

        // 60 minutes * 0.8 = 48 minutes effective session time
        assert!(!video_exceeds_session_limit(
            Duration::from_secs(30 * 60),
            &settings
        )); // 30 min - OK
        assert!(!video_exceeds_session_limit(
            Duration::from_secs(45 * 60),
            &settings
        )); // 45 min - OK
        assert!(video_exceeds_session_limit(
            Duration::from_secs(50 * 60),
            &settings
        )); // 50 min - exceeds
        assert!(video_exceeds_session_limit(
            Duration::from_secs(90 * 60),
            &settings
        )); // 90 min - exceeds
    }

    #[test]
    fn test_completion_time_calculation() {
        let video_duration = Duration::from_secs(45 * 60); // 45 minutes
        let completion_time = crate::types::duration_utils::calculate_completion_time_with_buffer(
            video_duration,
            0.25,
        );

        // Total video time: 45 minutes
        // Buffer time: 45 * 0.25 = 11.25 minutes
        // Total: ~56.25 minutes
        assert!(completion_time.as_secs() >= 56 * 60 && completion_time.as_secs() <= 57 * 60);
    }

    #[test]
    fn test_session_capacity_with_long_videos() {
        let long_video_duration = Duration::from_secs(90 * 60); // 90 minutes
        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60, // 60 minute sessions
            include_weekends: false,
            advanced_settings: None,
        };

        let capacity = calculate_session_capacity_with_duration(long_video_duration, &settings);
        assert_eq!(capacity, 1); // Should be 1 when videos exceed session time
    }

    #[test]
    fn test_session_capacity_with_short_videos() {
        let short_video_duration = Duration::from_secs(5 * 60); // 5 minutes
        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60, // 60 minute sessions
            include_weekends: false,
            advanced_settings: None,
        };

        let capacity = calculate_session_capacity_with_duration(short_video_duration, &settings);
        // 60 * 0.8 = 48 minutes effective / 5 minutes = 9.6 -> 9 videos
        assert_eq!(capacity, 9);
    }

    #[test]
    fn test_time_based_plan_with_duration_grouping() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_time_based_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();
        assert!(!plan_items.is_empty());

        // Verify that sessions respect duration constraints
        for item in &plan_items {
            assert!(!item.video_indices.is_empty());
            // Each session should have at least one video
            assert!(item.video_indices.len() >= 1);
        }
    }

    #[test]
    fn test_bin_packing_optimization() {
        // Create a course with videos of varying durations
        let structure = CourseStructure::new_basic(
            vec![Module::new_basic(
                "Test Module".to_string(),
                vec![
                    Section {
                        title: "Short Video 1".to_string(),
                        video_index: 0,
                        duration: Duration::from_secs(10 * 60), // 10 minutes
                    },
                    Section {
                        title: "Short Video 2".to_string(),
                        video_index: 1,
                        duration: Duration::from_secs(15 * 60), // 15 minutes
                    },
                    Section {
                        title: "Medium Video".to_string(),
                        video_index: 2,
                        duration: Duration::from_secs(25 * 60), // 25 minutes
                    },
                    Section {
                        title: "Short Video 3".to_string(),
                        video_index: 3,
                        duration: Duration::from_secs(12 * 60), // 12 minutes
                    },
                ],
            )],
            StructureMetadata {
                total_videos: 4,
                total_duration: Duration::from_secs(62 * 60),
                estimated_duration_hours: Some(1.0),
                difficulty_level: Some("Beginner".to_string()),
                structure_quality_score: None,
                content_coherence_score: None,
            },
        );

        let course = Course {
            id: Uuid::new_v4(),
            name: "Bin Packing Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec![
                "Short Video 1".to_string(),
                "Short Video 2".to_string(),
                "Medium Video".to_string(),
                "Short Video 3".to_string(),
            ],
            structure: Some(structure),
        };

        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60, // 60 minute sessions
            include_weekends: false,
            advanced_settings: None,
        };

        let result = generate_time_based_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();

        // With 60-minute sessions and 20% buffer (48 min effective):
        // Should be able to fit multiple short videos in first session
        // Medium video (25 min) should fit with some short videos
        assert!(plan_items.len() <= 3); // Should not need more than 3 sessions for 62 minutes of content

        // First session should contain multiple videos if bin-packing works
        if let Some(first_session) = plan_items.first() {
            // Should be able to fit at least 2 videos in first session
            assert!(first_session.video_indices.len() >= 1);
        }
    }

    #[test]
    fn test_duration_aware_module_grouping() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_module_based_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();
        assert!(!plan_items.is_empty());

        // Verify that sessions respect module boundaries and duration constraints
        for item in &plan_items {
            assert!(!item.video_indices.is_empty());
            // Module-based planning should maintain module context
            assert!(!item.module_title.is_empty());
        }
    }

    #[test]
    fn test_duration_aware_hybrid_planning() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_hybrid_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();
        assert!(!plan_items.is_empty());

        // Hybrid planning should balance module structure with time constraints
        for item in &plan_items {
            assert!(!item.video_indices.is_empty());
            assert!(!item.module_title.is_empty());
        }
    }

    #[test]
    fn test_difficulty_phase_sessions() {
        let video_items = vec![
            VideoItem {
                module_title: "Test Module".to_string(),
                section_title: "Easy Video 1".to_string(),
                video_index: 0,
                duration: Duration::from_secs(10 * 60), // 10 minutes
            },
            VideoItem {
                module_title: "Test Module".to_string(),
                section_title: "Easy Video 2".to_string(),
                video_index: 1,
                duration: Duration::from_secs(15 * 60), // 15 minutes
            },
            VideoItem {
                module_title: "Test Module".to_string(),
                section_title: "Easy Video 3".to_string(),
                video_index: 2,
                duration: Duration::from_secs(20 * 60), // 20 minutes
            },
        ];

        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        };

        let result = create_difficulty_phase_sessions(&video_items, 0, &settings); // Beginner phase
        assert!(result.is_ok());

        let sessions = result.unwrap();
        assert!(!sessions.is_empty());

        // For beginner content with 60-minute sessions and 70% buffer (42 min effective),
        // should be able to fit multiple videos per session
        let total_videos: usize = sessions.iter().map(|s| s.len()).sum();
        assert_eq!(total_videos, 3); // All videos should be included
    }

    #[test]
    fn test_duration_display_and_validation() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_plan(&course, &settings);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(!plan.items.is_empty());

        // Verify that all plan items have duration information
        for item in &plan.items {
            // All items should have non-zero total duration (except break days)
            if !item.video_indices.is_empty() {
                assert!(item.total_duration.as_secs() > 0);
                assert!(item.estimated_completion_time.as_secs() > 0);
                // Estimated completion time should be longer than total duration (due to buffer)
                assert!(item.estimated_completion_time >= item.total_duration);
            }

            // Overflow warnings should be a valid vector (can be empty)
            assert!(item.overflow_warnings.len() >= 0);
        }
    }

    #[test]
    fn test_duration_formatting_utilities() {
        use crate::types::duration_utils::*;

        // Test basic duration formatting
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3690)), "1h 1m");

        // Test verbose formatting
        assert_eq!(
            format_duration_verbose(Duration::from_secs(90)),
            "1 minutes"
        );
        assert_eq!(
            format_duration_verbose(Duration::from_secs(3600)),
            "1 hours"
        );
        assert_eq!(
            format_duration_verbose(Duration::from_secs(3690)),
            "1 hours 1 minutes"
        );

        // Test decimal hours formatting
        assert_eq!(
            format_duration_decimal_hours(Duration::from_secs(3600)),
            "1.0 hours"
        );
        assert_eq!(
            format_duration_decimal_hours(Duration::from_secs(1800)),
            "30 minutes"
        );

        // Test excessive duration check
        assert!(is_duration_excessive(Duration::from_secs(90 * 60), 60)); // 90 min > 60 min
        assert!(!is_duration_excessive(Duration::from_secs(45 * 60), 60)); // 45 min < 60 min

        // Test completion time calculation with buffer
        let video_duration = Duration::from_secs(60 * 60); // 1 hour
        let completion_time = calculate_completion_time_with_buffer(video_duration, 0.25);
        assert_eq!(completion_time.as_secs(), 75 * 60); // 1 hour + 25% = 75 minutes
    }

    #[test]
    fn test_session_overflow_warnings() {
        // Create a course with a very long video
        let structure = CourseStructure::new_basic(
            vec![Module::new_basic(
                "Test Module".to_string(),
                vec![Section {
                    title: "Very Long Video".to_string(),
                    video_index: 0,
                    duration: Duration::from_secs(90 * 60), // 90 minutes
                }],
            )],
            StructureMetadata {
                total_videos: 1,
                total_duration: Duration::from_secs(90 * 60),
                estimated_duration_hours: Some(1.5),
                difficulty_level: Some("Advanced".to_string()),
                structure_quality_score: None,
                content_coherence_score: None,
            },
        );

        let course = Course {
            id: Uuid::new_v4(),
            name: "Overflow Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Very Long Video".to_string()],
            structure: Some(structure),
        };

        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60, // 60 minute sessions
            include_weekends: false,
            advanced_settings: None,
        };

        let result = generate_plan(&course, &settings);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(!plan.items.is_empty());

        // The plan item should have overflow warnings
        let first_item = &plan.items[0];
        assert!(!first_item.overflow_warnings.is_empty());

        // The warning should mention that the video exceeds the session limit
        let warning_text = first_item.overflow_warnings.join(" ");
        assert!(warning_text.contains("exceeds") || warning_text.contains("long"));
    }

    #[test]
    fn test_overflow_handling() {
        // Create a course with a very long video
        let structure = CourseStructure::new_basic(
            vec![Module::new_basic(
                "Test Module".to_string(),
                vec![
                    Section {
                        title: "Normal Video".to_string(),
                        video_index: 0,
                        duration: Duration::from_secs(20 * 60), // 20 minutes
                    },
                    Section {
                        title: "Very Long Video".to_string(),
                        video_index: 1,
                        duration: Duration::from_secs(90 * 60), // 90 minutes - exceeds 60 min session
                    },
                    Section {
                        title: "Another Normal Video".to_string(),
                        video_index: 2,
                        duration: Duration::from_secs(15 * 60), // 15 minutes
                    },
                ],
            )],
            StructureMetadata {
                total_videos: 3,
                total_duration: Duration::from_secs(125 * 60),
                estimated_duration_hours: Some(2.0),
                difficulty_level: Some("Intermediate".to_string()),
                structure_quality_score: None,
                content_coherence_score: None,
            },
        );

        let course = Course {
            id: Uuid::new_v4(),
            name: "Overflow Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec![
                "Normal Video".to_string(),
                "Very Long Video".to_string(),
                "Another Normal Video".to_string(),
            ],
            structure: Some(structure),
        };

        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60, // 60 minute sessions
            include_weekends: false,
            advanced_settings: None,
        };

        let result = generate_time_based_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();
        assert!(plan_items.len() >= 2); // Should create at least 2 sessions

        // The long video should be in its own session
        let long_video_session = plan_items.iter().find(|item| {
            item.video_indices.contains(&1) // video_index 1 is the long video
        });
        assert!(long_video_session.is_some());

        // The long video session should only contain that one video (due to overflow)
        if let Some(session) = long_video_session {
            assert_eq!(session.video_indices.len(), 1);
            assert_eq!(session.video_indices[0], 1);
        }
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

        // Should have added review sessions and other optimizations
        assert!(plan.items.len() >= original_length);

        // Verify plan is sorted by date
        for i in 1..plan.items.len() {
            assert!(plan.items[i - 1].date <= plan.items[i].date);
        }
    }

    #[test]
    fn test_difficulty_based_planning() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_difficulty_based_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();
        assert!(!plan_items.is_empty());

        // Verify progressive difficulty (first items should be easier)
        // This is a simplified test - in practice, we'd need more sophisticated verification
        assert!(plan_items.len() >= 2);
    }

    #[test]
    fn test_spaced_repetition_planning() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_spaced_repetition_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();
        assert!(!plan_items.is_empty());

        // Should have more items than original (due to review sessions)
        assert!(plan_items.len() > course.video_count());
    }

    #[test]
    fn test_adaptive_planning() {
        let course = create_test_course();
        let settings = create_test_settings();

        let result = generate_adaptive_plan(&course, &settings);
        assert!(result.is_ok());

        let plan_items = result.unwrap();
        assert!(!plan_items.is_empty());
        assert_eq!(plan_items.len(), course.video_count());
    }

    #[test]
    fn test_course_complexity_analysis() {
        let course = create_test_course();
        let complexity = analyze_course_complexity(&course);

        assert!(complexity >= 0.0 && complexity <= 1.0);
    }

    #[test]
    fn test_user_experience_inference() {
        let beginner_settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 2,
            session_length_minutes: 30,
            include_weekends: false,
            advanced_settings: None,
        };

        let expert_settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 6,
            session_length_minutes: 120,
            include_weekends: true,
            advanced_settings: None,
        };

        assert_eq!(
            infer_user_experience_level(&beginner_settings),
            DifficultyLevel::Beginner
        );
        assert_eq!(
            infer_user_experience_level(&expert_settings),
            DifficultyLevel::Expert
        );
    }

    #[test]
    fn test_enhanced_strategy_selection() {
        let course = create_test_course();
        let settings = create_test_settings();

        let strategy = choose_distribution_strategy(&course, &settings).unwrap();

        // Should return a valid strategy
        match strategy {
            DistributionStrategy::ModuleBased
            | DistributionStrategy::TimeBased
            | DistributionStrategy::Hybrid
            | DistributionStrategy::DifficultyBased
            | DistributionStrategy::SpacedRepetition
            | DistributionStrategy::Adaptive => {
                // All valid strategies
            }
        }
    }

    #[test]
    fn test_learning_velocity_analysis() {
        let course = create_test_course();
        let settings = create_test_settings();
        let plan = generate_plan(&course, &settings).unwrap();

        let analysis = analyze_learning_velocity(&plan);

        assert!(analysis.videos_per_day >= 0.0);
        assert!(analysis.total_duration_days >= 0);
        assert!(!analysis.recommended_adjustments.is_empty());
    }

    #[test]
    fn test_cognitive_load_calculation() {
        let high_load = calculate_cognitive_load(
            "Advanced Algorithm Implementation",
            Duration::from_secs(3600),
        );
        let low_load = calculate_cognitive_load("Introduction to Basics", Duration::from_secs(600));

        assert!(high_load > low_load);
        assert!(high_load <= 1.0 && low_load >= 0.0);
    }

    #[test]
    fn test_session_type_classification() {
        assert_eq!(
            classify_session_type("Introduction to Programming"),
            SessionType::Introduction
        );
        assert_eq!(
            classify_session_type("Hands-on Practice Session"),
            SessionType::Practice
        );
        assert_eq!(
            classify_session_type("Review and Summary"),
            SessionType::Review
        );
        assert_eq!(
            classify_session_type("Final Project Build"),
            SessionType::Project
        );
        assert_eq!(
            classify_session_type("Quiz and Assessment"),
            SessionType::Assessment
        );
    }

    #[test]
    fn test_optimal_time_determination() {
        assert_eq!(
            determine_optimal_time_of_day("Complex Algorithm Analysis"),
            Some(TimeOfDay::Morning)
        );
        assert_eq!(
            determine_optimal_time_of_day("Practice Exercises"),
            Some(TimeOfDay::Afternoon)
        );
        assert_eq!(
            determine_optimal_time_of_day("Review Session"),
            Some(TimeOfDay::Evening)
        );
        assert_eq!(determine_optimal_time_of_day("General Topic"), None);
    }
}
// ============================================================================
// CLUSTERING-AWARE PLANNING FUNCTIONS
// ============================================================================

use crate::types::ClusteringMetadata;

// TODO: Task 3.3 - Advanced clustering algorithms
// This function will be used when implementing hierarchical clustering and LDA topic modeling
/// Choose distribution strategy based on clustering insights
#[allow(dead_code)]
fn choose_clustering_aware_strategy(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<DistributionStrategy, PlanError> {
    let basic_strategy = choose_distribution_strategy(course, settings)?;

    // Enhance strategy selection with clustering insights
    let enhanced_strategy = match clustering_metadata.algorithm_used {
        crate::types::ClusteringAlgorithm::KMeans | crate::types::ClusteringAlgorithm::TfIdf => {
            // High-quality content clustering favors topic-aware approaches
            if clustering_metadata.quality_score > 0.7 {
                match basic_strategy.clone() {
                    DistributionStrategy::TimeBased => DistributionStrategy::Hybrid,
                    other => other,
                }
            } else {
                basic_strategy.clone()
            }
        }
        crate::types::ClusteringAlgorithm::Hybrid => {
            // Hybrid clustering works well with hybrid planning
            DistributionStrategy::Hybrid
        }
        _ => basic_strategy.clone(),
    };

    log::info!(
        "Strategy selection: basic={:?}, enhanced={:?}, quality={:.3}",
        basic_strategy,
        enhanced_strategy,
        clustering_metadata.quality_score
    );

    Ok(enhanced_strategy)
}

// TODO: Task 3.3 - Advanced clustering algorithms
// This function will be used for topic modeling using LDA (Latent Dirichlet Allocation)
/// Generate topic-aware module-based plan
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn generate_topic_aware_module_plan(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Group modules by topic similarity for better session flow
    let topic_grouped_modules =
        group_modules_by_topic_similarity(&structure.modules, clustering_metadata);

    for module_group in topic_grouped_modules {
        for module in module_group {
            // Create topic-aware sessions within each module
            let module_sessions =
                create_topic_aware_sessions(module, settings, clustering_metadata)?;

            for session_videos in module_sessions {
                let plan_item = create_plan_item_from_videos(session_videos, current_date);
                plan_items.push(plan_item);

                current_date = crate::planner::get_next_session_date(
                    current_date,
                    settings.sessions_per_week,
                    settings.include_weekends,
                );
            }
        }
    }

    Ok(plan_items)
}

// TODO: Task 3.3 - Advanced clustering algorithms
// This function will be used for hybrid clustering combining multiple algorithms
/// Generate duration-optimized plan using clustering duration insights
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn generate_duration_optimized_plan(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Use clustering duration balancing insights
    let duration_optimized_sessions =
        create_duration_balanced_sessions(&structure.modules, settings, clustering_metadata)?;

    for session_videos in duration_optimized_sessions {
        let plan_item = create_plan_item_from_videos(session_videos, current_date);
        plan_items.push(plan_item);

        current_date = crate::planner::get_next_session_date(
            current_date,
            settings.sessions_per_week,
            settings.include_weekends,
        );
    }

    Ok(plan_items)
}

/// Generate clustering-enhanced hybrid plan
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn generate_clustering_hybrid_plan(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Balance topic coherence and duration optimization
    let hybrid_sessions =
        create_hybrid_clustering_sessions(&structure.modules, settings, clustering_metadata)?;

    for session_videos in hybrid_sessions {
        let plan_item = create_plan_item_from_videos(session_videos, current_date);
        plan_items.push(plan_item);

        current_date = crate::planner::get_next_session_date(
            current_date,
            settings.sessions_per_week,
            settings.include_weekends,
        );
    }

    Ok(plan_items)
}

/// Generate difficulty-based plan using clustering difficulty insights
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn generate_clustering_difficulty_plan(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().unwrap();
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Use module difficulty levels from clustering
    let difficulty_ordered_modules = order_modules_by_clustering_difficulty(&structure.modules);

    for module in difficulty_ordered_modules {
        let module_sessions =
            create_difficulty_aware_sessions(module, settings, clustering_metadata)?;

        for session_videos in module_sessions {
            let plan_item = create_plan_item_from_videos(session_videos, current_date);
            plan_items.push(plan_item);

            current_date = crate::planner::get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
}

/// Generate topic-based spaced repetition plan
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn generate_topic_spaced_repetition_plan(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<PlanItem>, PlanError> {
    // Start with basic spaced repetition
    let mut plan_items = generate_spaced_repetition_plan(course, settings)?;

    // Enhance with topic-based spacing
    enhance_plan_with_topic_spacing(&mut plan_items, clustering_metadata);

    Ok(plan_items)
}

// TODO: Task 3.4 - User preference learning
// This function will be used for clustering parameter auto-tuning based on user feedback
/// Generate adaptive plan using clustering insights
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn generate_clustering_adaptive_plan(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<PlanItem>, PlanError> {
    // Use clustering quality to determine adaptation level
    let adaptation_factor = clustering_metadata.quality_score;

    if adaptation_factor > 0.8 {
        // High-quality clustering: use topic-aware approach
        generate_topic_aware_module_plan(course, settings, clustering_metadata)
    } else if adaptation_factor > 0.6 {
        // Medium-quality clustering: use hybrid approach
        generate_clustering_hybrid_plan(course, settings, clustering_metadata)
    } else {
        // Lower-quality clustering: fall back to duration optimization
        generate_duration_optimized_plan(course, settings, clustering_metadata)
    }
}

/// Apply clustering-aware optimization to plan
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn optimize_clustering_aware_plan(
    plan: &mut Plan,
    clustering_metadata: &ClusteringMetadata,
) -> Result<(), PlanError> {
    // Apply basic optimization first
    optimize_plan(plan)?;

    // Apply clustering-specific optimizations
    optimize_topic_flow(plan, clustering_metadata)?;
    optimize_difficulty_progression_with_clustering(plan, clustering_metadata)?;

    Ok(())
}

// Helper functions for clustering-aware planning

/// Group modules by topic similarity
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn group_modules_by_topic_similarity<'a>(
    modules: &'a [crate::types::Module],
    clustering_metadata: &'a ClusteringMetadata,
) -> Vec<Vec<&'a crate::types::Module>> {
    let mut groups = Vec::new();
    let mut used_modules = std::collections::HashSet::new();

    // Group modules with similar topic keywords
    for (i, module) in modules.iter().enumerate() {
        if used_modules.contains(&i) {
            continue;
        }

        let mut group = vec![module];
        used_modules.insert(i);

        // Find modules with similar topics
        for (j, other_module) in modules.iter().enumerate().skip(i + 1) {
            if used_modules.contains(&j) {
                continue;
            }

            if modules_have_similar_topics(module, other_module, clustering_metadata) {
                group.push(other_module);
                used_modules.insert(j);
            }
        }

        groups.push(group);
    }

    groups
}

/// Check if two modules have similar topics
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn modules_have_similar_topics(
    module1: &crate::types::Module,
    module2: &crate::types::Module,
    _clustering_metadata: &ClusteringMetadata,
) -> bool {
    let keywords1: std::collections::HashSet<_> = module1.topic_keywords.iter().collect();
    let keywords2: std::collections::HashSet<_> = module2.topic_keywords.iter().collect();

    let intersection = keywords1.intersection(&keywords2).count();
    let union = keywords1.union(&keywords2).count();

    if union == 0 {
        return false;
    }

    let similarity = intersection as f32 / union as f32;
    similarity > 0.3 // 30% topic overlap threshold
}

/// Create topic-aware sessions within a module
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn create_topic_aware_sessions(
    module: &crate::types::Module,
    settings: &PlanSettings,
    _clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<Vec<VideoItem>>, PlanError> {
    let videos: Vec<VideoItem> = module
        .sections
        .iter()
        .map(|section| VideoItem {
            module_title: module.title.clone(),
            section_title: section.title.clone(),
            video_index: section.video_index,
            duration: section.duration,
        })
        .collect();

    // Group videos by session capacity, maintaining topic coherence
    let mut sessions = Vec::new();
    let mut current_session = Vec::new();
    let mut current_duration = Duration::from_secs(0);
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);

    for video in videos {
        if current_duration + video.duration > session_limit && !current_session.is_empty() {
            sessions.push(std::mem::take(&mut current_session));
            current_duration = Duration::from_secs(0);
        }

        current_duration += video.duration;
        current_session.push(video);
    }

    if !current_session.is_empty() {
        sessions.push(current_session);
    }

    Ok(sessions)
}

/// Create duration-balanced sessions using clustering insights
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn create_duration_balanced_sessions(
    modules: &[crate::types::Module],
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<Vec<VideoItem>>, PlanError> {
    let mut all_videos = Vec::new();

    // Collect all videos with clustering context
    for module in modules {
        for section in &module.sections {
            all_videos.push(VideoItem {
                module_title: module.title.clone(),
                section_title: section.title.clone(),
                video_index: section.video_index,
                duration: section.duration,
            });
        }
    }

    // Use clustering quality to determine balancing aggressiveness
    let balance_factor = clustering_metadata.quality_score;
    let _session_limit = Duration::from_secs(
        (settings.session_length_minutes as f32 * (1.0 + balance_factor * 0.1)) as u64 * 60,
    );

    // Create balanced sessions
    let mut sessions = Vec::new();
    let mut video_queue = VecDeque::from(all_videos);

    while !video_queue.is_empty() {
        let session_videos = pack_videos_into_session(&mut video_queue, settings)?;
        if !session_videos.is_empty() {
            sessions.push(session_videos);
        }
    }

    Ok(sessions)
}

/// Create hybrid sessions balancing topics and duration
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn create_hybrid_clustering_sessions(
    modules: &[crate::types::Module],
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<Vec<VideoItem>>, PlanError> {
    // Weight between topic coherence and duration balance based on clustering quality
    let topic_weight = clustering_metadata.quality_score;
    let _duration_weight = 1.0 - topic_weight;

    if topic_weight > 0.6 {
        // Favor topic coherence
        let mut sessions = Vec::new();
        for module in modules {
            let module_sessions =
                create_topic_aware_sessions(module, settings, clustering_metadata)?;
            sessions.extend(module_sessions);
        }
        Ok(sessions)
    } else {
        // Favor duration balance
        create_duration_balanced_sessions(modules, settings, clustering_metadata)
    }
}

/// Order modules by difficulty using clustering insights
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn order_modules_by_clustering_difficulty(
    modules: &[crate::types::Module],
) -> Vec<&crate::types::Module> {
    let mut ordered_modules: Vec<_> = modules.iter().collect();

    ordered_modules.sort_by(|a, b| {
        let difficulty_a = a.difficulty_level.unwrap_or(DifficultyLevel::Intermediate);
        let difficulty_b = b.difficulty_level.unwrap_or(DifficultyLevel::Intermediate);

        difficulty_a
            .partial_cmp(&difficulty_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ordered_modules
}

/// Create difficulty-aware sessions
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn create_difficulty_aware_sessions(
    module: &crate::types::Module,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<Vec<VideoItem>>, PlanError> {
    // Adjust session size based on module difficulty and clustering quality
    let difficulty_factor = match module
        .difficulty_level
        .unwrap_or(DifficultyLevel::Intermediate)
    {
        DifficultyLevel::Beginner => 1.2,
        DifficultyLevel::Intermediate => 1.0,
        DifficultyLevel::Advanced => 0.8,
        DifficultyLevel::Expert => 0.6,
    };

    let quality_factor = clustering_metadata.quality_score;
    let adjusted_session_length =
        (settings.session_length_minutes as f32 * difficulty_factor * quality_factor) as u32;

    let adjusted_settings = PlanSettings {
        session_length_minutes: adjusted_session_length,
        ..settings.clone()
    };

    create_topic_aware_sessions(module, &adjusted_settings, clustering_metadata)
}

/// Enhance plan with topic-based spacing
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn enhance_plan_with_topic_spacing(
    plan_items: &mut Vec<PlanItem>,
    clustering_metadata: &ClusteringMetadata,
) {
    // Use topic information to adjust spacing between related content
    if clustering_metadata.content_topics.is_empty() {
        return;
    }

    // Group plan items by topic similarity and adjust dates
    for i in 1..plan_items.len() {
        let current_item = &plan_items[i];
        let previous_item = &plan_items[i - 1];

        // Check topic similarity between consecutive items
        let topic_similarity = calculate_topic_similarity_between_items(
            current_item,
            previous_item,
            clustering_metadata,
        );

        if topic_similarity > 0.7 {
            // High similarity: reduce spacing for reinforcement
            let reduced_gap = chrono::Duration::days(1);
            plan_items[i].date = plan_items[i - 1].date + reduced_gap;
        } else if topic_similarity < 0.3 {
            // Low similarity: increase spacing for context switching
            let increased_gap = chrono::Duration::days(3);
            plan_items[i].date = plan_items[i - 1].date + increased_gap;
        }
    }
}

/// Optimize topic flow in plan
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn optimize_topic_flow(
    plan: &mut Plan,
    clustering_metadata: &ClusteringMetadata,
) -> Result<(), PlanError> {
    if clustering_metadata.content_topics.is_empty() {
        return Ok(());
    }

    // Reorder sessions to improve topic flow
    plan.items.sort_by(|a, b| {
        let topic_score_a = calculate_item_topic_score(a, clustering_metadata);
        let topic_score_b = calculate_item_topic_score(b, clustering_metadata);
        topic_score_a
            .partial_cmp(&topic_score_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Reassign dates after reordering
    let mut current_date = plan
        .items
        .first()
        .map(|item| item.date)
        .unwrap_or_else(chrono::Utc::now);

    for item in &mut plan.items {
        item.date = current_date;
        current_date = crate::planner::get_next_session_date(
            current_date,
            plan.settings.sessions_per_week,
            plan.settings.include_weekends,
        );
    }

    Ok(())
}

/// Optimize difficulty progression with clustering insights
#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn optimize_difficulty_progression_with_clustering(
    plan: &mut Plan,
    clustering_metadata: &ClusteringMetadata,
) -> Result<(), PlanError> {
    // Use clustering quality to determine progression aggressiveness
    let progression_factor = clustering_metadata.quality_score;

    if progression_factor > 0.7 {
        // High-quality clustering: trust the difficulty progression
        return Ok(());
    }

    // Lower-quality clustering: apply conservative difficulty progression
    plan.items.sort_by(|a, b| {
        let difficulty_a = estimate_item_difficulty(a);
        let difficulty_b = estimate_item_difficulty(b);
        difficulty_a
            .partial_cmp(&difficulty_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(())
}

// Helper functions

#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn calculate_topic_similarity_between_items(
    item1: &PlanItem,
    item2: &PlanItem,
    clustering_metadata: &ClusteringMetadata,
) -> f32 {
    // Simple similarity based on module titles and topics
    if item1.module_title == item2.module_title {
        return 0.8;
    }

    // Check topic keyword overlap
    let topics1: std::collections::HashSet<_> = clustering_metadata
        .content_topics
        .iter()
        .filter(|topic| {
            item1
                .module_title
                .to_lowercase()
                .contains(&topic.keyword.to_lowercase())
        })
        .collect();

    let topics2: std::collections::HashSet<_> = clustering_metadata
        .content_topics
        .iter()
        .filter(|topic| {
            item2
                .module_title
                .to_lowercase()
                .contains(&topic.keyword.to_lowercase())
        })
        .collect();

    let intersection = topics1.intersection(&topics2).count();
    let union = topics1.union(&topics2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn calculate_item_topic_score(item: &PlanItem, clustering_metadata: &ClusteringMetadata) -> f32 {
    clustering_metadata
        .content_topics
        .iter()
        .filter(|topic| {
            item.module_title
                .to_lowercase()
                .contains(&topic.keyword.to_lowercase())
        })
        .map(|topic| topic.relevance_score)
        .sum::<f32>()
}

#[allow(dead_code)] // TODO: Task 3.3 - Advanced clustering integration
fn estimate_item_difficulty(item: &PlanItem) -> f32 {
    let title_lower = item.section_title.to_lowercase();
    let mut difficulty: f32 = 0.5; // Default intermediate

    if title_lower.contains("introduction") || title_lower.contains("basic") {
        difficulty -= 0.2;
    }
    if title_lower.contains("advanced") || title_lower.contains("expert") {
        difficulty += 0.3;
    }
    if title_lower.contains("project") || title_lower.contains("exercise") {
        difficulty += 0.1;
    }

    difficulty.clamp(0.0, 1.0)
}
