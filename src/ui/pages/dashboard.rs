//! Dashboard page - Course overview and progress

use dioxus::prelude::*;

use crate::domain::entities::Course;
use crate::domain::ports::VideoRepository;
use crate::ui::Route;
use crate::ui::actions::{ImportResult, import_playlist};
use crate::ui::custom::{CourseCard, ImportPlaylistDialog};
use crate::ui::hooks::{use_load_courses, use_load_modules};
use crate::ui::state::AppState;

/// Dashboard showing all courses and overall progress.
#[component]
pub fn Dashboard() -> Element {
    let state = use_context::<AppState>();

    // Load courses from backend
    let mut courses = use_load_courses(state.backend.clone());

    // Import dialog state
    let mut import_open = use_signal(|| false);
    let mut import_status = use_signal(|| None::<String>);

    let backend = state.backend.clone();

    let handle_import = move |url: String| {
        let backend = backend.clone();

        spawn(async move {
            import_status.set(Some("Importing...".to_string()));

            match import_playlist(backend.clone(), url, None).await {
                ImportResult::Success { course_id: _, modules, videos } => {
                    import_status
                        .set(Some(format!("✓ Imported {} modules, {} videos", modules, videos)));

                    // Reload courses
                    if let Some(ref ctx) = backend {
                        use crate::domain::ports::CourseRepository;
                        if let Ok(loaded) = ctx.course_repo.find_all() {
                            courses.set(loaded);
                        }
                    }
                },
                ImportResult::Error(e) => {
                    import_status.set(Some(format!("✗ Error: {}", e)));
                },
            }
        });
    };

    rsx! {
        div {
            class: "p-6",

            // Header
            div {
                class: "flex items-center justify-between mb-6",
                h1 { class: "text-2xl font-bold", "Dashboard" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| import_open.set(true),
                    disabled: !state.has_youtube(),
                    "+ Import Playlist"
                }
            }

            // Import status message
            if let Some(ref status) = *import_status.read() {
                div {
                    class: if status.starts_with("✓") { "alert alert-success mb-4" } else if status.starts_with("✗") { "alert alert-error mb-4" } else { "alert alert-info mb-4" },
                    "{status}"
                }
            }

            // Status cards
            div {
                class: "grid grid-cols-3 gap-4 mb-6",
                StatusCard {
                    label: "YouTube API",
                    status: state.has_youtube(),
                }
                StatusCard {
                    label: "Gemini AI",
                    status: state.has_gemini(),
                }
                StatusCard {
                    label: "Backend",
                    status: state.has_backend(),
                }
            }

            // Course grid
            if courses.read().is_empty() {
                div {
                    class: "text-center py-12 bg-base-200 rounded-lg",
                    p { class: "text-xl mb-2", "No courses yet" }
                    p { class: "text-base-content/60", "Import a YouTube playlist to get started" }
                    div {
                        class: "flex justify-center gap-4 mt-4",
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| import_open.set(true),
                            disabled: !state.has_youtube(),
                            "Import Playlist"
                        }
                        Link {
                            to: Route::Settings {},
                            class: "btn btn-outline",
                            "Configure API Keys"
                        }
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

        // Import dialog
        ImportPlaylistDialog {
            open: import_open,
            on_import: handle_import,
        }
    }
}

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

#[component]
fn StatusCard(label: &'static str, status: bool) -> Element {
    let (icon, color) = if status { ("✓", "text-success") } else { ("✗", "text-error") };

    rsx! {
        div {
            class: "bg-base-200 p-4 rounded-lg flex items-center gap-3",
            span { class: "{color} text-xl", "{icon}" }
            span { "{label}" }
        }
    }
}
