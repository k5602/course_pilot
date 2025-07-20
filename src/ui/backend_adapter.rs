use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;
use dioxus::prelude::*;
use crate::types::{Course, Plan, Note};
use crate::storage::{self, database::Database, notes};

/// Progress information for plans and courses
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub completed_count: usize,
    pub total_count: usize,
    pub percentage: f32,
    pub estimated_time_remaining: Option<std::time::Duration>,
}

/// Async backend API trait for CRUD/search/export operations.
/// All methods are async and return Results for robust error handling.

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
            storage::load_courses(&db).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn get_course(&self, id: Uuid) -> Result<Option<Course>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            storage::get_course_by_id(&db, &id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn create_course(&self, course: Course) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            storage::save_course(&db, &course).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
    
    pub async fn update_course(&self, course: Course) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Verify course exists first
            let existing = storage::get_course_by_id(&db, &course.id)?;
            if existing.is_none() {
                return Err(anyhow::anyhow!("Course with id {} not found", course.id).into());
            }
            
            // Update the course
            storage::save_course(&db, &course).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
    
    pub async fn delete_course(&self, course_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            storage::delete_course(&db, &course_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    // --- Plans ---
    pub async fn get_plan_by_course(&self, course_id: Uuid) -> Result<Option<Plan>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::get_plan_by_course_id(&db, &course_id).map_err(Into::into))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn save_plan(&self, plan: Plan) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::save_plan(&db, &plan).map_err(Into::into))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn delete_plan(&self, plan_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::delete_plan(&db, &plan_id).map_err(Into::into))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    
    pub async fn update_plan_item_completion(
        &self, 
        plan_id: Uuid, 
        item_index: usize, 
        completed: bool
    ) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load plan
            let mut plan = storage::load_plan(&db, &plan_id)?
                .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;
            
            // Validate item index
            if item_index >= plan.items.len() {
                return Err(anyhow::anyhow!(
                    "Plan item index {} out of bounds (plan has {} items)", 
                    item_index, 
                    plan.items.len()
                ));
            }
            
            // Update item completion status
            plan.items[item_index].completed = completed;
            
            // Save updated plan
            storage::save_plan(&db, &plan).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
    
    pub async fn get_plan_progress(&self, plan_id: Uuid) -> Result<ProgressInfo> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let plan = storage::load_plan(&db, &plan_id)?
                .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;
            
            let total_count = plan.items.len();
            let completed_count = plan.items.iter().filter(|item| item.completed).count();
            let percentage = if total_count > 0 {
                (completed_count as f32 / total_count as f32) * 100.0
            } else {
                0.0
            };
            
            let estimated_time_remaining = if completed_count < total_count {
                let remaining_items = total_count - completed_count;
                let session_duration = std::time::Duration::from_secs(
                    (plan.settings.session_length_minutes as u64) * 60
                );
                Some(session_duration * remaining_items as u32)
            } else {
                None
            };
            
            Ok(ProgressInfo {
                completed_count,
                total_count,
                percentage,
                estimated_time_remaining,
            })
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
    
    pub async fn get_course_progress(&self, course_id: Uuid) -> Result<Option<ProgressInfo>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Get plan for this course
            if let Some(plan) = storage::get_plan_by_course_id(&db, &course_id)? {
                let total_count = plan.items.len();
                let completed_count = plan.items.iter().filter(|item| item.completed).count();
                let percentage = if total_count > 0 {
                    (completed_count as f32 / total_count as f32) * 100.0
                } else {
                    0.0
                };
                
                let estimated_time_remaining = if completed_count < total_count {
                    let remaining_items = total_count - completed_count;
                    let session_duration = std::time::Duration::from_secs(
                        (plan.settings.session_length_minutes as u64) * 60
                    );
                    Some(session_duration * remaining_items as u32)
                } else {
                    None
                };
                
                Ok(Some(ProgressInfo {
                    completed_count,
                    total_count,
                    percentage,
                    estimated_time_remaining,
                }))
            } else {
                Ok(None)
            }
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
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
    pub async fn export_course(&self, course_id: Uuid, format: crate::export::ExportFormat) -> Result<crate::export::ExportResult> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;
            
            // Load course data
            let course = storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;
            
            // Create export options
            let options = crate::export::ExportOptions {
                format,
                include_metadata: true,
                include_progress: true,
                include_timestamps: true,
                progress_callback: None,
            };
            
            // Export with validation and error handling
            course.export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Course export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
    
    pub async fn export_plan(&self, plan_id: Uuid, format: crate::export::ExportFormat) -> Result<crate::export::ExportResult> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;
            
            // Load plan data
            let plan = storage::load_plan(&db, &plan_id)?
                .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;
            
            // Create export options
            let options = crate::export::ExportOptions {
                format,
                include_metadata: true,
                include_progress: true,
                include_timestamps: true,
                progress_callback: None,
            };
            
            // Export with validation and error handling
            plan.export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Plan export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
    
    pub async fn export_notes(&self, course_id: Uuid, format: crate::export::ExportFormat) -> Result<crate::export::ExportResult> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;
            
            // Load notes data
            let conn = db.get_conn()?;
            let notes = notes::get_notes_by_course(&conn, course_id)?;
            
            if notes.is_empty() {
                return Err(anyhow::anyhow!("No notes found for course: {}", course_id));
            }
            
            // Create export options
            let options = crate::export::ExportOptions {
                format,
                include_metadata: true,
                include_progress: false, // Notes don't have progress
                include_timestamps: true,
                progress_callback: None,
            };
            
            // Export with validation and error handling
            notes.export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Notes export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
    
    pub async fn export_course_with_progress<F>(&self, course_id: Uuid, format: crate::export::ExportFormat, progress_callback: F) -> Result<crate::export::ExportResult>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;
            
            // Load course data
            let course = storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;
            
            // Create export options with progress callback
            let options = crate::export::ExportOptions {
                format,
                include_metadata: true,
                include_progress: true,
                include_timestamps: true,
                progress_callback: Some(Box::new(progress_callback)),
            };
            
            // Export with progress tracking
            course.export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Course export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
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
    use_future(move || {
        let backend = backend.clone();
        async move {
            backend.list_courses().await
        }
    })
}

// Additional hooks for plans, notes, and exports can be added following this pattern.
