//! Video player page - Sanctuary for focused learning.

use dioxus::prelude::*;

use crate::domain::ports::VideoRepository;
use crate::domain::value_objects::{CourseId, VideoId};
use crate::ui::Route;
use crate::ui::actions::start_exam;
use crate::ui::custom::YouTubePlayer;
use crate::ui::hooks::{use_load_modules, use_load_video};
use crate::ui::state::AppState;

/// Video player with controls and completion actions.
#[component]
pub fn VideoPlayer(course_id: String, video_id: String) -> Element {
    let mut state = use_context::<AppState>();
    let backend = state.backend.clone();
    let nav = use_navigator();

    // Parse IDs
    let course_id_vo = match CourseId::from_str(&course_id) {
        Ok(id) => id,
        Err(_) => return rsx! { div { class: "p-6 text-error", "Invalid Course ID" } },
    };

    let video_id_vo = match VideoId::from_str(&video_id) {
        Ok(id) => id,
        Err(_) => return rsx! { div { class: "p-6 text-error", "Invalid Video ID" } },
    };

    // Load data
    let video = use_load_video(backend.clone(), &video_id_vo);
    let modules = use_load_modules(backend.clone(), &course_id_vo);

    // Track current video in global state for AI companion context
    let video_id_for_state = video_id.clone();
    use_effect(move || {
        state.current_video_id.set(Some(video_id_for_state.clone()));
    });

    // Extract video data reactively
    let video_read = video.read();
    let v = match video_read.as_ref() {
        Some(v) => v.clone(),
        None => return rsx! { div { class: "p-6 animate-pulse", "Loading video..." } },
    };

    // Find current module name
    let module_title = modules
        .read()
        .iter()
        .find(|m| m.id() == v.module_id())
        .map(|m| m.title().to_string())
        .unwrap_or_else(|| "Module".to_string());

    // Clone data for closures
    let backend_for_complete = backend.clone();
    let backend_for_quiz = backend.clone();
    let video_id_for_complete = v.id().clone();
    let video_id_for_quiz = v.id().clone();
    let is_completed = v.is_completed();

    // Handlers
    let on_mark_complete = move |_| {
        if let Some(ctx) = backend_for_complete.as_ref() {
            let new_status = !is_completed;
            if let Err(e) = ctx.video_repo.update_completion(&video_id_for_complete, new_status) {
                log::error!("Failed to update completion: {}", e);
            }
        }
    };

    let on_take_quiz = move |_| {
        let backend_inner = backend_for_quiz.clone();
        let vid = video_id_for_quiz.clone();
        spawn(async move {
            match start_exam(backend_inner, vid).await {
                Ok(exam_id) => {
                    nav.push(Route::QuizView { exam_id: exam_id.as_uuid().to_string() });
                },
                Err(e) => {
                    log::error!("Failed to start exam: {}", e);
                },
            }
        });
    };

    rsx! {
        div {
            class: "p-6 h-full flex flex-col max-w-5xl mx-auto",

            // Header/Nav
            div { class: "flex justify-between items-center mb-6",
                Link {
                    to: Route::CourseView { course_id: course_id.clone() },
                    class: "btn btn-ghost btn-sm gap-2",
                    "← Back to Course"
                }
                div { class: "flex items-center gap-2 text-sm font-medium opacity-60",
                    span { "{module_title}" }
                    span { "•" }
                    span { "{v.duration_secs() / 60} min" }
                }
            }

            // Video player section
            div { class: "aspect-video w-full rounded-3xl overflow-hidden shadow-2xl bg-black border-4 border-base-300",
                YouTubePlayer { video_id: v.youtube_id().as_str() }
            }

            // Info & Actions
            div { class: "mt-8 flex flex-col md:flex-row md:items-start justify-between gap-6",
                div { class: "flex-1",
                    h1 { class: "text-3xl font-bold mb-2", "{v.title()}" }
                    p { class: "text-base-content/60",
                        if v.is_completed() {
                            span { class: "text-success font-medium flex items-center gap-1",
                                "✓ Completed"
                            }
                        } else {
                            span { "Not yet completed" }
                        }
                    }
                }

                div { class: "flex flex-wrap gap-3",
                    button {
                        class: if v.is_completed() { "btn btn-success" } else { "btn btn-outline btn-success" },
                        onclick: on_mark_complete,
                        if v.is_completed() { "✓ Completed" } else { "Mark Complete" }
                    }
                    button {
                        class: "btn btn-primary gap-2",
                        onclick: on_take_quiz,
                        "📝 Take Quiz"
                    }
                }
            }

            // Navigation Footer
            div { class: "mt-auto pt-12 flex justify-between border-t border-base-300",
                // Note: Logic for finding next/prev video would go here
                // For now we keep placeholders that go back to course view
                Link {
                    to: Route::CourseView { course_id: course_id.clone() },
                    class: "group flex items-center gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all",
                    div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center group-hover:bg-primary group-hover:text-primary-content transition-colors",
                        "←"
                    }
                    div {
                        p { class: "text-xs font-bold opacity-40 uppercase tracking-widest", "Previous" }
                        p { class: "font-medium", "Module Overview" }
                    }
                }

                Link {
                    to: Route::CourseView { course_id: course_id.clone() },
                    class: "group flex items-center text-right gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all",
                    div {
                        p { class: "text-xs font-bold opacity-40 uppercase tracking-widest", "Next" }
                        p { class: "font-medium", "Return to Course" }
                    }
                    div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center group-hover:bg-primary group-hover:text-primary-content transition-colors",
                        "→"
                    }
                }
            }
        }
    }
}
