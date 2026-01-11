//! Course list page

use dioxus::prelude::*;

use crate::domain::entities::Course;
use crate::domain::ports::VideoRepository;
use crate::ui::Route;
use crate::ui::custom::CourseCard;
use crate::ui::hooks::{use_load_courses, use_load_modules};
use crate::ui::state::AppState;

/// List of all imported courses.
#[component]
pub fn CourseList() -> Element {
    let state = use_context::<AppState>();
    let courses = use_load_courses(state.backend.clone());

    rsx! {
        div {
            class: "p-6",

            h1 { class: "text-2xl font-bold mb-6", "Courses" }

            if courses.read().is_empty() {
                div {
                    class: "text-center py-12 bg-base-200 rounded-lg",
                    p { class: "text-xl mb-2", "No courses yet" }
                    p { class: "text-base-content/60", "Import a YouTube playlist from the Dashboard to get started" }
                    Link {
                        to: Route::Dashboard {},
                        class: "btn btn-primary mt-4",
                        "Go to Dashboard"
                    }
                }
            } else {
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    for course in courses.read().iter() {
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
    let mut all_videos = use_signal(Vec::new);

    let course_id = course.id().clone();
    let backend_inner = backend.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend_inner {
            if let Ok(videos) = ctx.video_repo.find_by_course(&course_id) {
                all_videos.set(videos);
            }
        }
    });

    let module_list = modules.read();
    let video_list = all_videos.read();

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
