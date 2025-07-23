use crate::storage::{self, database::Database, notes};
use crate::types::{Course, Note, Plan};
use anyhow::Result;
use dioxus::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Progress information for plans and courses
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub completed_count: usize,
    pub total_count: usize,
    pub percentage: f32,
    pub estimated_time_remaining: Option<std::time::Duration>,
}

/// Folder validation result
#[derive(Debug, Clone, PartialEq)]
pub struct FolderValidation {
    pub is_valid: bool,
    pub video_count: usize,
    pub supported_files: Vec<PathBuf>,
    pub unsupported_files: Vec<PathBuf>,
    pub total_size: u64,
    pub error_message: Option<String>,
}

/// Supported video file extensions
const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ogv", "ts",
    "mts", "m2ts",
];

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
        tokio::task::spawn_blocking(move || storage::load_courses(&db).map_err(Into::into))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn get_course(&self, id: Uuid) -> Result<Option<Course>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::get_course_by_id(&db, &id).map_err(Into::into))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn create_course(&self, course: Course) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || storage::save_course(&db, &course).map_err(Into::into))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn update_course(&self, course: Course) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Verify course exists first
            let existing = storage::get_course_by_id(&db, &course.id)?;
            if existing.is_none() {
                return Err(anyhow::anyhow!("Course with id {} not found", course.id));
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
        tokio::task::spawn_blocking(move || {
            storage::get_plan_by_course_id(&db, &course_id).map_err(Into::into)
        })
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
        completed: bool,
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
                    (plan.settings.session_length_minutes as u64) * 60,
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
                        (plan.settings.session_length_minutes as u64) * 60,
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
            notes::get_notes_by_course(&conn, course_id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn list_notes_by_video(&self, video_id: Uuid) -> Result<Vec<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::get_notes_by_video(&conn, video_id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let db = self.db.clone();
        let query = query.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::search_notes(&conn, &query)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn search_notes_by_tags(&self, tags: &[String]) -> Result<Vec<Note>> {
        let db = self.db.clone();
        let tags_clone = tags.to_vec(); // Clone the tags to move into the closure
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            let tag_strs: Vec<&str> = tags_clone.iter().map(|s| s.as_str()).collect();
            let filters = notes::NoteSearchFilters {
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
            notes::search_notes_advanced(&conn, filters)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn get_note(&self, note_id: Uuid) -> Result<Option<Note>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::get_note_by_id(&conn, note_id)
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
            }
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }
    pub async fn delete_note(&self, note_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get_conn()?;
            notes::delete_note(&conn, note_id)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    // --- Export ---
    pub async fn export_course(
        &self,
        course_id: Uuid,
        format: crate::export::ExportFormat,
    ) -> Result<crate::export::ExportResult> {
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
            course
                .export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Course export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn export_plan(
        &self,
        plan_id: Uuid,
        format: crate::export::ExportFormat,
    ) -> Result<crate::export::ExportResult> {
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

    pub async fn export_notes(
        &self,
        course_id: Uuid,
        format: crate::export::ExportFormat,
    ) -> Result<crate::export::ExportResult> {
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
        format: crate::export::ExportFormat,
        progress_callback: F,
    ) -> Result<crate::export::ExportResult>
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
            course
                .export_with_options(options)
                .map_err(|e| anyhow::anyhow!("Course export failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    // --- Plan Generation ---

    /// Generate a new study plan for a course
    pub async fn generate_plan(
        &self,
        course_id: Uuid,
        settings: crate::types::PlanSettings,
    ) -> Result<crate::types::Plan> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load course data
            let course = storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            // Generate plan using planner module
            let plan = crate::planner::generate_plan(&course, &settings)
                .map_err(|e| anyhow::anyhow!("Plan generation failed: {}", e))?;

            // Save plan to database
            storage::save_plan(&db, &plan)?;

            Ok(plan)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    /// Regenerate an existing plan with new settings while preserving progress
    pub async fn regenerate_plan(
        &self,
        plan_id: Uuid,
        new_settings: crate::types::PlanSettings,
    ) -> Result<crate::types::Plan> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load existing plan
            let existing_plan = storage::load_plan(&db, &plan_id)?
                .ok_or_else(|| anyhow::anyhow!("Plan not found: {}", plan_id))?;

            // Load course data
            let course = storage::get_course_by_id(&db, &existing_plan.course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", existing_plan.course_id))?;

            // Generate new plan with new settings
            let mut new_plan = crate::planner::generate_plan(&course, &new_settings)
                .map_err(|e| anyhow::anyhow!("Plan regeneration failed: {}", e))?;

            // Preserve progress from existing plan
            preserve_plan_progress(&existing_plan, &mut new_plan);

            // Update the plan ID to maintain continuity
            new_plan.id = plan_id;

            // Save updated plan to database
            storage::save_plan(&db, &new_plan)?;

            Ok(new_plan)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    // --- Course Structuring ---

    /// Structure a course using NLP analysis
    pub async fn structure_course(&self, course_id: Uuid) -> Result<Course> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Load course data
            let mut course = storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            // Check if course already has structure
            if course.structure.is_some() {
                return Err(anyhow::anyhow!("Course is already structured"));
            }

            // Use NLP module to structure the course
            let structure = crate::nlp::structure_course(course.raw_titles.clone())
                .map_err(|e| anyhow::anyhow!("Course structuring failed: {}", e))?;

            // Update course with new structure
            course.structure = Some(structure);

            // Save updated course to database
            storage::save_course(&db, &course)?;

            Ok(course)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    /// Re-structure a course with progress callback
    pub async fn structure_course_with_progress<F>(
        &self,
        course_id: Uuid,
        progress_callback: F,
    ) -> Result<Course>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            progress_callback(0.0, "Loading course data...".to_string());

            // Load course data
            let mut course = storage::get_course_by_id(&db, &course_id)?
                .ok_or_else(|| anyhow::anyhow!("Course not found: {}", course_id))?;

            progress_callback(25.0, "Analyzing course content...".to_string());

            // Use NLP module to structure the course
            let structure = crate::nlp::structure_course(course.raw_titles.clone())
                .map_err(|e| anyhow::anyhow!("Course structuring failed: {}", e))?;

            progress_callback(75.0, "Saving structured course...".to_string());

            // Update course with new structure
            course.structure = Some(structure);

            // Save updated course to database
            storage::save_course(&db, &course)?;

            progress_callback(100.0, "Course structuring completed!".to_string());

            Ok(course)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    // --- File System Operations ---

    /// Open native folder browser dialog
    pub async fn browse_folder(&self) -> Result<Option<PathBuf>> {
        tokio::task::spawn_blocking(move || {
            use rfd::AsyncFileDialog;

            // Use async file dialog for better desktop integration
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                let folder = AsyncFileDialog::new()
                    .set_title("Select Course Folder")
                    .set_directory(dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")))
                    .pick_folder()
                    .await;

                Ok(folder.map(|f| f.path().to_path_buf()))
            })
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    /// Save exported data to a file using native save dialog
    pub async fn save_export_data(
        &self,
        export_result: crate::export::ExportResult,
    ) -> Result<PathBuf> {
        tokio::task::spawn_blocking(move || {
            use rfd::FileDialog;

            // Create a suggested filename based on the export result
            let filename = export_result.filename;

            // Determine file filter based on format
            let filter = match export_result.format {
                crate::export::ExportFormat::Json => ("JSON Files", &["json"]),
                crate::export::ExportFormat::Csv => ("CSV Files", &["csv"]),
                crate::export::ExportFormat::Pdf => ("PDF Files", &["pdf"]),
            };

            // Show save dialog
            let file_path = FileDialog::new()
                .set_title("Save Export")
                .set_file_name(&filename)
                .add_filter(filter.0, filter.1)
                .save_file();

            match file_path {
                Some(path) => {
                    // Write data to file
                    std::fs::write(&path, &export_result.data)
                        .map_err(|e| anyhow::anyhow!("Failed to write export file: {}", e))?;

                    Ok(path)
                }
                None => Err(anyhow::anyhow!("Export cancelled by user")),
            }
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    /// Validate folder and scan for video content
    pub async fn validate_folder(&self, path: &Path) -> Result<FolderValidation> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Ok(FolderValidation {
                    is_valid: false,
                    video_count: 0,
                    supported_files: Vec::new(),
                    unsupported_files: Vec::new(),
                    total_size: 0,
                    error_message: Some("Folder does not exist".to_string()),
                });
            }

            if !path.is_dir() {
                return Ok(FolderValidation {
                    is_valid: false,
                    video_count: 0,
                    supported_files: Vec::new(),
                    unsupported_files: Vec::new(),
                    total_size: 0,
                    error_message: Some("Path is not a directory".to_string()),
                });
            }

            let mut supported_files = Vec::new();
            let mut unsupported_files = Vec::new();
            let mut total_size = 0u64;

            // Recursively scan directory for video files
            for entry in walkdir::WalkDir::new(&path)
                .follow_links(false)
                .max_depth(3) // Limit depth to avoid infinite recursion
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    if let Some(extension) = entry.path().extension() {
                        if let Some(ext_str) = extension.to_str() {
                            let ext_lower = ext_str.to_lowercase();

                            // Get file size
                            if let Ok(metadata) = entry.metadata() {
                                total_size += metadata.len();
                            }

                            if SUPPORTED_VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                                supported_files.push(entry.path().to_path_buf());
                            } else if is_video_like_extension(&ext_lower) {
                                unsupported_files.push(entry.path().to_path_buf());
                            }
                        }
                    }
                }
            }

            let video_count = supported_files.len();
            let is_valid = video_count > 0;
            let error_message = if !is_valid {
                Some("No supported video files found in the selected folder".to_string())
            } else {
                None
            };

            Ok(FolderValidation {
                is_valid,
                video_count,
                supported_files,
                unsupported_files,
                total_size,
                error_message,
            })
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    /// Import course from local folder
    pub async fn import_from_local_folder(
        &self,
        folder_path: &Path,
        course_title: Option<String>,
    ) -> Result<Course> {
        let folder_path = folder_path.to_path_buf();
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || {
            // First validate the folder
            let validation = Self::validate_folder_sync(&folder_path)?;
            if !validation.is_valid {
                return Err(anyhow::anyhow!("Invalid folder: {}", 
                    validation.error_message.unwrap_or_else(|| "Unknown error".to_string())));
            }
            
            // Generate course title
            let course_title = course_title.unwrap_or_else(|| {
                folder_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Imported Course")
                    .to_string()
            });
            
            // Check if a course with the same title and similar content already exists
            let existing_courses = storage::load_courses(&db)?;
            for existing_course in &existing_courses {
                if existing_course.name == course_title {
                    // Check if it's the same folder by comparing video counts
                    if existing_course.raw_titles.len() == validation.video_count {
                        return Err(anyhow::anyhow!(
                            "A course with the title '{}' and similar content already exists. Please choose a different folder or rename the existing course.",
                            course_title
                        ));
                    }
                }
            }
            
            // Use the ingest module to import from local folder
            crate::ingest::local_folder::import_from_folder(&db, &folder_path, &course_title)
                .map_err(|e| anyhow::anyhow!("Local folder import failed: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    /// Synchronous version of validate_folder for internal use
    fn validate_folder_sync(path: &Path) -> Result<FolderValidation> {
        if !path.exists() {
            return Ok(FolderValidation {
                is_valid: false,
                video_count: 0,
                supported_files: Vec::new(),
                unsupported_files: Vec::new(),
                total_size: 0,
                error_message: Some("Folder does not exist".to_string()),
            });
        }

        if !path.is_dir() {
            return Ok(FolderValidation {
                is_valid: false,
                video_count: 0,
                supported_files: Vec::new(),
                unsupported_files: Vec::new(),
                total_size: 0,
                error_message: Some("Path is not a directory".to_string()),
            });
        }

        let mut supported_files = Vec::new();
        let mut unsupported_files = Vec::new();
        let mut total_size = 0u64;

        // Recursively scan directory for video files
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .max_depth(3) // Limit depth to avoid infinite recursion
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if let Some(ext_str) = extension.to_str() {
                        let ext_lower = ext_str.to_lowercase();

                        // Get file size
                        if let Ok(metadata) = entry.metadata() {
                            total_size += metadata.len();
                        }

                        if SUPPORTED_VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                            supported_files.push(entry.path().to_path_buf());
                        } else if is_video_like_extension(&ext_lower) {
                            unsupported_files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }

        let video_count = supported_files.len();
        let is_valid = video_count > 0;
        let error_message = if !is_valid {
            Some("No supported video files found in the selected folder".to_string())
        } else {
            None
        };

        Ok(FolderValidation {
            is_valid,
            video_count,
            supported_files,
            unsupported_files,
            total_size,
            error_message,
        })
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
        async move { backend.list_courses().await }
    })
}

// Additional hooks for plans, notes, and exports can be added following this pattern.

/// Helper function to check if an extension might be video-related
fn is_video_like_extension(ext: &str) -> bool {
    matches!(
        ext,
        "rm" | "rmvb"
            | "asf"
            | "divx"
            | "vob"
            | "dat"
            | "amv"
            | "f4v"
            | "f4p"
            | "f4a"
            | "f4b"
            | "mod"
            | "tod"
            | "mxf"
    )
}

/// Preserve progress from an existing plan when regenerating
fn preserve_plan_progress(existing_plan: &crate::types::Plan, new_plan: &mut crate::types::Plan) {
    use std::collections::HashMap;

    // Create a map of video indices to completion status from existing plan
    let mut completion_map: HashMap<Vec<usize>, bool> = HashMap::new();
    for item in &existing_plan.items {
        completion_map.insert(item.video_indices.clone(), item.completed);
    }

    // Apply completion status to matching items in new plan
    for item in &mut new_plan.items {
        if let Some(&completed) = completion_map.get(&item.video_indices) {
            item.completed = completed;
        }
    }
}
