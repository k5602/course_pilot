use crate::storage::database::Database;
use crate::types::Course;
use dioxus::prelude::*;
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::sync::Arc;

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

/// Import operations hook
#[derive(Clone)]
pub struct ImportManager {
    db: Arc<Database>,
    pub import_from_local_folder: Callback<(PathBuf, Option<String>)>,
    pub validate_folder: Callback<PathBuf>,
}

impl ImportManager {
    pub async fn browse_folder(&self) -> Result<Option<PathBuf>> {
        tokio::task::spawn_blocking(move || {
            use rfd::FileDialog;

            // Show folder picker dialog
            let folder = FileDialog::new()
                .set_title("Select Course Folder")
                .set_directory(dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")))
                .pick_folder();

            Ok(folder)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn validate_folder(&self, path: PathBuf) -> Result<FolderValidation> {
        tokio::task::spawn_blocking(move || {
            validate_folder_sync(&path)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn import_from_local_folder(
        &self,
        folder_path: PathBuf,
        course_title: Option<String>,
    ) -> Result<Course> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // First validate the folder
            let validation = validate_folder_sync(&folder_path)?;
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
            let existing_courses = crate::storage::load_courses(&db)?;
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
}

pub fn use_import_manager() -> ImportManager {
    let db = use_context::<Arc<Database>>();
    
    let import_from_local_folder = use_callback({
        let db = db.clone();
        move |(folder_path, course_title): (PathBuf, Option<String>)| {
            let db = db.clone();
            spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    // First validate the folder
                    let validation = validate_folder_sync(&folder_path)?;
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

                    // Use the ingest module to import from local folder
                    crate::ingest::local_folder::import_from_folder(&db, &folder_path, &course_title)
                        .map_err(|e| anyhow::anyhow!("Local folder import failed: {}", e))
                }).await;

                match result {
                    Ok(Ok(_)) => {
                        crate::ui::components::toast::toast::success("Course imported successfully");
                    }
                    Ok(Err(e)) => {
                        crate::ui::components::toast::toast::error(format!("Failed to import course: {}", e));
                    }
                    Err(e) => {
                        crate::ui::components::toast::toast::error(format!("Failed to import course: {}", e));
                    }
                }
            });
            // Return () to match expected callback type
        }
    });

    let validate_folder = use_callback(move |path: PathBuf| {
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                validate_folder_sync(&path)
            }).await;

            match result {
                Ok(Ok(_)) => {
                    // Validation successful - the UI will handle the result
                }
                Ok(Err(e)) => {
                    crate::ui::components::toast::toast::error(format!("Folder validation failed: {}", e));
                }
                Err(e) => {
                    crate::ui::components::toast::toast::error(format!("Folder validation failed: {}", e));
                }
            }
        });
        // Return () to match expected callback type
    });
    
    ImportManager { db, import_from_local_folder, validate_folder }
}

/// Hook for reactive folder validation
pub fn use_folder_validation(folder_path: Option<PathBuf>) -> Resource<Result<Option<FolderValidation>, anyhow::Error>> {
    let import_manager = use_import_manager();

    use_resource(move || {
        let import_manager = import_manager.clone();
        let folder_path = folder_path.clone();
        async move {
            if let Some(path) = folder_path {
                import_manager.validate_folder(path).await.map(Some)
            } else {
                Ok(None)
            }
        }
    })
}

/// Synchronous folder validation for internal use
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