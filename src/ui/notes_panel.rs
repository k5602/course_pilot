use crate::ui::components::SearchHistory;
use crate::ui::components::TagInput;
use crate::ui::components::modal::{Modal, confirmation_modal};
use crate::ui::components::{Badge, DropdownItem, DropdownTrigger, UnifiedDropdown};

use crate::ui::components::toast::toast;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaFloppyDisk, FaMagnifyingGlass, FaNoteSticky, FaTag,
};
use dioxus_motion::prelude::*;
use std::collections::HashSet;

// Use real Note type from backend/types
use uuid::Uuid;

/// Represents the mode for the notes panel
#[derive(Debug, Clone, PartialEq)]
pub enum NotesPanelMode {
    /// Show all notes across all courses
    AllNotes,
    /// Show notes for a specific course
    CourseNotes(Uuid),
    /// Show notes for a specific video within a course
    VideoNotes(Uuid, usize, String, String), // course_id, video_index, video_title, module_title
}

/// NotesPanel: Contextual panel that directly shows notes content
#[component]
pub fn NotesPanel(mode: NotesPanelMode) -> Element {
    match mode {
        NotesPanelMode::AllNotes => {
            rsx!(AllNotesTab {})
        }
        NotesPanelMode::CourseNotes(course_id) => {
            let video_id = None;
            rsx!(NotesTab {
                course_id: course_id,
                video_id: video_id,
                video_context: None
            })
        }
        NotesPanelMode::VideoNotes(course_id, video_index, video_title, module_title) => {
            let video_id = None; // We'll use video_index for now since we don't have video UUIDs
            let video_context = Some((video_index, video_title, module_title));
            rsx!(NotesTab {
                course_id: course_id,
                video_id: video_id,
                video_context: video_context
            })
        }
    }
}

