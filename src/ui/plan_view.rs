use crate::ui::hooks::use_plan;
use course_pilot::types;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaSquare};
use uuid::Uuid;

/// PlanView: Checklist of modules and sections with progress and session controls, wired to AppState/backend
#[component]
pub fn PlanView(course_id: Uuid) -> Element {
    let plan = use_plan(course_id);

    // Simulate async loading and error state (replace with real async logic as needed)
    let is_loading = false; // Set to true to simulate loading
    let has_error = false; // Set to true to simulate error

    if has_error {
        return rsx! {
            section {
                class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
                div { class: "text-error", "Failed to load study plan. Please try again." }
            }
        };
    }

    if is_loading {
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

    let plan_guard = plan.read();
    let plan = match plan_guard.as_ref() {
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
        (completed_sections as f32 / total_sections as f32 * 100.0).round() as u32
    } else {
        0
    };

    rsx! {
        section {
            class: "w-full max-w-3xl mx-auto px-4 py-8",
            h1 { class: "text-2xl font-bold mb-6", "Study Plan" }
            // Progress bar
            div {
                class: "w-full bg-base-300 rounded-full h-4 mb-6",
                div {
                    class: "bg-accent h-4 rounded-full transition-all duration-500",
                    style: "width: {progress}%;"
                }
            }
            // Checklist
            ul {
                class: "space-y-4",
                {plan.items.iter().enumerate().map(|(_i, item)| rsx! {
                    PlanChecklistItem { item: item.clone() }
                })}
            }
        }
    }
}

/// PlanChecklistItem: Single checklist item for a plan section/video
#[component]
fn PlanChecklistItem(item: types::PlanItem) -> Element {
    let check_icon = if item.completed {
        rsx! {
            Icon { icon: FaCheckDouble, class: "w-5 h-5 text-success" }
        }
    } else {
        rsx! {
            Icon { icon: FaSquare, class: "w-5 h-5 text-base-content/40" }
        }
    };

    let text_classes = if item.completed {
        "line-through text-base-content/40"
    } else {
        "text-base-content"
    };

    rsx! {
        li {
            class: "flex items-center gap-3 px-2 py-2 rounded hover:bg-base-300 transition-colors cursor-pointer",
            // on_click: move |_| { ...toggle complete... },
            {check_icon}
            span { class: "flex-1 text-sm {text_classes}", "{item.module_title} / {item.section_title}" }
            span { class: "text-xs text-base-content/60", "{item.date.format(\"%Y-%m-%d\")}" }
        }
    }
}
