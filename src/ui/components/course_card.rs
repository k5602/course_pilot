use course_pilot::types::ContextualPanelTab;
use crate::ui::hooks::use_app_state;
use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaEllipsis, FaPlus};

/// Props for the CourseCard component
#[derive(Props, PartialEq, Clone)]
pub struct CourseCardProps {
    pub id: usize,
    pub title: String,
    pub video_count: usize,
    pub total_duration: String,
    pub progress: f32, // 0.0 - 1.0
                       // Add more props as needed (e.g., on_export, on_delete)
}

/// DaisyUI-based CourseCard with progress, actions, and hover effects
#[component]
pub fn CourseCard(props: CourseCardProps) -> Element {
    let progress_percent = (props.progress * 100.0).round() as u32;
    let mut app_state = use_app_state();
    let mut scale = use_motion(1.0f32);
    let mut y = use_motion(0.0f32);

    let card_style = use_memo(move || {
        format!(
            "transform: translateY({}px) scale({});",
            y.get_value(),
            scale.get_value()
        )
    });

    rsx! {
        div {
            style: "{card_style}",
            onmouseenter: move |_| {
                scale.animate_to(1.02, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                y.animate_to(-5.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
            },
            onmouseleave: move |_| {
                scale.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                y.animate_to(0.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
            },
            div {
                class: "card w-full bg-base-100 shadow-xl border border-base-300 hover:border-primary transition-all duration-300 ease-in-out",
                div {
                    class: "card-body p-4",
                    div {
                        class: "flex justify-between items-start",
                        h2 {
                            class: "card-title text-lg",
                            "{props.title}"
                        }
                        div {
                            class: "dropdown dropdown-end",
                            label {
                                tabindex: 0,
                                class: "btn btn-ghost btn-sm btn-circle",
                                Icon { icon: FaEllipsis, width: 16, height: 16 }
                            }
                            ul {
                                tabindex: 0,
                                class: "dropdown-content menu p-2 shadow bg-base-200 rounded-box w-52",
                                li { a { "Edit Course" } }
                                li { a { "Archive Course" } }
                            }
                        }
                    }
                    p {
                        class: "text-sm text-base-content/70 h-12 overflow-hidden",
                        "{props.video_count} videos • {props.total_duration}"
                    }
                    div {
                        class: "card-actions justify-between items-center mt-4",
                        div {
                            class: "flex items-center gap-2 text-sm",
                            Icon { icon: FaCheckDouble, width: 16, height: 16, class: "text-success" }
                            span { "{progress_percent}% complete" }
                        }
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: move |_| {
                                let mut state = app_state.write();
                                state.contextual_panel.is_open = true;
                                state.contextual_panel.active_tab = ContextualPanelTab::Notes;
                            },
                            Icon { icon: FaPlus, width: 16, height: 16, class: "mr-2" }
                            "View Notes"
                        }
                    }
                }
            }
        }
    }
}
