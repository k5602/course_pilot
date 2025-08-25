/*!
Sequential planning utilities for the scheduler.

This module contains all logic related to generating study plans that
preserve the original, sequential order of videos. Use this path when
the course content has a strong pedagogical progression (e.g., "Lesson 1",
"Chapter 2", "Part 3", etc.) where reordering harms learning flow.

Public API:
- `should_use_sequential_planning(course) -> bool`
- `generate_sequential_plan(course, settings) -> Plan`

Internal helpers:
- Order-preserving bin-packing for sessions
- Sequential patterns detection in raw titles and topic keywords
- Lightweight optimization that does not change video order
*/

use std::collections::VecDeque;
use std::time::Duration;

use chrono::Utc;
use log::{debug, info, warn};

use crate::PlanError;
use crate::planner::validate_plan_settings;
use crate::types::{Course, CourseStructure, Plan, PlanItem, PlanSettings, TopicInfo};

use super::packing::VideoItem;

/// Decide whether the course should be scheduled with sequential planning.
///
/// Heuristics:
/// - If there is no structure, analyze raw titles for sequential patterns
/// - If structure exists and indicates sequential content in its titles, prefer sequential
/// - If clustering metadata exists but has low quality score, prefer sequential
pub fn should_use_sequential_planning(course: &Course) -> bool {
    // If no structure, analyze raw titles only
    let structure = match &course.structure {
        Some(s) => s,
        None => {
            return analyze_raw_titles_for_sequential_patterns(&course.raw_titles);
        }
    };

    // If clustering metadata exists and quality is low, prefer sequential
    if let Some(clustering) = &structure.clustering_metadata {
        if clustering.quality_score < 0.6 {
            debug!(
                "Low clustering quality score ({:.2}), considering sequential approach",
                clustering.quality_score
            );
            return true;
        }

        if has_sequential_topic_indicators(&clustering.content_topics) {
            debug!("Sequential topic indicators found in clustering metadata");
            return true;
        }
    } else {
        // Absence of clustering metadata often indicates preserve-order processing
        debug!("No clustering metadata found; sequential content likely");
        return true;
    }

    // Examine module titles for sequential patterns
    let module_titles: Vec<String> = structure.modules.iter().map(|m| m.title.clone()).collect();
    if analyze_raw_titles_for_sequential_patterns(&module_titles) {
        debug!("Sequential patterns detected in module titles");
        return true;
    }

    false
}

/// Create a sequential plan that preserves original video order.
///
/// This path packs videos into sessions respecting the session length while
/// maintaining strict order of items.
///
/// Note: This function does not re-arrange videos; it only groups them.
pub fn generate_sequential_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Plan, PlanError> {
    // Validate input
    validate_plan_settings(
        settings.sessions_per_week,
        settings.session_length_minutes,
        settings.start_date,
    )
    .map_err(PlanError::InvalidSettings)?;

    info!(
        "Generating sequential plan for course '{}' with {} videos",
        course.name,
        course.video_count()
    );

    let plan_items = if let Some(structure) = &course.structure {
        generate_sequential_plan_from_structure(structure, settings)?
    } else {
        generate_sequential_plan_from_raw_titles(course, settings)?
    };

    let mut plan = Plan::new(course.id, settings.clone());
    plan.items = plan_items;

    // Apply basic optimization that preserves order
    optimize_sequential_plan(&mut plan)?;

    info!(
        "Generated sequential plan with {} sessions (order preserved)",
        plan.items.len()
    );

    Ok(plan)
}

