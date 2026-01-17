//! Video player page - Sanctuary for focused learning.

use dioxus::prelude::*;
use std::str::FromStr;

use crate::domain::ports::VideoRepository;
use crate::domain::value_objects::{CourseId, VideoId};
use crate::ui::Route;
use crate::ui::actions::start_exam;
use crate::ui::custom::YouTubePlayer;
use crate::ui::hooks::{use_load_modules, use_load_video, use_load_videos_by_course};
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
    let all_videos = use_load_videos_by_course(backend.clone(), &course_id_vo);

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

    // Compute prev/next videos
    let videos_list = all_videos.read();
    let current_idx = videos_list.iter().position(|vid| vid.id() == v.id());

    let prev_video =
        current_idx.and_then(|idx| if idx > 0 { videos_list.get(idx - 1).cloned() } else { None });

    let next_video = current_idx.and_then(|idx| videos_list.get(idx + 1).cloned());

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

            // AI Summary Section
            SummarySection {
                youtube_id: v.youtube_id().as_str().to_string(),
                video_title: v.title().to_string(),
            }

            // Navigation Footer
            div { class: "mt-auto pt-12 flex justify-between border-t border-base-300",
                // Previous video
                if let Some(pv) = prev_video {
                    Link {
                        to: Route::VideoPlayer {
                            course_id: course_id.clone(),
                            video_id: pv.id().as_uuid().to_string()
                        },
                        class: "group flex items-center gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all",
                        div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center group-hover:bg-primary group-hover:text-primary-content transition-colors",
                            "←"
                        }
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest", "Previous" }
                            p { class: "font-medium truncate max-w-[200px]", "{pv.title()}" }
                        }
                    }
                } else {
                    Link {
                        to: Route::CourseView { course_id: course_id.clone() },
                        class: "group flex items-center gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all opacity-50",
                        div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center",
                            "←"
                        }
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest", "Previous" }
                            p { class: "font-medium", "Back to Course" }
                        }
                    }
                }

                // Next video
                if let Some(nv) = next_video {
                    Link {
                        to: Route::VideoPlayer {
                            course_id: course_id.clone(),
                            video_id: nv.id().as_uuid().to_string()
                        },
                        class: "group flex items-center text-right gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all",
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest", "Next" }
                            p { class: "font-medium truncate max-w-[200px]", "{nv.title()}" }
                        }
                        div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center group-hover:bg-primary group-hover:text-primary-content transition-colors",
                            "→"
                        }
                    }
                } else {
                    Link {
                        to: Route::CourseView { course_id: course_id.clone() },
                        class: "group flex items-center text-right gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all opacity-50",
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest", "Complete" }
                            p { class: "font-medium", "Back to Course" }
                        }
                        div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center",
                            "✓"
                        }
                    }
                }
            }
        }
    }
}

/// Summary generation state
#[derive(Clone, PartialEq)]
enum SummaryState {
    Empty,
    Loading(String),
    Ready(String),
    Error(String),
}

