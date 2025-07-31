use dioxus::prelude::*;
use crate::gemini::{GeminiClient, ConversationHistory, ChatMessage, CourseContext, CourseSourceType};
use crate::types::{Route, VideoContext};
use crate::ui::use_app_state;
use crate::ui::components::ChatMarkdownRenderer;
use crate::storage::database::Database;

#[component]
pub fn GeminiChatbot() -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut input_text = use_signal(String::new);
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    let mut conversation_history = use_signal(ConversationHistory::new);
    let mut gemini_client = use_signal(GeminiClient::new);
    let mut is_initialized = use_signal(|| false);

    let app_state = use_app_state();
    let current_route = use_route::<Route>();
    let video_context = app_state.read().contextual_panel.video_context.clone();

    // Initialize the Gemini client and set up course context
    use_effect(move || {
        if !is_initialized() {
            let route = current_route.clone();
            let video_ctx = video_context.clone();
            spawn(async move {
                let mut client = gemini_client.write();
                match client.initialize().await {
                    Ok(()) => {
                        is_initialized.set(true);
                        
                        // Set up course context based on current route
                        if let Route::PlanView { course_id } = route {
                            if let Ok(course_uuid) = uuid::Uuid::parse_str(&course_id) {
                                spawn(async move {
                                    match setup_course_context(course_uuid, video_ctx).await {
                                        Ok(context) => {
                                            conversation_history.write().set_course_context(context);
                                        }
                                        Err(e) => {
                                            log::error!("Failed to setup course context: {}", e);
                                        }
                                    }
                                });
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to initialize Gemini client: {}", e)));
                    }
                }
            });
        }
    });

    let mut send_message = move |message: String| {
        if message.trim().is_empty() || is_loading() {
            return;
        }

        let message = message.trim().to_string();
        is_loading.set(true);
        error_message.set(None);

        // Add user message to display
        messages.write().push(ChatMessage {
            role: "user".to_string(),
            content: message.clone(),
            timestamp: chrono::Utc::now(),
        });

        // Add to conversation history
        conversation_history.write().add_message("user".to_string(), message.clone());

        let client = gemini_client.read().clone();
        let history = conversation_history.read().clone();

        spawn(async move {
            match client.send_message(&message, &history).await {
                Ok(response) => {
                    // Add assistant response to display
                    messages.write().push(ChatMessage {
                        role: "assistant".to_string(),
                        content: response.message.clone(),
                        timestamp: chrono::Utc::now(),
                    });

                    // Add to conversation history
                    conversation_history.write().add_message("assistant".to_string(), response.message);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to get response: {}", e)));
                }
            }
            is_loading.set(false);
        });

        input_text.set(String::new());
    };

    let mut handle_suggestion_click = move |suggestion: String| {
        send_message(suggestion);
    };

    let handle_send = move |_| {
        send_message(input_text());
    };

    let handle_key_down = move |e: KeyboardEvent| {
        if e.key() == dioxus::events::Key::Enter && !e.modifiers().shift() {
            e.prevent_default();
            send_message(input_text());
        }
    };

    // Get course context for display
    let course_context = conversation_history.read().course_context.clone();

    rsx! {
        div { class: "flex flex-col h-full bg-base-100",
            // Chat header with course context
            div { class: "navbar bg-base-200 border-b border-base-300 px-4 py-2 shrink-0",
                div { class: "flex-1",
                    h2 { class: "text-lg font-semibold text-base-content",
                        "Course Assistant"
                    }
                    if let Some(context) = &course_context {
                        p { class: "text-sm text-base-content/60 truncate",
                            "{context.course_name}"
                        }
                    }
                }
                
                div { class: "flex-none",
                    div { class: "badge badge-primary badge-sm",
                        "🤖 Gemini"
                    }
                }
            }
            
            // Messages area
            div { class: "flex-1 overflow-y-auto p-4 space-y-4",
                if messages.read().is_empty() && !is_loading() {
                    div { class: "text-center py-8",
                        div { class: "text-6xl mb-4", "🤖" }
                        h3 { class: "text-xl font-semibold text-base-content mb-2",
                            "Hi! I'm your course assistant"
                        }
                        p { class: "text-base-content/70 max-w-md mx-auto mb-4",
                            "I can help you understand the course content, answer questions, and provide learning guidance."
                        }
                        
                        if !is_initialized() {
                            div { class: "loading loading-spinner loading-md" }
                            p { class: "text-sm text-base-content/60 mt-2",
                                "Initializing..."
                            }
                        } else if let Some(_context) = &course_context {
                            div { class: "flex flex-wrap gap-2 justify-center mt-4",
                                button {
                                    class: "btn btn-sm btn-outline btn-primary",
                                    onclick: move |_| handle_suggestion_click("Tell me about this course".to_string()),
                                    "Tell me about this course"
                                }
                                button {
                                    class: "btn btn-sm btn-outline btn-primary",
                                    onclick: move |_| handle_suggestion_click("What should I study first?".to_string()),
                                    "What should I study first?"
                                }
                                button {
                                    class: "btn btn-sm btn-outline btn-primary",
                                    onclick: move |_| handle_suggestion_click("Explain the course structure".to_string()),
                                    "Explain the course structure"
                                }
                            }
                        }
                    }
                } else {
                    for message in messages.read().iter() {
                        ChatMessageBubble { message: message.clone() }
                    }
                },
                
                if is_loading() {
                    div { class: "flex justify-start",
                        div { class: "chat chat-start",
                            div { class: "chat-bubble chat-bubble-secondary",
                                span { class: "loading loading-dots loading-sm" }
                            }
                        }
                    }
                },
            }
            
            // Error display
            if let Some(error) = error_message() {
                div { class: "alert alert-error mx-4 mb-2",
                    span { "⚠️ {error}" }
                    button { 
                        class: "btn btn-sm btn-ghost",
                        onclick: move |_| error_message.set(None),
                        "✕"
                    }
                }
            }
            
            // Input area
            div { class: "border-t border-base-300 p-4 bg-base-100 shrink-0",
                div { class: "flex gap-2",
                    textarea {
                        class: "textarea textarea-bordered flex-1 resize-none",
                        placeholder: if is_initialized() { 
                            "Ask me anything about the course..." 
                        } else { 
                            "Initializing chatbot..." 
                        },
                        rows: "2",
                        value: "{input_text}",
                        disabled: !is_initialized() || is_loading(),
                        oninput: move |e| input_text.set(e.value().clone()),
                        onkeydown: handle_key_down,
                    }
                    
                    button {
                        class: "btn btn-primary btn-square",
                        disabled: input_text().trim().is_empty() || !is_initialized() || is_loading(),
                        onclick: handle_send,
                        if is_loading() {
                            span { class: "loading loading-spinner loading-sm" }
                        } else {
                            "📤"
                        }
                    }
                }
                
                div { class: "flex justify-between items-center mt-2 text-xs text-base-content/60",
                    span { "Press Enter to send, Shift+Enter for new line" }
                    span { "{input_text().len()}/1000" }
                }
            }
        }
    }
}

#[component]
fn ChatMessageBubble(message: ChatMessage) -> Element {
    let is_user = message.role == "user";
    let chat_class = if is_user { "chat chat-end" } else { "chat chat-start" };
    let bubble_class = if is_user { 
        "chat-bubble chat-bubble-primary" 
    } else { 
        "chat-bubble chat-bubble-secondary" 
    };
    let user_icon = if is_user { "👤" } else { "🤖" };
    let user_label = if is_user { "You" } else { "Assistant" };
    let formatted_time = message.timestamp.format("%H:%M").to_string();

    rsx! {
        div { 
            class: "{chat_class}",
            div { 
                class: "chat-image avatar",
                div { 
                    class: "w-8 h-8 rounded-full bg-base-300 flex items-center justify-center",
                    "{user_icon}"
                }
            }
            div { 
                class: "chat-header text-xs opacity-60 mb-1",
                "{user_label}"
                time { 
                    class: "ml-1", 
                    "{formatted_time}" 
                }
            }
            div { 
                class: "{bubble_class}",
                ChatMarkdownRenderer { content: message.content.clone() }
            }
        }
    }
}

async fn setup_course_context(course_id: uuid::Uuid, video_context: Option<VideoContext>) -> anyhow::Result<CourseContext> {
    let db_path = std::path::PathBuf::from("course_pilot.db");
    let db = Database::new(&db_path)?;
    
    // Get course information
    let course = crate::storage::get_course_by_id(&db, &course_id)?
        .ok_or_else(|| anyhow::anyhow!("Course not found"))?;
    
    // For now, we'll create a generic source type since we don't store source URLs
    // In a future enhancement, we could add source tracking to the database
    let source_type = CourseSourceType::Local { 
        folder_path: format!("Course: {}", course.name) 
    };
    
    Ok(CourseContext {
        course_id,
        course_name: course.name,
        course_structure: course.structure,
        youtube_playlist_url: None, // Not stored in current schema
        current_video_context: video_context,
        source_type,
    })
}