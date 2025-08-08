//! Enhanced data ingestion module for Course Pilot
//!
//! This module provides functionality for importing course content from various sources
//! with integrated clustering and automatic course structuring.

pub mod local_folder;
pub mod youtube;

// Re-export main import functions
pub use local_folder::{import_from_local_folder, import_from_local_folder_with_analysis, LocalImportResult};
pub use youtube::import_from_youtube;

// Re-export error types
pub use crate::ImportError;

// Import content analysis types
use crate::nlp::sequential_detection::{ContentTypeAnalysis, ContentType, ProcessingRecommendation};

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
            if s.video_id.is_empty() || s.url.is_empty() {
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

        let mut videos: Vec<crate::types::VideoMetadata> = Vec::with_capacity(import_result.sections.len());
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
    PreserveSequentialOrder,  // Use sequential processing to preserve order
    ApplyClusteringAnalysis,  // Use clustering algorithms for thematic content
    RequestUserChoice,        // Present user with choice dialog (fallback to sequential for now)
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

    // Validate that YouTube import preserved all required metadata
    for (index, section) in sections.iter().enumerate() {
        if section.video_id.is_empty() {
            return Err(ImportError::Network(format!(
                "YouTube import failed: video {} '{}' has empty video_id",
                index + 1,
                section.title
            )));
        }
        if section.url.is_empty() {
            return Err(ImportError::Network(format!(
                "YouTube import failed: video {} '{}' has empty URL",
                index + 1,
                section.title
            )));
        }
    }

    progress_callback(ImportProgress {
        stage: ImportStage::Processing,
        progress: 0.3,
        message: format!("Imported {} videos with complete metadata", sections.len()),
        clustering_stage: None,
    });

    // Stage 3: Create course with structured video metadata
    let course_name = course_title.unwrap_or_else(|| metadata.title.clone());
    let raw_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    let videos: Vec<crate::types::VideoMetadata> = sections.iter().enumerate().map(|(_index, s)| {
        // Log what we're creating for debugging
        log::info!("Creating VideoMetadata: title='{}', video_id='{}', url='{}'", s.title, s.video_id, s.url);
        
        // Create VideoMetadata with complete YouTube metadata
        let mut video_metadata = crate::types::VideoMetadata::new_youtube_with_playlist(
            s.title.clone(),
            s.video_id.clone(),
            s.url.clone(),
            s.playlist_id.clone(),
            s.original_index,
        );
        
        // Set additional metadata fields
        video_metadata.duration_seconds = Some(s.duration.as_secs_f64());
        video_metadata.thumbnail_url = s.thumbnail_url.clone();
        video_metadata.description = s.description.clone();
        video_metadata.author = s.author.clone();
        
        video_metadata
    }).collect();
    
    // Validate that all imported metadata is complete before proceeding
    for (index, video) in videos.iter().enumerate() {
        if !video.is_metadata_complete() {
            return Err(ImportError::Network(format!(
                "Incomplete metadata for video {}: '{}'. Missing required fields for YouTube video.",
                index + 1,
                video.title
            )));
        }
    }
    
    log::info!("Successfully validated metadata for {} YouTube videos", videos.len());
    
    // Log metadata preservation statistics
    let videos_with_thumbnails = videos.iter().filter(|v| v.thumbnail_url.is_some()).count();
    let videos_with_descriptions = videos.iter().filter(|v| v.description.is_some()).count();
    let videos_with_authors = videos.iter().filter(|v| v.author.is_some()).count();
    let videos_with_playlist_ids = videos.iter().filter(|v| v.playlist_id.is_some()).count();
    
    log::info!(
        "Metadata preservation stats: thumbnails={}/{}, descriptions={}/{}, authors={}/{}, playlist_ids={}/{}",
        videos_with_thumbnails, videos.len(),
        videos_with_descriptions, videos.len(),
        videos_with_authors, videos.len(),
        videos_with_playlist_ids, videos.len()
    );
    
    let mut course = Course::new_with_videos(course_name, videos);

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

/// Enhanced local folder import with content type detection and appropriate processing
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

    // Stage 2: Import raw content with content analysis
    progress_callback(ImportProgress {
        stage: ImportStage::Processing,
        progress: 0.1,
        message: "Scanning video files and detecting content patterns...".to_string(),
        clustering_stage: None,
    });

    let import_result = local_folder::import_from_local_folder_with_analysis(folder_path)
        .map_err(|e| ImportError::FileSystem(format!("Folder import failed: {e}")))?;

    progress_callback(ImportProgress {
        stage: ImportStage::Processing,
        progress: 0.3,
        message: format!(
            "Found {} video files (content type: {:?}, confidence: {:.2})",
            import_result.sections.len(),
            import_result.content_analysis.content_type,
            import_result.content_analysis.confidence_score
        ),
        clustering_stage: None,
    });

    // Stage 3: Create course with structured video metadata
    let raw_titles: Vec<String> = import_result.sections.iter().map(|s| s.title.clone()).collect();
    let videos: Vec<crate::types::VideoMetadata> = import_result.sections.iter().map(|s| {
        let mut video_metadata = crate::types::VideoMetadata::new_local_with_index(
            s.title.clone(),
            s.file_path.clone().unwrap_or_default(),
            s.original_index,
        );
        video_metadata.duration_seconds = Some(s.duration.as_secs_f64());
        video_metadata
    }).collect();
    let mut course = Course::new_with_videos(course_title, videos);

    // Stage 4: Apply appropriate processing based on content analysis
    let processing_decision = determine_processing_strategy(&import_result.content_analysis);
    
    match processing_decision {
        ProcessingStrategy::PreserveSequentialOrder => {
            progress_callback(ImportProgress {
                stage: ImportStage::Processing,
                progress: 0.5,
                message: "Preserving sequential order based on detected patterns...".to_string(),
                clustering_stage: None,
            });

            // Create sequential structure without clustering
            let structure = create_sequential_structure(&import_result)?;
            course.structure = Some(structure);

            progress_callback(ImportProgress {
                stage: ImportStage::Processing,
                progress: 0.8,
                message: "Sequential structure created successfully".to_string(),
                clustering_stage: None,
            });
        }
        ProcessingStrategy::ApplyClusteringAnalysis => {
            progress_callback(ImportProgress {
                stage: ImportStage::TfIdfAnalysis,
                progress: 0.4,
                message: "Applying clustering analysis to thematic content...".to_string(),
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
        }
        ProcessingStrategy::RequestUserChoice => {
            progress_callback(ImportProgress {
                stage: ImportStage::Processing,
                progress: 0.5,
                message: "Content type ambiguous - using fallback sequential processing...".to_string(),
                clustering_stage: None,
            });

            // For now, default to sequential processing when user choice would be needed
            // In a full implementation, this would trigger a user dialog
            let structure = create_sequential_structure(&import_result)?;
            course.structure = Some(structure);

            progress_callback(ImportProgress {
                stage: ImportStage::Processing,
                progress: 0.8,
                message: "Fallback sequential structure created".to_string(),
                clustering_stage: None,
            });
        }
    }

    // Stage 5: Save course with processing metadata
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
            "Successfully imported and structured course: {} (processing: {:?})",
            course.name,
            processing_decision
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

/// Determine processing strategy based on content analysis
fn determine_processing_strategy(analysis: &ContentTypeAnalysis) -> ProcessingStrategy {
    match analysis.recommendation {
        ProcessingRecommendation::PreserveOrder => {
            log::info!("Using sequential processing strategy (confidence: {:.2})", analysis.confidence_score);
            ProcessingStrategy::PreserveSequentialOrder
        }
        ProcessingRecommendation::ApplyClustering => {
            log::info!("Using clustering processing strategy (confidence: {:.2})", analysis.confidence_score);
            ProcessingStrategy::ApplyClusteringAnalysis
        }
        ProcessingRecommendation::UserChoice => {
            log::info!("Content analysis suggests user choice (confidence: {:.2})", analysis.confidence_score);
            // For now, default to sequential processing when user choice would be needed
            // In a full implementation, this would trigger a user dialog
            ProcessingStrategy::RequestUserChoice
        }
        ProcessingRecommendation::FallbackProcessing => {
            log::info!("Using fallback sequential processing (confidence: {:.2})", analysis.confidence_score);
            ProcessingStrategy::RequestUserChoice
        }
    }
}

/// Create sequential structure preserving original order
fn create_sequential_structure(import_result: &local_folder::LocalImportResult) -> Result<crate::types::CourseStructure, ImportError> {
    // Create sections from import result
    let mut sections = Vec::new();
    for (index, local_section) in import_result.sections.iter().enumerate() {
        let section = crate::types::Section {
            title: local_section.title.clone(),
            video_index: index,
            duration: local_section.duration,
        };
        sections.push(section);
    }

    // Calculate total duration
    let total_duration: std::time::Duration = sections.iter().map(|s| s.duration).sum();

    // Create a single module containing all videos in sequential order
    let module = crate::types::Module {
        title: "Course Content".to_string(),
        sections,
        total_duration,
        similarity_score: None,
        topic_keywords: Vec::new(),
        difficulty_level: None,
    };

    // Determine content type string and processing strategy from analysis
    let (content_type_detected, processing_strategy_used, original_order_preserved) = match import_result.content_analysis.content_type {
        ContentType::Sequential => {
            ("Sequential".to_string(), "PreserveOrder".to_string(), true)
        }
        ContentType::Thematic => {
            ("Thematic".to_string(), "PreserveOrder".to_string(), true) // Still preserving order in this case
        }
        ContentType::Mixed => {
            ("Mixed".to_string(), "PreserveOrder".to_string(), true)
        }
        ContentType::Ambiguous => {
            ("Ambiguous".to_string(), "FallbackProcessing".to_string(), true)
        }
    };

    // Create course structure with sequential metadata
    let structure = crate::types::CourseStructure {
        modules: vec![module],
        metadata: crate::types::StructureMetadata {
            total_videos: import_result.sections.len(),
            total_duration,
            estimated_duration_hours: Some(total_duration.as_secs_f32() / 3600.0),
            difficulty_level: Some("Beginner".to_string()),
            structure_quality_score: Some(import_result.content_analysis.confidence_score),
            content_coherence_score: Some(import_result.content_analysis.confidence_score),
            content_type_detected: Some(content_type_detected),
            original_order_preserved: Some(original_order_preserved),
            processing_strategy_used: Some(processing_strategy_used),
        },
        clustering_metadata: None,
    };

    Ok(structure)
}
