use crate::ui::components::badge::Badge;
use crate::ui::components::course_card::CourseCard;
use crate::ui::components::modal_confirmation::{ActionMenu, CircularProgress, DropdownItem};
use crate::ui::components::progress_ring::ProgressRing;
use crate::ui::components::toast::toast;
use crate::ui::hooks::use_courses;
use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// Dashboard: Responsive grid of CourseCards, wired to AppState/backend
#[component]
pub fn Dashboard() -> Element {
    let courses = use_courses();

    // Simulate async loading state (replace with real async logic as needed)
    let is_loading = false; // Set to true to simulate loading
    let has_error = false; // Set to true to simulate error

    let courses_guard = courses.read();

    // Animate CourseCard presence/layout
    let mut grid_opacity = use_motion(0.0f32);
    let mut grid_y = use_motion(-24.0f32);

    use_effect(move || {
        grid_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        grid_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let grid_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px);",
            grid_opacity.get_value(),
            grid_y.get_value()
        )
    });

    rsx! {
        section {
            class: "w-full max-w-7xl mx-auto px-4 py-8",
            h1 { class: "text-2xl font-bold mb-6", "Your Courses" }
            if has_error {
                div {
                    class: "flex flex-col items-center justify-center py-12 text-error",
                    "Failed to load courses. Please try again."
                }
            } else if is_loading {
                div {
                    class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
                    {(0..3).map(|_| rsx! {
                        div {
                            class: "card bg-base-200 shadow-xl animate-pulse",
                            div { class: "card-body pb-4",
                                div { class: "h-6 w-2/3 bg-base-300 rounded mb-2" }
                                div { class: "h-4 w-1/2 bg-base-300 rounded mb-2" }
                                div { class: "h-2 w-full bg-base-300 rounded mb-2" }
                                div { class: "h-8 w-1/d bg-base-300 rounded mt-4" }
                            }
                        }
                    })}
                }
            } else if courses_guard.is_empty() {
                div {
                    class: "flex flex-col items-center justify-center py-12 text-base-content/60",
                    "No courses found. Click 'Add New Course' to get started."
                }
            } else {
                div {
                    class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
                    style: "{grid_style}",
                    {courses_guard.iter().enumerate().map(|(idx, course)| {
                        let progress = 0.0; // TODO: wire up real progress
                        let status = if progress >= 1.0 {
                            "Completed".to_string()
                        } else if progress > 0.0 {
                            "In Progress".to_string()
                        } else {
                            "Not Started".to_string()
                        };
                        let badge_color = if progress >= 1.0 {
                            Some("success".to_string())
                        } else if progress > 0.0 {
                            Some("accent".to_string())
                        } else {
                            Some("base-300".to_string())
                        };
                        let actions = vec![
                            DropdownItem {
                                label: "Edit Course".to_string(),
                                icon: None,
                                on_select: None,
                                children: None,
                                disabled: false,
                            },
                            DropdownItem {
                                label: "Export".to_string(),
                                icon: None,
                                on_select: None,
                                children: None,
                                disabled: false,
                            },
                            DropdownItem {
                                label: "Delete".to_string(),
                                icon: None,
                                on_select: None,
                                children: None,
                                disabled: false,
                            },
                        ];
                        rsx! {
                            div {
                                class: "relative",
                                CourseCard {
                                    id: idx,
                                    title: course.name.clone(),
                                    video_count: course.raw_titles.len(),
                                    total_duration: course.structure.as_ref().map(|s| {
                                        let secs = s.aggregate_total_duration().as_secs();
                                        let hours = secs / 3600;
                                        let mins = (secs % 3600) / 60;
                                        format!("{}h {}m", hours, mins)
                                    }).unwrap_or_else(|| "N/A".to_string()),
                                    progress,
                                }
                                div {
                                    class: "absolute top-2 left-2 z-10",
                                    Badge {
                                        label: status.clone(),
                                        color: badge_color.clone(),
                                        icon: None,
                                        class: None,
                                    }
                                }
                                div {
                                    class: "absolute top-2 right-2 z-10",
                                    ActionMenu {
                                        actions: actions.clone(),
                                        class: None,
                                    }
                                }
                                div {
                                    class: "absolute bottom-2 right-2 z-10",
                                    ProgressRing {
                                        value: (progress * 100.0).round() as u32,
                                        max: Some(100),
                                        color: Some("primary".to_string()),
                                        size: Some(36),
                                        thickness: Some(4),
                                        label: None,
                                    }
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}

// Example stub: show toast on course action (to be wired to real actions)
fn _on_course_action(action: &str) {
    toast::info(format!("Course action: {}", action));
}
