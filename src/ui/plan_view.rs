use crate::ui::components::toast::toast;
use crate::ui::hooks::use_plan_resource;
use crate::types::{self, PlanExt};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaSquare};
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;
use uuid::Uuid;

/// PlanView: Checklist of modules and sections with progress and session controls, wired to AppState/backend
#[component]
pub fn PlanView(course_id: Uuid) -> Element {
    let plan_resource = use_plan_resource(course_id);
    
    // Show loading toast only once when plan is None - separate the read and write
    use_effect(use_reactive!(|plan_resource| {
        if plan_resource.read().is_none() {
            spawn(async move {
                toast::info("Loading study plan...");
            });
        }
    }));
    
    match &*plan_resource.read_unchecked() {
        None => {
            return rsx! {
                section {
                    class: "w-full max-w-3xl mx-auto px-4 py-8",
                    h1 { class: "text-2xl font-bold mb-6", "Study Plan" }
                    div { class: "h-4 w-1/2 bg-base-300 rounded mb-6 animate-pulse" },
                    div { class: "h-3 w-full bg-base-300 rounded mb-6 animate-pulse" },
                    div { class: "h-2 w-1/3 bg-base-300 rounded animate-pulse" }
                }
            };
        }
        Some(Err(err)) => {
            let error_msg = format!("Failed to load study plan: {err}");
            // Show error toast in a spawn to avoid render-time signal writes
            spawn(async move {
                toast::error(error_msg);
            });
            return rsx! {
                section {
                    class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
                    div { class: "text-error", "Failed to load study plan." }
                }
            };
        }
        Some(Ok(Some(plan))) => {
            // Use enhanced progress calculation
            let (completed_sections, total_sections, progress_percentage) = plan.calculate_progress();
            let progress = progress_percentage.round() as u8;

            // Animate checklist presence
            let mut list_opacity = use_motion(0.0f32);
            let mut list_y = use_motion(-16.0f32);

            use_effect(move || {
                list_opacity.animate_to(
                    1.0,
                    AnimationConfig::new(AnimationMode::Tween(Tween::default())),
                );
                list_y.animate_to(
                    0.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring::default())),
                );
            });

            let list_style = use_memo(move || {
                format!(
                    "opacity: {}; transform: translateY({}px);",
                    list_opacity.get_value(),
                    list_y.get_value()
                )
            });

            return rsx! {
                section {
                    class: "w-full max-w-3xl mx-auto px-4 py-8",
                    h1 { class: "text-2xl font-bold mb-6", "Study Plan" }
                    // Circular progress indicator
                    div { class: "flex items-center gap-4 mb-6",
                        crate::ui::components::modal_confirmation::CircularProgress {
                            value: progress,
                            size: Some(56),
                            color: Some("accent".to_string()),
                            label: Some(format!("{}/{} Complete", completed_sections, total_sections)),
                            class: Some("mr-2".to_string()),
                        }
                        crate::ui::components::modal_confirmation::Badge {
                            label: if progress == 100 { "Completed".to_string() } else { "In Progress".to_string() },
                            color: Some(if progress == 100 { "success".to_string() } else { "accent".to_string() }),
                            class: Some("ml-2".to_string()),
                        }
                        // Example destructive action
                        crate::ui::components::modal_confirmation::ActionMenu {
                            actions: vec![
                                crate::ui::components::modal_confirmation::DropdownItem {
                                    label: "Clear Plan".to_string(),
                                    icon: None,
                                    on_select: Some(EventHandler::new(|_| {
                                        crate::ui::components::toast::toast::warning("Clear plan (stub)");
                                    })),
                                    children: None,
                                    disabled: false,
                                }
                            ],
                            class: Some("ml-auto".to_string()),
                        }
                    }
                    // Checklist
                    ul {
                        class: "space-y-4",
                        style: "{list_style}",
                        {plan.items.iter().enumerate().map(|(idx, item)| rsx! {
                            PlanChecklistItem { 
                                plan_id: plan.id,
                                item: item.clone(), 
                                item_index: idx 
                            }
                        })}
                    }
                }
            };
        }
        Some(Ok(None)) => {
            // No plan exists for this course
            return rsx! {
                section {
                    class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
                    h1 { class: "text-2xl font-bold mb-6", "Study Plan" }
                    div { class: "text-base-content/60 text-center", 
                        "No study plan found for this course."
                        br {}
                        "Create a plan to start tracking your progress."
                    }
                    button {
                        class: "btn btn-primary mt-4",
                        onclick: move |_| {
                            spawn(async move {
                                toast::info("Plan creation not implemented yet");
                            });
                        },
                        "Create Study Plan"
                    }
                }
            };
        }
    }
}

/// PlanChecklistItem: Single checklist item for a plan section/video
#[component]
fn PlanChecklistItem(
    plan_id: Uuid,
    item: types::PlanItem, 
    item_index: usize
) -> Element {
    let toggle_completion = crate::ui::hooks::use_toggle_plan_item_action();
    let mut local_completed = use_signal(|| item.completed);
    
    // Sync local state with prop changes
    use_effect(move || {
        local_completed.set(item.completed);
    });
    
    let toggle_handler = move |_| {
        let new_state = !local_completed();
        
        // Optimistic update
        local_completed.set(new_state);
        
        // Call backend
        toggle_completion(plan_id, item_index, new_state);
    };

    let check_icon = if local_completed() {
        rsx! {
            Icon { icon: FaCheckDouble, class: "w-5 h-5 text-success" }
        }
    } else {
        rsx! {
            Icon { icon: FaSquare, class: "w-5 h-5 text-base-content/40" }
        }
    };
    let status_badge = rsx! {
        crate::ui::components::modal_confirmation::Badge {
            label: if local_completed() { "Done".to_string() } else { "Pending".to_string() },
            color: Some(if local_completed() { "success".to_string() } else { "accent".to_string() }),
            class: Some("ml-2".to_string()),
        }
    };

    let text_classes = if local_completed() {
        "line-through text-base-content/40"
    } else {
        "text-base-content"
    };

    // Animate checklist item presence/completion
    let mut item_opacity = use_motion(0.0f32);
    let mut item_x = use_motion(-12.0f32);

    use_effect(move || {
        item_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        item_x.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let item_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateX({}px); transition: opacity 0.3s, transform 0.3s;",
            item_opacity.get_value(),
            item_x.get_value()
        )
    });

    rsx! {
        li {
            class: "flex items-center gap-3 px-2 py-2 rounded hover:bg-base-300 transition-colors cursor-pointer",
            style: "{item_style}",
            onclick: toggle_handler,
            {check_icon}
            span { class: "flex-1 text-sm {text_classes}", "{item.module_title} / {item.section_title}" }
            {status_badge}
            span { class: "text-xs text-base-content/60", "{item.date.format(\"%Y-%m-%d\")}" }
        }
    }
}
