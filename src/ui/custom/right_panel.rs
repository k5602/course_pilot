//! Right panel with Notes and AI Chat tabs

use crate::application::use_cases::{DeleteNoteInput, LoadNoteInput, SaveNoteInput};
use crate::domain::ports::VideoRepository;
use crate::domain::value_objects::VideoId;
use crate::ui::custom::{MarkdownRenderer, TagBadge};
use crate::ui::state::{AppState, ChatMessage, ChatRole, RightPanelTab};
use dioxus::prelude::*;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

/// Right side panel with Notes and AI Chat tabs.
#[component]
pub fn RightPanel() -> Element {
    let mut state = use_context::<AppState>();
    let current_tab = *state.right_panel_tab.read();

    rsx! {
        aside {
            class: "w-80 h-full bg-base-200 border-l border-base-300 flex flex-col",

            // Tab headers
            div {
                class: "flex border-b border-base-300",

                TabButton {
                    label: "Notes",
                    active: current_tab == RightPanelTab::Notes,
                    onclick: move |_| state.right_panel_tab.set(RightPanelTab::Notes),
                }
                TabButton {
                    label: "AI Chat",
                    active: current_tab == RightPanelTab::AiChat,
                    onclick: move |_| state.right_panel_tab.set(RightPanelTab::AiChat),
                }
            }

            // Tab content
            div {
                class: "flex-1 overflow-auto p-5",
                match current_tab {
                    RightPanelTab::Notes => rsx! { NotesEditor {} },
                    RightPanelTab::AiChat => rsx! { AiChatView {} },
                }
            }
        }
    }
}

