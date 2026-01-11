//! Right panel with Notes and AI Chat tabs

use dioxus::prelude::*;

use crate::ui::state::{AppState, ChatMessage, ChatRole, RightPanelTab};

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
                class: "flex-1 overflow-auto p-4",
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

    rsx! {
        div {
            class: "h-full flex flex-col",

            if video_id.is_some() {
                textarea {
                    class: "textarea textarea-bordered flex-1 resize-none",
                    placeholder: "Take notes on this video...",
                }
            } else {
                div {
                    class: "text-base-content/50 text-center mt-8",
                    "Select a video to take notes"
                }
            }
        }
    }
}

/// AI Chat companion view.
#[component]
fn AiChatView() -> Element {
    let state = use_context::<AppState>();
    let messages = state.chat_history.read();
    let video_id = state.current_video_id.read().clone();
    let has_gemini = state.has_gemini();

    let mut input_value = use_signal(String::new);
    let is_loading = use_signal(|| false);
    let error_msg = use_signal(|| None::<String>);

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
                        class: "text-base-content/50 text-center mt-8",
                        if video_id.is_none() {
                            "Select a video to ask questions"
                        } else if !has_gemini {
                            "Add a Gemini API key in Settings to enable AI Chat"
                        } else {
                            "Ask questions about the current video"
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
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: video_id.is_none() || !has_gemini || *is_loading.read() || input_value.read().trim().is_empty(),
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
                class: "max-w-[80%] px-4 py-2 rounded-lg {bg}",
                "{message.content}"
            }
        }
    }
}
