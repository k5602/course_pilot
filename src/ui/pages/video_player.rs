//! Video player page - Sanctuary for focused learning.

use dioxus::prelude::*;
use std::str::FromStr;

use crate::application::ServiceFactory;
use crate::application::use_cases::UpdatePreferencesInput;
use crate::domain::ports::VideoRepository;
use crate::domain::value_objects::{CourseId, ExamDifficulty, VideoId};
use crate::ui::Route;
use crate::ui::actions::{import_subtitle_for_video, start_exam};
use crate::ui::custom::{
    ErrorAlert, LocalVideoPlayer, MarkdownRenderer, Spinner, SuccessAlert, YouTubePlayer,
};
use crate::ui::hooks::{
    use_load_modules_state, use_load_video_state, use_load_videos_by_course_state,
};
use crate::ui::state::AppState;

/// Video player with controls and completion actions.
#[component]
pub fn VideoPlayer(course_id: String, video_id: String) -> Element {
    let mut state = use_context::<AppState>();
    let backend = state.backend.clone();
    let nav = use_navigator();

    {
        let mut state = state.clone();
        let backend = state.backend.clone();
        use_effect(move || {
            if let Some(ref ctx) = backend {
                let use_case = ServiceFactory::preferences(ctx);
                match use_case.load() {
                    Ok(prefs) => {
                        state.right_panel_visible.set(prefs.right_panel_visible());
                        state.onboarding_completed.set(prefs.onboarding_completed());
                    },
                    Err(e) => {
                        log::error!("Failed to load preferences: {}", e);
                        state.right_panel_visible.set(true);
                    },
                }
            } else {
                state.right_panel_visible.set(true);
            }
        });
    }

    // Parse IDs
    let course_id_vo = match CourseId::from_str(&course_id) {
        Ok(id) => id,
        Err(_) => {
            return rsx! {
                div { class: "p-6 text-error", "Invalid Course ID" }
            };
        },
    };

    let video_id_vo = match VideoId::from_str(&video_id) {
        Ok(id) => id,
        Err(_) => {
            return rsx! {
                div { class: "p-6 text-error", "Invalid Video ID" }
            };
        },
    };

    // Load data
    let (video, video_state) = use_load_video_state(backend.clone(), &video_id_vo);
    let (modules, modules_state) = use_load_modules_state(backend.clone(), &course_id_vo);
    let (all_videos, videos_state) =
        use_load_videos_by_course_state(backend.clone(), &course_id_vo);

    // Track current video in global state for AI companion context
    let video_id_for_state = video_id.clone();
    let course_id_for_state = course_id.clone();
    use_effect(move || {
        state.current_video_id.set(Some(video_id_for_state.clone()));
        state
            .last_video_by_course
            .write()
            .insert(course_id_for_state.clone(), video_id_for_state.clone());
    });

    // Extract video data reactively
    let video_read = video.read();
    let v = match video_read.as_ref() {
        Some(v) => v.clone(),
        None => {
            if let Some(ref err) = *video_state.error.read() {
                return rsx! {
                    div { class: "p-6",
                        ErrorAlert { message: err.clone(), on_dismiss: None }
                    }
                };
            }
            return rsx! {
                div { class: "p-6",
                    Spinner { message: Some("Loading video...".to_string()) }
                }
            };
        },
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
    let is_completed_now = video.read().as_ref().map(|v| v.is_completed()).unwrap_or(false);
    let is_local_video = use_signal(|| false);
    let has_transcript = use_signal(|| false);
    {
        let mut is_local_video = is_local_video;
        let mut has_transcript = has_transcript;
        use_effect(move || {
            let value = video.read();
            let (local, transcript) = value
                .as_ref()
                .map(|video| {
                    (
                        video.local_path().is_some(),
                        video.transcript().map(|t| !t.trim().is_empty()).unwrap_or(false),
                    )
                })
                .unwrap_or((false, false));
            is_local_video.set(local);
            has_transcript.set(transcript);
        });
    }
    let mut quiz_num_questions = use_signal(|| 5u8);
    let mut quiz_difficulty = use_signal(|| ExamDifficulty::Medium);
    let quiz_disabled = !state.has_gemini() || (*is_local_video.read() && !*has_transcript.read());
    let action_status = use_signal(|| None::<(bool, String)>);

    // Handlers
    let mut action_status_complete = action_status;
    let mut video_for_complete = video;
    let on_mark_complete = move |_| {
        if let Some(ctx) = backend_for_complete.as_ref() {
            let current_completed =
                video_for_complete.read().as_ref().map(|v| v.is_completed()).unwrap_or(false);
            let new_status = !current_completed;
            if let Err(e) = ctx.video_repo.update_completion(&video_id_for_complete, new_status) {
                log::error!("Failed to update completion: {}", e);
                action_status_complete
                    .set(Some((false, format!("Failed to update completion: {}", e))));
                return;
            }

            if let Ok(Some(updated)) = ctx.video_repo.find_by_id(&video_id_for_complete) {
                video_for_complete.set(Some(updated));
            }

            let message = if new_status { "Marked as completed." } else { "Marked as incomplete." };
            action_status_complete.set(Some((true, message.to_string())));
        } else {
            action_status_complete.set(Some((false, "Backend not available".to_string())));
        }
    };

    let mut action_status_quiz = action_status;
    let on_take_quiz = move |_| {
        let backend_inner = backend_for_quiz.clone();
        let vid = video_id_for_quiz.clone();
        let num_questions = *quiz_num_questions.read();
        let difficulty = *quiz_difficulty.read();
        spawn(async move {
            match start_exam(backend_inner, vid, num_questions, difficulty).await {
                Ok(exam_id) => {
                    nav.push(Route::QuizView { exam_id: exam_id.as_uuid().to_string() });
                },
                Err(e) => {
                    log::error!("Failed to start exam: {}", e);
                    action_status_quiz.set(Some((false, format!("Failed to start exam: {}", e))));
                },
            }
        });
    };

    let on_toggle_panel = {
        let mut state = state.clone();
        let backend_for_toggle = state.backend.clone();
        move |_| {
            let new_value = !*state.right_panel_visible.read();
            state.right_panel_visible.set(new_value);

            let Some(ref ctx) = backend_for_toggle else {
                return;
            };

            let use_case = ServiceFactory::preferences(ctx);
            match use_case.load() {
                Ok(prefs) => {
                    let input = UpdatePreferencesInput {
                        ml_boundary_enabled: prefs.ml_boundary_enabled(),
                        cognitive_limit_minutes: prefs.cognitive_limit_minutes(),
                        right_panel_visible: new_value,
                        onboarding_completed: *state.onboarding_completed.read(),
                    };
                    if let Err(e) = use_case.update(input) {
                        log::error!("Failed to persist right panel preference: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("Failed to load preferences for right panel: {}", e);
                },
            }
        }
    };

    let on_transcript_update = {
        let backend = backend.clone();
        let video_id_for_refresh = v.id().clone();
        let mut video_for_refresh = video;
        move |_| {
            if let Some(ctx) = backend.as_ref() {
                if let Ok(Some(updated)) = ctx.video_repo.find_by_id(&video_id_for_refresh) {
                    video_for_refresh.set(Some(updated));
                }
            }
        }
    };

    rsx! {
        div { class: "p-6 min-h-full flex flex-col max-w-5xl mx-auto",

            if let Some((is_success, ref msg)) = *action_status.read() {
                if is_success {
                    SuccessAlert { message: msg.clone(), on_dismiss: None }
                } else {
                    ErrorAlert { message: msg.clone(), on_dismiss: None }
                }
            }

            if let Some(ref err) = *video_state.error.read() {
                ErrorAlert { message: err.clone(), on_dismiss: None }
            }
            if let Some(ref err) = *modules_state.error.read() {
                ErrorAlert { message: err.clone(), on_dismiss: None }
            }
            if let Some(ref err) = *videos_state.error.read() {
                ErrorAlert { message: err.clone(), on_dismiss: None }
            }

            // Header/Nav
            div { class: "flex flex-wrap justify-between items-center mb-6 gap-2",
                Link {
                    to: Route::CourseView {
                        course_id: course_id.clone(),
                    },
                    class: "btn btn-ghost btn-sm gap-2",
                    "← Back to Course"
                }
                div { class: "flex items-center gap-3",
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: on_toggle_panel,
                        title: "Toggle notes & AI panel",
                        if *state.right_panel_visible.read() {
                            "Hide Panel"
                        } else {
                            "Show Panel"
                        }
                    }
                    div { class: "flex items-center gap-2 text-sm font-medium opacity-60",
                        span { "{module_title}" }
                        span { "•" }
                        span { "{v.duration_secs() / 60} min" }
                    }
                }
            }

            // Video player section
            div { class: "aspect-video w-full rounded-3xl overflow-hidden shadow-2xl bg-black border-4 border-base-300",
                if let Some(path) = v.local_path() {
                    LocalVideoPlayer { path: path.to_string() }
                } else if let Some(youtube_id) = v.youtube_id() {
                    YouTubePlayer { video_id: youtube_id.as_str().to_string() }
                } else {
                    div { class: "flex items-center justify-center w-full h-full text-base-content/60",
                        "Video source unavailable."
                    }
                }
            }

            // Info & Actions
            div { class: "mt-8 flex flex-col md:flex-row md:items-start justify-between gap-6",
                div { class: "flex-1",
                    h1 { class: "text-3xl font-bold mb-2", "{v.title()}" }
                    p { class: "text-base-content/60",
                        if is_completed_now {
                            span { class: "text-success font-medium flex items-center gap-1",
                                "✓ Completed"
                            }
                        } else {
                            span { "Not yet completed" }
                        }
                    }
                }

                div { class: "flex flex-wrap items-center gap-3",
                    button {
                        class: if is_completed_now { "btn btn-success" } else { "btn btn-outline btn-success" },
                        onclick: on_mark_complete,
                        if is_completed_now {
                            "✓ Completed"
                        } else {
                            "Mark Complete"
                        }
                    }

                    div { class: "flex items-center gap-2",
                        span { class: "text-xs uppercase tracking-wide opacity-60", "Questions" }
                        input {
                            class: "input input-bordered input-sm w-20",
                            r#type: "number",
                            min: "1",
                            max: "20",
                            value: "{quiz_num_questions}",
                            disabled: quiz_disabled,
                            oninput: move |e| {
                                let parsed = e.value().trim().parse::<u8>().ok().unwrap_or(5);
                                quiz_num_questions.set(parsed.clamp(1, 20));
                            },
                        }
                    }

                    div { class: "flex items-center gap-2",
                        span { class: "text-xs uppercase tracking-wide opacity-60", "Difficulty" }
                        select {
                            class: "select select-bordered select-sm",
                            value: "{quiz_difficulty.read().as_str()}",
                            disabled: quiz_disabled,
                            oninput: move |e| {
                                let next = ExamDifficulty::from_str(&e.value())
                                    .unwrap_or(ExamDifficulty::Medium);
                                quiz_difficulty.set(next);
                            },
                            option { value: "easy", "Easy" }
                            option { value: "medium", "Medium" }
                            option { value: "hard", "Hard" }
                        }
                    }

                    button {
                        class: "btn btn-primary gap-2",
                        onclick: on_take_quiz,
                        disabled: quiz_disabled,
                        title: if !state.has_gemini() {
                            "Configure Gemini API key in Settings"
                        } else if *is_local_video.read() && !*has_transcript.read() {
                            "Local videos need subtitles to enable quizzes"
                        } else {
                            ""
                        },
                        "📝 Take Quiz"
                    }
                }

                if *is_local_video.read() && !*has_transcript.read() {
                    p { class: "text-xs text-warning mt-2",
                        "Local videos need subtitles (SRT/VTT) to enable summaries and quizzes."
                    }
                }
            }

            // AI Summary Section
            SummarySection {
                video_id: v.id().as_uuid().to_string(),
                is_local: is_local_video,
                has_transcript,
                on_transcript_update: on_transcript_update,
            }

            // Navigation Footer
            div { class: "mt-auto pt-12 flex justify-between border-t border-base-300",
                // Previous video
                if let Some(pv) = prev_video {
                    Link {
                        to: Route::VideoPlayer {
                            course_id: course_id.clone(),
                            video_id: pv.id().as_uuid().to_string(),
                        },
                        class: "group flex items-center gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all",
                        div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center group-hover:bg-primary group-hover:text-primary-content transition-colors",
                            "←"
                        }
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest",
                                "Previous"
                            }
                            p { class: "font-medium truncate max-w-[200px]", "{pv.title()}" }
                        }
                    }
                } else {
                    Link {
                        to: Route::CourseView {
                            course_id: course_id.clone(),
                        },
                        class: "group flex items-center gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all opacity-50",
                        div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center",
                            "←"
                        }
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest",
                                "Previous"
                            }
                            p { class: "font-medium", "Back to Course" }
                        }
                    }
                }

                // Next video
                if let Some(nv) = next_video {
                    Link {
                        to: Route::VideoPlayer {
                            course_id: course_id.clone(),
                            video_id: nv.id().as_uuid().to_string(),
                        },
                        class: "group flex items-center text-right gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all",
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest",
                                "Next"
                            }
                            p { class: "font-medium truncate max-w-[200px]", "{nv.title()}" }
                        }
                        div { class: "w-10 h-10 rounded-full bg-base-300 flex items-center justify-center group-hover:bg-primary group-hover:text-primary-content transition-colors",
                            "→"
                        }
                    }
                } else {
                    Link {
                        to: Route::CourseView {
                            course_id: course_id.clone(),
                        },
                        class: "group flex items-center text-right gap-4 p-4 rounded-2xl hover:bg-base-200 transition-all opacity-50",
                        div {
                            p { class: "text-xs font-bold opacity-40 uppercase tracking-widest",
                                "Complete"
                            }
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
    Ready { summary: String, cached: bool },
    Error(String),
}

/// AI Summary section with cached transcript + summary persistence
#[component]
fn SummarySection(
    video_id: String,
    is_local: Signal<bool>,
    has_transcript: Signal<bool>,
    on_transcript_update: EventHandler<()>,
) -> Element {
    let state = use_context::<AppState>();
    let mut summary_state = use_signal(|| SummaryState::Empty);
    let mut expanded = use_signal(|| false);
    let summary_disabled = !state.has_gemini() || (*is_local.read() && !*has_transcript.read());

    let backend = state.backend.clone();
    let video_id_clone = video_id.clone();
    let attach_status = use_signal(|| None::<String>);

    let on_attach_subtitle = {
        let backend = state.backend.clone();
        let video_id = video_id_clone.clone();
        let on_transcript_update = on_transcript_update;
        move |_| {
            let Some(path) =
                rfd::FileDialog::new().add_filter("Subtitles", &["srt", "vtt", "txt"]).pick_file()
            else {
                return;
            };

            let backend = backend.clone();
            let video_id = video_id.clone();
            let mut attach_status = attach_status;
            let on_transcript_update = on_transcript_update;
            spawn(async move {
                let video_id_vo = match VideoId::from_str(&video_id) {
                    Ok(id) => id,
                    Err(_) => {
                        attach_status.set(Some("Invalid video ID".to_string()));
                        return;
                    },
                };

                attach_status.set(Some("Importing subtitles...".to_string()));
                match import_subtitle_for_video(backend, video_id_vo, path.display().to_string())
                    .await
                {
                    Ok(len) => {
                        attach_status
                            .set(Some(format!("Subtitles attached ({} chars cleaned).", len)));
                        on_transcript_update.call(());
                    },
                    Err(e) => {
                        attach_status.set(Some(format!("Subtitle import failed: {e}")));
                    },
                }
            });
        }
    };

    {
        let backend = backend.clone();
        let video_id = video_id.clone();
        let mut summary_state = summary_state;
        use_effect(move || {
            let Some(ref ctx) = backend else {
                return;
            };
            let video_id_vo = match VideoId::from_str(&video_id) {
                Ok(id) => id,
                Err(_) => return,
            };

            if let Some(use_case) = crate::application::ServiceFactory::summarize_video(ctx) {
                spawn(async move {
                    let input = crate::application::use_cases::SummarizeVideoInput {
                        video_id: video_id_vo,
                        force_refresh: false,
                    };
                    if let Ok(result) = use_case.execute(input).await {
                        if result.cached {
                            summary_state
                                .set(SummaryState::Ready { summary: result.summary, cached: true });
                        }
                    }
                });
            }
        });
    }

    let generate_summary = move |force_refresh: bool| {
        let backend = backend.clone();
        let video_id = video_id_clone.clone();

        spawn(async move {
            summary_state.set(SummaryState::Loading("Generating summary...".to_string()));

            let video_id_vo = match VideoId::from_str(&video_id) {
                Ok(id) => id,
                Err(_) => {
                    summary_state.set(SummaryState::Error("Invalid Video ID".to_string()));
                    return;
                },
            };

            if let Some(ref ctx) = backend {
                if let Some(use_case) = crate::application::ServiceFactory::summarize_video(ctx) {
                    let input = crate::application::use_cases::SummarizeVideoInput {
                        video_id: video_id_vo,
                        force_refresh,
                    };

                    match use_case.execute(input).await {
                        Ok(result) => {
                            summary_state.set(SummaryState::Ready {
                                summary: result.summary,
                                cached: result.cached,
                            });
                            on_transcript_update.call(());
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
        div { class: "mt-8 bg-base-200 rounded-2xl overflow-hidden",

            // Header (clickable to expand)
            button {
                class: "w-full p-4 flex items-center justify-between hover:bg-base-300 transition-colors",
                onclick: move |_| {
                    let current = *expanded.read();
                    expanded.set(!current);
                },

                div { class: "flex items-center gap-3",
                    span { class: "text-xl", "✨" }
                    span { class: "font-bold", "AI Summary" }
                    match &*summary_state.read() {
                        SummaryState::Ready { cached, .. } => rsx! {
                            span { class: "badge badge-success badge-sm",
                                if *cached {
                                    "Cached"
                                } else {
                                    "Ready"
                                }
                            }
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
                div { class: "p-4 pt-0",

                    match &*summary_state.read() {
                        SummaryState::Empty => rsx! {
                            div { class: "text-center py-8",
                                if *is_local.read() && !*has_transcript.read() {
                                    p { class: "text-base-content/60 mb-2",
                                        "Local videos need a subtitle file (SRT/VTT) before summaries can be generated."
                                    }
                                    p { class: "text-xs text-warning mb-4",
                                        "Attach subtitles to store a cleaned transcript for this video."
                                    }
                                    button {
                                        class: "btn btn-outline btn-primary btn-sm",
                                        onclick: on_attach_subtitle,
                                        "Attach Subtitles"
                                    }
                                    if let Some(ref status) = *attach_status.read() {
                                        p { class: "text-xs text-base-content/60 mt-2", "{status}" }
                                    }
                                } else {
                                    p { class: "text-base-content/60 mb-4",
                                        "Generate an AI summary from the video transcript"
                                    }
                                }
                                button {
                                    class: "btn btn-primary",
                                    onclick: move |_| generate_summary(false),
                                    disabled: summary_disabled,
                                    "✨ Generate Summary"
                                }
                                if !state.has_gemini() {
                                    p { class: "text-sm text-warning mt-2", "Configure Gemini API key in Settings" }
                                }
                            }
                        },
                        SummaryState::Loading(msg) => rsx! {
                            div { class: "flex flex-col items-center py-8",
                                div { class: "loading loading-spinner loading-lg text-primary" }
                                p { class: "text-base-content/60 mt-4", "{msg}" }
                            }
                        },
                        SummaryState::Ready { summary, cached } => rsx! {
                            div { class: "space-y-4",
                                if *cached {
                                    p { class: "text-xs text-base-content/60", "Loaded from cache" }
                                }
                                div { class: "prose prose-sm max-w-none",
                                    MarkdownRenderer { src: summary.clone() }
                                }
                                div { class: "flex justify-end",
                                    button {
                                        class: "btn btn-outline btn-primary btn-sm",
                                        onclick: move |_| generate_summary(true),
                                        "Regenerate"
                                    }
                                }
                            }
                        },
                        SummaryState::Error(err) => rsx! {
                            div { class: "text-center py-8",
                                p { class: "text-error mb-4", "{err}" }
                                button {
                                    class: "btn btn-outline btn-primary",
                                    onclick: move |_| generate_summary(false),
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
