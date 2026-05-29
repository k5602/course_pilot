use adw::prelude::*;
use adw::{ViewStack, ViewSwitcher};
use std::fmt::Write;

use crate::application::ServiceFactory;
use crate::application::use_cases::{LoadNoteInput, SaveNoteInput};
use crate::domain::ports::VideoRepository;
use crate::domain::value_objects::VideoId;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;

/// Opens the dynamic popup Notes Window for the currently active video in the workspace.
pub fn open_notes_window(state: SharedState) {
    let s = state.borrow();

    let video_id_str = match &s.current_video_id {
        Some(id) => id.clone(),
        None => {
            Toast::show("Select a video to start taking notes!");
            return;
        },
    };

    let backend = match &s.backend {
        Some(ctx) => ctx.clone(),
        None => {
            Toast::show("No active backend connected.");
            return;
        },
    };
    drop(s);

    // Fetch video details synchronously from repository
    let video_title = match video_id_str.parse::<VideoId>() {
        Ok(vid) => backend
            .video_repo
            .find_by_id(&vid)
            .ok()
            .flatten()
            .map(|v| v.title().to_string())
            .unwrap_or_else(|| "Video Notes".to_string()),
        Err(_) => "Video Notes".to_string(),
    };

    // Load initial content from SQLite
    let initial_content = match video_id_str.parse::<VideoId>() {
        Ok(vid) => ServiceFactory::notes(&backend)
            .load_note(LoadNoteInput { video_id: vid })
            .ok()
            .flatten()
            .map(|n| n.content)
            .unwrap_or_default(),
        Err(_) => String::new(),
    };

    // Construct the popup window
    let window = gtk::Window::new();
    window.set_title(Some(&format!("Notes: {}", video_title)));
    window.set_default_size(750, 600);
    window.set_modal(true);
    window.add_css_class("notes-window");

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 12);

    // Header Bar
    let header = adw::HeaderBar::new();

    let title_widget =
        adw::WindowTitle::new(&format!("Notes: {}", video_title), "Markdown & LaTeX editor");
    header.set_title_widget(Some(&title_widget));

    // Split/Tab layout
    let stack = ViewStack::new();
    stack.set_vexpand(true);

    let switcher = ViewSwitcher::new();
    switcher.set_stack(Some(&stack));
    switcher.set_policy(adw::ViewSwitcherPolicy::Wide);
    header.set_title_widget(Some(&switcher));

    // Save Button
    let save_btn = gtk::Button::with_label("Save Note");
    save_btn.add_css_class("suggested-action");
    header.pack_end(&save_btn);

    // Insert Reference Button
    let ref_btn = gtk::Button::new();
    ref_btn.set_icon_name("link-symbolic");
    ref_btn.set_tooltip_text(Some("Insert Reference"));
    header.pack_start(&ref_btn);

    main_box.append(&header);

    // 1. Edit (Type Mode) Panel
    let editor_scroll = gtk::ScrolledWindow::new();
    editor_scroll.set_vexpand(true);

    let editor = gtk::TextView::new();
    editor.set_wrap_mode(gtk::WrapMode::Word);
    editor.set_margin_start(8);
    editor.set_margin_end(8);
    editor.set_margin_top(8);
    editor.set_margin_bottom(8);
    editor.add_css_class("notes-editor-panel");
    editor.buffer().set_text(&initial_content);

    editor_scroll.set_child(Some(&editor));
    stack.add_titled(&editor_scroll, Some("edit"), "Type Mode");

    // 2. Preview Mode Panel
    let preview_scroll = gtk::ScrolledWindow::new();
    preview_scroll.set_vexpand(true);

    let preview_label = gtk::Label::new(None);
    preview_label.set_wrap(true);
    preview_label.set_halign(gtk::Align::Start);
    preview_label.set_valign(gtk::Align::Start);
    preview_label.set_margin_start(16);
    preview_label.set_margin_end(16);
    preview_label.set_margin_top(16);
    preview_label.set_margin_bottom(16);
    preview_label.add_css_class("notes-preview-panel");

    preview_scroll.set_child(Some(&preview_label));
    stack.add_titled(&preview_scroll, Some("preview"), "Preview Mode");

    main_box.append(&stack);
    window.set_child(Some(&main_box));

    // Connect Dynamic Preview compilation on visible-child changes
    let preview_label_cl = preview_label.clone();
    let editor_cl = editor.clone();
    stack.connect_visible_child_notify(move |st| {
        if st.visible_child_name().as_deref() == Some("preview") {
            let buffer = editor_cl.buffer();
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
            let pango_markup = parse_markdown_to_pango(&text);
            preview_label_cl.set_markup(&pango_markup);
        }
    });

    // Save notes to DB on click
    let editor_save = editor.clone();
    let state_save = state.clone();
    let vid_save = video_id_str.clone();
    save_btn.connect_clicked(move |_| {
        let buffer = editor_save.buffer();
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
        let s = state_save.borrow();
        if let Some(ref ctx) = s.backend
            && let Ok(vid) = vid_save.parse::<VideoId>()
        {
            match ServiceFactory::notes(ctx)
                .save_note(SaveNoteInput { video_id: vid, content: text })
            {
                Ok(_) => {
                    Toast::show("Notes saved successfully.");
                },
                Err(e) => {
                    Toast::show_error(&format!("Failed to save notes: {}", e));
                },
            }
        }
    });

    // Reference Injection
    let editor_ref = editor.clone();
    let ref_title = video_title;
    ref_btn.connect_clicked(move |_| {
        let buffer = editor_ref.buffer();
        let ref_md = format!("**Reference:** *{}*\n", ref_title);
        buffer.insert_at_cursor(&ref_md);
    });

    window.present();
}

