use crate::storage::database::Database;
use crate::types::{Plan, PlanSettings};
use dioxus::prelude::*;
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;

/// Progress information for plans and courses
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub completed_count: usize,
    pub total_count: usize,
    pub percentage: f32,
    pub estimated_time_remaining: Option<std::time::Duration>,
}

/// Plan management hook with all plan-related operations
#[derive(Clone)]
pub struct PlanManager {
    db: Arc<Database>,
    pub generate_plan: Callback<(Uuid, PlanSettings)>,
}

impl PlanManager {
    pub async fn get_plan_by_course(&self, course_id: Uuid) -> Result<Option<Plan>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::storage::get_plan_by_course_id(&db, &course_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn save_plan(&self, plan: Plan) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::storage::save_plan(&db, &plan).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn delete_plan(&self, plan_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::storage::delete_plan(&db, &plan_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn update_plan_item_completion(
        &self,
        plan_id: Uuid,
        item_index: usize,
        completed: bool,
    ) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load plan
            let mut plan = crate::storage::load_plan(&db, &plan_id)?
                .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

            // Validate item index
            if item_index >= plan.items.len() {
                return Err(anyhow::anyhow!(
                    "Plan item index {} out of bounds (plan has {} items)",
                    item_index,
                    plan.items.len()
                ));
            }

            // Update item completion status
            plan.items[item_index].completed = completed;

            // Save updated plan
            crate::storage::save_plan(&db, &plan).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn get_plan_progress(&self, plan_id: Uuid) -> Result<ProgressInfo> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let plan = crate::storage::load_plan(&db, &plan_id)?
                .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

            let total_count = plan.items.len();
            let completed_count = plan.items.iter().filter(|item| item.completed).count();
            let percentage = if total_count > 0 {
                (completed_count as f32 / total_count as f32) * 100.0
            } else {
                0.0
            };

            let estimated_time_remaining = if completed_count < total_count {
                let remaining_items = total_count - completed_count;
                let session_duration = std::time::Duration::from_secs(
                    (plan.settings.session_length_minutes as u64) * 60,
                );
                Some(session_duration * remaining_items as u32)
            } else {
                None
            };

            Ok(ProgressInfo {
                completed_count,
                total_count,
                percentage,
                estimated_time_remaining,
            })
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn get_course_progress(&self, course_id: Uuid) -> Result<Option<ProgressInfo>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Get plan for this course
            if let Some(plan) = crate::storage::get_plan_by_course_id(&db, &course_id)? {
                let total_count = plan.items.len();
                let completed_count = plan.items.iter().filter(|item| item.completed).count();
                let percentage = if total_count > 0 {
                    (completed_count as f32 / total_count as f32) * 100.0
                } else {
                    0.0
                };

                let estimated_time_remaining = if completed_count < total_count {
                    let remaining_items = total_count - completed_count;
                    let session_duration = std::time::Duration::from_secs(
                        (plan.settings.session_length_minutes as u64) * 60,
                    );
                    Some(session_duration * remaining_items as u32)
                } else {
                    None
                };

                Ok(Some(ProgressInfo {
                    completed_count,
                    total_count,
                    percentage,
                    estimated_time_remaining,
                }))
            } else {
                Ok(None)
            }
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn generate_plan(
        &self,
        course_id: Uuid,
        settings: PlanSettings,
    ) -> Result<Plan> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load course data
            let course = crate::storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            // Generate plan using planner module
            let plan = crate::planner::generate_plan(&course, &settings)
                .map_err(|e| anyhow::anyhow!("Plan generation failed: {}", e))?;

            // Save plan to database
            crate::storage::save_plan(&db, &plan)?;

            Ok(plan)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn regenerate_plan(
        &self,
        plan_id: Uuid,
        new_settings: PlanSettings,
    ) -> Result<Plan> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load existing plan
            let existing_plan = crate::storage::load_plan(&db, &plan_id)?
                .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

            // Load course data
            let course = crate::storage::get_course_by_id(&db, &existing_plan.course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", existing_plan.course_id))?;

            // Generate new plan with new settings
            let mut new_plan = crate::planner::generate_plan(&course, &new_settings)
                .map_err(|e| anyhow::anyhow!("Plan regeneration failed: {}", e))?;

            // Preserve progress from existing plan
            preserve_plan_progress(&existing_plan, &mut new_plan);

            // Update the plan ID to maintain continuity
            new_plan.id = plan_id;

            // Save updated plan to database
            crate::storage::save_plan(&db, &new_plan)?;

            Ok(new_plan)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
}

pub fn use_plan_manager() -> PlanManager {
    let db = use_context::<Arc<Database>>();
    
    let generate_plan = use_callback({
        let db = db.clone();
        move |(course_id, settings): (Uuid, PlanSettings)| {
            let db = db.clone();
            spawn(async move {
                let result: Result<Result<crate::types::Plan, anyhow::Error>, _> = tokio::task::spawn_blocking(move || {
                    // Load course data
                    let course = crate::storage::get_course_by_id(&db, &course_id)?
                        .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

                    // Generate plan using planner module
                    let plan = crate::planner::generate_plan(&course, &settings)
                        .map_err(|e| anyhow::anyhow!("Plan generation failed: {}", e))?;

                    // Save plan to database
                    crate::storage::save_plan(&db, &plan).map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

                    Ok(plan)
                }).await;

                match result {
                    Ok(Ok(_)) => {
                        crate::ui::components::toast::toast::success("Study plan created successfully");
                    }
                    Ok(Err(e)) => {
                        crate::ui::components::toast::toast::error(format!("Failed to create plan: {}", e));
                    }
                    Err(e) => {
                        crate::ui::components::toast::toast::error(format!("Failed to create plan: {}", e));
                    }
                }
            });
            // Return () to match expected callback type
        }
    });
    
    PlanManager { db, generate_plan }
}

/// Hook for reactive plan resource loading
pub fn use_plan_resource(course_id: Uuid) -> Resource<Result<Option<Plan>, anyhow::Error>> {
    let plan_manager = use_plan_manager();

    use_resource(move || {
        let plan_manager = plan_manager.clone();
        async move {
            plan_manager.get_plan_by_course(course_id).await
        }
    })
}

/// Preserve progress from an existing plan when regenerating
fn preserve_plan_progress(existing_plan: &Plan, new_plan: &mut Plan) {
    use std::collections::HashMap;

    // Create a map of video indices to completion status from existing plan
    let mut completion_map: HashMap<Vec<usize>, bool> = HashMap::new();
    for item in &existing_plan.items {
        completion_map.insert(item.video_indices.clone(), item.completed);
    }

    // Apply completion status to matching items in new plan
    for item in &mut new_plan.items {
        if let Some(&completed) = completion_map.get(&item.video_indices) {
            item.completed = completed;
        }
    }
}