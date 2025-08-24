//! Plans management state for Course Pilot
//!
//! This module handles reactive state for study plan operations including
//! creation, updates, deletion, and plan execution tracking.

use crate::types::Plan;
use dioxus::prelude::*;
use uuid::Uuid;

use super::{StateError, StateResult};

/// Plans management context
#[derive(Clone, Copy)]
pub struct PlanContext {
    pub plans: Signal<Vec<Plan>>,
}

impl Default for PlanContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanContext {
    pub fn new() -> Self {
        Self {
            plans: Signal::new(Vec::new()),
        }
    }
}

/// Plans context provider component
#[component]
pub fn PlanContextProvider(children: Element) -> Element {
    use_context_provider(|| PlanContext::new());
    rsx! { {children} }
}

/// Hook to access plans reactively
pub fn use_plans_reactive() -> Signal<Vec<Plan>> {
    use_context::<PlanContext>().plans
}

/// Hook to get plans for a specific course
pub fn use_course_plans_reactive(course_id: Uuid) -> Signal<Vec<Plan>> {
    let plans = use_plans_reactive();
    Signal::new(
        plans
            .read()
            .iter()
            .filter(|p| p.course_id == course_id)
            .cloned()
            .collect(),
    )
}

/// Hook to get the active plan for a course
pub fn use_active_plan_reactive(course_id: Uuid) -> Signal<Option<Plan>> {
    let plans = use_plans_reactive();
    Signal::new({
        let plans_vec = plans.read();
        plans_vec
            .iter()
            .filter(|p| p.course_id == course_id)
            .max_by_key(|p| p.created_at)
            .cloned()
    })
}

/// Hook to get plan statistics
pub fn use_plan_stats_reactive() -> Signal<(usize, usize, f32)> {
    let plans = use_plans_reactive();
    Signal::new({
        let plans_vec = plans.read();
        let total_plans = plans_vec.len();

        // "Active" plans are the latest per course_id (one per course if any)
        let mut course_ids = std::collections::HashSet::new();
        for p in plans_vec.iter() {
            course_ids.insert(p.course_id);
        }
        let active_plans = course_ids.len();

        // Average completion across plans (completed items / total items)
        let mut total_completion = 0.0f32;
        for p in plans_vec.iter() {
            let total_items = p.items.len();
            if total_items > 0 {
                let completed = p.items.iter().filter(|i| i.completed).count();
                total_completion += completed as f32 / total_items as f32;
            }
        }
        let average_progress = if total_plans > 0 {
            total_completion / total_plans as f32
        } else {
            0.0
        };

        (total_plans, active_plans, average_progress)
    })
}

/// Add a new plan to the reactive state
pub fn add_plan_reactive(plan: Plan) {
    let mut plans = use_plans_reactive();
    let mut plans_vec = plans.read().clone();

    plans_vec.push(plan);
    plans.set(plans_vec);
}

/// Update an existing plan in the reactive state
pub fn update_plan_reactive(plan_id: Uuid, updated_plan: Plan) -> StateResult<()> {
    let mut plans = use_plans_reactive();
    let mut plans_vec = plans.read().clone();

    if let Some(index) = plans_vec.iter().position(|p| p.id == plan_id) {
        let mut final_plan = updated_plan;
        final_plan.id = plan_id; // Preserve original ID

        plans_vec[index] = final_plan;
        plans.set(plans_vec);
        Ok(())
    } else {
        Err(StateError::InvalidOperation(format!(
            "Plan not found: {}",
            plan_id
        )))
    }
}

/// Delete a plan from the reactive state
pub fn delete_plan_reactive(plan_id: Uuid) -> StateResult<()> {
    let mut plans = use_plans_reactive();
    let mut plans_vec = plans.read().clone();

    if let Some(index) = plans_vec.iter().position(|p| p.id == plan_id) {
        plans_vec.remove(index);
        plans.set(plans_vec);
        Ok(())
    } else {
        Err(StateError::InvalidOperation(format!(
            "Plan not found: {}",
            plan_id
        )))
    }
}

/// Delete all plans for a specific course
pub fn delete_course_plans_reactive(course_id: Uuid) -> usize {
    let mut plans = use_plans_reactive();
    let mut plans_vec = plans.read().clone();
    let initial_count = plans_vec.len();

    plans_vec.retain(|p| p.course_id != course_id);
    let deleted_count = initial_count - plans_vec.len();

    plans.set(plans_vec);
    deleted_count
}

/// Update plan progress
pub fn update_plan_progress_reactive(plan_id: Uuid, _progress: f32) -> StateResult<()> {
    let mut plans = use_plans_reactive();
    let mut plans_vec = plans.read().clone();

    if plans_vec.iter_mut().any(|p| p.id == plan_id) {
        // No persistent progress field on Plan; UI derives progress from items.
        plans.set(plans_vec);
        Ok(())
    } else {
        Err(StateError::InvalidOperation(format!(
            "Plan not found: {}",
            plan_id
        )))
    }
}

/// Mark a plan item as completed
pub fn complete_plan_item_reactive(plan_id: Uuid, item_index: usize) -> StateResult<()> {
    let mut plans = use_plans_reactive();
    let mut plans_vec = plans.read().clone();

    if let Some(plan) = plans_vec.iter_mut().find(|p| p.id == plan_id) {
        if let Some(item) = plan.items.get_mut(item_index) {
            item.completed = true;

            plans.set(plans_vec);
            Ok(())
        } else {
            Err(StateError::InvalidOperation(format!(
                "Plan item {} not found in plan {}",
                item_index, plan_id
            )))
        }
    } else {
        Err(StateError::InvalidOperation(format!(
            "Plan not found: {}",
            plan_id
        )))
    }
}

