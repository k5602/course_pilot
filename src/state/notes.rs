//! Notes management state for Course Pilot
//!
//! This module handles reactive state for notes operations including
//! creation, updates, deletion, and video-specific note management.

use crate::types::Note;
use dioxus::prelude::*;
use uuid::Uuid;

use super::{StateError, StateResult};

/// Notes management context
#[derive(Clone, Copy)]
pub struct NotesContext {
    pub notes: Signal<Vec<Note>>,
}

impl Default for NotesContext {
    fn default() -> Self {
        Self::new()
    }
}

impl NotesContext {
    pub fn new() -> Self {
        Self {
            notes: Signal::new(Vec::new()),
        }
    }
}

/// Notes context provider component
#[component]
pub fn NotesContextProvider(children: Element) -> Element {
    use_context_provider(|| NotesContext::new());
    rsx! { {children} }
}

/// Hook to access notes reactively
pub fn use_notes_reactive() -> Signal<Vec<Note>> {
    use_context::<NotesContext>().notes
}

/// Hook to get notes for a specific course
pub fn use_course_notes_reactive(course_id: Uuid) -> Signal<Vec<Note>> {
    let notes = use_notes_reactive();
    Signal::new(
        notes
            .read()
            .iter()
            .filter(|n| n.course_id == course_id)
            .cloned()
            .collect(),
    )
}

/// Hook to get notes for a specific video
pub fn use_video_notes_reactive(course_id: Uuid, video_index: usize) -> Signal<Vec<Note>> {
    let notes = use_notes_reactive();
    Signal::new(
        notes
            .read()
            .iter()
            .filter(|n| n.course_id == course_id && n.video_index == Some(video_index))
            .cloned()
            .collect(),
    )
}

/// Hook to get course-level notes (not associated with specific videos)
pub fn use_course_level_notes_reactive(course_id: Uuid) -> Signal<Vec<Note>> {
    let notes = use_notes_reactive();
    Signal::new(
        notes
            .read()
            .iter()
            .filter(|n| n.course_id == course_id && n.video_index.is_none())
            .cloned()
            .collect(),
    )
}

/// Add a new note to the reactive state
pub fn add_note_reactive(note: Note) {
    let mut notes = use_notes_reactive();
    let mut notes_vec = notes.read().clone();
    notes_vec.push(note);
    notes.set(notes_vec);
}

/// Update an existing note in the reactive state
pub fn update_note_reactive(note_id: Uuid, updated_note: Note) -> StateResult<()> {
    let mut notes = use_notes_reactive();
    let mut notes_vec = notes.read().clone();

    if let Some(index) = notes_vec.iter().position(|n| n.id == note_id) {
        let mut final_note = updated_note;
        final_note.id = note_id; // Preserve original ID
        final_note.updated_at = chrono::Utc::now();

        notes_vec[index] = final_note;
        notes.set(notes_vec);
        Ok(())
    } else {
        Err(StateError::InvalidOperation(format!(
            "Note not found: {}",
            note_id
        )))
    }
}

/// Delete a note from the reactive state
pub fn delete_note_reactive(note_id: Uuid) -> StateResult<()> {
    let mut notes = use_notes_reactive();
    let mut notes_vec = notes.read().clone();

    if let Some(index) = notes_vec.iter().position(|n| n.id == note_id) {
        notes_vec.remove(index);
        notes.set(notes_vec);
        Ok(())
    } else {
        Err(StateError::InvalidOperation(format!(
            "Note not found: {}",
            note_id
        )))
    }
}

/// Delete all notes for a specific course
pub fn delete_course_notes_reactive(course_id: Uuid) -> usize {
    let mut notes = use_notes_reactive();
    let mut notes_vec = notes.read().clone();
    let initial_count = notes_vec.len();

    notes_vec.retain(|n| n.course_id != course_id);
    let deleted_count = initial_count - notes_vec.len();

    notes.set(notes_vec);
    deleted_count
}

/// Delete all notes for a specific video
pub fn delete_video_notes_reactive(course_id: Uuid, video_index: usize) -> usize {
    let mut notes = use_notes_reactive();
    let mut notes_vec = notes.read().clone();
    let initial_count = notes_vec.len();

    notes_vec.retain(|n| !(n.course_id == course_id && n.video_index == Some(video_index)));
    let deleted_count = initial_count - notes_vec.len();

    notes.set(notes_vec);
    deleted_count
}

/// Get notes statistics
pub fn get_notes_stats_reactive() -> (usize, usize, usize) {
    let notes = use_notes_reactive();
    let notes_vec = notes.read();

    let total_notes = notes_vec.len();
    let video_notes = notes_vec.iter().filter(|n| n.video_index.is_some()).count();
    let course_notes = total_notes - video_notes;

    (total_notes, course_notes, video_notes)
}

