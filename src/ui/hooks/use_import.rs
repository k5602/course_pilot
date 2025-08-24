use crate::storage::database::Database;
use crate::types::Course;
use crate::ui::toast_helpers;
use anyhow::Result;
use dioxus::prelude::*;
use std::path::{Path, PathBuf};
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
        tokio::task::spawn_blocking(move || generate_folder_preview_sync(&path))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
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
                            validation
                                .error_message
                                .unwrap_or_else(|| "Unknown error".to_string())
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
                    }
                    Ok(Err(e)) => {
                        toast_helpers::error(format!("Failed to import course: {e}"));
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to import course: {e}"));
                    }
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
                }
                Ok(Err(e)) => {
                    toast_helpers::error(format!("Folder validation failed: {e}"));
                }
                Err(e) => {
                    toast_helpers::error(format!("Folder validation failed: {e}"));
                }
            }
        });
        // Return () to match expected callback type
    });

    let generate_folder_preview = use_callback(move |path: PathBuf| {
        spawn(async move {
            let result =
                tokio::task::spawn_blocking(move || generate_folder_preview_sync(&path)).await;

            match result {
                Ok(Ok(_)) => {
                    // Preview generation successful - the UI will handle the result
                }
                Ok(Err(e)) => {
                    toast_helpers::error(format!("Preview generation failed: {e}"));
                }
                Err(e) => {
                    toast_helpers::error(format!("Preview generation failed: {e}"));
                }
            }
        });
        // Return () to match expected callback type
    });

    ImportManager {
        db,
        import_from_local_folder,
        validate_folder,
        generate_folder_preview,
    }
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
fn generate_folder_preview_sync(path: &Path) -> Result<LocalFolderPreview> {
    // First validate the folder
    let validation = validate_folder_sync(path)?;

    if !validation.is_valid {
        return Err(anyhow::anyhow!(
            "Cannot generate preview for invalid folder: {}",
            validation
                .error_message
                .unwrap_or_else(|| "Unknown error".to_string())
        ));
    }

    // Generate course title from folder name
    let title = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Imported Course")
        .to_string();

    // Use the enhanced local ingest to scan recursively (matching validation behavior)
    let ingest = crate::ingest::local_folder::EnhancedLocalIngest::new();
    let video_files = ingest
        .scan_directory_recursive(path, None)
        .map_err(|e| anyhow::anyhow!("Failed to scan folder for preview: {}", e))?;

    // Convert video files to preview videos
    let mut videos = Vec::new();
    let mut total_duration = std::time::Duration::new(0, 0);

    for (index, video_file) in video_files.iter().enumerate() {
        // Extract title from file path
        let title = video_file
            .path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| clean_filename_title(s))
            .unwrap_or_else(|| video_file.name.clone());

        // Try to get video duration (this might be slow for many files, but needed for preview)
        let duration = probe_video_duration(&video_file.path);

        let format = video_file
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_uppercase();

        videos.push(LocalVideoPreview {
            title,
            duration,
            index,
            file_size: video_file.size,
            format,
        });

        if let Some(dur) = duration {
            total_duration += dur;
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

/// Clean and normalize titles extracted from filenames
fn clean_filename_title(title: &str) -> String {
    title
        .trim()
        // Replace common separators with spaces
        .replace(['_', '-', '.'], " ")
        // Remove common video quality indicators
        .replace("1080p", "")
        .replace("720p", "")
        .replace("480p", "")
        .replace("4K", "")
        .replace("HD", "")
        // Remove common brackets and their contents if they contain metadata
        .split('[')
        .next()
        .unwrap_or(title)
        .split('(')
        .next()
        .unwrap_or(title)
        // Normalize whitespace
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Probe video duration using Symphonia (pure Rust, no external CLI)
fn probe_video_duration(path: &std::path::Path) -> Option<std::time::Duration> {
    use std::fs::File;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    // Open the media source.
    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Provide a hint based on the file extension.
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    // Probe the media format.
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;

    let mut format = probed.format;

    // Use the default track for duration calculations.
    let track = format.default_track()?;

    // Preferred: derive duration from track time_base and n_frames if available.
    if let (Some(tb), Some(n_frames)) = (track.codec_params.time_base, track.codec_params.n_frames)
    {
        let t = tb.calc_time(n_frames);
        let secs = t.seconds as f64 + (t.frac as f64 / 1_000_000_000.0);
        return Some(std::time::Duration::from_secs_f64(secs.max(0.0)));
    }

    // Fallback: iterate packets to get the last timestamp for the default track.
    let mut last_ts: Option<u64> = None;
    let tb = track.codec_params.time_base;
    let track_id = track.id;
    let _ = track;

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() == track_id {
            last_ts = Some(packet.ts());
        }
    }

    if let (Some(tb), Some(ts)) = (tb, last_ts) {
        let t = tb.calc_time(ts);
        let secs = t.seconds as f64 + (t.frac as f64 / 1_000_000_000.0);
        return Some(std::time::Duration::from_secs_f64(secs.max(0.0)));
    }

    None
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
