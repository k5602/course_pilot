use crate::ui::components::modal_confirmation::{
    ActionMenu, Badge, CircularProgress, DropdownItem,
};
use crate::ui::hooks::use_app_state;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaDownload, FaPen, FaTrash};
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;

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
    let progress_percent = (props.progress * 100.0).round() as u8;
    let _app_state = use_app_state();
    let mut scale = use_motion(1.0f32);
    let mut y = use_motion(0.0f32);

    let card_style = use_memo(move || {
        format!(
            "transform: translateY({}px) scale({});",
            y.get_value(),
            scale.get_value()
        )
    });

    // Example status logic (replace with real logic as needed)
    let status = if progress_percent >= 100 {
        "Completed"
    } else if progress_percent > 0 {
        "In Progress"
    } else {
        "Not Started"
    };

    // Example actions for ActionMenu/EnhancedDropdown
    let actions = vec![
        DropdownItem {
            label: "Edit Course".to_string(),
            icon: Some(rsx!(Icon {
                icon: FaPen,
                class: "w-4 h-4"
            })),
            on_select: None,
            children: None,
            disabled: false,
        },
        DropdownItem {
            label: "Export".to_string(),
            icon: Some(rsx!(Icon {
                icon: FaDownload,
                class: "w-4 h-4"
            })),
            on_select: None,
            children: None,
            disabled: false,
        },
        DropdownItem {
            label: "Delete".to_string(),
            icon: Some(rsx!(Icon {
                icon: FaTrash,
                class: "w-4 h-4 text-error"
            })),
            on_select: None,
            children: None,
            disabled: false,
        },
    ];

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
                            class: "card-title text-lg flex items-center gap-2",
                            "{props.title}"
                            Badge { label: status.to_string(), color: Some(if status == "Completed" { "success".to_string() } else if status == "In Progress" { "accent".to_string() } else { "base-300".to_string() }), class: Some("ml-2".to_string()) }
                        }
                        ActionMenu { actions: actions.clone(), class: Some("".to_string()) }
                    }
                    p {
                        class: "text-sm text-base-content/70 h-12 overflow-hidden",
                        "{props.video_count} videos \u{2022} {props.total_duration}"
                    }
                    div {
                        class: "card-actions justify-between items-center mt-4",
                        div {
                            class: "flex items-center gap-2 text-sm",
                            CircularProgress { value: progress_percent, size: Some(36), color: Some("accent".to_string()), label: None, class: Some("mr-2".to_string()) }
                            span { "{progress_percent}% complete" }
                        }
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: move |_| {
                                // Example: open notes panel (update as needed)
                            },
                            Icon { icon: FaPen, class: "mr-2 w-4 h-4" }
                            "View Notes"
                        }
                    }
                }
            }
        }
    }
}
