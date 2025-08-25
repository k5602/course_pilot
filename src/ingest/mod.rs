//! Enhanced data ingestion module for Course Pilot
//!
//! This module provides functionality for importing course content from various sources
//! with integrated clustering and automatic course structuring.

pub mod local_folder;
pub mod youtube;

// Re-export main import functions
pub use local_folder::{
    LocalImportResult, import_from_local_folder, import_from_local_folder_with_analysis,
};
pub use youtube::import_from_youtube;

// Re-export error types
pub use crate::ImportError;

// Import basic types needed for ingest operations
use crate::types::{Course, ImportStage};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

static VIDEO_DURATION_CACHE: Lazy<
    Mutex<HashMap<std::path::PathBuf, (u64, Option<std::time::SystemTime>, std::time::Duration)>>,
> = Lazy::new(|| Mutex::new(HashMap::new()));

// Common validation utilities

/// Validate that a string could be a valid YouTube playlist URL
pub fn is_valid_youtube_url(url: &str) -> bool {
    url.contains("youtube.com") && (url.contains("playlist") || url.contains("list="))
}

/// Validate that a path exists and is a directory
pub fn is_valid_directory(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

/// Common video file extensions
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "mp2", "mpe", "mpv",
    "m2v", "3gp", "3g2", "ts", "mts", "m2ts", "ogv", "qt", "yuv", "drc", "svi", "mxf", "roq",
    "nsv", "f4v", "f4p", "f4a", "f4b", "asf", "rm", "rmvb", "vob",
];

/// Check if a file has a video extension
pub fn is_video_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return VIDEO_EXTENSIONS.contains(&ext_str.to_lowercase().as_str());
        }
    }
    false
}