/// Converts simple Markdown and LaTeX formatting into GTK/Pango markup representation.
fn parse_markdown_to_pango(markdown: &str) -> String {
    // Escape standard XML tags so they don't corrupt the Pango parser
    let escaped = glib::markup_escape_text(markdown).to_string();

    // Pre-allocate for String instead of Vec<String> join pattern
    let mut result = String::with_capacity(escaped.len() + (escaped.len() / 4));
    let mut in_code_block = false;

    // Line-by-line processor for headings and code blocks
    for line in escaped.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if !result.is_empty() {
                result.push('\n');
            }
            if in_code_block {
                result.push_str("<span font_family=\"monospace\" foreground=\"#e06c75\">");
            } else {
                result.push_str("</span>");
            }
            continue;
        }

        if in_code_block {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
            continue;
        }

        if !result.is_empty() {
            result.push('\n');
        }

        if let Some(content) = trimmed.strip_prefix("### ") {
            write!(result, "<span size=\"large\" weight=\"bold\">{}</span>", content).unwrap();
        } else if let Some(content) = trimmed.strip_prefix("## ") {
            write!(result, "<span size=\"x-large\" weight=\"bold\">{}</span>", content).unwrap();
        } else if let Some(content) = trimmed.strip_prefix("# ") {
            write!(result, "<span size=\"xx-large\" weight=\"bold\">{}</span>", content).unwrap();
        } else if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
            let content = &trimmed[2..];
            write!(result, "  • {}", content).unwrap();
        } else {
            result.push_str(line);
        }
    }

    let mut body = result;

    // Asterisk parser for Bold (**) and Italic (*)
    let mut bold_state = false;
    let mut italic_state = false;
    let mut out = String::with_capacity(body.len() + 32);
    let bytes = body.as_bytes();
    let mut pos = 0;
    let len = bytes.len();

    while pos < len {
        if bytes.get(pos..pos + 2) == Some(b"**") {
            out.push_str(if bold_state { "</b>" } else { "<b>" });
            bold_state = !bold_state;
            pos += 2;
        } else if bytes[pos] == b'*' {
            out.push_str(if italic_state { "</i>" } else { "<i>" });
            italic_state = !italic_state;
            pos += 1;
        } else {
            let start = pos;
            while pos < len && bytes[pos] != b'*' {
                pos += 1;
            }
            out.push_str(&body[start..pos]);
        }
    }
    body = out;

    // LaTeX math syntax processor ($$ LaTeX block $$, $ LaTeX inline $)
    let mut out = String::with_capacity(body.len() + 256);
    let parts = body.split("$$");
    for (idx, part) in parts.enumerate() {
        if idx % 2 == 1 {
            out.push_str("\n<span background=\"#2e3440\" foreground=\"#a3be8c\" font_family=\"monospace\"><b>   [LaTeX Block Equation]   </b>\n   ");
            out.push_str(part.trim());
            out.push_str("</span>\n");
        } else {
            let inline = part.split('$');
            for (i2, p2) in inline.enumerate() {
                if i2 % 2 == 1 {
                    out.push_str("<span foreground=\"#a3be8c\" font_family=\"monospace\"><i>");
                    out.push_str(p2);
                    out.push_str("</i></span>");
                } else {
                    out.push_str(p2);
                }
            }
        }
    }
    body = out;

    body
}
