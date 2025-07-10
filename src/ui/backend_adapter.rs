use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;
use dioxus::prelude::*;
use course_pilot::types::{Course, Plan, Note};
use course_pilot::storage::database::Database;

/// Async backend API trait for CRUD/search/export operations.
/// All methods are async and return Results for robust error handling.
use async_trait::async_trait;

/// Concrete backend implementation using the pooled Database.
pub struct Backend {
    pub db: Arc<Database>,
}

impl Backend {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl Backend {
    // --- Courses ---
    pub async fn list_courses(&self) -> Result<Vec<Course>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            storage::load_courses(&db)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn get_course(&self, id: Uuid) -> Result<Option<Course>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            storage::get_course_by_id(&db, &id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn create_course(&self, course: Course) -> Result<()> {
        unimplemented!()
    }
    pub async fn update_course(&self, course: Course) -> Result<()> {
        unimplemented!()
    }
    pub async fn delete_course(&self, id: Uuid) -> Result<()> {
        unimplemented!()
    }

    // --- Plans ---
    pub async fn get_plan_by_course(&self, course_id: Uuid) -> Result<Option<Plan>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::get_plan_by_course_id(&db, &course_id))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn save_plan(&self, plan: Plan) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::save_plan(&db, &plan))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn delete_plan(&self, plan_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::delete_plan(&db, &plan_id))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    // --- Notes ---
    pub async fn list_notes_by_course(&self, course_id: Uuid) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::get_notes_by_course(&conn, course_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn list_notes_by_video(&self, video_id: Uuid) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::get_notes_by_video(&conn, video_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn get_note(&self, note_id: Uuid) -> Result<Option<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::get_note_by_id(&conn, note_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn save_note(&self, note: Note) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            // If note exists, update; else, create
            let exists = notes::get_note_by_id(&conn, note.id)?.is_some();
            if exists {
                notes::update_note(&conn, &note)
            } else {
                notes::create_note(&conn, &note)
            }.map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn delete_note(&self, note_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::delete_note(&conn, note_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    // --- Export ---
    pub async fn export_course(&self, _course_id: Uuid) -> Result<Vec<u8>> {
        // Placeholder: implement actual export logic as needed
        Err(anyhow::anyhow!("Not implemented"))
    }
    pub async fn export_plan(&self, _plan_id: Uuid) -> Result<Vec<u8>> {
        // Placeholder: implement actual export logic as needed
        Err(anyhow::anyhow!("Not implemented"))
    }
    pub async fn export_notes(&self, course_id: Uuid) -> Result<Vec<u8>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            let md = notes::export_notes_markdown_by_course(&conn, course_id)?;
            Ok(md.into_bytes())
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
}

/// Dioxus hooks for async backend actions.
/// These hooks wrap the BackendApi trait and provide ergonomic, reactive access for UI components.

pub fn use_backend_adapter() -> std::sync::Arc<Backend> {
    use_context::<std::sync::Arc<Backend>>()
}

/// Example: use_async_courses returns a signal with the list of courses and loading/error state.
pub fn use_async_courses() -> UseFuture {
    let backend = use_backend_adapter();
    use_future(move || async move {
        backend.list_courses().await.unwrap_or_default()
    })
}

// Additional hooks for plans, notes, and exports can be added following this pattern.
