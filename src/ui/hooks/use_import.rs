use crate::storage::database::Database;
use crate::storage::settings::AppSettings;
use crate::types::Course;
use crate::ui::toast_helpers;
use anyhow::Result;
use dioxus::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

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

/// Import preview data for local folders
#[derive(Debug, Clone, PartialEq)]
pub struct LocalFolderPreview {
    pub title: String,
    pub video_count: usize,
    pub total_duration: Option<std::time::Duration>,
    pub videos: Vec<LocalVideoPreview>,
    pub total_size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVideoPreview {
    pub title: String,
    pub duration: Option<std::time::Duration>,
    pub index: usize,
    pub file_size: u64,
    pub format: String,
}

/// Supported video file extensions
const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ogv", "ts",
    "mts", "m2ts",
];

/// Import operations hook with improved async patterns
#[derive(Clone)]
pub struct ImportManager {
    db: Arc<Database>,
    pub import_from_local_folder: Callback<(PathBuf, Option<String>)>,
    pub validate_folder: Callback<PathBuf>,
    pub generate_folder_preview: Callback<PathBuf>,
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
        tokio::task::spawn_blocking(move || validate_folder_sync(&path))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn generate_folder_preview(&self, path: PathBuf) -> Result<LocalFolderPreview> {
        self.generate_folder_preview_with_cancel(path, CancellationToken::new()).await
    }

    pub async fn generate_folder_preview_with_cancel(
        &self,
        path: PathBuf,
        cancel_token: CancellationToken,
    ) -> Result<LocalFolderPreview> {
        // Validate first in a blocking task
        let validation = tokio::task::spawn_blocking({
            let path = path.clone();
            move || validate_folder_sync(&path)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))?;

        if !validation.is_valid {
            return Err(anyhow::anyhow!(
                "Cannot generate preview for invalid folder: {}",
                validation.error_message.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        // Load ingest-related preferences
        let settings = AppSettings::load().unwrap_or_default();
        let import_prefs = settings.get_import_preferences().clone();

        // Respect cancellation toggle from settings
        let effective_token = if import_prefs.preview_cancellation_enabled {
            cancel_token.clone()
        } else {
            CancellationToken::new()
        };

        let ingest = crate::ingest::local_folder::LocalIngest::new();
        let video_files = ingest
            .scan_directory_recursive_async(
                path.clone(),
                None,
                Some(128), // batch size for scanning
                Some(effective_token.clone()),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to scan folder for preview: {}", e))?;

        // Prepare preview items and parallelize duration probing with bounded concurrency
        let mut videos = Vec::with_capacity(video_files.len());
        let mut total_duration = std::time::Duration::new(0, 0);

        // Precompute work items
        let work: Vec<_> = video_files
            .iter()
            .enumerate()
            .map(|(index, vf)| {
                let title = vf
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(crate::ingest::clean_title)
                    .unwrap_or_else(|| vf.name.clone());

                let format = vf
                    .path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_uppercase();

                (index, vf.path.clone(), title, vf.size, format)
            })
            .collect();

        // Bounded parallelism (configurable via settings)
        let max_concurrency = import_prefs.preview_probe_max_concurrency.max(1);
        for chunk in work.chunks(max_concurrency) {
            if effective_token.is_cancelled() {
                return Err(anyhow::anyhow!("Preview generation cancelled"));
            }

            // Spawn blocking duration probes
            let mut handles = Vec::with_capacity(chunk.len());
            for (_, path, _, _, _) in chunk.iter() {
                let path = path.clone();
                let cancel = effective_token.clone();
                handles.push(tokio::task::spawn_blocking(move || {
                    if cancel.is_cancelled() {
                        return None;
                    }
                    crate::ingest::probe_video_duration(&path)
                }));
            }

            // Collect results preserving chunk order
            for (i, handle) in handles.into_iter().enumerate() {
                if effective_token.is_cancelled() {
                    return Err(anyhow::anyhow!("Preview generation cancelled"));
                }
                let duration =
                    handle.await.map_err(|e| anyhow::anyhow!(format!("Join error: {}", e)))?;

                let (index, _path, title, size, format) = &chunk[i];

                if let Some(d) = duration {
                    total_duration += d;
                }

                videos.push(LocalVideoPreview {
                    title: title.clone(),
                    duration,
                    index: *index,
                    file_size: *size,
                    format: format.clone(),
                });
            }
        }

        Ok(LocalFolderPreview {
            title: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Imported Course")
                .to_string(),
            video_count: videos.len(),
            total_duration: Some(total_duration),
            videos,
            total_size: validation.total_size,
        })
    }

    pub async fn import_from_local_folder(
        &self,
        folder_path: PathBuf,
        course_title: Option<String>,
    ) -> Result<Course> {
        // Generate course title
        let course_title = course_title.unwrap_or_else(|| {
            folder_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Imported Course")
                .to_string()
        });

        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::ingest::local_folder::import_from_folder(&db, &folder_path, &course_title)
                .map_err(|e| anyhow::anyhow!("Import error: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn import_from_youtube(
        &self,
        _playlist_url: String,
        _api_key: String,
        _course_title: Option<String>,
    ) -> Result<Course> {
        let _db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // This would need to be implemented in the youtube module
            Err(anyhow::anyhow!("YouTube import not yet implemented"))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn validate_youtube_playlist(&self, url: &str, _api_key: &str) -> Result<bool> {
        let url = url.to_string();
        tokio::task::spawn_blocking(move || {
            // This would need to be implemented
            Ok(crate::ingest::is_valid_youtube_url(&url))
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
                        return Err(anyhow::anyhow!(
                            "Invalid folder: {}",
                            validation.error_message.unwrap_or_else(|| "Unknown error".to_string())
                        ));
                    }

                    // Generate course title
                    let course_title = course_title.unwrap_or_else(|| {
                        folder_path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Imported Course")
                            .to_string()
                    });

                    // Use the ingest module to import from local folder
                    crate::ingest::local_folder::import_from_folder(
                        &db,
                        &folder_path,
                        &course_title,
                    )
                    .map_err(|e| anyhow::anyhow!("Local folder import failed: {}", e))
                })
                .await;

                match result {
                    Ok(Ok(_)) => {
                        toast_helpers::success("Course imported successfully");
                    },
                    Ok(Err(e)) => {
                        toast_helpers::error(format!("Failed to import course: {e}"));
                    },
                    Err(e) => {
                        toast_helpers::error(format!("Failed to import course: {e}"));
                    },
                }
            });
            // Return () to match expected callback type
        }
    });

    let validate_folder = use_callback(move |path: PathBuf| {
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || validate_folder_sync(&path)).await;

            match result {
                Ok(Ok(_)) => {
                    // Validation successful - the UI will handle the result
                },
                Ok(Err(e)) => {
                    toast_helpers::error(format!("Folder validation failed: {e}"));
                },
                Err(e) => {
                    toast_helpers::error(format!("Folder validation failed: {e}"));
                },
            }
        });
        // Return () to match expected callback type
    });

    let generate_folder_preview = use_callback(move |path: PathBuf| {
        spawn(async move {
            let result = generate_folder_preview_async(path).await;

            match result {
                Ok(_) => {
                    // Preview generation successful - the UI will handle the result
                },
                Err(e) => {
                    toast_helpers::error(format!("Preview generation failed: {e}"));
                },
            }
        });
        // Return () to match expected callback type
    });

    ImportManager { db, import_from_local_folder, validate_folder, generate_folder_preview }
}

