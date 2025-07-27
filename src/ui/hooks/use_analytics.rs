use crate::storage::database::Database;
use crate::types::{Course, AdvancedSchedulerSettings, DifficultyLevel, DistributionStrategy};
use dioxus::prelude::*;
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;

/// Analytics and AI recommendations hook
#[derive(Clone)]
pub struct AnalyticsManager {
    db: Arc<Database>,
    pub structure_course: Callback<Uuid>,
}

impl AnalyticsManager {
    pub async fn get_available_scheduling_strategies(&self) -> Result<Vec<DistributionStrategy>> {
        Ok(DistributionStrategy::all())
    }

    pub async fn get_available_difficulty_levels(&self) -> Result<Vec<DifficultyLevel>> {
        Ok(DifficultyLevel::all())
    }

    pub async fn validate_advanced_scheduler_settings(&self, settings: &AdvancedSchedulerSettings) -> Result<Vec<String>> {
        let settings = settings.clone();
        tokio::task::spawn_blocking(move || {
            let mut errors = Vec::new();

            // Validate custom intervals if provided
            if let Some(ref intervals) = settings.custom_intervals {
                if intervals.is_empty() {
                    errors.push("Custom intervals cannot be empty".to_string());
                }

                // Check for negative intervals
                for &interval in intervals {
                    if interval <= 0 {
                        errors.push("Custom intervals must be positive".to_string());
                        break;
                    }
                }

                // Check for reasonable maximum interval (1 year = 365 days)
                for &interval in intervals {
                    if interval > 365 {
                        errors.push("Custom intervals should not exceed 365 days".to_string());
                        break;
                    }
                }

                // Check for ascending order
                let mut sorted_intervals = intervals.clone();
                sorted_intervals.sort();
                if *intervals != sorted_intervals {
                    errors.push("Custom intervals should be in ascending order".to_string());
                }
            }

            // Validate strategy-specific settings
            match settings.strategy {
                DistributionStrategy::SpacedRepetition => {
                    if !settings.spaced_repetition_enabled {
                        errors.push(
                            "Spaced repetition must be enabled for SpacedRepetition strategy"
                                .to_string(),
                        );
                    }
                }
                DistributionStrategy::DifficultyBased => {
                    if !settings.difficulty_adaptation {
                        errors.push(
                            "Difficulty adaptation should be enabled for DifficultyBased strategy"
                                .to_string(),
                        );
                    }
                }
                DistributionStrategy::Adaptive => {
                    if !settings.cognitive_load_balancing {
                        errors.push(
                            "Cognitive load balancing should be enabled for Adaptive strategy"
                                .to_string(),
                        );
                    }
                }
                _ => {} // Other strategies don't have specific requirements
            }

            Ok(errors)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn get_recommended_advanced_settings(
        &self,
        course_id: Uuid,
        user_experience: DifficultyLevel,
    ) -> Result<AdvancedSchedulerSettings> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load course data
            let course = crate::storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            // Analyze course characteristics
            let total_videos = course.video_count();
            let has_structure = course.structure.is_some();

            // Determine recommended strategy based on course and user characteristics
            let recommended_strategy = match (user_experience, total_videos, has_structure) {
                // Beginners benefit from spaced repetition
                (DifficultyLevel::Beginner, _, _) => {
                    DistributionStrategy::SpacedRepetition
                }
                // Large courses need adaptive scheduling
                (_, videos, _) if videos > 50 => DistributionStrategy::Adaptive,
                // Well-structured courses can use module-based approach
                (_, _, true) => DistributionStrategy::ModuleBased,
                // Default to hybrid for balanced approach
                _ => DistributionStrategy::Hybrid,
            };

            // Create recommended settings
            let spaced_repetition_enabled = matches!(
                recommended_strategy,
                DistributionStrategy::SpacedRepetition
            );

            let recommended_settings = AdvancedSchedulerSettings {
                strategy: recommended_strategy,
                difficulty_adaptation: matches!(
                    user_experience,
                    DifficultyLevel::Beginner | DifficultyLevel::Intermediate
                ),
                spaced_repetition_enabled,
                cognitive_load_balancing: total_videos > 20,
                user_experience_level: user_experience,
                custom_intervals: None, // Use default intervals
                max_session_duration_minutes: None,
                min_break_between_sessions_hours: None,
                prioritize_difficult_content: matches!(
                    user_experience,
                    DifficultyLevel::Advanced | DifficultyLevel::Expert
                ),
                adaptive_pacing: true,
            };

            Ok(recommended_settings)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn structure_course(&self, course_id: Uuid) -> Result<Course> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load course data
            let mut course = crate::storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            // Check if course already has structure
            if course.structure.is_some() {
                return Err(anyhow::anyhow!("Course is already structured"));
            }

            // Use NLP module to structure the course
            let structure = crate::nlp::structure_course(course.raw_titles.clone())
                .map_err(|e| anyhow::anyhow!("Course structuring failed: {}", e))?;

            // Update course with new structure
            course.structure = Some(structure);

            // Save updated course to database
            crate::storage::save_course(&db, &course)?;

            Ok(course)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn structure_course_with_progress<F>(
        &self,
        course_id: Uuid,
        progress_callback: F,
    ) -> Result<Course>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            progress_callback(0.0, "Loading course data...".to_string());

            // Load course data
            let mut course = crate::storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            progress_callback(25.0, "Analyzing course content...".to_string());

            // Use NLP module to structure the course
            let structure = crate::nlp::structure_course(course.raw_titles.clone())
                .map_err(|e| anyhow::anyhow!("Course structuring failed: {}", e))?;

            progress_callback(75.0, "Saving structured course...".to_string());

            // Update course with new structure
            course.structure = Some(structure);

            // Save updated course to database
            crate::storage::save_course(&db, &course)?;

            progress_callback(100.0, "Course structuring completed!".to_string());

            Ok(course)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
}

pub fn use_analytics_manager() -> AnalyticsManager {
    let db = use_context::<Arc<Database>>();
    
    let structure_course = use_callback({
        let db = db.clone();
        move |course_id: Uuid| {
            let db = db.clone();
            spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    // Load course data
                    let mut course = crate::storage::get_course_by_id(&db, &course_id)?
                        .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

                    // Check if course already has structure
                    if course.structure.is_some() {
                        return Err(anyhow::anyhow!("Course is already structured"));
                    }

                    // Use NLP module to structure the course
                    let structure = crate::nlp::structure_course(course.raw_titles.clone())
                        .map_err(|e| anyhow::anyhow!("Course structuring failed: {}", e))?;

                    // Update course with new structure
                    course.structure = Some(structure);

                    // Save updated course to database
                    crate::storage::save_course(&db, &course)?;

                    Ok(course)
                }).await;

                match result {
                    Ok(Ok(_)) => {
                        crate::ui::components::toast::toast::success("Course structured successfully");
                    }
                    Ok(Err(e)) => {
                        crate::ui::components::toast::toast::error(format!("Failed to structure course: {}", e));
                    }
                    Err(e) => {
                        crate::ui::components::toast::toast::error(format!("Failed to structure course: {}", e));
                    }
                }
            });
            // Return () to match expected callback type
        }
    });
    
    AnalyticsManager { db, structure_course }
}

/// Hook for reactive AI recommendations
pub fn use_ai_recommendations(course_id: Uuid, user_experience: DifficultyLevel) -> Resource<Result<AdvancedSchedulerSettings, anyhow::Error>> {
    let analytics_manager = use_analytics_manager();

    use_resource(move || {
        let analytics_manager = analytics_manager.clone();
        async move {
            analytics_manager.get_recommended_advanced_settings(course_id, user_experience).await
        }
    })
}