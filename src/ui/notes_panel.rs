use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaBars, FaCirclePlay, FaDownload, FaEllipsis, FaFloppyDisk, FaPen, FaTrash,
};

// Use real Note type from backend/types
use uuid::Uuid;

/// NotesPanel: Contextual panel with tabs for Notes and Player
#[component]
pub fn NotesPanel() -> Element {
    // Tab state: 0 = Notes, 1 = Player
    let mut tab = use_signal(|| 0);

    // For demo, use a fixed course_id and video_id (replace with router param or prop)
    let course_id = Uuid::nil();
    let video_id = None;

    // Simulate async loading and error state (replace with real async logic as needed)
    let is_loading = false; // Set to true to simulate loading
    let has_error = false; // Set to true to simulate error

    if has_error {
        return rsx! {
            div {
                class: "flex flex-col h-full w-full items-center justify-center",
                div { class: "text-error", "Failed to load notes. Please try again." }
            }
        };
    }

    if is_loading {
        return rsx! {
            div {
                class: "flex flex-col h-full w-full p-4",
                div { class: "tabs tabs-boxed flex gap-2 p-2 bg-base-100/80 mb-4 animate-pulse" }
                {(0..3).map(|_| rsx! {
                    div { class: "card bg-base-200 shadow-sm p-4 mb-4 animate-pulse",
                        div { class: "h-4 w-1/2 bg-base-300 rounded mb-2" }
                        div { class: "h-3 w-full bg-base-300 rounded mb-2" }
                        div { class: "h-2 w-1/3 bg-base-300 rounded" }
                    }
                })}
            }
        };
    }

    rsx! {
        div {
            class: "flex flex-col h-full w-full",
            // Tabs
            div {
                class: "tabs tabs-boxed flex gap-2 p-2 bg-base-100/80",
                button {
                    class: if *tab.read() == 0 { "tab tab-active flex items-center gap-1" } else { "tab flex items-center gap-1" },
                    onclick: move |_| tab.set(0),
                    Icon { icon: FaBars, class: "w-5 h-5" },
                    "Notes"
                }
                button {
                    class: if *tab.read() == 1 { "tab tab-active flex items-center gap-1" } else { "tab flex items-center gap-1" },
                    onclick: move |_| tab.set(1),
                    Icon { icon: FaCirclePlay, class: "w-5 h-5" },
                    "Player"
                }
            }
            // Tab content
            div {
                class: "flex-1 overflow-y-auto p-4",
                match *tab.read() {
                    0 => rsx!(NotesTab { course_id, video_id }),
                    1 => rsx!(PlayerTab {}),
                    _ => rsx!({}),
                }
            }
        }
    }
}

/// NotesTab: List of notes and markdown editor (wired to backend)
#[component]
fn NotesTab(course_id: uuid::Uuid, video_id: Option<uuid::Uuid>) -> Element {
    let notes = crate::ui::hooks::use_notes(course_id, video_id);

    rsx! {
        div {
            class: "space-y-6",
            // Notes list
            {
                let notes_vec: Vec<_> = notes.read().iter().cloned().collect();
                rsx! {
                    div {
                        class: "space-y-4",
                        {notes_vec.iter().map(|note| rsx! {
                            NoteCard {
                                content: note.content.clone(),
                                timestamp: note.timestamp,
                                tags: note.tags.clone(),
                                created_at: note.created_at.to_string(),
                                updated_at: note.updated_at.to_string()
                            }
                        })}
                    }
                }
            }
            // Markdown editor (placeholder)
            div {
                class: "mt-6",
                h3 { class: "text-base font-semibold mb-2", "Add/Edit Note" }
                textarea {
                    class: "textarea textarea-bordered w-full min-h-[100px] mb-2",
                    placeholder: "Write your note in markdown..."
                }
                div {
                    class: "flex gap-2",
                    button {
                        class: "btn btn-accent btn-sm flex items-center gap-1",
                        // on_click: move |_| { /* save note logic */ }
                        Icon { icon: FaFloppyDisk, class: "w-5 h-5" },
                        "Save"
                    }
                    button {
                        class: "btn btn-outline btn-sm flex items-center gap-1",
                        // on_click: move |_| { /* edit note logic */ }
                        Icon { icon: FaPen, class: "w-5 h-5" },
                        "Edit"
                    }
                }
            }
        }
    }
}

/// NoteCard: Single note display
#[derive(Props, PartialEq, Clone)]
struct NoteCardProps {
    content: String,
    timestamp: Option<u32>,
    tags: Vec<String>,
    created_at: String,
    updated_at: String,
}

#[component]
fn NoteCard(props: NoteCardProps) -> Element {
    let ts = props
        .timestamp
        .map(|t| format!(" at {}s", t))
        .unwrap_or_default();

    let note_for_render = course_pilot::types::Note {
        content: props.content.to_string(),
        // The rest are dummy fields, only content is used for rendering
        id: uuid::Uuid::nil(),
        course_id: uuid::Uuid::nil(),
        video_id: None,
        timestamp: None,
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let rendered_html = course_pilot::storage::notes::render_note_html(&note_for_render);

    rsx! {
        div {
            class: "card bg-base-200 shadow-sm p-4 relative",
            // Dropdown/context menu for note actions
            div {
                class: "absolute top-2 right-2 dropdown dropdown-end z-10",
                tabindex: "0",
                button {
                    class: "btn btn-ghost btn-xs rounded-full hover:bg-base-300",
                    tabindex: "0",
                    Icon { icon: FaEllipsis, class: "w-5 h-5" }
                }
                ul {
                    class: "dropdown-content menu p-2 shadow bg-base-200 rounded-box w-36 z-50",
                    li {
                        button {
                            class: "flex items-center gap-2",
                            // on_click: move |_| { /* export note logic */ },
                            Icon { icon: FaDownload, class: "w-4 h-4" }
                            "Export"
                        }
                    }
                    li {
                        button {
                            class: "flex items-center gap-2",
                            // on_click: move |_| { /* edit note logic */ },
                            Icon { icon: FaPen, class: "w-4 h-4" }
                            "Edit"
                        }
                    }
                    li {
                        button {
                            class: "flex items-center gap-2 text-error",
                            // on_click: move |_| { /* delete note logic */ },
                            Icon { icon: FaTrash, class: "w-4 h-4" }
                            "Delete"
                        }
                    }
                }
            }
            div {
                class: "flex items-center gap-2 mb-1",
                span { class: "text-xs text-base-content/60", "{props.created_at}" }
                if ts.len() > 0 {
                    span { class: "badge badge-outline badge-xs ml-2", "{ts}" }
                }
            }
            div {
                class: "prose prose-sm max-w-none mb-2",
                dangerous_inner_html: "{rendered_html}"
            }
            div {
                class: "flex gap-2 flex-wrap",
                {props.tags.iter().map(|tag| rsx! {
                    span { class: "badge badge-accent badge-outline badge-xs", "#{tag}" }
                })}
            }
            div {
                class: "text-xs text-base-content/40 mt-1",
                "Updated: {props.updated_at}"
            }
        }
    }
}

/// PlayerTab: Placeholder for embedded video player
#[component]
fn PlayerTab() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full w-full",
            div {
                class: "w-full aspect-video bg-base-300 rounded-lg flex items-center justify-center text-base-content/40",
                "Video Player Placeholder"
            }
            p {
                class: "mt-4 text-sm text-base-content/60",
                "The built-in player will appear here."
            }
        }
    }
}
