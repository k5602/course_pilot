use crate::ui::components::toast::toast;
use crate::ui::hooks::use_plan;
use course_pilot::types;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaSquare};
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;
use uuid::Uuid;

/// PlanView: Checklist of modules and sections with progress and session controls, wired to AppState/backend
#[component]
pub fn PlanView(course_id: Uuid) -> Element {
    let plan_future = use_plan(course_id);
    match plan_future.state() {
        UseFutureState::Pending => {
            toast::info("Loading study plan...");
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
        UseFutureState::Errored(err) => {
            toast::error(format!("Failed to load study plan: {err}"));
            return rsx! {
                section {
                    class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
                    div { class: "text-error", {format!("Failed to load study plan: {err}")}}
                }
            };
        }
        UseFutureState::Ready => {
            let plan_opt = plan_future.data();
            let plan = match plan_opt.and_then(|x| x.as_ref()) {
                Some(plan) => plan,
                None => {
                    return rsx! {
                        section {
                            class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
                            div { class: "text-base-content/60", "No study plan found for this course." }
                        }
                    };
                }
            };

            let total_sections = plan.items.len();
            let completed_sections = plan.items.iter().filter(|s| s.completed).count();
            let progress = if total_sections > 0 {
                (completed_sections as f32 / total_sections as f32 * 100.0).round() as u8
            } else {
                0
            };

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
                            PlanChecklistItem { item: item.clone(), idx }
                        })}
                    }
                }
            };
        }
    }
}

/// PlanChecklistItem: Single checklist item for a plan section/video
#[component]
fn PlanChecklistItem(item: types::PlanItem, idx: usize) -> Element {
    let check_icon = if item.completed {
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
            label: if item.completed { "Done".to_string() } else { "Pending".to_string() },
            color: Some(if item.completed { "success".to_string() } else { "accent".to_string() }),
            class: Some("ml-2".to_string()),
        }
    };

    let text_classes = if item.completed {
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

    // Modal state for destructive action
    let mut show_delete_modal = use_signal(|| false);

    rsx! {
        li {
            class: "flex items-center gap-3 px-2 py-2 rounded hover:bg-base-300 transition-colors cursor-pointer",
            style: "{item_style}",
            // on_click: move |_| { ...toggle complete... },
            {check_icon}
            span { class: "flex-1 text-sm {text_classes}", "{item.module_title} / {item.section_title}" }
            {status_badge}
            span { class: "text-xs text-base-content/60", "{item.date.format(\"%Y-%m-%d\")}" }
            crate::ui::components::modal_confirmation::ActionMenu {
                actions: vec![
                    crate::ui::components::modal_confirmation::DropdownItem {
                        label: "Delete".to_string(),
                        icon: None,
                        on_select: Some(EventHandler::new({
                            let mut show_delete_modal = show_delete_modal.clone();
                            move |_| show_delete_modal.set(true)
                        })),
                        children: None,
                        disabled: false,
                    }
                ],
                class: Some("ml-2".to_string()),
            }
            crate::ui::components::modal_confirmation::ModalConfirmation {
                open: show_delete_modal(),
                title: "Delete Plan Item".to_string(),
                message: "Are you sure you want to delete this plan item? This action cannot be undone.".to_string(),
                confirm_label: Some("Delete".to_string()),
                cancel_label: Some("Cancel".to_string()),
                confirm_color: Some("error".to_string()),
                on_confirm: Some(EventHandler::new({
                    let mut show_delete_modal = show_delete_modal.clone();
    // PlanItem has no id field; use another unique identifier if needed
    // let plan_id = item.id;
                    move |_| {
                        show_delete_modal.set(false);
                        // Async delete using backend_adapter
                        let backend_adapter = crate::ui::hooks::use_backend_adapter();
                        let show_toast = crate::ui::hooks::use_show_toast();
                        spawn(async move {
                            match backend_adapter.delete_plan(plan_id).await {
                                Ok(_) => show_toast("Plan item deleted", crate::ui::components::toast::ToastVariant::Success),
                                Err(e) => show_toast(&format!("Failed to delete plan item: {e}"), crate::ui::components::toast::ToastVariant::Error),
                            }
                        });
                    }
                })),
                on_cancel: Some(EventHandler::new({
                    let mut show_delete_modal = show_delete_modal.clone();
                    move |_| show_delete_modal.set(false)
                })),
            }
        }
    }
}