/// Search notes by content
pub fn search_notes_reactive(query: &str) -> Vec<Note> {
    let notes = use_notes_reactive();
    let notes_vec = notes.read();
    let query_lower = query.to_lowercase();

    notes_vec
        .iter()
        .filter(|note| {
            note.content.to_lowercase().contains(&query_lower)
                || note
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect()
}

/// Get notes by tag
pub fn get_notes_by_tag_reactive(tag: &str) -> Vec<Note> {
    let notes = use_notes_reactive();
    let notes_vec = notes.read();

    notes_vec
        .iter()
        .filter(|note| note.tags.contains(&tag.to_string()))
        .cloned()
        .collect()
}

/// Get all unique tags from notes
pub fn get_all_tags_reactive() -> Vec<String> {
    let notes = use_notes_reactive();
    let notes_vec = notes.read();

    let mut all_tags: Vec<String> = notes_vec
        .iter()
        .flat_map(|note| note.tags.clone())
        .collect();

    all_tags.sort();
    all_tags.dedup();
    all_tags
}

/// Get tag statistics (tag -> count)
pub fn get_tag_statistics_reactive() -> Vec<(String, usize)> {
    let notes = use_notes_reactive();
    let notes_vec = notes.read();

    let mut tag_counts = std::collections::HashMap::new();

    for note in notes_vec.iter() {
        for tag in &note.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut stats: Vec<(String, usize)> = tag_counts.into_iter().collect();
    stats.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
    stats
}

/// Check if a note exists by ID
pub fn note_exists_reactive(note_id: Uuid) -> bool {
    let notes = use_notes_reactive();
    notes.read().iter().any(|n| n.id == note_id)
}

/// Get a note by ID
pub fn get_note_reactive(note_id: Uuid) -> Option<Note> {
    let notes = use_notes_reactive();
    notes.read().iter().find(|n| n.id == note_id).cloned()
}

/// Get all notes
pub fn get_notes_reactive() -> Vec<Note> {
    let notes = use_notes_reactive();
    notes.read().clone()
}

/// Legacy hook functions for compatibility
pub fn use_notes() -> Signal<Vec<Note>> {
    use_notes_reactive()
}

/// Non-reactive note operations for backend integration
pub fn add_note(note: Note) {
    add_note_reactive(note);
}

pub fn update_note(note_id: Uuid, updated_note: Note) -> StateResult<()> {
    update_note_reactive(note_id, updated_note)
}

pub fn delete_note(note_id: Uuid) -> StateResult<()> {
    delete_note_reactive(note_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_notes_context_creation() {
        let context = NotesContext::new();
        assert!(context.notes.read().is_empty());
    }

    #[test]
    fn test_note_statistics() {
        // This would need a proper Dioxus context for full testing
        // For now, we test the logic with mock data
        let notes = vec![
            Note {
                id: Uuid::new_v4(),
                course_id: Uuid::new_v4(),
                video_index: Some(1),
                video_id: None,
                timestamp: None,
                content: "Content 1".to_string(),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Note {
                id: Uuid::new_v4(),
                course_id: Uuid::new_v4(),
                video_index: None,
                video_id: None,
                timestamp: None,
                content: "Content 2".to_string(),
                tags: vec!["tag1".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let video_notes = notes.iter().filter(|n| n.video_index.is_some()).count();
        let course_notes = notes.len() - video_notes;

        assert_eq!(video_notes, 1);
        assert_eq!(course_notes, 1);
    }

    #[test]
    fn test_tag_collection() {
        let notes = vec![
            Note {
                id: Uuid::new_v4(),
                course_id: Uuid::new_v4(),
                video_index: Some(1),
                video_id: None,
                timestamp: None,
                content: "Content 1".to_string(),
                tags: vec!["rust".to_string(), "programming".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Note {
                id: Uuid::new_v4(),
                course_id: Uuid::new_v4(),
                video_index: None,
                video_id: None,
                timestamp: None,
                content: "Content 2".to_string(),
                tags: vec!["rust".to_string(), "tutorial".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let mut all_tags: Vec<String> = notes.iter().flat_map(|note| note.tags.clone()).collect();
        all_tags.sort();
        all_tags.dedup();

        assert_eq!(all_tags.len(), 3);
        assert!(all_tags.contains(&"rust".to_string()));
        assert!(all_tags.contains(&"programming".to_string()));
        assert!(all_tags.contains(&"tutorial".to_string()));
    }
}
