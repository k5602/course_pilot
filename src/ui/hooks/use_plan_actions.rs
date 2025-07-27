use crate::types::Plan;
use crate::ui::components::toast::toast;
use dioxus::prelude::*;
use uuid::Uuid;

/// Hook for toggling plan item completion status
pub fn use_toggle_plan_item_action() -> Callback<(Uuid, usize)> {
    let backend = crate::ui::hooks::use_backend_adapter();

    use_callback(move |(plan_id, item_index): (Uuid, usize)| {
        let backend = backend.clone();
        spawn(async move {
            match backend
                .update_plan_item_completion(plan_id, item_index, true)
                .await
            {
                Ok(_) => {
                    toast::success("Item status updated");
                }
                Err(e) => {
                    toast::error(format!("Failed to update item: {e}"));
                }
            }
        });
    })
}

/// Hook for managing plan resources
pub fn use_plan_resource(course_id: Uuid) -> Resource<Result<Option<Plan>, anyhow::Error>> {
    let backend = crate::ui::hooks::use_backend_adapter();

    use_resource(move || {
        let backend = backend.clone();
        async move { backend.get_plan_by_course(course_id).await }
    })
}
