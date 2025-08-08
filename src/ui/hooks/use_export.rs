use crate::export::{ExportFormat, ExportResult};
use crate::ui::toast_helpers;
use anyhow::Result;
use dioxus::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

/// Export operations hook
#[derive(Clone)]
pub struct ExportManager {
    db: Arc<crate::storage::database::Database>,
    pub export_course_with_progress: Callback<(
        Uuid,
        crate::export::ExportFormat,
        Box<dyn Fn(f32, String) + Send + Sync>,
    )>,
    pub save_export_data: Callback<crate::export::ExportResult>,
}

impl ExportManager {
    pub async fn export_course(
        &self,
        course_id: Uuid,
        format: ExportFormat,
    ) -> Result<ExportResult> {
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

    pub async fn export_notes(
        &self,
        course_id: Uuid,
        format: ExportFormat,
    ) -> Result<ExportResult> {
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
                include_progress: false,
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

    pub async fn save_export_data(
        &self,
        export_result: ExportResult,
    ) -> Result<std::path::PathBuf> {
        let file_path = crate::export::io::default_output_path(&export_result.filename);
        let saved = crate::export::io::save_bytes_atomic(&file_path, &export_result.data).await?;
        Ok(saved)
    }
}

pub fn use_export_manager() -> ExportManager {
    let db = use_context::<Arc<crate::storage::database::Database>>();

    let export_course_with_progress = use_callback({
        let db = db.clone();
        move |(course_id, format, progress_callback): (
            Uuid,
            crate::export::ExportFormat,
            Box<dyn Fn(f32, String) + Send + Sync>,
        )| {
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
                })
                .await;

                match result {
                    Ok(Ok(_)) => {
                        toast_helpers::success("Course exported successfully");
                    }
                    Ok(Err(e)) => {
                        toast_helpers::error(format!("Failed to export course: {e}"));
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to export course: {e}"));
                    }
                }
            });
            // Return () to match expected callback type
        }
    });

    let save_export_data = use_callback(move |export_result: crate::export::ExportResult| {
        spawn(async move {
            let file_path = crate::export::io::default_output_path(&export_result.filename);
            let result = crate::export::io::save_bytes_atomic(&file_path, &export_result.data).await;

            match result {
                Ok(saved_path) => {
                    toast_helpers::success(format!("Export saved to: {}", saved_path.display()));
                }
                Err(e) => {
                    toast_helpers::error(format!("Failed to save export: {e}"));
                }
            }
        });
        // Return () to match expected callback type
    });

    ExportManager {
        db,
        export_course_with_progress,
        save_export_data,
    }
}

// Placeholder action hooks - these will be implemented later if needed
pub fn use_export_course_action() {
    // Placeholder
}

pub fn use_export_plan_action() {
    // Placeholder
}

pub fn use_export_notes_action() {
    // Placeholder
}
