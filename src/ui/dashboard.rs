use crate::ui::components::badge::Badge;
use crate::ui::components::course_card::CourseCard;
use crate::ui::components::modal_confirmation::{ActionMenu, DropdownItem};
use crate::ui::components::progress_ring::ProgressRing;
use crate::ui::components::toast::toast;
use crate::ui::components::modal::Modal;
use crate::ui::hooks::use_backend_adapter;
use crate::types::Course;

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// Dashboard: Responsive grid of CourseCards, wired to AppState/backend
#[component]
pub fn Dashboard() -> Element {
    // Async resource for courses using backend adapter
    let backend = use_backend_adapter();
    let courses_resource = use_resource(move || {
        let backend = backend.clone();
        async move {
            backend.list_courses().await
        }
    });

    let courses_state = courses_resource.read_unchecked();
    let is_loading = matches!(*courses_state, None);
    let has_error = matches!(*courses_state, Some(Err(_)));
    let courses_data = match &*courses_state {
        Some(Ok(data)) => Some(data),
        _ => None,
    };

    let mut show_add_course_modal = use_signal(|| false);
    let mut new_course_name = use_signal(|| "".to_string());


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
            div {
                class: "flex items-center justify-between mb-6",
                h1 { class: "text-2xl font-bold", "Your Courses" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| show_add_course_modal.set(true),
                    "Add New Course"
                }
            }

            // Add Course Modal
            Modal {
                open: show_add_course_modal(),
                on_close: move |_| show_add_course_modal.set(false),
                title: "Add New Course".to_string(),
                actions: rsx! {
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| show_add_course_modal.set(false),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let course_name = new_course_name();
                            if !course_name.is_empty() {
                                let backend = use_backend_adapter();
                                let new_course = Course::new(course_name.to_string(), vec![]);
                                // Async event handler for course creation
                                spawn(async move {
                                    if let Err(e) = backend.create_course(new_course).await {
                                        toast::error(&format!("Failed to add course: {e}"));
                                    }
                                });
                                show_add_course_modal.set(false);
                                new_course_name.set("".to_string());
                            } else {
                                toast::error("Course name cannot be empty.");
                            }
                        },
                        "Create"
                    }
                },
                div {
                    class: "form-control w-full",
                    label {
                        class: "label",
                        span { class: "label-text", "Course Name" }
                    }
                    input {
                        r#type: "text",
                        placeholder: "Enter course name",
                        class: "input input-bordered w-full",
                        value: new_course_name.clone(),
                        oninput: move |evt| new_course_name.set(evt.value()),
                    }
                }
            }

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
            } else if courses_data.map_or(true, |data| data.is_empty()) {
                div {
                    class: "flex flex-col items-center justify-center py-12 text-base-content/60",
                    "No courses found. Click 'Add New Course' to get started."
                }
            } else {
                div {
                    class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
                    style: "{grid_style}",
                    {courses_data.map_or_else(Vec::new, |courses| {
                        courses.iter().enumerate().map(|(idx, course)| {
                            rsx! {
                                CourseCardWithProgress {
                                    key: "{course.id}",
                                    course: course.clone(),
                                    index: idx,
                                }
                            }
                        }).collect::<Vec<_>>()
                    }).into_iter()}
                }
            }
        }
    }
}

/// CourseCardWithProgress: Individual course card that loads and displays real progress data
#[component]
fn CourseCardWithProgress(course: Course, index: usize) -> Element {
    let backend = use_backend_adapter();
    
    // Load progress data for this course
    let progress_resource = use_resource(move || {
        let backend = backend.clone();
        let course_id = course.id;
        async move {
            backend.get_course_progress(course_id).await
        }
    });
    
    let (progress, status, badge_color) = match &*progress_resource.read_unchecked() {
        Some(Ok(Some(progress_info))) => {
            let progress_percentage = progress_info.percentage / 100.0; 
            let status = if progress_percentage >= 1.0 {
                "Completed".to_string()
            } else if progress_percentage > 0.0 {
                "In Progress".to_string()
            } else {
                "Not Started".to_string()
            };
            let badge_color = if progress_percentage >= 1.0 {
                Some("success".to_string())
            } else if progress_percentage > 0.0 {
                Some("accent".to_string())
            } else {
                Some("neutral".to_string())
            };
            (progress_percentage, status, badge_color)
        },
        Some(Ok(None)) => {
            // No plan exists for this course
            (0.0, "Not Started".to_string(), Some("neutral".to_string()))
        },
        Some(Err(_)) => {
            // Error loading progress
            (0.0, "Error".to_string(), Some("error".to_string()))
        },
        None => {
            // Still loading
            (0.0, "Loading...".to_string(), Some("neutral".to_string()))
        }
    };
    
    let actions = vec![
        DropdownItem {
            label: "View Plan".to_string(),
            icon: None,
            on_select: Some(EventHandler::new(move |_| {
                // TODO: Navigate to plan view
                toast::info("Plan view navigation not implemented yet");
            })),
            children: None,
            disabled: false,
        },
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
                id: index,
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
}
