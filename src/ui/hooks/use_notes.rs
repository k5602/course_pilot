use crate::storage::database::Database;
use crate::types::Note;
use dioxus::prelude::*;
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;

/// Notes management hook
#[derive(Clone)]
pub struct NotesManager {
    db: Arc<Database>,
}

impl NotesManager {
    pub async fn list_all_notes(&self) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            crate::storage::notes::get_all_notes(&conn)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn list_notes_by_course(&self, course_id: Uuid) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            crate::storage::notes::get_notes_by_course(&conn, course_id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn list_notes_by_course_and_video_index(
        &self,
        course_id: Uuid,
        video_index: Option<usize>,
    ) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            if let Some(video_index) = video_index {
                crate::storage::notes::get_notes_by_video_index(&conn, course_id, video_index)
            } else {
                crate::storage::notes::get_notes_by_course(&conn, course_id)
            }
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn list_notes_by_video(&self, video_id: Uuid) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            crate::storage::notes::get_notes_by_video(&conn, video_id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn list_notes_by_video_index(
        &self,
        course_id: Uuid,
        video_index: usize,
    ) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            crate::storage::notes::get_notes_by_video_index(&conn, course_id, video_index)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let db = self.db.clone();
        let query = query.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            crate::storage::notes::search_notes(&conn, &query)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn search_notes_by_tags(&self, tags: &[String]) -> Result<Vec<Note>> {
        let db = self.db.clone();
        let tags_clone = tags.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            let tag_strs: Vec<&str> = tags_clone.iter().map(|s| s.as_str()).collect();
            let filters = crate::storage::notes::NoteSearchFilters {
                course_id: None,
                video_id: None,
                content: None,
                tags: Some(&tag_strs),
                timestamp_min: None,
                timestamp_max: None,
                created_after: None,
                created_before: None,
                updated_after: None,
                updated_before: None,
            };
            crate::storage::notes::search_notes_advanced(&conn, filters)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn get_note(&self, note_id: Uuid) -> Result<Option<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            crate::storage::notes::get_note_by_id(&conn, note_id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn save_note(&self, note: Note) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            // If note exists, update; else, create
            let exists = crate::storage::notes::get_note_by_id(&conn, note.id)?.is_some();
            if exists {
                crate::storage::notes::update_note(&conn, &note)
            } else {
                crate::storage::notes::create_note(&conn, &note)
            }
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn delete_note(&self, note_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            crate::storage::notes::delete_note(&conn, note_id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
}

pub fn use_notes_manager() -> NotesManager {
    let db = use_context::<Arc<Database>>();
    
    NotesManager { db }
}

/// Hook for loading notes with video index
pub fn use_notes_with_video_index_resource(
    course_id: Uuid,
    video_index: Option<usize>,
) -> Resource<Result<Vec<Note>, anyhow::Error>> {
    let notes_manager = use_notes_manager();

    use_resource(move || {
        let notes_manager = notes_manager.clone();
        async move {
            notes_manager.list_notes_by_course_and_video_index(course_id, video_index).await
        }
    })
}

/// Hook for loading all notes
pub fn use_all_notes_resource() -> Resource<Result<Vec<Note>, anyhow::Error>> {
    let notes_manager = use_notes_manager();

    use_resource(move || {
        let notes_manager = notes_manager.clone();
        async move {
            notes_manager.list_all_notes().await
        }
    })
}

/// Hook for saving notes
pub fn use_save_note_action() -> impl Fn(Note) {
    let notes_manager = use_notes_manager();

    move |note: Note| {
        let notes_manager = notes_manager.clone();
        spawn(async move {
            match notes_manager.save_note(note).await {
                Ok(_) => {
                    // Note saved successfully
                }
                Err(e) => {
                    eprintln!("Failed to save note: {}", e);
                }
            }
        });
    }
}

/// Hook for deleting notes
pub fn use_delete_note_action() -> impl Fn(Uuid) {
    let notes_manager = use_notes_manager();

    move |note_id: Uuid| {
        let notes_manager = notes_manager.clone();
        spawn(async move {
            match notes_manager.delete_note(note_id).await {
                Ok(_) => {
                    // Note deleted successfully
                }
                Err(e) => {
                    eprintln!("Failed to delete note: {}", e);
                }
            }
        });
    }
}