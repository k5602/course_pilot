//! Enhanced data ingestion module for Course Pilot
//!
//! This module provides functionality for importing course content from various sources
//! with integrated clustering and automatic course structuring.

pub mod local_folder;
pub mod youtube;

// Re-export main import functions
pub use local_folder::import_from_local_folder;
pub use youtube::import_from_youtube;

// Re-export error types
pub use crate::ImportError;

// Enhanced import functions with clustering integration
use crate::nlp::structure_course;
use crate::storage::database::Database;
use crate::types::{Course, ImportJob, ImportStage};
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

/// Enhanced YouTube import with automatic clustering
pub async fn import_and_structure_youtube(
    url: &str,
    api_key: &str,
    course_title: Option<String>,
    db: &Database,
    progress_callback: impl Fn(ImportProgress) + Send + 'static,
) -> Result<Course, ImportError> {
    // Stage 1: Starting
    progress_callback(ImportProgress {
        stage: ImportStage::Fetching,
        progress: 0.0,
        message: "Initializing YouTube import...".to_string(),
        clustering_stage: None,
    });

    // Stage 2: Import raw content
    progress_callback(ImportProgress {
        stage: ImportStage::Fetching,
        progress: 0.1,
        message: "Fetching playlist data...".to_string(),
        clustering_stage: None,
    });

    let (sections, metadata) = youtube::import_from_youtube(url, api_key)
        .await
        .map_err(|e| ImportError::Network(format!("YouTube import failed: {e}")))?;

    progress_callback(ImportProgress {
        stage: ImportStage::Processing,
        progress: 0.3,
        message: format!("Imported {} videos", sections.len()),
        clustering_stage: None,
    });

    // Stage 3: Create course with raw titles
    let course_name = course_title.unwrap_or_else(|| metadata.title.clone());
    let raw_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    let mut course = Course::new(course_name, raw_titles.clone());

    // Stage 4: Structure using advanced clustering
    progress_callback(ImportProgress {
        stage: ImportStage::TfIdfAnalysis,
        progress: 0.4,
        message: "Analyzing content structure...".to_string(),
        clustering_stage: Some(0),
    });

    // Create clustering progress callback
    let clustering_progress = |stage: u8, message: String| {
        let progress = 0.4 + (stage as f32 / 4.0) * 0.4; // 0.4 to 0.8
        progress_callback(ImportProgress {
            stage: ImportStage::KMeansClustering,
            progress,
            message,
            clustering_stage: Some(stage),
        });
    };

    // Perform clustering with progress tracking
    let structure = structure_course_with_progress(raw_titles, clustering_progress)?;
    course.structure = Some(structure);

    // Stage 5: Save course with clustering metadata
    progress_callback(ImportProgress {
        stage: ImportStage::Saving,
        progress: 0.9,
        message: "Saving course structure...".to_string(),
        clustering_stage: None,
    });

    crate::storage::save_course(db, &course)
        .map_err(|e| ImportError::Database(format!("Failed to save course: {e}")))?;

    // Stage 6: Complete
    progress_callback(ImportProgress {
        stage: ImportStage::Saving, // Use Saving as final stage since we don't have Complete
        progress: 1.0,
        message: format!(
            "Successfully imported and structured course: {}",
            course.name
        ),
        clustering_stage: None,
    });

    Ok(course)
}

/// Enhanced local folder import with automatic clustering
pub async fn import_and_structure_local_folder(
    folder_path: &Path,
    course_title: String,
    db: &Database,
    progress_callback: impl Fn(ImportProgress) + Send + 'static,
) -> Result<Course, ImportError> {
    // Stage 1: Starting
    progress_callback(ImportProgress {
        stage: ImportStage::Fetching,
        progress: 0.0,
        message: "Initializing folder import...".to_string(),
        clustering_stage: None,
    });

    // Stage 2: Import raw content
    progress_callback(ImportProgress {
        stage: ImportStage::Processing,
        progress: 0.1,
        message: "Scanning video files...".to_string(),
        clustering_stage: None,
    });

    let sections = local_folder::import_from_local_folder(folder_path)
        .map_err(|e| ImportError::FileSystem(format!("Folder import failed: {e}")))?;

    progress_callback(ImportProgress {
        stage: ImportStage::Processing,
        progress: 0.3,
        message: format!("Found {} video files", sections.len()),
        clustering_stage: None,
    });

    // Stage 3: Create course with raw titles
    let raw_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    let mut course = Course::new(course_title, raw_titles.clone());

    // Stage 4: Structure using advanced clustering
    progress_callback(ImportProgress {
        stage: ImportStage::TfIdfAnalysis,
        progress: 0.4,
        message: "Analyzing content structure...".to_string(),
        clustering_stage: Some(0),
    });

    // Create clustering progress callback
    let clustering_progress = |stage: u8, message: String| {
        let progress = 0.4 + (stage as f32 / 4.0) * 0.4; // 0.4 to 0.8
        progress_callback(ImportProgress {
            stage: ImportStage::KMeansClustering,
            progress,
            message,
            clustering_stage: Some(stage),
        });
    };

    // Perform clustering with progress tracking
    let structure = structure_course_with_progress(raw_titles, clustering_progress)?;
    course.structure = Some(structure);

    // Stage 5: Save course with clustering metadata
    progress_callback(ImportProgress {
        stage: ImportStage::Saving,
        progress: 0.9,
        message: "Saving course structure...".to_string(),
        clustering_stage: None,
    });

    crate::storage::save_course(db, &course)
        .map_err(|e| ImportError::Database(format!("Failed to save course: {e}")))?;

    // Stage 6: Complete
    progress_callback(ImportProgress {
        stage: ImportStage::Saving, // Use Saving as final stage since we don't have Complete
        progress: 1.0,
        message: format!(
            "Successfully imported and structured course: {}",
            course.name
        ),
        clustering_stage: None,
    });

    Ok(course)
}

/// Structure course with progress tracking
fn structure_course_with_progress(
    titles: Vec<String>,
    progress_callback: impl Fn(u8, String),
) -> Result<crate::types::CourseStructure, ImportError> {
    progress_callback(0, "Starting content analysis...".to_string());

    // Simulate progress through clustering stages
    std::thread::sleep(std::time::Duration::from_millis(100));
    progress_callback(1, "Performing TF-IDF analysis...".to_string());

    std::thread::sleep(std::time::Duration::from_millis(200));
    progress_callback(2, "Clustering content...".to_string());

    std::thread::sleep(std::time::Duration::from_millis(150));
    progress_callback(3, "Optimizing clusters...".to_string());

    std::thread::sleep(std::time::Duration::from_millis(100));
    progress_callback(4, "Finalizing structure...".to_string());

    structure_course(titles).map_err(|e| ImportError::Network(format!("Clustering failed: {e}")))
}

/// Create import job for tracking
pub fn create_import_job(message: String) -> ImportJob {
    ImportJob::new(message)
}

/// Update import job with progress
pub fn update_import_job_progress(job: &mut ImportJob, progress: &ImportProgress) {
    job.update_stage_progress(
        progress.stage.clone(),
        progress.progress * 100.0,
        progress.message.clone(),
    );

    // Update status based on progress
    if progress.progress >= 1.0 {
        job.mark_completed();
    }
}
