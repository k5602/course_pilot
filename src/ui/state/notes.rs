//! Notes State Management
//!
//! Focused state management for notes using modern Dioxus signals

use dioxus::prelude::*;
use uuid::Uuid;
use crate::types::Note;

/// Notes state context
#[derive(Clone, Copy)]
pub struct NotesState {
    pub notes: Signal<Vec<Note>>,
}

/// Provide notes context
pub fn provide_notes_context(initial_notes: Vec<Note>) {
    use_context_provider(|| NotesState {
        notes: Signal::new(initial_notes),
    });
}

/// Hook to get notes state
pub fn use_notes_state() -> NotesState {
    use_context::<NotesState>()
}

/// Hook for reactive access to all notes
pub fn use_notes() -> ReadOnlySignal<Vec<Note>> {
    let state = use_notes_state();
    state.notes.into()
}

/// Hook for reactive access to notes for a specific course
pub fn use_course_notes(course_id: Uuid) -> Memo<Vec<Note>> {
    let notes = use_notes();
    use_memo(move || {
        notes.read()
            .iter()
            .filter(|note| note.course_id == course_id)
            .cloned()
            .collect()
    })
}

/// Hook for tag statistics
pub fn use_tag_statistics() -> Memo<std::collections::HashMap<String, usize>> {
    let notes = use_notes();
    use_memo(move || {
        let mut stats = std::collections::HashMap::new();
        for note in notes.read().iter() {
            for tag in &note.tags {
                *stats.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        stats
    })
}

/// Actions for notes
pub mod actions {
    use super::*;
    
    /// Add a note
    pub fn add_note(note: Note) {
        let state = use_notes_state();
        state.notes.write().push(note);
        log::info!("Note added successfully");
    }
    
    /// Update a note
    pub fn update_note(id: Uuid, updated_note: Note) -> Result<(), String> {
        let state = use_notes_state();
        let mut notes = state.notes.write();
        
        if let Some(index) = notes.iter().position(|n| n.id == id) {
            notes[index] = updated_note;
            log::info!("Note {id} updated successfully");
            Ok(())
        } else {
            Err(format!("Note {id} not found"))
        }
    }
    
    /// Delete a note
    pub fn delete_note(id: Uuid) -> Result<(), String> {
        let state = use_notes_state();
        let mut notes = state.notes.write();
        let initial_len = notes.len();
        
        notes.retain(|n| n.id != id);
        
        if notes.len() == initial_len {
            Err(format!("Note {id} not found"))
        } else {
            log::info!("Note {id} deleted successfully");
            Ok(())
        }
    }
}