/// Clean and normalize video titles
pub fn clean_title(title: &str) -> String {
    // Normalize separators and strip common noise
    let base = title
        .trim()
        .replace(['_', '-', '.'], " ")
        .replace("1080p", "")
        .replace("720p", "")
        .replace("480p", "")
        .replace("2160p", "")
        .replace("1440p", "")
        .replace("4K", "")
        .replace("HD", "");

    // Remove bracketed metadata like [Official], (2021), etc.
    let base = base
        .split('[')
        .next()
        .unwrap_or(&base)
        .split('(')
        .next()
        .unwrap_or(&base)
        .to_string();

    base.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Probe video duration using mp4 and matroska crates. Returns None if not determinable quickly.
pub fn probe_video_duration(path: &std::path::Path) -> Option<std::time::Duration> {
    use std::fs::File;
    use std::io::BufReader;

    // Build a lightweight fingerprint from metadata for cache validation.
    let (len, mtime) = std::fs::metadata(path)
        .map(|m| (m.len(), m.modified().ok()))
        .unwrap_or((0, None));

    // Fast path: return cached duration if file is unchanged.
    if let Ok(cache) = VIDEO_DURATION_CACHE.lock() {
        if let Some((cached_len, cached_mtime, dur)) = cache.get(path) {
            if *cached_len == len && *cached_mtime == mtime {
                return Some(*dur);
            }
        }
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())?;

    let computed = match ext.as_str() {
        // ISO Base Media File Format (mp4/m4v/mov)
        "mp4" | "m4v" | "mov" => {
            let f = File::open(path).ok()?;
            let size = f.metadata().ok()?.len();
            let reader = BufReader::new(f);
            // mp4::Mp4Reader provides duration if available.
            let mp4 = mp4::Mp4Reader::read_header(reader, size).ok()?;
            Some(mp4.duration())
        }

        // Matroska containers (mkv/webm)
        "mkv" | "webm" => {
            // Use matroska::get_from to read Info with duration.
            let info = matroska::get_from::<_, matroska::Info>(path).ok()??;
            if let Some(dur) = info.duration {
                let d = std::time::Duration::from_secs(dur.as_secs())
                    + std::time::Duration::from_nanos(dur.subsec_nanos() as u64);
                Some(d)
            } else {
                None
            }
        }

        // Others not handled by these parsers.
        _ => None,
    };

    // Update cache on success and return.
    if let Some(d) = computed {
        if let Ok(mut cache) = VIDEO_DURATION_CACHE.lock() {
            cache.insert(path.to_path_buf(), (len, mtime, d));
        }
        Some(d)
    } else {
        None
    }
}

/// Progress tracking for integrated import operations
#[derive(Debug, Clone)]
pub struct ImportProgress {
    pub stage: ImportStage,
    pub progress: f32, // 0.0 to 1.0
    pub message: String,
    pub clustering_stage: Option<u8>, // 0-4 for clustering progress
}

/// Ingest-only service (order-preserving, metadata-complete, no structuring, no DB I/O)
/// NOTE: Intentionally free functions for now to avoid introducing async-trait; callers can wrap
/// preserve original_index; they DO NOT call NLP or save to storage.
pub mod ingest_only {
    use super::*;

    /// Ingest a YouTube playlist preserving order and metadata without structuring or saving.
    pub async fn ingest_youtube_only(
        url: &str,
        api_key: &str,
        course_title: Option<String>,
        mut progress_callback: Option<impl FnMut(ImportProgress) + Send + 'static>,
    ) -> Result<Course, ImportError> {
        if let Some(cb) = progress_callback.as_mut() {
            cb(ImportProgress {
                stage: ImportStage::Fetching,
                progress: 0.0,
                message: "Initializing YouTube ingest...".to_string(),
                clustering_stage: None,
            });
        }

        if let Some(cb) = progress_callback.as_mut() {
            cb(ImportProgress {
                stage: ImportStage::Fetching,
                progress: 0.2,
                message: "Fetching playlist data...".to_string(),
                clustering_stage: None,
            });
        }

        let (sections, metadata) = youtube::import_from_youtube(url, api_key)
            .await
            .map_err(|e| ImportError::Network(format!("YouTube import failed: {e}")))?;

        // Validate required fields and build videos
        let mut videos: Vec<crate::types::VideoMetadata> = Vec::with_capacity(sections.len());
        for (i, s) in sections.iter().enumerate() {
            log::info!(
                "Processing YouTube section {}: title='{}', video_id='{}', url='{}', playlist_id={:?}",
                i,
                s.title,
                s.video_id,
                s.url,
                s.playlist_id
            );

            if s.video_id.is_empty() || s.url.is_empty() {
                log::error!(
                    "Incomplete metadata for section {}: video_id='{}', url='{}'",
                    i,
                    s.video_id,
                    s.url
                );
                return Err(ImportError::Network(format!(
                    "Incomplete metadata for YouTube item {} '{}'",
                    i + 1,
                    s.title
                )));
            }

            let mut v = crate::types::VideoMetadata::new_youtube_with_playlist(
                s.title.clone(),
                s.video_id.clone(),
                s.url.clone(),
                s.playlist_id.clone(),
                s.original_index,
            );

            log::info!(
                "Created VideoMetadata: video_id={:?}, source_url={:?}, is_local={}",
                v.video_id,
                v.source_url,
                v.is_local
            );
            v.duration_seconds = Some(s.duration.as_secs_f64());
            v.thumbnail_url = s.thumbnail_url.clone();
            v.description = s.description.clone();
            v.author = s.author.clone();
            if !v.is_metadata_complete() {
                return Err(ImportError::Network(format!(
                    "Incomplete metadata after build for video {}: '{}'",
                    i + 1,
                    v.title
                )));
            }
            videos.push(v);
        }

        if let Some(cb) = progress_callback.as_mut() {
            cb(ImportProgress {
                stage: ImportStage::Processing,
                progress: 0.8,
                message: format!("Prepared {} videos (order preserved)", videos.len()),
                clustering_stage: None,
            });
        }

        let name = course_title.unwrap_or_else(|| metadata.title.clone());
        let course = Course::new_with_videos(name, videos);

        if let Some(cb) = progress_callback.as_mut() {
            cb(ImportProgress {
                stage: ImportStage::Saving,
                progress: 1.0,
                message: "Ingest complete (no structuring, no save)".to_string(),
                clustering_stage: None,
            });
        }

        Ok(course)
    }

    /// Ingest a local folder preserving file order and metadata without structuring or saving.
    pub fn ingest_local_folder_only(
        folder_path: &Path,
        course_title: String,
        mut progress_callback: Option<impl FnMut(ImportProgress) + Send + 'static>,
    ) -> Result<Course, ImportError> {
        if let Some(cb) = progress_callback.as_mut() {
            cb(ImportProgress {
                stage: ImportStage::Fetching,
                progress: 0.0,
                message: "Scanning local folder...".to_string(),
                clustering_stage: None,
            });
        }

        let import_result = local_folder::import_from_local_folder_with_analysis(folder_path)
            .map_err(|e| ImportError::FileSystem(format!("Folder import failed: {e}")))?;

        let mut videos: Vec<crate::types::VideoMetadata> =
            Vec::with_capacity(import_result.sections.len());
        for s in &import_result.sections {
            let mut v = crate::types::VideoMetadata::new_local_with_index(
                s.title.clone(),
                s.file_path.clone().unwrap_or_default(),
                s.original_index,
            );
            v.duration_seconds = Some(s.duration.as_secs_f64());
            if !v.is_metadata_complete() {
                return Err(ImportError::FileSystem(format!(
                    "Incomplete local metadata for '{}'",
                    v.title
                )));
            }
            videos.push(v);
        }

        if let Some(cb) = progress_callback.as_mut() {
            cb(ImportProgress {
                stage: ImportStage::Processing,
                progress: 0.8,
                message: format!("Prepared {} videos (order preserved)", videos.len()),
                clustering_stage: None,
            });
        }

        let course = Course::new_with_videos(course_title, videos);

        if let Some(cb) = progress_callback.as_mut() {
            cb(ImportProgress {
                stage: ImportStage::Saving,
                progress: 1.0,
                message: "Ingest complete (no structuring, no save)".to_string(),
                clustering_stage: None,
            });
        }

        Ok(course)
    }
}

/// Processing strategy for local folder content based on analysis
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStrategy {
    PreserveSequentialOrder, // Use sequential processing to preserve order
    ApplyClusteringAnalysis, // Use clustering algorithms for thematic content
    RequestUserChoice,       // Present user with choice dialog (fallback to sequential for now)
}
