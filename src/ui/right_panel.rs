use std::rc::Rc;

use adw::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::AskCompanionInput;
use crate::domain::value_objects::VideoId;
use crate::ui::state::{ChatMessage, ChatRole, MAX_CHAT_HISTORY_PER_VIDEO, SharedState};

pub struct RightPanel {
    widget: gtk::Box,
    chat_input: gtk::Entry,
    chat_spinner: gtk::Spinner,
    chat_history_box: gtk::Box,
    chat_scroll: gtk::ScrolledWindow,
    state: SharedState,
    placeholder: adw::StatusPage,
    content_area: gtk::Box,
    context_text: gtk::TextView,
}

fn rebuild_chat_history(
    chat_box: &gtk::Box,
    state: &SharedState,
    video_id: &str,
    scroll: &gtk::ScrolledWindow,
) {
    while let Some(child) = chat_box.first_child() {
        chat_box.remove(&child);
    }

    let s = state.borrow();
    let history = s.chat_history_by_video.get(video_id).cloned().unwrap_or_default();
    for msg in &history {
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        row_box.set_hexpand(true);

        let bubble = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let label = gtk::Label::new(Some(&msg.content));
        label.set_wrap(true);
        label.set_xalign(0.0);
        label.set_selectable(true);
        bubble.append(&label);

        match msg.role {
            ChatRole::User => {
                bubble.add_css_class("chat-bubble-user");
                let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                spacer.set_hexpand(true);
                row_box.append(&spacer);
                row_box.append(&bubble);
            },
            ChatRole::Assistant => {
                bubble.add_css_class("chat-bubble-assistant");
                row_box.append(&bubble);
                let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                spacer.set_hexpand(true);
                row_box.append(&spacer);
            },
        }

        chat_box.append(&row_box);
    }
    drop(s);

    // Scroll to the bottom on the next main-loop cycle (after GTK computes size adjustments)
    let scroll_cl = scroll.clone();
    glib::idle_add_local(move || {
        let vadj = scroll_cl.vadjustment();
        vadj.set_value(vadj.upper() - vadj.page_size());
        glib::ControlFlow::Break
    });
}

impl RightPanel {
    pub fn new(state: SharedState) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_width_request(320);
        widget.add_css_class("right-panel");

        let placeholder = adw::StatusPage::new();
        placeholder.set_title("No Video Selected");
        placeholder
            .set_description(Some("Select a video to start chatting with your AI Companion."));
        placeholder.set_icon_name(Some("dialog-question-symbolic"));
        widget.append(&placeholder);

        let content_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_area.set_vexpand(true);

        let chat_area = gtk::Box::new(gtk::Orientation::Vertical, 8);
        chat_area.set_vexpand(true);
        chat_area.set_margin_start(8);
        chat_area.set_margin_end(8);
        chat_area.set_margin_top(8);
        chat_area.set_margin_bottom(8);

        let chat_scroll = gtk::ScrolledWindow::new();
        chat_scroll.set_vexpand(true);
        chat_scroll.set_hexpand(true);

        let chat_history_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        chat_history_box.set_vexpand(true);
        chat_scroll.set_child(Some(&chat_history_box));

        let context_expander = gtk::Expander::new(Some("Video Context (optional)"));
        let context_text = gtk::TextView::new();
        context_text.set_wrap_mode(gtk::WrapMode::Word);
        context_text.set_height_request(60);
        context_text.set_vexpand(false);
        context_expander.set_child(Some(&context_text));
        context_expander.set_margin_bottom(4);

        let chat_bottom = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let chat_input = gtk::Entry::new();
        chat_input.set_hexpand(true);
        chat_input.set_placeholder_text(Some("Ask a question..."));

        let chat_spinner = gtk::Spinner::new();
        chat_spinner.set_halign(gtk::Align::Center);
        chat_spinner.set_valign(gtk::Align::Center);
        chat_spinner.set_margin_start(4);
        chat_spinner.set_margin_end(4);

        let send_btn = gtk::Button::with_label("Send");
        send_btn.add_css_class("suggested-action");

