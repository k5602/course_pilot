//! Course list page

use dioxus::prelude::*;

use crate::domain::entities::Course;
use crate::ui::Route;
use crate::ui::custom::{CardSkeleton, CourseCard, ErrorAlert};
use crate::ui::hooks::{use_load_courses, use_load_modules, use_load_videos_by_course};
use crate::ui::state::AppState;

/// List of all imported courses.
#[component]
pub fn CourseList() -> Element {
    let state = use_context::<AppState>();

    {
        let mut state = state.clone();
        use_effect(move || {
            state.right_panel_visible.set(false);
            state.current_video_id.set(None);
        });
    }

    let courses = use_load_courses(state.backend.clone());
    let courses_state = courses.state.clone();

    rsx! {
        div { class: "p-6",

            h1 { class: "text-2xl font-bold mb-6", "Courses" }

            if let Some(ref err) = *courses_state.error.read() {
                ErrorAlert { message: err.clone(), on_dismiss: None }
            }

            if *courses_state.is_loading.read() && courses.data.read().is_empty() {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    CardSkeleton {}
                    CardSkeleton {}
                    CardSkeleton {}
                    CardSkeleton {}
                    CardSkeleton {}
                    CardSkeleton {}
                }
            } else if courses.data.read().is_empty() {
                div { class: "text-center py-12 bg-base-200 rounded-lg",
                    p { class: "text-xl mb-2", "No courses yet" }
                    p { class: "text-base-content/60",
                        "Import a YouTube playlist from the Dashboard to get started"
                    }
                    Link {
                        to: Route::Dashboard {},
                        class: "btn btn-primary mt-4",
                        "Go to Dashboard"
                    }
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    for course in courses.data.read().iter() {
                        CourseCardWithStats {
                            key: "{course.id().as_uuid()}",
                            course: course.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Course card with computed stats from backend.
#[component]
fn CourseCardWithStats(course: Course) -> Element {
    let state = use_context::<AppState>();
    let backend = state.backend.clone();

    let modules = use_load_modules(backend.clone(), course.id());
    let videos = use_load_videos_by_course(backend.clone(), course.id());

    let module_list = modules.data.read();
    let video_list = videos.data.read();

    let module_count = module_list.len();
    let completed_modules = if video_list.is_empty() {
        0
    } else {
        module_list
            .iter()
            .filter(|m| {
                let module_videos: Vec<_> =
                    video_list.iter().filter(|v| v.module_id() == m.id()).collect();
                !module_videos.is_empty() && module_videos.iter().all(|v| v.is_completed())
            })
            .count()
    };

    rsx! {
        CourseCard {
            id: course.id().as_uuid().to_string(),
            name: course.name().to_string(),
            module_count,
            completed_modules,
        }
    }
}
