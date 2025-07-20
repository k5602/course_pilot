use crate::ui::components::modal_confirmation::{
    ActionMenu, AdvancedTabs, Badge, ModalConfirmation,
};
use crate::ui::components::toast::toast;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaDownload, FaFloppyDisk, FaPen, FaTrash,
};
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;

// Use real Note type from backend/types
use uuid::Uuid;

/// NotesPanel: Contextual panel with tabs for Notes and Player
#[component]
pub fn NotesPanel(course_id: Option<Uuid>) -> Element {
    // Use the provided course_id or fall back to nil for backward compatibility
    let course_id = course_id.unwrap_or(Uuid::nil());
    let video_id = None;

    // Use async notes resource and handle loading/error state
    let notes_resource = crate::ui::hooks::use_notes_resource(course_id, video_id);

    match &*notes_resource.read_unchecked() {
        None => {
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
        Some(Err(_err)) => {
            return rsx! {
                div {
                    class: "flex flex-col h-full w-full items-center justify-center",
                    div { class: "text-error", "Failed to load notes" }
                }
            };
        }
        Some(Ok(_notes)) => {
            // AdvancedTabs for Notes/Player
            let tabs = vec![
                crate::ui::components::modal_confirmation::TabData {
                    label: "Notes".to_string(),
                    content: rsx!(NotesTab {
                        course_id: course_id,
                        video_id: video_id
                    }),
                    closable: false,
                },
                crate::ui::components::modal_confirmation::TabData {
                    label: "Player".to_string(),
                    content: rsx!(PlayerTab {}),
                    closable: false,
                },
            ];
            let mut selected = use_signal(|| 0);

            rsx! {
                div {
                    class: "flex flex-col h-full w-full",
                    AdvancedTabs {
                        tabs: tabs.clone(),
                        selected: selected(),
                        on_select: move |idx| selected.set(idx),
                    }
                }
            }
        }
    }
}

/// NotesTab: List of notes and markdown editor (wired to backend)
#[component]
fn NotesTab(course_id: uuid::Uuid, video_id: Option<uuid::Uuid>) -> Element {
    let notes_resource = crate::ui::hooks::use_notes_resource(course_id, video_id);

    match &*notes_resource.read_unchecked() {
        None => {
            return rsx! {
                div {
                    class: "space-y-6",
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
        Some(Err(_err)) => {
            return rsx! {
                div {
                    class: "space-y-6 text-error",
                    "Failed to load notes."
                }
            };
        }
        Some(Ok(notes_vec)) => {
            rsx! {
                div {
                    class: "space-y-6",
                    // Notes list
                    div {
                        class: "space-y-4",
                        {notes_vec.iter().map(|note| rsx! {
                            NoteCard {
                                content: note.content.clone(),
                                timestamp: note.timestamp,
                                tags: note.tags.clone(),
                                created_at: note.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                updated_at: note.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                            }
                        })}
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

    let note_for_render = crate::types::Note {
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
    let rendered_html = crate::storage::notes::render_note_html(&note_for_render);

    // Animation for note card presence
    let mut card_opacity = use_motion(0.0f32);
    let mut card_y = use_motion(12.0f32);

    use_effect(move || {
        card_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        card_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let card_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px); transition: opacity 0.3s, transform 0.3s;",
            card_opacity.get_value(),
            card_y.get_value()
        )
    });

    // Modal state for delete confirmation
    let mut show_delete_modal = use_signal(|| false);

    // ActionMenu for note actions
    let actions = vec![
        crate::ui::components::modal_confirmation::DropdownItem {
            label: "Export".to_string(),
            icon: Some(rsx!(Icon {
                icon: FaDownload,
                class: "w-4 h-4"
            })),
            on_select: Some(EventHandler::new(|_| toast::info("Exported note (stub)"))),
            children: None,
            disabled: false,
        },
        crate::ui::components::modal_confirmation::DropdownItem {
            label: "Edit".to_string(),
            icon: Some(rsx!(Icon {
                icon: FaPen,
                class: "w-4 h-4"
            })),
            on_select: Some(EventHandler::new(|_| toast::info("Edit note (stub)"))),
            children: None,
            disabled: false,
        },
        crate::ui::components::modal_confirmation::DropdownItem {
            label: "Delete".to_string(),
            icon: Some(rsx!(Icon {
                icon: FaTrash,
                class: "w-4 h-4"
            })),
            on_select: Some(EventHandler::new({
                let mut show_delete_modal = show_delete_modal.clone();
                move |_| show_delete_modal.set(true)
            })),
            children: None,
            disabled: false,
        },
    ];

    rsx! {
        div {
            class: "card bg-base-200 shadow-sm p-4 relative",
            style: "{card_style}",
            // ActionMenu for note actions
            div {
                class: "absolute top-2 right-2 z-10",
                ActionMenu { actions: actions.clone() }
            }
            div {
                class: "flex items-center gap-2 mb-1",
                span { class: "text-xs text-base-content/60", "{props.created_at}" }
                if ts.len() > 0 {
                    Badge { label: ts.clone(), color: Some("accent".to_string()), class: Some("badge-outline badge-xs ml-2".to_string()) }
                }
            }
            div {
                class: "prose prose-sm max-w-none mb-2",
                dangerous_inner_html: "{rendered_html}"
            }
            div {
                class: "flex gap-2 flex-wrap",
                {props.tags.iter().map(|tag| rsx! {
                    Badge { label: format!("#{tag}"), color: Some("accent".to_string()), class: Some("badge-outline badge-xs".to_string()) }
                })}
            }
            div {
                class: "text-xs text-base-content/40 mt-1",
                "Updated: {props.updated_at}"
            }
            // ModalConfirmation for delete
            ModalConfirmation {
                open: show_delete_modal(),
                title: "Delete Note",
                message: "Are you sure you want to delete this note? This action cannot be undone.",
                confirm_label: Some("Delete".to_string()),
                cancel_label: Some("Cancel".to_string()),
                confirm_color: Some("error".to_string()),
                on_confirm: move |_| {
                    show_delete_modal.set(false);
                    toast::success("Note deleted (stub)");
                },
                on_cancel: move |_| show_delete_modal.set(false),
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
