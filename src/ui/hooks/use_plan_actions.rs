use crate::storage::core::Database;
use crate::types::Plan;
use crate::ui::toast_helpers;
use anyhow::Result;
use dioxus::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

/// Hook for toggling plan item completion status
pub fn use_toggle_plan_item_action() -> Callback<(Uuid, usize)> {
    let db = use_context::<Arc<Database>>();

    use_callback(move |(plan_id, item_index): (Uuid, usize)| {
        let db = db.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
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
                plan.items[item_index].completed = true;

                // Save updated plan
                crate::storage::save_plan(&db, &plan).map_err(Into::into)
            })
            .await;

            match result {
                Ok(Ok(_)) => {
                    toast_helpers::success("Item status updated");
                },
                Ok(Err(e)) => {
                    toast_helpers::error(format!("Failed to update item: {e}"));
                },
                Err(e) => {
                    toast_helpers::error(format!("Failed to update item: {e}"));
                },
            }
        });
    })
}

/// Hook for managing plan resources
pub fn use_plan_resource(course_id: Uuid) -> Resource<Result<Option<Plan>, anyhow::Error>> {
    let db = use_context::<Arc<Database>>();

    use_resource(move || {
        let db = db.clone();
        async move {
            tokio::task::spawn_blocking(move || {
                crate::storage::get_plan_by_course_id(&db, &course_id).map_err(Into::into)
            })
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
        }
    })
}