/// AI Summary section with transcript fetching and summarization
#[component]
fn SummarySection(youtube_id: String, video_title: String) -> Element {
    let state = use_context::<AppState>();
    let mut summary_state = use_signal(|| SummaryState::Empty);
    let mut expanded = use_signal(|| false);

    let backend = state.backend.clone();
    let youtube_id_clone = youtube_id.clone();
    let video_title_clone = video_title.clone();

    let generate_summary = move |_| {
        let backend = backend.clone();
        let youtube_id = youtube_id_clone.clone();
        let video_title = video_title_clone.clone();

        spawn(async move {
            summary_state.set(SummaryState::Loading("Fetching transcript...".to_string()));

            // Step 1: Fetch transcript
            let transcript_adapter =
                match crate::infrastructure::transcript::TranscriptAdapter::new() {
                    Ok(adapter) => adapter,
                    Err(e) => {
                        summary_state.set(SummaryState::Error(format!("Failed to init: {}", e)));
                        return;
                    },
                };

            let transcript = match transcript_adapter.fetch_transcript(&youtube_id).await {
                Ok(t) => t,
                Err(e) => {
                    summary_state.set(SummaryState::Error(format!("No transcript: {}", e)));
                    return;
                },
            };

            summary_state.set(SummaryState::Loading("Generating summary...".to_string()));

            // Step 2: Generate summary with LLM
            if let Some(ref ctx) = backend {
                if let Some(ref llm) = ctx.llm {
                    use crate::domain::ports::SummarizerAI;
                    match llm.summarize_transcript(&transcript, &video_title).await {
                        Ok(summary) => {
                            summary_state.set(SummaryState::Ready(summary));
                        },
                        Err(e) => {
                            summary_state
                                .set(SummaryState::Error(format!("Summary failed: {}", e)));
                        },
                    }
                } else {
                    summary_state.set(SummaryState::Error("Gemini API not configured".to_string()));
                }
            } else {
                summary_state.set(SummaryState::Error("Backend not available".to_string()));
            }
        });
    };

    rsx! {
        div {
            class: "mt-8 bg-base-200 rounded-2xl overflow-hidden",

            // Header (clickable to expand)
            button {
                class: "w-full p-4 flex items-center justify-between hover:bg-base-300 transition-colors",
                onclick: move |_| {
                    let current = *expanded.read();
                    expanded.set(!current);
                },

                div {
                    class: "flex items-center gap-3",
                    span { class: "text-xl", "✨" }
                    span { class: "font-bold", "AI Summary" }
                    match &*summary_state.read() {
                        SummaryState::Ready(_) => rsx! {
                            span { class: "badge badge-success badge-sm", "Ready" }
                        },
                        SummaryState::Loading(_) => rsx! {
                            span { class: "badge badge-warning badge-sm", "Loading" }
                        },
                        SummaryState::Error(_) => rsx! {
                            span { class: "badge badge-error badge-sm", "Error" }
                        },
                        SummaryState::Empty => rsx! {},
                    }
                }

                span {
                    class: "transition-transform",
                    style: if *expanded.read() { "transform: rotate(180deg)" } else { "" },
                    "▼"
                }
            }

            // Content (expanded)
            if *expanded.read() {
                div {
                    class: "p-4 pt-0",

                    match &*summary_state.read() {
                        SummaryState::Empty => rsx! {
                            div {
                                class: "text-center py-8",
                                p { class: "text-base-content/60 mb-4", "Generate an AI summary from the video transcript" }
                                button {
                                    class: "btn btn-primary",
                                    onclick: generate_summary,
                                    disabled: !state.has_gemini(),
                                    "✨ Generate Summary"
                                }
                                if !state.has_gemini() {
                                    p { class: "text-sm text-warning mt-2", "Configure Gemini API key in Settings" }
                                }
                            }
                        },
                        SummaryState::Loading(msg) => rsx! {
                            div {
                                class: "flex flex-col items-center py-8",
                                div { class: "loading loading-spinner loading-lg text-primary" }
                                p { class: "text-base-content/60 mt-4", "{msg}" }
                            }
                        },
                        SummaryState::Ready(summary) => rsx! {
                            div {
                                class: "prose prose-sm max-w-none",
                                // Render summary as formatted text
                                for line in summary.lines() {
                                    if line.starts_with("# ") || line.starts_with("## ") {
                                        h3 { class: "font-bold text-lg mt-4 mb-2", "{line.trim_start_matches('#').trim()}" }
                                    } else if line.starts_with("- ") || line.starts_with("* ") {
                                        li { class: "ml-4", "{line.trim_start_matches('-').trim_start_matches('*').trim()}" }
                                    } else if !line.trim().is_empty() {
                                        p { class: "mb-2", "{line}" }
                                    }
                                }
                            }
                        },
                        SummaryState::Error(err) => rsx! {
                            div {
                                class: "text-center py-8",
                                p { class: "text-error mb-4", "{err}" }
                                button {
                                    class: "btn btn-outline btn-primary",
                                    onclick: generate_summary,
                                    "Try Again"
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
