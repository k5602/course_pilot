use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use super::CourseGrid;
use crate::ui::components::import_modal::{ImportModal, ImportSettings, ImportSource};
use crate::ui::components::toast::toast;
use crate::ui::hooks::{use_course_manager, use_modal_manager};

/// Clean dashboard component with proper separation of concerns
#[component]
pub fn Dashboard() -> Element {
    let course_manager = use_course_manager();
    let add_course_modal = use_modal_manager(false);

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

    // Handle import from the import modal
    let handle_import = {
        let _course_manager = course_manager.clone();
        let add_course_modal = add_course_modal.clone();

        move |(source, _input, _settings): (ImportSource, String, ImportSettings)| {
            match source {
                ImportSource::LocalFolder => {
                    // Local folder import is handled by the ImportModal component itself
                    // Just close the modal as it will handle the actual import process
                    add_course_modal.close.call(());
                }
                ImportSource::YouTube => {
                    // Handle YouTube import - this will be handled by the YouTubeImportForm component
                    // The form will handle the actual import and close the modal
                }
                ImportSource::OtherResources => {
                    // This shouldn't be called since the button is disabled
                    toast::info("Other resources import is coming soon");
                    add_course_modal.close.call(());
                }
            }
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

        // Add Course Modal (now uses ImportModal with tabs)
        ImportModal {
            open: add_course_modal.is_open,
            on_close: move |_| add_course_modal.close.call(()),
            on_import: handle_import,
            on_course_imported: Some(EventHandler::new({
                let course_manager = course_manager.clone();
                move |_| {
                    course_manager.refresh.call(());
                }
            })),
        }
    }
}

/// Render dashboard content based on course manager state
fn render_dashboard_content(
    course_manager: &crate::ui::hooks::use_courses::CourseManager,
) -> Element {
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
