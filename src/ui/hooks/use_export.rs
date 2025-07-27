use crate::export::{ExportFormat, ExportResult};
use dioxus::prelude::*;
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;

/// Export operations hook
#[derive(Clone)]
pub struct ExportManager {
    db: Arc<crate::storage::database::Database>,
    pub export_course_with_progress: Callback<(Uuid, crate::export::ExportFormat, Box<dyn Fn(f32, String) + Send + Sync>)>,
    pub save_export_data: Callback<crate::export::ExportResult>,
}

impl ExportManager {
    pub async fn export_course(&self, course_id: Uuid, format: ExportFormat) -> Result<ExportResult> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;

            // Load course data
            let course = crate::storage::get_course_by_id(&db, &course_id)?
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
            course
                .export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Course export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn export_plan(&self, plan_id: Uuid, format: ExportFormat) -> Result<ExportResult> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;

            // Load plan data
            let plan = crate::storage::load_plan(&db, &plan_id)?
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

    pub async fn export_notes(&self, course_id: Uuid, format: ExportFormat) -> Result<ExportResult> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;

            // Load notes data
            let conn = db.get_conn()?;
            let notes = crate::storage::notes::get_notes_by_course(&conn, course_id)?;

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
            notes
                .export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Notes export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn export_course_with_progress<F>(
        &self,
        course_id: Uuid,
        format: ExportFormat,
        progress_callback: Box<F>,
    ) -> Result<ExportResult>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            use crate::export::Exportable;

            // Load course data
            let course = crate::storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            // Create export options with progress callback
            let options = crate::export::ExportOptions {
                format,
                include_metadata: true,
                include_progress: true,
                include_timestamps: true,
                progress_callback: Some(progress_callback),
            };

            // Export with progress tracking
            course
                .export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Course export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn save_export_data(&self, export_result: ExportResult) -> Result<std::path::PathBuf> {
        // This is a placeholder for saving export data to a file
        // The actual implementation would depend on the export format and user preferences
        let file_path = std::path::PathBuf::from(&export_result.filename);
        
        // Write the data to the file
        tokio::fs::write(&file_path, &export_result.data).await?;
        
        Ok(file_path)
    }
}

pub fn use_export_manager() -> ExportManager {
    let db = use_context::<Arc<crate::storage::database::Database>>();
    
    let export_course_with_progress = use_callback({
        let db = db.clone();
        move |(course_id, format, progress_callback): (Uuid, crate::export::ExportFormat, Box<dyn Fn(f32, String) + Send + Sync>)| {
            let db = db.clone();
            spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    use crate::export::Exportable;

                    // Load course data
                    let course = crate::storage::get_course_by_id(&db, &course_id)?
                        .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

                    // Create export options with progress callback
                    let options = crate::export::ExportOptions {
                        format,
                        include_metadata: true,
                        include_progress: true,
                        include_timestamps: true,
                        progress_callback: Some(progress_callback),
                    };

                    // Export with progress tracking
                    course
                        .export_with_options(options)
                        .map_err(|e| anyhow::anyhow!("Course export failed: {}", e))
                }).await;

                match result {
                    Ok(Ok(_)) => {
                        crate::ui::components::toast::toast::success("Course exported successfully");
                    }
                    Ok(Err(e)) => {
                        crate::ui::components::toast::toast::error(format!("Failed to export course: {}", e));
                    }
                    Err(e) => {
                        crate::ui::components::toast::toast::error(format!("Failed to export course: {}", e));
                    }
                }
            });
            // Return () to match expected callback type
        }
    });

    let save_export_data = use_callback(move |export_result: crate::export::ExportResult| {
        spawn(async move {
            let file_path = std::path::PathBuf::from(&export_result.filename);
            
            let result = tokio::fs::write(&file_path, &export_result.data).await;
            
            match result {
                Ok(_) => {
                    crate::ui::components::toast::toast::success(format!("Export saved to: {}", file_path.display()));
                }
                Err(e) => {
                    crate::ui::components::toast::toast::error(format!("Failed to save export: {}", e));
                }
            }
        });
        // Return () to match expected callback type
    });
    
    ExportManager { db, export_course_with_progress, save_export_data }
}

// Placeholder action hooks - these will be implemented later if needed
pub fn use_export_course_action() -> () {
    // Placeholder
}

pub fn use_export_plan_action() -> () {
    // Placeholder
}

pub fn use_export_notes_action() -> () {
    // Placeholder
}