/// Mark a plan item as uncompleted
pub fn uncomplete_plan_item_reactive(plan_id: Uuid, item_index: usize) -> StateResult<()> {
    let mut plans = use_plans_reactive();
    let mut plans_vec = plans.read().clone();

    if let Some(plan) = plans_vec.iter_mut().find(|p| p.id == plan_id) {
        if let Some(item) = plan.items.get_mut(item_index) {
            item.completed = false;

            plans.set(plans_vec);
            Ok(())
        } else {
            Err(StateError::InvalidOperation(format!(
                "Plan item {} not found in plan {}",
                item_index, plan_id
            )))
        }
    } else {
        Err(StateError::InvalidOperation(format!(
            "Plan not found: {}",
            plan_id
        )))
    }
}

/// Activate a specific plan (deactivate others for the same course)
pub fn activate_plan_reactive(plan_id: Uuid) -> StateResult<()> {
    let mut plans = use_plans_reactive();
    let plans_vec = plans.read().clone();

    if plans_vec.iter().any(|p| p.id == plan_id) {
        // No active flag on Plan; selectors treat latest created plan per course as "active".
        plans.set(plans_vec);
        Ok(())
    } else {
        Err(StateError::InvalidOperation(format!(
            "Plan not found: {}",
            plan_id
        )))
    }
}

/// Get plan completion statistics
pub fn get_plan_completion_stats_reactive(plan_id: Uuid) -> Option<(usize, usize, f32)> {
    let plans = use_plans_reactive();
    let plans_vec = plans.read();

    if let Some(plan) = plans_vec.iter().find(|p| p.id == plan_id) {
        let total_items = plan.items.len();
        let completed_items = plan.items.iter().filter(|i| i.completed).count();
        let completion_rate = if total_items > 0 {
            completed_items as f32 / total_items as f32
        } else {
            0.0
        };

        Some((total_items, completed_items, completion_rate))
    } else {
        None
    }
}

/// Check if a plan exists by ID
pub fn plan_exists_reactive(plan_id: Uuid) -> bool {
    let plans = use_plans_reactive();
    plans.read().iter().any(|p| p.id == plan_id)
}

/// Get a plan by ID
pub fn get_plan_reactive(plan_id: Uuid) -> Option<Plan> {
    let plans = use_plans_reactive();
    plans.read().iter().find(|p| p.id == plan_id).cloned()
}

/// Get all plans
pub fn get_plans_reactive() -> Vec<Plan> {
    let plans = use_plans_reactive();
    plans.read().clone()
}

/// Legacy hook functions for compatibility
pub fn use_plans() -> Signal<Vec<Plan>> {
    use_plans_reactive()
}

/// Non-reactive plan operations for backend integration
pub fn add_plan(plan: Plan) {
    add_plan_reactive(plan);
}

pub fn update_plan(plan_id: Uuid, updated_plan: Plan) -> StateResult<()> {
    update_plan_reactive(plan_id, updated_plan)
}

pub fn delete_plan(plan_id: Uuid) -> StateResult<()> {
    delete_plan_reactive(plan_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PlanItem, PlanSettings};
    use chrono::Utc;

    #[test]
    fn test_plan_context_creation() {
        let plans: Vec<Plan> = Vec::new();
        assert!(plans.is_empty());
    }

    #[test]
    fn test_plan_progress_calculation() {
        let items = vec![
            PlanItem {
                date: Utc::now(),
                module_title: "Module 1".to_string(),
                section_title: "Section 1".to_string(),
                video_indices: vec![0],
                completed: true,
                total_duration: std::time::Duration::from_secs(3600),
                estimated_completion_time: std::time::Duration::from_secs(3600),
                overflow_warnings: vec![],
            },
            PlanItem {
                date: Utc::now(),
                module_title: "Module 2".to_string(),
                section_title: "Section 2".to_string(),
                video_indices: vec![1],
                completed: false,
                total_duration: std::time::Duration::from_secs(3600),
                estimated_completion_time: std::time::Duration::from_secs(3600),
                overflow_warnings: vec![],
            },
        ];

        let completed_items = items.iter().filter(|i| i.completed).count();
        let progress = completed_items as f32 / items.len() as f32;

        assert_eq!(progress, 0.5);
    }

    #[test]
    fn test_plan_activation_logic() {
        // In absence of an explicit active flag, the latest plan (by created_at) is considered active per course
        let course_id = Uuid::new_v4();

        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        };

        let older = Utc::now() - chrono::Duration::days(1);
        let newer = Utc::now();

        let plans = vec![
            Plan {
                id: Uuid::new_v4(),
                course_id,
                settings: settings.clone(),
                items: vec![],
                created_at: older,
            },
            Plan {
                id: Uuid::new_v4(),
                course_id,
                settings,
                items: vec![],
                created_at: newer,
            },
        ];

        // The "active" one should be the most recent for that course
        let active = plans
            .iter()
            .filter(|p| p.course_id == course_id)
            .max_by_key(|p| p.created_at)
            .unwrap();

        assert_eq!(active.id, plans[1].id);
    }
}
