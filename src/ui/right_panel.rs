use adw::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::{AskCompanionInput, LoadNoteInput, SaveNoteInput};
use crate::domain::value_objects::VideoId;
use crate::ui::state::{ChatMessage, ChatRole, SharedState};

pub struct RightPanel {
    widget: gtk::Box,
    notes_text: gtk::TextView,
    chat_input: gtk::Entry,
    chat_history_box: gtk::Box,
    state: SharedState,
    placeholder: adw::StatusPage,
    content_area: gtk::Box,
    context_text: gtk::TextView,
}

impl RightPanel {
    pub fn new(state: SharedState) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_width_request(320);
        widget.add_css_class("right-panel");

        let placeholder = adw::StatusPage::new();
        placeholder.set_title("No Video Selected");
        placeholder.set_description(Some("Select a video to start taking notes."));
        placeholder.set_icon_name(Some("edit-note-symbolic"));
        widget.append(&placeholder);

        let content_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_area.set_vexpand(true);

        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        let switcher = adw::ViewSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_policy(adw::ViewSwitcherPolicy::Wide);

        let notes_scroll = gtk::ScrolledWindow::new();
        notes_scroll.set_vexpand(true);
        let notes_vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        notes_vbox.set_margin_start(8);
        notes_vbox.set_margin_end(8);
        notes_vbox.set_margin_top(8);
        notes_vbox.set_margin_bottom(8);

        let notes_text = gtk::TextView::new();
        notes_text.set_wrap_mode(gtk::WrapMode::Word);
        notes_text.set_vexpand(true);
        notes_text.add_css_class("notes-text-view");

        let save_btn = gtk::Button::with_label("Save");
        save_btn.add_css_class("suggested-action");

        notes_vbox.append(&notes_text);
        notes_vbox.append(&save_btn);
        notes_scroll.set_child(Some(&notes_vbox));
        stack.add_titled(&notes_scroll, Some("notes"), "Notes");

        let chat_scroll = gtk::ScrolledWindow::new();
        chat_scroll.set_vexpand(true);
        let chat_vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        chat_vbox.set_margin_start(8);
        chat_vbox.set_margin_end(8);
        chat_vbox.set_margin_top(8);
        chat_vbox.set_margin_bottom(8);

        let chat_history_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        chat_history_box.set_vexpand(true);

        let chat_bottom = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let chat_input = gtk::Entry::new();
        chat_input.set_hexpand(true);
        chat_input.set_placeholder_text(Some("Ask a question..."));
        let send_btn = gtk::Button::with_label("Send");
        send_btn.add_css_class("suggested-action");

        chat_bottom.append(&chat_input);
        chat_bottom.append(&send_btn);

        chat_vbox.append(&chat_history_box);

        let context_expander = gtk::Expander::new(Some("Video Context (optional)"));
        let context_text = gtk::TextView::new();
        context_text.set_wrap_mode(gtk::WrapMode::Word);
        context_text.set_height_request(60);
        context_text.set_vexpand(false);
        context_expander.set_child(Some(&context_text));
        context_expander.set_margin_bottom(4);
        chat_vbox.append(&context_expander);

        chat_vbox.append(&chat_bottom);
        chat_scroll.set_child(Some(&chat_vbox));
        stack.add_titled(&chat_scroll, Some("chat"), "AI Chat");

        content_area.append(&switcher);
        content_area.append(&stack);

        widget.append(&content_area);

        let result = Self {
            widget,
            notes_text,
            chat_input,
            chat_history_box,
            state: state.clone(),
            placeholder,
            content_area,
            context_text,
        };

        result.connect_signals(state.clone(), send_btn, save_btn);

        result.refresh();

