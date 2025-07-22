use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::ui::components::modal::Modal;
use crate::ui::components::toast::toast;
use crate::ui::components::youtube_import_form::YouTubeImportForm;
use crate::ui::hooks::{use_course_manager, use_modal_manager, use_form_manager};
use crate::types::Course;
use super::CourseGrid;

/// Clean dashboard component with proper separation of concerns
#[component]
pub fn Dashboard() -> Element {
    let course_manager = use_course_manager();
    let add_course_modal = use_modal_manager(false);
    let import_modal = use_modal_manager(false);
    let course_name_form = use_form_manager("".to_string());
    
    // Animation for grid entrance
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

    // Handle course creation
    let handle_create_course = {
        let course_manager = course_manager.clone();
        let add_course_modal = add_course_modal.clone();
        let course_name_form = course_name_form.clone();
        
        move |_| {
            let name = course_name_form.value.trim().to_string();
            if name.is_empty() {
                toast::error("Course name cannot be empty");
                return;
            }
            
            course_manager.create_course.call(name);
            add_course_modal.close.call(());
            course_name_form.reset.call(());
        }
    };

    // Handle import completion
    let handle_import_complete = {
        let course_manager = course_manager.clone();
        let import_modal = import_modal.clone();
        
        move |_course: Course| {
            // Refresh the courses list by triggering the course manager
            // The course manager will automatically refresh when a new course is created
            import_modal.close.call(());
            toast::success("Course imported and added to your library!");
        }
    };

    // Handle import error
    let handle_import_error = {
        let import_modal = import_modal.clone();
        
        move |error: String| {
            toast::error(&format!("Import failed: {}", error));
            // Keep modal open so user can try again
        }
    };

    rsx! {
        section {
            class: "w-full max-w-7xl mx-auto px-4 py-8",
            
            // Header
            div {
                class: "flex items-center justify-between mb-6",
                h1 { class: "text-2xl font-bold", "Your Courses" }
                div { class: "flex gap-2",
                    button {
                        class: "btn btn-outline",
                        onclick: move |_| import_modal.open.call(()),
                        "Import Course"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| add_course_modal.open.call(()),
                        "Add New Course"
                    }
                }
            }

            // Course grid with loading/error states
            div {
                style: "{grid_style}",
                {render_dashboard_content(&course_manager)}
            }
        }

        // Add Course Modal
        Modal {
            open: add_course_modal.is_open,
            on_close: move |_| {
                add_course_modal.close.call(());
                course_name_form.reset.call(());
            },
            title: "Add New Course".to_string(),
            actions: rsx! {
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| {
                        add_course_modal.close.call(());
                        course_name_form.reset.call(());
                    },
                    "Cancel"
                }
                button {
                    class: "btn btn-primary",
                    onclick: handle_create_course,
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
                    value: course_name_form.value,
                    oninput: move |evt| course_name_form.set_value.call(evt.value()),
                }
            }
        }

        // Import Course Modal
        Modal {
            open: import_modal.is_open,
            on_close: move |_| import_modal.close.call(()),
            title: "Import Course from YouTube".to_string(),
            actions: rsx! {
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| import_modal.close.call(()),
                    "Cancel"
                }
            },
            
            YouTubeImportForm {
                on_import_complete: handle_import_complete,
                on_import_error: handle_import_error,
            }
        }
    }
}

/// Render dashboard content based on course manager state
fn render_dashboard_content(course_manager: &crate::ui::hooks::use_courses::CourseManager) -> Element {
    if let Some(error) = &course_manager.error {
        return rsx! {
            div {
                class: "flex flex-col items-center justify-center py-12 text-error",
                "Failed to load courses: {error}"
                button {
                    class: "btn btn-outline btn-sm mt-4",
                    onclick: move |_| {
                        // Trigger refresh by showing a toast message
                        toast::info("Please refresh the page to retry loading courses");
                    },
                    "Retry"
                }
            }
        };
    }

    if course_manager.is_loading {
        return rsx! {
            div {
                class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
                {(0..3).map(|_| rsx! {
                    div {
                        class: "card bg-base-200 shadow-xl animate-pulse",
                        div { 
                            class: "card-body pb-4",
                            div { class: "h-6 w-2/3 bg-base-300 rounded mb-2" }
                            div { class: "h-4 w-1/2 bg-base-300 rounded mb-2" }
                            div { class: "h-2 w-full bg-base-300 rounded mb-2" }
                            div { class: "h-8 w-1/3 bg-base-300 rounded mt-4" }
                        }
                    }
                })}
            }
        };
    }

    if course_manager.courses.is_empty() {
        return rsx! {
            div {
                class: "flex flex-col items-center justify-center py-12 text-base-content/60",
                "No courses found. Click 'Add New Course' to get started."
            }
        };
    }

    rsx! {
        CourseGrid { courses: course_manager.courses.clone() }
    }
}