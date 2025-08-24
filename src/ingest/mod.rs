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
use std::path::Path;

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
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg",
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
    title
        .trim()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
/// them behind their own service objects. These functions build a Course with videos populated and
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

// Functions removed: import_and_structure_youtube, import_and_structure_local_folder,
// structure_course_with_progress, create_import_job, update_import_job_progress
// These functions violated the ingest contract by mixing import, structuring, and saving.
// Use ingest_only functions instead for clean separation of concerns.
//
// The ProcessingStrategy enum remains for potential future use by other modules
// that need to determine how to process course content after ingest.