        result
    }

    fn connect_signals(&self, state: SharedState, send_btn: gtk::Button, save_btn: gtk::Button) {
        let notes_text = self.notes_text.clone();
        save_btn.connect_clicked(move |_| {
            let state = state.borrow();
            let video_id = match &state.current_video_id {
                Some(id) => id.clone(),
                None => return,
            };
            let text = notes_text
                .buffer()
                .text(&notes_text.buffer().start_iter(), &notes_text.buffer().end_iter(), false)
                .as_str()
                .to_string();
            if let Some(ref ctx) = state.backend
                && let Ok(vid) = video_id.parse::<VideoId>()
            {
                let _ = ServiceFactory::notes(ctx)
                    .save_note(SaveNoteInput { video_id: vid, content: text });
            }
            drop(state);
        });

        let chat_history_box = self.chat_history_box.clone();
        let chat_input = self.chat_input.clone();
        let state_clone = self.state.clone();
        let context_text = self.context_text.clone();

        send_btn.connect_clicked(move |_| {
            let question = chat_input.text().as_str().to_string();
            if question.trim().is_empty() {
                return;
            }
            chat_input.set_text("");

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
                s.chat_history
                    .push(ChatMessage { role: ChatRole::User, content: question.clone() });
                s.chat_history_by_video
                    .entry(video_id.clone())
                    .or_default()
                    .push(ChatMessage { role: ChatRole::User, content: question.clone() });
            }

            let chat_box = chat_history_box.clone();
            let state = state_clone.clone();
            let vid_for_spawn = video_id.clone();
            let vid = video_id;

            let local_context = {
                let buffer = context_text.buffer();
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let s = text.as_str().trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            };

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

            glib::idle_add_local(move || match rx.try_recv() {
                Ok(response) => {
                    {
                        let mut s = state.borrow_mut();
                        s.chat_history.push(ChatMessage {
                            role: ChatRole::Assistant,
                            content: response.clone(),
                        });
                        s.chat_history_by_video.entry(vid.clone()).or_default().push(ChatMessage {
                            role: ChatRole::Assistant,
                            content: response.clone(),
                        });
                    }

                    while let Some(child) = chat_box.first_child() {
                        chat_box.remove(&child);
                    }

                    let s = state.borrow();
                    for msg in &s.chat_history {
                        let label = gtk::Label::new(None);
                        label.set_wrap(true);
                        label.set_xalign(0.0);
                        label.set_margin_bottom(4);
                        let prefix = match msg.role {
                            ChatRole::User => "You: ",
                            ChatRole::Assistant => "AI: ",
                        };
                        label.set_markup(&format!(
                            "<b>{}</b>{}",
                            prefix,
                            glib::markup_escape_text(&msg.content)
                        ));
                        chat_box.append(&label);
                    }
                    glib::ControlFlow::Break
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            });
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

        let notes_text = self.notes_text.clone();
        let chat_box = self.chat_history_box.clone();

        if let Some(ref ctx) = state.backend {
            if let Ok(vid) = video_id.parse::<VideoId>() {
                match ServiceFactory::notes(ctx).load_note(LoadNoteInput { video_id: vid }) {
                    Ok(Some(note_view)) => {
                        notes_text.buffer().set_text(&note_view.content);
                    },
                    _ => {
                        notes_text.buffer().set_text("");
                    },
                }
            }
        } else {
            notes_text.buffer().set_text("");
        }

        let history = state.chat_history_by_video.get(&video_id).cloned().unwrap_or_default();

        drop(state);
        {
            let mut s = self.state.borrow_mut();
            s.chat_history = history.clone();
        }

        while let Some(child) = chat_box.first_child() {
            chat_box.remove(&child);
        }

        for msg in &history {
            let label = gtk::Label::new(None);
            label.set_wrap(true);
            label.set_xalign(0.0);
            label.set_margin_bottom(4);
            let prefix = match msg.role {
                ChatRole::User => "You: ",
                ChatRole::Assistant => "AI: ",
            };
            label.set_markup(&format!(
                "<b>{}</b>{}",
                prefix,
                glib::markup_escape_text(&msg.content)
            ));
            chat_box.append(&label);
        }
    }
}