/// Hook for reactive folder validation
pub fn use_folder_validation(
    folder_path: Option<PathBuf>,
) -> Resource<Result<Option<FolderValidation>, anyhow::Error>> {
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

/// Hook for reactive folder preview generation
pub fn use_folder_preview(
    folder_path: Option<PathBuf>,
) -> Resource<Result<Option<LocalFolderPreview>, anyhow::Error>> {
    let import_manager = use_import_manager();

    use_resource(move || {
        let import_manager = import_manager.clone();
        let folder_path = folder_path.clone();
        async move {
            if let Some(path) = folder_path {
                import_manager.generate_folder_preview(path).await.map(Some)
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

/// Generate preview data for a local folder
async fn generate_folder_preview_async(path: PathBuf) -> Result<LocalFolderPreview> {
    // First validate the folder in a blocking task
    let validation = tokio::task::spawn_blocking({
        let path = path.clone();
        move || validate_folder_sync(&path)
    })
    .await
    .map_err(|e| anyhow::anyhow!(format!("Join error: {}", e)))??;

    if !validation.is_valid {
        return Err(anyhow::anyhow!(
            "Cannot generate preview for invalid folder: {}",
            validation.error_message.unwrap_or_else(|| "Unknown error".to_string())
        ));
    }

    // Generate course title from folder name
    let title =
        path.file_name().and_then(|name| name.to_str()).unwrap_or("Imported Course").to_string();

    // Async, cancellable recursive scan with batching
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let ingest = crate::ingest::local_folder::LocalIngest::new();
    let video_files = ingest
        .scan_directory_recursive_async(path.clone(), None, Some(128), Some(cancel_token.clone()))
        .await
        .map_err(|e| anyhow::anyhow!(format!("Failed to scan folder for preview: {}", e)))?;

    // Prepare preview items and parallelize duration probing with bounded concurrency
    let mut videos = Vec::with_capacity(video_files.len());
    let mut total_duration = std::time::Duration::new(0, 0);

    // Build work items
    let work: Vec<_> = video_files
        .iter()
        .enumerate()
        .map(|(index, vf)| {
            let title = vf
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(crate::ingest::clean_title)
                .unwrap_or_else(|| vf.name.clone());

            let format = vf
                .path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown")
                .to_uppercase();

            (index, vf.path.clone(), title, vf.size, format)
        })
        .collect();

    let max_concurrency = 8usize;
    for chunk in work.chunks(max_concurrency) {
        if cancel_token.is_cancelled() {
            return Err(anyhow::anyhow!("Preview generation cancelled"));
        }

        // Spawn blocking duration probes
        let mut handles = Vec::with_capacity(chunk.len());
        for (_, p, _, _, _) in chunk.iter() {
            let path = p.clone();
            let cancel = cancel_token.clone();
            handles.push(tokio::task::spawn_blocking(move || {
                if cancel.is_cancelled() {
                    return None;
                }
                crate::ingest::probe_video_duration(&path)
            }));
        }

        // Collect results preserving order within chunk
        for (i, handle) in handles.into_iter().enumerate() {
            if cancel_token.is_cancelled() {
                return Err(anyhow::anyhow!("Preview generation cancelled"));
            }
            let duration =
                handle.await.map_err(|e| anyhow::anyhow!(format!("Join error: {}", e)))?;

            let (index, _path, title, size, format) = &chunk[i];

            if let Some(d) = duration {
                total_duration += d;
            }

            videos.push(LocalVideoPreview {
                title: title.clone(),
                duration,
                index: *index,
                file_size: *size,
                format: format.clone(),
            });
        }
    }

    Ok(LocalFolderPreview {
        title,
        video_count: videos.len(),
        total_duration: Some(total_duration),
        videos,
        total_size: validation.total_size,
    })
}

// Removed: use crate::ingest::clean_title instead

// Removed: use crate::ingest::probe_video_duration instead

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
