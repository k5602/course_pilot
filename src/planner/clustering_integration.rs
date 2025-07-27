//! Clustering-aware planning functions for enhanced study plan generation
//!
//! This module provides clustering-aware planning capabilities that leverage
//! the advanced clustering metadata to create more intelligent study plans.

use crate::PlanError;
use crate::planner::get_next_session_date;
use crate::types::{ClusteringMetadata, Course, DifficultyLevel, Plan, PlanItem, PlanSettings};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::time::Duration;

/// Choose distribution strategy based on clustering insights
pub fn choose_clustering_aware_strategy(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<crate::types::DistributionStrategy, PlanError> {
    use crate::planner::choose_distribution_strategy;

    let basic_strategy = choose_distribution_strategy(course, settings)?;

    // Enhance strategy selection with clustering insights
    let enhanced_strategy = match clustering_metadata.algorithm_used {
        crate::types::ClusteringAlgorithm::KMeans | crate::types::ClusteringAlgorithm::TfIdf => {
            // High-quality content clustering favors topic-aware approaches
            if clustering_metadata.quality_score > 0.7 {
                match basic_strategy.clone() {
                    crate::types::DistributionStrategy::TimeBased => {
                        crate::types::DistributionStrategy::Hybrid
                    }
                    other => other,
                }
            } else {
                basic_strategy.clone()
            }
        }
        crate::types::ClusteringAlgorithm::Hybrid => {
            // Hybrid clustering works well with hybrid planning
            crate::types::DistributionStrategy::Hybrid
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

/// Generate topic-aware module-based plan
pub fn generate_topic_aware_module_plan(
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

                current_date = get_next_session_date(
                    current_date,
                    settings.sessions_per_week,
                    settings.include_weekends,
                );
            }
        }
    }

    Ok(plan_items)
}

/// Generate duration-optimized plan using clustering duration insights
pub fn generate_duration_optimized_plan(
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

        current_date = get_next_session_date(
            current_date,
            settings.sessions_per_week,
            settings.include_weekends,
        );
    }

    Ok(plan_items)
}

/// Generate clustering-enhanced hybrid plan
pub fn generate_clustering_hybrid_plan(
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

        current_date = get_next_session_date(
            current_date,
            settings.sessions_per_week,
            settings.include_weekends,
        );
    }

    Ok(plan_items)
}

/// Generate difficulty-based plan using clustering difficulty insights
pub fn generate_clustering_difficulty_plan(
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

            current_date = get_next_session_date(
                current_date,
                settings.sessions_per_week,
                settings.include_weekends,
            );
        }
    }

    Ok(plan_items)
}

/// Generate topic-based spaced repetition plan
pub fn generate_topic_spaced_repetition_plan(
    course: &Course,
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<PlanItem>, PlanError> {
    use crate::planner::generate_spaced_repetition_plan;

    // Start with basic spaced repetition
    let mut plan_items = generate_spaced_repetition_plan(course, settings)?;

    // Enhance with topic-based spacing
    enhance_plan_with_topic_spacing(&mut plan_items, clustering_metadata);

    Ok(plan_items)
}

/// Generate adaptive plan using clustering insights
pub fn generate_clustering_adaptive_plan(
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
pub fn optimize_clustering_aware_plan(
    plan: &mut Plan,
    clustering_metadata: &ClusteringMetadata,
) -> Result<(), PlanError> {
    use crate::planner::optimize_plan;

    // Apply basic optimization first
    optimize_plan(plan)?;

    // Apply clustering-specific optimizations
    optimize_topic_flow(plan, clustering_metadata)?;
    optimize_difficulty_progression_with_clustering(plan, clustering_metadata)?;

    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Helper struct for video items in clustering-aware planning
#[derive(Debug, Clone)]
struct VideoItem {
    module_title: String,
    section_title: String,
    video_index: usize,
    duration: Duration,
}

/// Group modules by topic similarity
fn group_modules_by_topic_similarity<'a>(
    modules: &'a [crate::types::Module],
    clustering_metadata: &ClusteringMetadata,
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

    // Create balanced sessions using existing bin packing
    let mut sessions = Vec::new();
    let mut video_queue = VecDeque::from(all_videos);

    while !video_queue.is_empty() {
        let session_videos = pack_videos_into_clustering_session(&mut video_queue, settings)?;
        if !session_videos.is_empty() {
            sessions.push(session_videos);
        }
    }

    Ok(sessions)
}

/// Pack videos into session for clustering-aware planning
fn pack_videos_into_clustering_session(
    video_queue: &mut VecDeque<VideoItem>,
    settings: &PlanSettings,
) -> Result<Vec<VideoItem>, PlanError> {
    let mut session_videos = Vec::new();
    let mut current_duration = Duration::from_secs(0);
    let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    let effective_limit = Duration::from_secs((session_limit.as_secs() as f32 * 0.9) as u64);

    while let Some(video) = video_queue.pop_front() {
        if current_duration + video.duration <= effective_limit {
            current_duration += video.duration;
            session_videos.push(video);
        } else {
            // Put video back and try to fit smaller ones
            video_queue.push_front(video);
            break;
        }
    }

    Ok(session_videos)
}

/// Create hybrid sessions balancing topics and duration
fn create_hybrid_clustering_sessions(
    modules: &[crate::types::Module],
    settings: &PlanSettings,
    clustering_metadata: &ClusteringMetadata,
) -> Result<Vec<Vec<VideoItem>>, PlanError> {
    // Weight between topic coherence and duration balance based on clustering quality
    let topic_weight = clustering_metadata.quality_score;

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
        current_date = get_next_session_date(
            current_date,
            plan.settings.sessions_per_week,
            plan.settings.include_weekends,
        );
    }

    Ok(())
}

/// Optimize difficulty progression with clustering insights
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

/// Create plan item from video items
fn create_plan_item_from_videos(videos: Vec<VideoItem>, date: DateTime<Utc>) -> PlanItem {
    let total_duration: Duration = videos.iter().map(|v| v.duration).sum();
    let estimated_completion_time =
        crate::types::duration_utils::calculate_completion_time_with_buffer(total_duration, 0.25);

    let video_indices: Vec<usize> = videos.iter().map(|v| v.video_index).collect();
    let module_title = videos
        .first()
        .map(|v| v.module_title.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let section_title = if videos.len() == 1 {
        videos[0].section_title.clone()
    } else {
        format!("{} (+{} more)", videos[0].section_title, videos.len() - 1)
    };

    // Check for overflow warnings
    let overflow_warnings = videos
        .iter()
        .filter(|v| crate::types::duration_utils::is_duration_excessive(v.duration, 60)) // Assume 60 min default
        .map(|v| format!("Video '{}' may be too long", v.section_title))
        .collect();

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

// Helper functions for topic analysis

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

fn estimate_item_difficulty(item: &PlanItem) -> f32 {
    let title_lower = item.section_title.to_lowercase();
    let mut difficulty = 0.5f32; // Default intermediate

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