/// Generate sequential plan items from a structured course.
pub fn generate_sequential_plan_from_structure(
    structure: &CourseStructure,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Collect videos preserving original order (by section.video_index)
    let mut all_videos: Vec<VideoItem> = Vec::new();
    for module in &structure.modules {
        for section in &module.sections {
            all_videos.push(VideoItem {
                module_title: module.title.clone(),
                section_title: section.title.clone(),
                video_index: section.video_index,
                duration: section.duration,
            });
        }
    }
    all_videos.sort_by_key(|v| v.video_index);

    // Pack into sessions while preserving order
    let mut queue: VecDeque<VideoItem> = VecDeque::from(all_videos);
    while !queue.is_empty() {
        let session_videos = pack_videos_into_sequential_session(&mut queue, settings)?;
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

/// Generate sequential plan items from raw titles (fallback path).
pub fn generate_sequential_plan_from_raw_titles(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    let mut queue: VecDeque<VideoItem> = VecDeque::new();
    for (index, title) in course.raw_titles.iter().enumerate() {
        queue.push_back(VideoItem {
            module_title: "Sequential Content".to_string(),
            section_title: title.clone(),
            video_index: index,
            duration: Duration::from_secs(600), // fallback 10 minutes
        });
    }

    while !queue.is_empty() {
        let session_videos = pack_videos_into_sequential_session(&mut queue, settings)?;
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

/// Order-preserving session packer.
///
/// Unlike the general-purpose packer, this strictly preserves the queue order.
/// It greedily takes items from the front until the effective session limit is reached.
/// If the very first item exceeds the hard limit, we accept it (with a warning)
/// to avoid blocking progress.
fn pack_videos_into_sequential_session(
    video_queue: &mut VecDeque<VideoItem>,
    settings: &PlanSettings,
) -> Result<Vec<VideoItem>, PlanError> {
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    let effective_session_limit =
        Duration::from_secs(((session_limit.as_secs() as f32) * 0.8) as u64);

    let mut session_videos = Vec::new();
    let mut session_duration = Duration::from_secs(0);

    while let Some(front) = video_queue.front() {
        let video_duration = front.duration;

        // Handle oversized first video
        if video_exceeds_session_limit(video_duration, session_limit) {
            if session_videos.is_empty() {
                let v = video_queue.pop_front().expect("peeked front exists");
                warn!(
                    "Video '{}' ({:.1} min) exceeds session limit ({} min) but included to preserve order",
                    v.section_title,
                    video_duration.as_secs_f32() / 60.0,
                    settings.session_length_minutes
                );
                session_videos.push(v);
            }
            break;
        }

        // Greedy take while fits; always take at least one item
        if session_duration + video_duration <= effective_session_limit || session_videos.is_empty()
        {
            let v = video_queue.pop_front().expect("peeked front exists");
            session_duration += video_duration;
            session_videos.push(v);
        } else {
            break;
        }
    }

    Ok(session_videos)
}

/// Sequential-safe optimization that preserves order.
///
/// - Adds duration overflow warnings
/// - Ensures proper date spacing according to settings
pub(crate) fn optimize_sequential_plan(plan: &mut Plan) -> Result<(), PlanError> {
    // 1) Overflow warnings for very long sessions
    for item in &mut plan.items {
        let target_secs = (plan.settings.session_length_minutes as u64) * 60;
        if item.total_duration.as_secs() > target_secs.saturating_mul(120).saturating_div(100) {
            item.overflow_warnings.push(format!(
                "Session duration ({}) significantly exceeds target ({})",
                crate::types::duration_utils::format_duration(item.total_duration),
                crate::types::duration_utils::format_duration(Duration::from_secs(target_secs))
            ));
        }
    }

    // 2) Recompute dates monotonically using planner helper
    let mut current_date = plan.settings.start_date;
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

/// Helper: create a PlanItem from packed videos and a target date.
fn create_plan_item_from_videos(videos: Vec<VideoItem>, date: chrono::DateTime<Utc>) -> PlanItem {
    let module_title = videos
        .first()
        .map(|v| v.module_title.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let section_title = if videos.len() == 1 {
        videos[0].section_title.clone()
    } else {
        format!(
            "{} + {} more",
            videos[0].section_title,
            videos.len().saturating_sub(1)
        )
    };

    let video_indices: Vec<usize> = videos.iter().map(|v| v.video_index).collect();
    let total_duration: Duration = videos.iter().map(|v| v.duration).sum();

    // Estimate completion time with a 25% buffer for notes/breaks
    let estimated_completion_time =
        crate::types::duration_utils::calculate_completion_time_with_buffer(total_duration, 0.25);

    PlanItem {
        date,
        module_title,
        section_title,
        video_indices,
        completed: false,
        total_duration,
        estimated_completion_time,
        overflow_warnings: Vec::new(),
    }
}

/// Detect sequential patterns in raw titles.
///
/// Signals:
/// - Titles containing ["lesson","part","chapter","module","step","tutorial"] with numbers
/// - Titles with ["introduction","getting started","basics","fundamentals","overview"]
fn analyze_raw_titles_for_sequential_patterns(titles: &[String]) -> bool {
    if titles.len() < 2 {
        return false;
    }

    let mut sequential_hits = 0usize;
    for title in titles {
        let t = title.to_lowercase();

        let has_seq_word = t.contains("lesson")
            || t.contains("part")
            || t.contains("chapter")
            || t.contains("module")
            || t.contains("step")
            || t.contains("tutorial");

        if has_seq_word && t.chars().any(|c| c.is_ascii_digit()) {
            sequential_hits += 1;
            continue;
        }

        if t.contains("introduction")
            || t.contains("getting started")
            || t.contains("basics")
            || t.contains("fundamentals")
            || t.contains("overview")
        {
            sequential_hits += 1;
        }
    }

    let ratio = sequential_hits as f32 / titles.len() as f32;
    ratio > 0.4
}

/// Check if topic keywords imply a sequential curriculum.
fn has_sequential_topic_indicators(topics: &[TopicInfo]) -> bool {
    if topics.is_empty() {
        return false;
    }

    const SEQ: [&str; 17] = [
        "introduction",
        "basic",
        "fundamentals",
        "getting started",
        "overview",
        "lesson",
        "part",
        "chapter",
        "module",
        "step",
        "tutorial",
        "beginner",
        "intermediate",
        "advanced",
        "final",
        "conclusion",
        "recap",
    ];

    let mut hits = 0usize;
    for topic in topics {
        let key = topic.keyword.to_lowercase();
        if SEQ.iter().any(|k| key.contains(k)) {
            hits += 1;
        }
    }

    (hits as f32 / topics.len() as f32) > 0.3
}

/// Helper: oversized video predicate using strict session limit.
#[inline]
fn video_exceeds_session_limit(video_duration: Duration, strict_limit: Duration) -> bool {
    video_duration > strict_limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Module, Section, StructureMetadata};
    use uuid::Uuid;

    fn make_structure() -> CourseStructure {
        let sections = vec![
            Section {
                title: "Introduction".to_string(),
                video_index: 0,
                duration: Duration::from_secs(10 * 60),
            },
            Section {
                title: "Lesson 1".to_string(),
                video_index: 1,
                duration: Duration::from_secs(15 * 60),
            },
            Section {
                title: "Lesson 2".to_string(),
                video_index: 2,
                duration: Duration::from_secs(20 * 60),
            },
        ];
        let modules = vec![Module::new_basic("Module 1".into(), sections)];
        let metadata = StructureMetadata {
            total_videos: 3,
            total_duration: Duration::from_secs(45 * 60),
            estimated_duration_hours: None,
            difficulty_level: None,
            structure_quality_score: None,
            content_coherence_score: None,
            content_type_detected: None,
            original_order_preserved: Some(true),
            processing_strategy_used: Some("PreserveOrder".into()),
        };
        CourseStructure::new_basic(modules, metadata)
    }

    fn make_course() -> Course {
        Course {
            id: Uuid::new_v4(),
            name: "Seq".into(),
            created_at: Utc::now(),
            raw_titles: vec!["Introduction".into(), "Lesson 1".into(), "Lesson 2".into()],
            videos: vec![],
            structure: Some(make_structure()),
        }
    }

    fn settings() -> PlanSettings {
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        }
    }

    #[test]
    fn detects_sequential_patterns_in_titles() {
        let titles = vec!["Introduction".into(), "Lesson 1".into(), "Lesson 2".into()];
        assert!(analyze_raw_titles_for_sequential_patterns(&titles));
    }

    #[test]
    fn generates_sequential_plan_from_structure() {
        let c = make_course();
        let s = settings();
        let items = generate_sequential_plan_from_structure(c.structure.as_ref().unwrap(), &s)
            .expect("plan items");
        assert!(!items.is_empty());
        // Order preserved: first index should be 0
        assert_eq!(items[0].video_indices.first().copied(), Some(0));
    }

    #[test]
    fn sequential_plan_smoke_test() {
        let c = make_course();
        let s = settings();
        let plan = generate_sequential_plan(&c, &s).expect("plan");
        assert!(!plan.items.is_empty());
    }
}
