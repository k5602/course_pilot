//! Course view page - modules and videos with real data

use dioxus::prelude::*;

use crate::domain::ports::VideoRepository;
use crate::domain::value_objects::CourseId;
use crate::ui::Route;
use crate::ui::custom::VideoItem;
use crate::ui::hooks::{use_load_course, use_load_modules};
use crate::ui::state::AppState;

/// Detailed course view with modules accordion.
#[component]
pub fn CourseView(course_id: String) -> Element {
    let state = use_context::<AppState>();

    // Parse course ID
    let course_id_parsed = CourseId::from_str(&course_id);

    // Load course and modules
    let course = match &course_id_parsed {
        Ok(id) => use_load_course(state.backend.clone(), id),
        Err(_) => use_signal(|| None),
    };

    let modules = match &course_id_parsed {
        Ok(id) => use_load_modules(state.backend.clone(), id),
        Err(_) => use_signal(Vec::new),
    };

    let all_videos = match &course_id_parsed {
        Ok(id) => {
            let mut videos = use_signal(Vec::new);
            let id = id.clone();
            let backend = state.backend.clone();
            use_effect(move || {
                if let Some(ref ctx) = backend {
                    if let Ok(loaded) = ctx.video_repo.find_by_course(&id) {
                        videos.set(loaded);
                    }
                }
            });
            videos
        },
        Err(_) => use_signal(Vec::new),
    };

    let total_videos = all_videos.read().len();
    let completed_videos = all_videos.read().iter().filter(|v| v.is_completed()).count();
    let progress = if total_videos > 0 {
        (completed_videos as f32 / total_videos as f32) * 100.0
    } else {
        0.0
    };

    rsx! {
        div {
            class: "p-6",

            // Back button
            Link {
                to: Route::CourseList {},
                class: "btn btn-ghost btn-sm mb-4",
                "← Back to Courses"
            }

            // Course header
            if let Some(ref c) = *course.read() {
                div {
                    h1 { class: "text-2xl font-bold mb-2", "{c.name()}" }
                    if let Some(desc) = c.description() {
                        p { class: "text-base-content/70 mb-4", "{desc}" }
                    }
                }
            } else {
                h1 { class: "text-2xl font-bold mb-2", "Course: {course_id}" }
            }

            // Progress bar
            div {
                class: "w-full max-w-md bg-base-300 rounded-full h-3 mb-6",
                div {
                    class: "bg-primary h-3 rounded-full transition-all",
                    style: "width: {progress}%",
                }
            }

            // Modules accordion
            div {
                class: "space-y-4",

                if modules.read().is_empty() {
                    div {
                        class: "text-center py-8 bg-base-200 rounded-lg",
                        p { class: "text-base-content/60", "No modules found" }
                    }
                } else {
                    for module in modules.read().iter() {
                        ModuleAccordion {
                            course_id: course_id.clone(),
                            module_id: module.id().as_uuid().to_string(),
                            title: module.title().to_string(),
                        }
                    }
                }
            }
        }
    }
}

/// Module accordion with lazy-loaded videos.
#[component]
fn ModuleAccordion(course_id: String, module_id: String, title: String) -> Element {
    let state = use_context::<AppState>();

    // Load videos for this module
    let mut videos = use_signal(Vec::new);
    let module_id_clone = module_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = state.backend {
            if let Ok(mid) = crate::domain::value_objects::ModuleId::from_str(&module_id_clone) {
                if let Ok(loaded) = ctx.video_repo.find_by_module(&mid) {
                    videos.set(loaded);
                }
            }
        }
    });

    rsx! {
        div {
            class: "collapse collapse-arrow bg-base-200",
            input { r#type: "checkbox", checked: true }
            div {
                class: "collapse-title font-medium",
                "{title}"
                span {
                    class: "text-sm text-base-content/60 ml-2",
                    "({videos.read().len()} videos)"
                }
            }
            div {
                class: "collapse-content",
                if videos.read().is_empty() {
                    p { class: "text-base-content/60 py-2", "No videos in this module" }
                } else {
                    for video in videos.read().iter() {
                        VideoItem {
                            course_id: course_id.clone(),
                            video_id: video.id().as_uuid().to_string(),
                            title: video.title().to_string(),
                            duration_secs: video.duration_secs(),
                            is_completed: video.is_completed(),
                        }
                    }
                }
            }
        }
    }
}
