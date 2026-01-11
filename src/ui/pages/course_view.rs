//! Course view page - modules and videos with real data

use dioxus::prelude::*;

use crate::application::{ServiceFactory, use_cases::PlanSessionInput};
use crate::domain::ports::{CourseRepository, VideoRepository};
use crate::domain::value_objects::{CourseId, SessionPlan};
use crate::ui::Route;
use crate::ui::custom::VideoItem;
use crate::ui::hooks::{use_load_course, use_load_modules};
use crate::ui::state::AppState;

/// Detailed course view with modules accordion.
#[component]
pub fn CourseView(course_id: String) -> Element {
    let state = use_context::<AppState>();
    let nav = use_navigator();

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

    // State for modals
    let mut show_delete_modal = use_signal(|| false);
    let mut show_session_modal = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);
    let mut session_plans = use_signal(Vec::<SessionPlan>::new);
    let mut cognitive_limit = use_signal(|| 45u32);

    // Delete course handler
    let backend_for_delete = state.backend.clone();
    let course_id_for_delete = course_id_parsed.clone();
    let on_delete_confirm = move |_| {
        if let Ok(ref cid) = course_id_for_delete {
            if let Some(ref ctx) = backend_for_delete {
                is_deleting.set(true);
                match ctx.course_repo.delete(cid) {
                    Ok(_) => {
                        log::info!("Course deleted successfully");
                        nav.push(Route::CourseList {});
                    },
                    Err(e) => {
                        log::error!("Failed to delete course: {}", e);
                        is_deleting.set(false);
                    },
                }
            }
        }
    };

    // Session planning handler
    let backend_for_session = state.backend.clone();
    let course_id_for_session = course_id_parsed.clone();
    let on_plan_sessions = move |_| {
        if let Ok(ref cid) = course_id_for_session {
            if let Some(ref ctx) = backend_for_session {
                let use_case = ServiceFactory::plan_session(ctx);
                let input = PlanSessionInput {
                    course_id: cid.clone(),
                    cognitive_limit_minutes: *cognitive_limit.read(),
                };
                match use_case.execute(input) {
                    Ok(plans) => session_plans.set(plans),
                    Err(e) => log::error!("Failed to plan sessions: {}", e),
                }
            }
        }
    };

    rsx! {
        div {
            class: "p-6",

            // Back button and actions row
            div {
                class: "flex justify-between items-center mb-4",
                Link {
                    to: Route::CourseList {},
                    class: "btn btn-ghost btn-sm",
                    "← Back to Courses"
                }

                div { class: "flex gap-2",
                    // Plan sessions button
                    button {
                        class: "btn btn-ghost btn-sm text-primary hover:bg-primary/10",
                        onclick: move |_| {
                            session_plans.set(Vec::new());
                            show_session_modal.set(true);
                        },
                        "📅 Plan Study Sessions"
                    }

                    // Delete button
                    button {
                        class: "btn btn-ghost btn-sm text-error hover:bg-error/10",
                        onclick: move |_| show_delete_modal.set(true),
                        "🗑️ Delete"
                    }
                }
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

        // Delete confirmation modal
        if *show_delete_modal.read() {
            div {
                class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                onclick: move |_| show_delete_modal.set(false),

                div {
                    class: "bg-base-100 rounded-2xl p-6 max-w-md mx-4 shadow-2xl",
                    onclick: |e| e.stop_propagation(),

                    h3 { class: "text-xl font-bold mb-4", "Delete Course?" }
                    p { class: "text-base-content/70 mb-6",
                        "This will permanently delete this course, all its modules, videos, and any associated quizzes. This action cannot be undone."
                    }

                    div { class: "flex justify-end gap-3",
                        button {
                            class: "btn btn-ghost",
                            onclick: move |_| show_delete_modal.set(false),
                            "Cancel"
                        }
                        button {
                            class: "btn btn-error",
                            disabled: *is_deleting.read(),
                            onclick: on_delete_confirm,
                            if *is_deleting.read() {
                                span { class: "loading loading-spinner loading-sm" }
                            } else {
                                "Delete"
                            }
                        }
                    }
                }
            }
        }

        // Session planning modal
        if *show_session_modal.read() {
            div {
                class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                onclick: move |_| show_session_modal.set(false),

                div {
                    class: "bg-base-100 rounded-2xl p-6 max-w-lg mx-4 shadow-2xl max-h-[80vh] overflow-y-auto",
                    onclick: |e| e.stop_propagation(),

                    h3 { class: "text-xl font-bold mb-4", "📅 Plan Your Study Sessions" }

                    // Cognitive limit slider
                    div { class: "mb-6",
                        label { class: "block text-sm font-medium mb-2",
                            "Daily study time: {cognitive_limit} minutes"
                        }
                        input {
                            r#type: "range",
                            class: "range range-primary w-full",
                            min: "15",
                            max: "120",
                            step: "5",
                            value: "{cognitive_limit}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    cognitive_limit.set(val);
                                }
                            },
                        }
                        div { class: "flex justify-between text-xs text-base-content/50 mt-1",
                            span { "15 min" }
                            span { "45 min" }
                            span { "120 min" }
                        }
                    }

                    button {
                        class: "btn btn-primary w-full mb-4",
                        onclick: on_plan_sessions,
                        "Generate Study Plan"
                    }

                    // Session results
                    if !session_plans.read().is_empty() {
                        div { class: "space-y-3",
                            p { class: "text-sm text-base-content/70 mb-3",
                                "Estimated {session_plans.read().len()} days to complete:"
                            }
                            for plan in session_plans.read().iter() {
                                div {
                                    class: "bg-base-200 rounded-xl p-4",
                                    div { class: "flex justify-between items-center mb-2",
                                        span { class: "font-bold", "Day {plan.day}" }
                                        span { class: "text-sm text-base-content/60",
                                            "{plan.total_duration_secs / 60} min"
                                        }
                                    }
                                    p { class: "text-sm text-base-content/70",
                                        "{plan.video_indices.len()} video(s)"
                                    }
                                }
                            }
                        }
                    }

                    div { class: "mt-6 flex justify-end",
                        button {
                            class: "btn btn-ghost",
                            onclick: move |_| show_session_modal.set(false),
                            "Close"
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