/// NotesTab: List of notes and markdown editor (wired to backend)
#[component]
fn NotesTab(
    course_id: uuid::Uuid,
    video_id: Option<uuid::Uuid>,
    video_context: Option<(usize, String, String)>, // (video_index, video_title, module_title)
) -> Element {
    // Use a unified resource that handles video_index filtering
    let video_index = video_context.as_ref().map(|(index, _, _)| *index);
    let mut notes_resource =
        crate::ui::hooks::use_notes_with_video_index_resource(course_id, video_index);
    let mut search_query = use_signal(String::new);
    let mut selected_tags = use_signal(Vec::new);
    let mut show_tag_filter = use_signal(|| false);
    let mut show_search = use_signal(|| false);
    let mut show_search_history = use_signal(|| false);
    let mut note_content = use_signal(String::new);
    let mut editing_note_id = use_signal(|| None::<uuid::Uuid>);
    let mut editing_note_tags = use_signal(Vec::new);

    // Search history
    let mut recent_searches = use_signal(|| {
        // Load from local storage in a real app
        Vec::new()
    });
    let mut saved_searches = use_signal(|| {
        // Load from local storage in a real app
        Vec::new()
    });

    // Get all available tags from notes
    let all_tags = use_memo(move || {
        if let Some(Ok(notes)) = &*notes_resource.read_unchecked() {
            let mut tags = HashSet::new();
            for note in notes {
                for tag in &note.tags {
                    tags.insert(tag.clone());
                }
            }
            tags.into_iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    });

    // Filter notes based on search query and selected tags
    let filtered_notes = use_memo(move || {
        if let Some(Ok(notes)) = &*notes_resource.read_unchecked() {
            notes
                .iter()
                .filter(|note| {
                    // Filter by search query
                    let matches_query = search_query().is_empty()
                        || note
                            .content
                            .to_lowercase()
                            .contains(&search_query().to_lowercase());

                    // Filter by selected tags
                    let matches_tags = selected_tags().is_empty()
                        || selected_tags().iter().any(|tag| note.tags.contains(tag));

                    matches_query && matches_tags
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    });

    // Handle tag selection
    let handle_tag_selection = move |tags: Vec<String>| {
        selected_tags.set(tags);
    };

    // Handle editing note tags
    let handle_editing_tags_change = move |tags: Vec<String>| {
        editing_note_tags.set(tags);
    };

    // Handle save note
    let save_note_action = crate::ui::hooks::use_save_note_action();

    let handle_save_note = {
        let save_note_action = save_note_action.clone();
        let video_context = video_context.clone();
        move |_| {
            let content = note_content();
            if content.trim().is_empty() {
                toast::warning("Note content cannot be empty");
                return;
            }

            let note = match editing_note_id() {
                Some(id) => {
                    // Update existing note
                    if let Some(Ok(notes)) = &*notes_resource.read_unchecked() {
                        if let Some(existing_note) = notes.iter().find(|n| n.id == id) {
                            let mut updated_note = existing_note.clone();
                            updated_note.content = content;
                            updated_note.tags = editing_note_tags();
                            updated_note.updated_at = chrono::Utc::now();
                            // Preserve or update video_index if we have video context
                            if let Some((video_index, _, _)) = &video_context {
                                updated_note.video_index = Some(*video_index);
                            }
                            updated_note
                        } else {
                            toast::error("Failed to find note to update");
                            return;
                        }
                    } else {
                        toast::error("Failed to load notes");
                        return;
                    }
                }
                None => {
                    // Create new note
                    crate::types::Note {
                        id: uuid::Uuid::new_v4(),
                        course_id,
                        video_id,
                        video_index: video_context.as_ref().map(|(index, _, _)| *index),
                        content,
                        timestamp: None, // Could add timestamp input for video notes
                        tags: editing_note_tags(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                }
            };

            save_note_action(note);

            // Reset form
            note_content.set(String::new());
            editing_note_id.set(None);
            editing_note_tags.set(Vec::new());

            // Refresh notes
            notes_resource.restart();
        }
    };

    // Handle edit note
    let mut handle_edit_note = move |note: crate::types::Note| {
        note_content.set(note.content);
        editing_note_id.set(Some(note.id));
        editing_note_tags.set(note.tags);
    };

    // Handle cancel edit
    let handle_cancel_edit = move |_| {
        note_content.set(String::new());
        editing_note_id.set(None);
        editing_note_tags.set(Vec::new());
    };

    // Tag statistics
    let tag_stats = use_memo(move || {
        if let Some(Ok(notes)) = &*notes_resource.read_unchecked() {
            let mut stats = std::collections::HashMap::new();
            for note in notes {
                for tag in &note.tags {
                    *stats.entry(tag.clone()).or_insert(0) += 1;
                }
            }
            stats
        } else {
            std::collections::HashMap::new()
        }
    });

    // Handle search
    let mut handle_search = move |_| {
        let query = search_query();
        if !query.is_empty() {
            // Add to recent searches if not already there
            let mut searches = recent_searches();
            if !searches.contains(&query) {
                searches.insert(0, query.clone());
                // Limit to 10 recent searches
                if searches.len() > 10 {
                    searches.pop();
                }
                recent_searches.set(searches);
            }
        }
    };

    // Handle search history selection
    let handle_search_history_select = move |query: String| {
        search_query.set(query);
        show_search_history.set(false);
        handle_search(());
    };

    // Handle save search
    let handle_save_search = move |query: String| {
        let mut searches = saved_searches();
        if !searches.contains(&query) {
            searches.push(query);
            saved_searches.set(searches);
            toast::success("Search saved");
        }
    };

    // Handle delete search
    let handle_delete_search = move |query: String| {
        // Remove from recent searches
        let mut searches = recent_searches();
        searches.retain(|s| s != &query);
        recent_searches.set(searches);

        // Remove from saved searches
        let mut saved = saved_searches();
        saved.retain(|s| s != &query);
        saved_searches.set(saved);
    };

    // Handle clear all searches
    let handle_clear_all_searches = move |_| {
        recent_searches.set(Vec::new());
        toast::info("Search history cleared");
    };

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
        Some(Ok(_)) => {
            // Extract temporary values to avoid borrowing issues
            let tag_stats_data = tag_stats.read_unchecked();
            let filtered_notes_data = filtered_notes();

            rsx! {
                div {
                    class: "space-y-6",

                    // Video context header
                    if let Some((video_index, video_title, module_title)) = video_context {
                        div {
                            class: "bg-primary/10 border border-primary/20 rounded-lg p-3 mb-4",
                            div {
                                class: "flex items-center gap-2 mb-1",
                                Icon { icon: FaNoteSticky, class: "w-4 h-4 text-primary" }
                                h3 { class: "font-medium text-sm text-primary", "Video Notes" }
                            }
                            p { class: "text-xs text-base-content/70", "Module: {module_title}" }
                            p { class: "text-xs text-base-content/70", "Video: {video_title}" }
                            p { class: "text-xs text-base-content/70", "Video #{video_index + 1}" }
                        }
                    }

                    // Search and filter controls
                    div {
                        class: "flex flex-wrap gap-2 mb-4",
                        // Search button
                        button {
                            class: "btn btn-sm btn-outline gap-1",
                            onclick: move |_| show_search.set(!show_search()),
                            Icon { icon: FaMagnifyingGlass, class: "w-4 h-4" }
                            "Search"
                        }
                        // Tag filter button
                        button {
                            class: "btn btn-sm btn-outline gap-1",
                            onclick: move |_| show_tag_filter.set(!show_tag_filter()),
                            Icon { icon: FaTag, class: "w-4 h-4" }
                            "Filter by Tags"
                        }
                        // Tag statistics
                        div {
                            class: "flex-grow text-right text-xs text-base-content/60",
                            "Found {filtered_notes().len()} notes"
                        }
                    }

                    // Search input
                    if show_search() {
                        div {
                            class: "mb-4 animate-in fade-in",
                            div {
                                class: "join w-full",
                                input {
                                    class: "input input-bordered join-item w-full",
                                    placeholder: "Search notes...",
                                    value: "{search_query}",
                                    oninput: move |e| search_query.set(e.value().clone()),
                                    onfocus: move |_| show_search_history.set(true),
                                }
                                button {
                                    class: "btn join-item",
                                    onclick: move |_| handle_search(()),
                                    disabled: search_query().is_empty(),
                                    "Search"
                                }
                                button {
                                    class: "btn join-item",
                                    onclick: move |_| search_query.set(String::new()),
                                    disabled: search_query().is_empty(),
                                    "Clear"
                                }
                            }

                            // Search history dropdown
                            if show_search_history() && (!recent_searches().is_empty() || !saved_searches().is_empty()) {
                                div {
                                    class: "relative",
                                    div {
                                        class: "absolute z-10 mt-1 w-full bg-base-200 shadow-lg rounded-md p-3",
                                        SearchHistory {
                                            recent_searches: recent_searches(),
                                            saved_searches: saved_searches(),
                                            on_select: handle_search_history_select,
                                            on_save: Some(EventHandler::new(handle_save_search)),
                                            on_delete: Some(EventHandler::new(handle_delete_search)),
                                            on_clear_all: Some(EventHandler::new(handle_clear_all_searches)),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Tag filter
                    if show_tag_filter() {
                        div {
                            class: "mb-4 animate-in fade-in",
                            TagInput {
                                tags: selected_tags(),
                                suggestions: all_tags(),
                                on_tags_change: handle_tag_selection,
                                placeholder: "Filter by tags...".to_string(),
                            }

                            // Tag statistics
                            if !tag_stats_data.is_empty() {
                                div {
                                    class: "mt-2 flex flex-wrap gap-1",
                                    {tag_stats_data.iter().map(|(tag, count)| {
                                        let is_selected = selected_tags().contains(tag);
                                        let tag_clone = tag.clone();
                                        let badge_class = if is_selected { "badge-accent" } else { "badge-outline" };
                                        rsx! {
                                            div {
                                                key: "{tag}",
                                                class: "badge badge-sm {badge_class}",
                                                onclick: move |_| {
                                                    let mut new_tags = selected_tags();
                                                    if is_selected {
                                                        new_tags.retain(|t| t != &tag_clone);
                                                    } else {
                                                        new_tags.push(tag_clone.clone());
                                                    }
                                                    selected_tags.set(new_tags);
                                                },
                                                "#{tag} ({count})"
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }

                    // Notes list
                    div {
                        class: "space-y-4",
                        if filtered_notes_data.is_empty() {
                            div {
                                class: "text-center py-8 text-base-content/60",
                                if !search_query().is_empty() || !selected_tags().is_empty() {
                                    "No notes match your search criteria"
                                } else {
                                    "No notes yet. Create your first note below."
                                }
                            }
                        } else {
                            {filtered_notes_data.iter().map(|note| {
                                let note_clone = note.clone();
                                rsx! {
                                    NoteCard {
                                        key: "{note.id}",
                                        content: note.content.clone(),
                                        timestamp: note.timestamp,
                                        tags: note.tags.clone(),
                                        created_at: note.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                        updated_at: note.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                                        on_edit: move |_| handle_edit_note(note_clone.clone()),
                                        search_highlight: search_query().clone(),
                                    }
                                }
                            })}
                        }
                    }

                    // Markdown editor
                    div {
                        class: "mt-6",
                        h3 {
                            class: "text-base font-semibold mb-2",
                            if editing_note_id().is_some() {
                                "Edit Note"
                            } else {
                                "Add New Note"
                            }
                        }
                        textarea {
                            class: "textarea textarea-bordered w-full min-h-[100px] mb-2",
                            placeholder: "Write your note in markdown...",
                            value: "{note_content}",
                            oninput: move |e| note_content.set(e.value().clone()),
                        }

                        // Tag input for note
                        div {
                            class: "mb-2",
                            TagInput {
                                tags: editing_note_tags(),
                                suggestions: all_tags(),
                                on_tags_change: handle_editing_tags_change,
                                placeholder: "Add tags to your note...".to_string(),
                            }
                        }

                        div {
                            class: "flex gap-2",
                            button {
                                class: "btn btn-accent btn-sm flex items-center gap-1",
                                onclick: handle_save_note,
                                Icon { icon: FaFloppyDisk, class: "w-5 h-5" },
                                if editing_note_id().is_some() {
                                    "Update"
                                } else {
                                    "Save"
                                }
                            }
                            if editing_note_id().is_some() {
                                button {
                                    class: "btn btn-outline btn-sm flex items-center gap-1",
                                    onclick: handle_cancel_edit,
                                    "Cancel"
                                }
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
    #[props(default)]
    search_highlight: String,
    #[props(default)]
    on_edit: EventHandler<()>,
}

#[component]
fn NoteCard(props: NoteCardProps) -> Element {
    let ts = props
        .timestamp
        .map(|t| format!(" at {t}s"))
        .unwrap_or_default();

    let note_for_render = crate::types::Note {
        content: props.content.to_string(),
        // The rest are dummy fields, only content is used for rendering
        id: uuid::Uuid::nil(),
        course_id: uuid::Uuid::nil(),
        video_id: None,
        video_index: None,
        timestamp: None,
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Render HTML with search highlighting if needed
    let rendered_html = if props.search_highlight.is_empty() {
        crate::storage::notes::render_note_html(&note_for_render)
    } else {
        let html = crate::storage::notes::render_note_html(&note_for_render);
        highlight_search_term(&html, &props.search_highlight)
    };

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

    // Delete note action
    let _delete_note_action = crate::ui::hooks::use_delete_note_action();

    // Dropdown items for note actions
    let note_actions = vec![
        DropdownItem {
            label: "Export".to_string(),
            icon: Some("📤".to_string()),
            on_select: Some(EventHandler::new(|_| toast::info("Exported note (stub)"))),
            disabled: false,
            divider: false,
        },
        DropdownItem {
            label: "Edit".to_string(),
            icon: Some("✏️".to_string()),
            on_select: Some(EventHandler::new({
                let on_edit = props.on_edit;
                move |_| on_edit.call(())
            })),
            disabled: false,
            divider: false,
        },
        DropdownItem {
            label: "Delete".to_string(),
            icon: Some("🗑️".to_string()),
            on_select: Some(EventHandler::new({
                let mut show_delete_modal = show_delete_modal;
                move |_| show_delete_modal.set(true)
            })),
            disabled: false,
            divider: true,
        },
    ];

    rsx! {
        div {
            class: "card bg-base-200 shadow-sm p-4 relative",
            style: "{card_style}",
            // Note actions dropdown
            div {
                class: "absolute top-2 right-2 z-10",
                UnifiedDropdown {
                    items: note_actions,
                    trigger: DropdownTrigger::DotsMenu,
                    position: "dropdown-end".to_string(),
                }
            }
            div {
                class: "flex items-center gap-2 mb-1",
                span { class: "text-xs text-base-content/60", "{props.created_at}" }
                if !ts.is_empty() {
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
            // Delete confirmation modal using unified Modal
            Modal {
                variant: confirmation_modal(
                    "Are you sure you want to delete this note? This action cannot be undone.",
                    "Delete",
                    "Cancel", 
                    "error",
                    Some(Callback::new(move |_| {
                        show_delete_modal.set(false);
                        // We would need the actual note ID here to delete it
                        // For now, just show a success message
                        toast::success("Note deleted");
                    })),
                    Some(Callback::new(move |_| show_delete_modal.set(false)))
                ),
                open: show_delete_modal(),
                title: Some("Delete Note".to_string()),
                on_close: Some(Callback::new(move |_| show_delete_modal.set(false))),
            }
        }
    }
}

/// Helper function to highlight search terms in HTML content
fn highlight_search_term(html: &str, search_term: &str) -> String {
    if search_term.is_empty() {
        return html.to_string();
    }

    // In a real implementation, you would want to use a proper HTML parser
    // to avoid breaking HTML tags, but this is a simple demonstration
    let search_term_lower = search_term.to_lowercase();
    let mut result = html.to_string();

    // Find all occurrences of the search term (case-insensitive)
    let mut positions = Vec::new();
    let html_lower = html.to_lowercase();
    let mut start = 0;

    while let Some(pos) = html_lower[start..].find(&search_term_lower) {
        let absolute_pos = start + pos;
        positions.push(absolute_pos);
        start = absolute_pos + search_term_lower.len();
    }

    // Replace from end to beginning to avoid position shifts
    for pos in positions.iter().rev() {
        let end_pos = *pos + search_term_lower.len();
        let original_term = &html[*pos..end_pos];
        let highlighted =
            format!("<mark class=\"bg-accent/30 text-accent-content\">{original_term}</mark>");
        result.replace_range(*pos..end_pos, &highlighted);
    }

    result
}

/// AllNotesTab: Display all notes across all courses
#[component]
fn AllNotesTab() -> Element {
    let notes_resource = crate::ui::hooks::use_all_notes_resource();
    let mut search_query = use_signal(String::new);
    let mut show_search = use_signal(|| false);

    // Extract notes early to avoid borrowing issues
    let notes_result = match &*notes_resource.read_unchecked() {
        None => {
            return rsx! {
                div {
                    class: "p-4",
                    div { class: "text-center", "Loading all notes..." }
                    div { class: "text-xs text-base-content/60 mt-2", "Debug: Resource is None (still loading)" }
                }
            };
        }
        Some(Err(err)) => {
            return rsx! {
                div {
                    class: "p-4",
                    div { class: "text-error text-center", "Error loading notes: {err}" }
                    div { class: "text-xs text-base-content/60 mt-2", "Debug: Resource returned error" }
                }
            };
        }
        Some(Ok(notes)) => {
            log::info!("Debug: Loaded {} notes from database", notes.len());
            notes.clone()
        }
    };

    // Filter notes based on search query
    let filtered_notes = use_memo(move || {
        let query = search_query.read().to_lowercase();
        if query.is_empty() {
            notes_result.clone()
        } else {
            notes_result
                .iter()
                .filter(|note| {
                    note.content.to_lowercase().contains(&query)
                        || note
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))
                })
                .cloned()
                .collect()
        }
    });

    // In a future implementation, this could navigate to the specific course or open an edit modal
    let handle_edit_note = move |note: crate::types::Note| {
        crate::ui::components::toast::toast::info(&format!(
            "Note editing from All Notes view will be implemented in a future update. Note: '{}'",
            if note.content.len() > 50 {
                format!("{}...", &note.content[..50])
            } else {
                note.content
            }
        ));
    };

    rsx! {
        div {
            class: "flex flex-col h-full w-full p-4",

            // Search bar
            div {
                class: "mb-4",
                div {
                    class: "flex items-center gap-2",
                    button {
                        class: "btn btn-ghost btn-square btn-sm",
                        onclick: move |_| show_search.set(!show_search()),
                        Icon { icon: FaMagnifyingGlass, class: "w-4 h-4" }
                    }
                    h2 { class: "text-lg font-semibold flex-1", "All Notes ({filtered_notes().len()})" }
                }

                if show_search() {
                    div {
                        class: "mt-2",
                        input {
                            r#type: "text",
                            placeholder: "Search notes...",
                            class: "input input-bordered w-full input-sm",
                            value: search_query(),
                            oninput: move |evt| search_query.set(evt.value()),
                        }
                    }
                }
            }

            // Notes list
            div {
                class: "flex-1 overflow-y-auto space-y-3",
                if filtered_notes().is_empty() {
                    div {
                        class: "text-center text-base-content/60 p-8",
                        if search_query().is_empty() {
                            "No notes found. Create some notes to get started!"
                        } else {
                            "No notes match your search."
                        }
                    }
                } else {
                    {filtered_notes().into_iter().map(|note| {
                        let created_at = note.created_at.format("%Y-%m-%d %H:%M").to_string();
                        let updated_at = note.updated_at.format("%Y-%m-%d %H:%M").to_string();
                        let handle_edit_note = handle_edit_note.clone();
                        let note_for_edit = note.clone();

                        rsx! {
                            NoteCard {
                                key: "{note.id}",
                                content: note.content.clone(),
                                timestamp: note.timestamp,
                                tags: note.tags.clone(),
                                created_at: created_at,
                                updated_at: updated_at,
                                search_highlight: search_query(),
                                on_edit: move |_| handle_edit_note(note_for_edit.clone()),
                            }
                        }
                    })}
                }
            }
        }
    }
}
