//! Dashboard page - Course overview and progress

use dioxus::prelude::*;

use crate::domain::entities::Course;
use crate::domain::ports::{TagRepository, VideoRepository};
use crate::domain::value_objects::TagId;
use crate::ui::Route;
use crate::ui::actions::{ImportResult, import_playlist};
use crate::ui::custom::{
    CardSkeleton, CourseCard, ErrorAlert, ImportPlaylistDialog, TagFilterChip,
};
use crate::ui::hooks::{use_load_courses_state, use_load_modules, use_load_tags};
use crate::ui::state::AppState;

/// Dashboard showing all courses and overall progress.
#[component]
pub fn Dashboard() -> Element {
    let state = use_context::<AppState>();

    {
        let mut state = state.clone();
        use_effect(move || {
            state.right_panel_visible.set(false);
            state.current_video_id.set(None);
        });
    }

    // Load courses and tags from backend
    let (mut courses, courses_state) = use_load_courses_state(state.backend.clone());
    let all_tags = use_load_tags(state.backend.clone());

    // Search and filter state
    let mut search_query = use_signal(String::new);
    let mut selected_tags = use_signal(Vec::<TagId>::new);

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

    // Filter courses by search query and selected tags
    let backend_filter = state.backend.clone();
    let filtered_courses: Vec<Course> = courses
        .read()
        .iter()
        .filter(|course| {
            // Filter by search query
            let query = search_query.read();
            let matches_search = query.is_empty()
                || course.name().to_lowercase().contains(&query.to_lowercase())
                || course
                    .description()
                    .map(|d| d.to_lowercase().contains(&query.to_lowercase()))
                    .unwrap_or(false);

            // Filter by selected tags
            let sel_tags = selected_tags.read();
            let matches_tags = if sel_tags.is_empty() {
                true
            } else {
                // Check if course has any of the selected tags
                if let Some(ref ctx) = backend_filter {
                    ctx.tag_repo
                        .find_by_course(course.id())
                        .map(|course_tags| course_tags.iter().any(|ct| sel_tags.contains(ct.id())))
                        .unwrap_or(false)
                } else {
                    true
                }
            };

            matches_search && matches_tags
        })
        .cloned()
        .collect();

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

                        // Loading/error state for courses
                        if let Some(ref err) = *courses_state.error.read() {
                            ErrorAlert { message: err.clone(), on_dismiss: None }
                        }

            // Search bar
            div {
                class: "mb-4",
                div {
                    class: "relative",
                    span {
                        class: "absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40",
                        "🔍"
                    }
                    input {
                        class: "input input-bordered w-full pl-10",
                        r#type: "text",
                        placeholder: "Search courses...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
            }

            // Tag filter
            {
                let tags_list = all_tags.read().clone();
                let has_tags = !tags_list.is_empty();

                if has_tags {
                    rsx! {
                        div {
                            class: "flex flex-wrap gap-2 mb-4",

                            // "All" button
                            button {
                                class: if selected_tags.read().is_empty() {
                                    "px-3 py-1 rounded-full text-sm font-medium bg-primary text-primary-content"
                                } else {
                                    "px-3 py-1 rounded-full text-sm font-medium bg-base-200 text-base-content hover:bg-base-300"
                                },
                                onclick: move |_| selected_tags.set(Vec::new()),
                                "All"
                            }

                            // Tag chips
                            for tag in tags_list.iter() {
                                {
                                    let tag_id = tag.id().clone();
                                    let tag_id_for_check = tag_id.clone();
                                    let tag_id_for_toggle = tag_id.clone();
                                    let is_active = selected_tags.read().contains(&tag_id_for_check);
                                    rsx! {
                                        TagFilterChip {
                                            key: "{tag_id.as_uuid()}",
                                            tag: tag.clone(),
                                            active: is_active,
                                            on_click: move |_| {
                                                let mut tags = selected_tags.write();
                                                if tags.contains(&tag_id_for_toggle) {
                                                    let tid = tag_id_for_toggle.clone();
                                                    tags.retain(|t| *t != tid);
                                                } else {
                                                    tags.push(tag_id_for_toggle.clone());
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
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
                        if *courses_state.is_loading.read() && courses.read().is_empty() {
                            div {
                                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                                CardSkeleton {}
                                CardSkeleton {}
                                CardSkeleton {}
                                CardSkeleton {}
                                CardSkeleton {}
                                CardSkeleton {}
                            }
                        } else if filtered_courses.is_empty() {
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
                                    class: "text-center py-12 bg-base-200 rounded-lg",
                                    p { class: "text-xl mb-2", "No matching courses" }
                                    p { class: "text-base-content/60", "Try adjusting your search or filters" }
                                }
                            }
                        } else {
                            div {
                                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                                for course in filtered_courses.iter() {
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