        chat_bottom.append(&chat_input);
        chat_bottom.append(&chat_spinner);
        chat_bottom.append(&send_btn);

        chat_area.append(&chat_scroll);
        chat_area.append(&context_expander);
        chat_area.append(&chat_bottom);

        content_area.append(&chat_area);
        widget.append(&content_area);

        let result = Self {
            widget,
            chat_input,
            chat_spinner,
            chat_history_box,
            chat_scroll,
            state: state.clone(),
            placeholder,
            content_area,
            context_text,
        };

        result.connect_signals(state, send_btn);
        result.refresh();

        result
    }

    fn connect_signals(&self, _state: SharedState, send_btn: gtk::Button) {
        let chat_history_box = self.chat_history_box.clone();
        let chat_input = self.chat_input.clone();
        let chat_spinner = self.chat_spinner.clone();
        let send_btn_cl = send_btn.clone();
        let state_clone = self.state.clone();
        let context_text = self.context_text.clone();
        let chat_scroll = self.chat_scroll.clone();

        let perform_send = Rc::new(move || {
            let question = chat_input.text().as_str().to_string();
            if question.trim().is_empty() {
                return;
            }
            chat_input.set_text("");

            chat_spinner.start();
            chat_input.set_sensitive(false);
            send_btn_cl.set_sensitive(false);

            let (video_id, backend) = {
                let s = state_clone.borrow();
                let vid = match &s.current_video_id {
                    Some(id) => id.clone(),
                    None => return,
                };
                if !s.has_backend() || !s.has_gemini() {
                    return;
                }
                (vid, s.backend.as_ref().cloned())
            };

            {
                let mut s = state_clone.borrow_mut();
                let history = s.chat_history_by_video.entry(video_id.clone()).or_default();
                history.push(ChatMessage { role: ChatRole::User, content: question.clone() });
                history.push(ChatMessage {
                    role: ChatRole::Assistant,
                    content: "Thinking…".to_string(),
                });
                if history.len() > MAX_CHAT_HISTORY_PER_VIDEO {
                    let excess = history.len() - MAX_CHAT_HISTORY_PER_VIDEO;
                    history.drain(0..excess);
                }
            }

            // Save user message to repository
            if let Some(ctx) = &backend
                && let Ok(parsed_video_id) = video_id.parse::<VideoId>()
            {
                let domain_msg = crate::domain::ports::ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    video_id: parsed_video_id,
                    role: crate::domain::ports::ChatRole::User,
                    content: question.clone(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                let _ = ctx.chat_repo.save(&domain_msg);
            }

            // Immediately show user's question and "Thinking..." bubble
            rebuild_chat_history(&chat_history_box, &state_clone, &video_id, &chat_scroll);

            let chat_box = chat_history_box.clone();
            let state = state_clone.clone();
            let vid_for_spawn = video_id.clone();
            let vid = video_id;
            let scroll_for_spawn = chat_scroll.clone();

            let local_context = {
                let buffer = context_text.buffer();
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let s = text.as_str().trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            };

            let backend_for_save = backend.clone();
            let (tx, rx) = std::sync::mpsc::channel::<String>();

            crate::infrastructure::tokio_bridge::spawn(async move {
                let response = match backend {
                    Some(ctx) => match ServiceFactory::ask_companion(&ctx) {
                        Some(uc) => match vid_for_spawn.parse::<VideoId>() {
                            Ok(video_id) => {
                                match uc
                                    .execute(AskCompanionInput {
                                        video_id,
                                        question,
                                        local_context,
                                    })
                                    .await
                                {
                                    Ok(answer) => answer,
                                    Err(e) => format!("AI error: {}", e),
                                }
                            },
                            Err(_) => "Invalid video ID.".to_string(),
                        },
                        None => "AI companion not available.".to_string(),
                    },
                    None => "No backend connected.".to_string(),
                };
                let _ = tx.send(response);
            });

            let chat_spinner_cl = chat_spinner.clone();
            let chat_input_cl = chat_input.clone();
            let send_btn_cl2 = send_btn_cl.clone();

            glib::idle_add_local(move || match rx.try_recv() {
                Ok(response) => {
                    {
                        let mut s = state.borrow_mut();
                        let history = s.chat_history_by_video.entry(vid.clone()).or_default();
                        if let Some(last) = history.last_mut() {
                            if last.role == ChatRole::Assistant && last.content == "Thinking…" {
                                last.content = response.clone();
                            } else {
                                history.push(ChatMessage {
                                    role: ChatRole::Assistant,
                                    content: response.clone(),
                                });
                            }
                        } else {
                            history.push(ChatMessage {
                                role: ChatRole::Assistant,
                                content: response.clone(),
                            });
                        }
                        if history.len() > MAX_CHAT_HISTORY_PER_VIDEO {
                            let excess = history.len() - MAX_CHAT_HISTORY_PER_VIDEO;
                            history.drain(0..excess);
                        }
                    }

                    // Save assistant response to repository
                    if let Some(ctx) = &backend_for_save
                        && let Ok(parsed_video_id) = vid.parse::<VideoId>()
                    {
                        let domain_msg = crate::domain::ports::ChatMessage {
                            id: uuid::Uuid::new_v4().to_string(),
                            video_id: parsed_video_id,
                            role: crate::domain::ports::ChatRole::Assistant,
                            content: response,
                            created_at: chrono::Utc::now().to_rfc3339(),
                        };
                        let _ = ctx.chat_repo.save(&domain_msg);
                    }

                    rebuild_chat_history(&chat_box, &state, &vid, &scroll_for_spawn);
                    chat_spinner_cl.stop();
                    chat_input_cl.set_sensitive(true);
                    send_btn_cl2.set_sensitive(true);
                    glib::ControlFlow::Break
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    {
                        let mut s = state.borrow_mut();
                        let history = s.chat_history_by_video.entry(vid.clone()).or_default();
                        if let Some(last) = history.last_mut()
                            && last.role == ChatRole::Assistant
                            && last.content == "Thinking…"
                        {
                            last.content = "Failed to receive response.".to_string();
                        }
                    }
                    rebuild_chat_history(&chat_box, &state, &vid, &scroll_for_spawn);
                    chat_spinner_cl.stop();
                    chat_input_cl.set_sensitive(true);
                    send_btn_cl2.set_sensitive(true);
                    glib::ControlFlow::Break
                },
            });
        });

        let perform_send_cl1 = perform_send.clone();
        self.chat_input.connect_activate(move |_| {
            perform_send_cl1();
        });

        let perform_send_cl2 = perform_send;
        send_btn.connect_clicked(move |_| {
            perform_send_cl2();
        });
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn refresh(&self) {
        let state = self.state.borrow();
        let video_id = match &state.current_video_id {
            Some(id) => id.clone(),
            None => {
                self.placeholder.set_visible(true);
                self.content_area.set_visible(false);
                return;
            },
        };

        self.placeholder.set_visible(false);
        self.content_area.set_visible(true);
        self.context_text.buffer().set_text("");

        // Fetch companion chat history from SQLite repository if available
        let mut loaded_messages = None;
        if let Some(ctx) = &state.backend
            && let Ok(parsed_video_id) = video_id.parse::<VideoId>()
            && let Ok(messages) = ctx.chat_repo.find_by_video(&parsed_video_id)
        {
            let ui_messages: Vec<ChatMessage> = messages
                .into_iter()
                .map(|msg| {
                    let ui_role = match msg.role {
                        crate::domain::ports::ChatRole::User => ChatRole::User,
                        crate::domain::ports::ChatRole::Assistant => ChatRole::Assistant,
                    };
                    ChatMessage { role: ui_role, content: msg.content }
                })
                .collect();
            loaded_messages = Some(ui_messages);
        }
        drop(state);

        if let Some(ui_messages) = loaded_messages {
            let mut s = self.state.borrow_mut();
            s.chat_history_by_video.insert(video_id.clone(), ui_messages);
        }

        rebuild_chat_history(&self.chat_history_box, &self.state, &video_id, &self.chat_scroll);
    }
}