#[component]
fn TabButton(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let active_class = if active { "border-b-2 border-primary text-primary" } else { "" };

    rsx! {
        button {
            class: "flex-1 py-3 text-center hover:bg-base-300 transition-colors {active_class}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

/// Notes editor for current video.
#[component]
fn NotesEditor() -> Element {
    let state = use_context::<AppState>();
    let video_id = state.current_video_id.read().clone();
    let note_text = use_signal(String::new);
    let course_tags = use_signal(Vec::<crate::domain::entities::Tag>::new);
    let load_error = use_signal(|| None::<String>);
    let is_loading = use_signal(|| false);
    let save_status = use_signal(|| None::<String>);
    let save_seq = use_signal(|| 0u64);
    let is_saving = use_signal(|| false);

    {
        let state = state.clone();
        let video_id = video_id.clone();
        let mut note_text = note_text;
        let mut course_tags = course_tags;
        let mut load_error = load_error;
        let mut is_loading = is_loading;
        let mut save_status = save_status;
        let mut save_seq = save_seq;
        let mut is_saving = is_saving;
        use_effect(move || {
            load_error.set(None);
            course_tags.set(Vec::new());
            note_text.set(String::new());
            save_status.set(None);
            save_seq.set(0);
            is_saving.set(false);

            let Some(id) = video_id.clone() else {
                return;
            };
            let vid = match VideoId::from_str(&id) {
                Ok(v) => v,
                Err(_) => {
                    load_error.set(Some("Invalid video ID format".to_string()));
                    return;
                },
            };

            let Some(ctx) = state.backend.clone() else {
                load_error.set(Some("Backend not available".to_string()));
                return;
            };

            is_loading.set(true);
            let mut note_text = note_text;
            let mut course_tags = course_tags;
            let mut load_error = load_error;
            let mut is_loading = is_loading;
            spawn(async move {
                let use_case = crate::application::ServiceFactory::notes(&ctx);
                match use_case.load_note(LoadNoteInput { video_id: vid }) {
                    Ok(Some(note_view)) => {
                        note_text.set(note_view.content);
                        course_tags.set(note_view.course_tags);
                    },
                    Ok(None) => {},
                    Err(e) => {
                        load_error.set(Some(format!("Failed to load note: {}", e)));
                    },
                }
                is_loading.set(false);
            });
        });
    }

    let on_note_input = {
        let state = state.clone();
        let video_id = video_id.clone();
        let mut note_text = note_text;
        let mut load_error = load_error;
        let mut save_status = save_status;
        let mut save_seq = save_seq;
        let mut is_saving = is_saving;

        move |e: Event<FormData>| {
            let text = e.value();
            note_text.set(text.clone());
            load_error.set(None);

            let Some(id) = video_id.clone() else {
                return;
            };
            let vid = match VideoId::from_str(&id) {
                Ok(v) => v,
                Err(_) => {
                    load_error.set(Some("Invalid video ID format".to_string()));
                    return;
                },
            };

            let Some(ctx) = state.backend.clone() else {
                load_error.set(Some("Backend not available".to_string()));
                return;
            };

            let current_seq = *save_seq.read() + 1;
            save_seq.set(current_seq);
            save_status.set(None);
            is_saving.set(true);

            let mut course_tags = course_tags;
            let mut load_error = load_error;
            let mut save_status = save_status;
            let mut is_saving = is_saving;
            let save_seq_check = save_seq;
            spawn(async move {
                sleep(Duration::from_millis(500)).await;
                if *save_seq_check.read() != current_seq {
                    return;
                }

                let use_case = crate::application::ServiceFactory::notes(&ctx);

                if text.trim().is_empty() {
                    if let Err(e) = use_case.delete_note(DeleteNoteInput { video_id: vid }) {
                        load_error.set(Some(format!("Failed to delete note: {}", e)));
                    } else {
                        course_tags.set(Vec::new());
                        save_status.set(Some("Cleared".to_string()));
                    }
                    is_saving.set(false);
                    return;
                }

                match use_case.save_note(SaveNoteInput { video_id: vid, content: text }) {
                    Ok(note_view) => {
                        course_tags.set(note_view.course_tags);
                        save_status.set(Some("Saved".to_string()));
                    },
                    Err(e) => {
                        load_error.set(Some(format!("Failed to save note: {}", e)));
                    },
                }
                is_saving.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "h-full flex flex-col",

            if video_id.is_some() {
                div { class: "flex items-center justify-between gap-2 mb-2",
                    if !course_tags.read().is_empty() {
                        div { class: "flex flex-wrap gap-2",
                            for tag in course_tags.read().iter() {
                                TagBadge { tag: tag.clone() }
                            }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        if *is_loading.read() {
                            span { class: "text-xs text-base-content/60", "Loading..." }
                        }
                        if *is_saving.read() {
                            span { class: "text-xs text-base-content/60", "Saving..." }
                        } else if let Some(ref status) = *save_status.read() {
                            span { class: "text-xs text-base-content/60", "{status}" }
                        }
                    }
                }

                if let Some(ref err) = *load_error.read() {
                    div { class: "text-error text-xs mb-2", "{err}" }
                }

                textarea {
                    class: "textarea textarea-bordered resize-none text-sm leading-6",
                    placeholder: "Take notes on this video...",
                    value: "{note_text.read()}",
                    oninput: on_note_input,
                }

                div {
                    class: "mt-4 flex-1 overflow-auto rounded-lg bg-base-100 p-4",

                    if note_text.read().trim().is_empty() {
                        p { class: "text-base-content/50", "Markdown preview will appear once you add notes" }
                    } else {
                        MarkdownRenderer {
                            src: note_text.read().clone(),
                            class: Some("prose prose-base leading-7 max-w-none".to_string()),
                        }
                    }
                }
            } else {
                div {
                    class: "text-base-content/50 text-center mt-8",
                    "Select a video to start notes. Your notes save automatically."
                }
            }
        }
    }
}

/// AI Chat companion view.
#[component]
fn AiChatView() -> Element {
    let mut state = use_context::<AppState>();
    let video_id = state.current_video_id.read().clone();
    let messages = {
        let all = state.chat_history_by_video.read();
        video_id.as_ref().and_then(|id| all.get(id)).cloned().unwrap_or_default()
    };
    let has_gemini = state.has_gemini();
    let has_transcript = use_signal(|| None::<bool>);

    {
        let video_id = video_id.clone();
        let backend = state.backend.clone();
        let mut has_transcript = has_transcript;
        use_effect(move || {
            has_transcript.set(None);

            let Some(id) = video_id.clone() else {
                return;
            };
            let vid = match VideoId::from_str(&id) {
                Ok(v) => v,
                Err(_) => return,
            };
            let Some(ctx) = backend.as_ref() else {
                return;
            };

            match ctx.video_repo.find_by_id(&vid) {
                Ok(Some(video)) => {
                    let available =
                        video.transcript().map(|t| !t.trim().is_empty()).unwrap_or(false);
                    has_transcript.set(Some(available));
                },
                Ok(None) => has_transcript.set(Some(false)),
                Err(_) => has_transcript.set(Some(false)),
            }
        });
    }

    let mut input_value = use_signal(String::new);
    let mut is_loading = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);

    {
        let video_id = video_id.clone();
        let mut input_value = input_value;
        let mut is_loading = is_loading;
        let mut error_msg = error_msg;
        use_effect(move || {
            let _ = video_id.clone();
            input_value.set(String::new());
            is_loading.set(false);
            error_msg.set(None);
        });
    }

    let video_id_closure = video_id.clone();
    // Closure captures Clone-able items (Signals, Option<String>), so it is Clone.
    let on_send = move || {
        let question = input_value.read().trim().to_string();
        if question.is_empty() || *is_loading.read() {
            return;
        }

        if let Some(vid_str) = video_id_closure.clone() {
            // Parse VideoId (UUID)
            let vid = match VideoId::from_str(&vid_str) {
                Ok(id) => id,
                Err(_) => {
                    error_msg.set(Some("Invalid video ID format".to_string()));
                    return;
                },
            };

            // Add user message immediately
            {
                let mut history = state.chat_history_by_video.write();
                let entry = history.entry(vid_str.clone()).or_default();
                entry.push(ChatMessage { role: ChatRole::User, content: question.clone() });
            }
            input_value.set(String::new());
            is_loading.set(true);
            error_msg.set(None);

            let service_context = state.backend.clone();

            spawn(async move {
                if let Some(ctx) = service_context {
                    if let Some(use_case) = crate::application::ServiceFactory::ask_companion(&ctx)
                    {
                        let input = crate::application::use_cases::AskCompanionInput {
                            video_id: vid,
                            question,
                        };

                        match use_case.execute(input).await {
                            Ok(response) => {
                                let mut history = state.chat_history_by_video.write();
                                let entry = history.entry(vid_str.clone()).or_default();
                                entry.push(ChatMessage {
                                    role: ChatRole::Assistant,
                                    content: response,
                                });
                            },
                            Err(e) => {
                                error_msg.set(Some(format!("Error: {}", e)));
                            },
                        }
                    } else {
                        error_msg.set(Some("Chat service not available".to_string()));
                    }
                }
                is_loading.set(false);
            });
        }
    };

    let mut on_send_click = on_send.clone();
    let mut on_send_key = on_send.clone();

    rsx! {
        div {
            class: "h-full flex flex-col",

            // Chat messages
            div {
                class: "flex-1 overflow-auto space-y-3",
                for msg in messages.iter() {
                    ChatBubble { message: msg.clone() }
                }

                if *is_loading.read() {
                    div {
                        class: "flex justify-start",
                        div {
                            class: "max-w-[80%] px-4 py-2 rounded-lg bg-base-300",
                            span { class: "loading loading-dots loading-sm" }
                        }
                    }
                }

                if messages.is_empty() && !*is_loading.read() {
                    div {
                        class: "text-base-content/50 text-center mt-8 space-y-2",
                        if video_id.is_none() {
                            "Select a video to ask questions"
                        } else if !has_gemini {
                            "Add a Gemini API key in Settings to enable AI Chat"
                        } else {
                            "Ask questions about the current video"
                        }
                        if video_id.is_some() && has_gemini && *has_transcript.read() == Some(false) {
                            p {
                                class: "text-xs text-warning",
                                "Transcript not available yet. Generate a summary to fetch it."
                            }
                        }
                        if video_id.is_some() && has_gemini {
                            p {
                                class: "text-xs text-base-content/60",
                                "Answers improve when a transcript, summary, or notes are available."
                            }
                        }
                    }
                }
            }

            // Error message
            if let Some(ref err) = *error_msg.read() {
                div {
                    class: "text-error text-sm mb-2",
                    "{err}"
                }
            }

            // Input area
            div {
                class: "pt-4 border-t border-base-300",
                div {
                    class: "flex gap-2",
                    input {
                        class: "input input-bordered flex-1",
                        placeholder: if video_id.is_none() { "Select a video first..." } else if !has_gemini { "Configure Gemini API key..." } else { "Ask a question..." },
                        value: "{input_value}",
                        disabled: video_id.is_none() || !has_gemini || *is_loading.read(),
                        oninput: move |e| input_value.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                on_send_key();
                            }
                        },
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: video_id.is_none() || !has_gemini || *is_loading.read() || input_value.read().trim().is_empty(),
                        onclick: move |_| on_send_click(),
                        "Send"
                    }
                }
                if !has_gemini {
                    p {
                        class: "text-xs text-warning mt-2",
                        "AI Chat requires a Gemini API key. Configure in Settings."
                    }
                }
            }
        }
    }
}

#[component]
fn ChatBubble(message: ChatMessage) -> Element {
    let (align, bg) = match message.role {
        ChatRole::User => ("justify-end", "bg-primary text-primary-content"),
        ChatRole::Assistant => ("justify-start", "bg-base-300"),
    };

    rsx! {
        div {
            class: "flex {align}",
            div {
                class: "max-w-[80%] px-4 py-3 rounded-lg {bg}",
                MarkdownRenderer {
                    src: message.content,
                    class: Some("prose prose-base leading-7 max-w-none".to_string()),
                }
            }
        }
    }
}
