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
    let plan = use_plan(course_id);

    // Simulate async loading and error state (replace with real async logic as needed)
    let is_loading = false; // Set to true to simulate loading
    let has_error = false; // Set to true to simulate error

    if has_error {
        toast::error("Failed to load study plan. Please try again.");
        return rsx! {
            section {
                class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
                div { class: "text-error", "Failed to load study plan. Please try again." }
            }
        };
    }

    if is_loading {
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
                style: "{list_style}",
                {plan.items.iter().enumerate().map(|(idx, item)| rsx! {
                    PlanChecklistItem { item: item.clone(), idx }
                })}
            }
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

    rsx! {
        li {
            class: "flex items-center gap-3 px-2 py-2 rounded hover:bg-base-300 transition-colors cursor-pointer",
            style: "{item_style}",
            // on_click: move |_| { ...toggle complete... },
            {check_icon}
            span { class: "flex-1 text-sm {text_classes}", "{item.module_title} / {item.section_title}" }
            span { class: "text-xs text-base-content/60", "{item.date.format(\"%Y-%m-%d\")}" }
        }
    }
}
