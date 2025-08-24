//! Local folder import functionality
//!
//! This module provides functionality to scan local directories for video files
//! and extract their titles and durations for course creation.
//! Enhanced with recursive directory scanning, nested folder support, and sequential pattern detection.

use crate::ImportError;
use crate::nlp::sequential_detection::{
    ContentType, ContentTypeAnalysis, ProcessingRecommendation, detect_sequential_patterns,
};
use crate::storage::{self, database::Database};
use crate::types::Course;
use chrono::Utc;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::Uuid;
use walkdir::WalkDir;

/// Struct representing a local video section with title and duration
#[derive(PartialEq, Debug, Clone)]
pub struct LocalVideoSection {
    pub title: String,
    pub duration: std::time::Duration,
    pub file_path: Option<String>,
    pub original_index: usize, // Preserve import order for sequential detection
}

/// Enhanced import result with content type analysis
#[derive(Debug, Clone)]
pub struct LocalImportResult {
    pub sections: Vec<LocalVideoSection>,
    pub content_analysis: ContentTypeAnalysis,
    pub sorting_applied: SortingMethod,
}

/// Sorting method applied to local video files
#[derive(Debug, Clone, PartialEq)]
pub enum SortingMethod {
    Natural,        // Natural sorting (default)
    Alphabetical,   // Alphabetical sorting
    CreationTime,   // Sorted by file creation time
    PreservedOrder, // Original order preserved due to sequential detection
}

/// Enhanced local ingest with nested folder support
pub struct EnhancedLocalIngest {
    video_extensions: HashSet<String>,
}

impl Default for EnhancedLocalIngest {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedLocalIngest {
    pub fn new() -> Self {
        let mut video_extensions = HashSet::new();
        video_extensions.extend(
            [
                "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ogv", "ts", "mts",
                "m2ts", "mpg", "mpeg", "f4v", "asf", "rm", "rmvb", "vob", "drc",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        Self { video_extensions }
    }

    /// Scans a directory recursively for video files
    pub fn scan_directory_recursive(
        &self,
        root_path: &Path,
        mut progress_callback: Option<&mut dyn FnMut(crate::ingest::ImportProgress)>,
    ) -> Result<Vec<VideoFile>, ImportError> {
        log::info!("Recursively scanning directory: {}", root_path.display());

        if !root_path.exists() {
            return Err(ImportError::FileSystem(format!(
                "Path does not exist: {}",
                root_path.display()
            )));
        }

        if !root_path.is_dir() {
            return Err(ImportError::FileSystem(format!(
                "Path is not a directory: {}",
                root_path.display()
            )));
        }

        // First pass: count total files for progress reporting
        let mut total_files = 0;
        if progress_callback.is_some() {
            if let Some(cb) = progress_callback.as_mut() {
                cb(crate::ingest::ImportProgress {
                    stage: crate::types::ImportStage::Fetching,
                    progress: 0.0,
                    message: "Counting video files...".to_string(),
                    clustering_stage: None,
                });
            }

            for entry in WalkDir::new(root_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if self.is_video_file(entry.path()) {
                    total_files += 1;
                }
            }

            if let Some(cb) = progress_callback.as_mut() {
                cb(crate::ingest::ImportProgress {
                    stage: crate::types::ImportStage::Fetching,
                    progress: 0.0,
                    message: format!("Found {total_files} video files"),
                    clustering_stage: None,
                });
            }
        }

        // Second pass: process files
        let mut video_files = Vec::new();
        let mut processed_files = 0;

        for entry in WalkDir::new(root_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(err) => {
                    log::warn!("Error accessing path: {err}");
                    None
                }
            })
            .filter(|e| e.file_type().is_file())
        {
            if self.is_video_file(entry.path()) {
                processed_files += 1;

                if let Some(cb) = progress_callback.as_mut() {
                    let progress = if total_files > 0 {
                        (processed_files as f32 / total_files as f32) * 100.0
                    } else {
                        0.0
                    };

                    cb(crate::ingest::ImportProgress {
                        stage: crate::types::ImportStage::Processing,
                        progress,
                        message: format!("Processing: {}", entry.path().display()),
                        clustering_stage: None,
                    });
                }

                let video_file = VideoFile {
                    path: entry.path().to_path_buf(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                    relative_path: entry
                        .path()
                        .strip_prefix(root_path)
                        .unwrap_or(entry.path())
                        .to_path_buf(),
                };

                video_files.push(video_file);
            }
        }

        if video_files.is_empty() {
            return Err(ImportError::NoContent);
        }

        log::info!(
            "Found {} video files in {} (recursive)",
            video_files.len(),
            root_path.display()
        );
        Ok(video_files)
    }

    /// Asynchronously scans a directory recursively for video files with cancellation support
    pub async fn scan_directory_recursive_async(
        &self,
        root_path: PathBuf,
        mut progress_callback: Option<
            Box<dyn FnMut(crate::ingest::ImportProgress) + Send + 'static>,
        >,
        batch_size: Option<usize>,
        cancel_token: Option<tokio_util::sync::CancellationToken>,
    ) -> Result<Vec<VideoFile>, ImportError> {
        // Use tokio to avoid blocking the async runtime
        let video_extensions = self.video_extensions.clone();

        tokio::task::spawn_blocking(move || {
            let mut total_files = 0;
            let mut processed_files = 0;
            let mut video_files = Vec::new();

            // First pass: count total files
            if let Some(cb) = progress_callback.as_mut() {
                cb(crate::ingest::ImportProgress {
                    stage: crate::types::ImportStage::Fetching,
                    progress: 0.0,
                    message: "Counting video files...".to_string(),
                    clustering_stage: None,
                });
            }

            for entry in WalkDir::new(&root_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if let Some(ext) = entry.path().extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if video_extensions.contains(&ext_lower) {
                        total_files += 1;
                    }
                }
            }

            if let Some(cb) = progress_callback.as_mut() {
                cb(crate::ingest::ImportProgress {
                    stage: crate::types::ImportStage::Fetching,
                    progress: 0.0,
                    message: format!("Found {total_files} video files"),
                    clustering_stage: None,
                });
            }

            // Second pass: process files in batches if requested
            let batch_size = batch_size.unwrap_or(usize::MAX); // Default to processing all at once
            let entries: Vec<_> = WalkDir::new(&root_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| match e {
                    Ok(entry) => Some(entry),
                    Err(err) => {
                        log::warn!("Error accessing path: {err}");
                        None
                    }
                })
                .filter(|e| e.file_type().is_file())
                .collect();

            // Process in batches
            for chunk in entries.chunks(batch_size) {
                // Check for cancellation between batches
                if let Some(token) = &cancel_token {
                    if token.is_cancelled() {
                        log::info!("Directory scan cancelled by user");
                        return Ok(video_files);
                    }
                }

                for entry in chunk {
                    if let Some(ext) = entry.path().extension() {
                        let ext_lower = ext.to_string_lossy().to_lowercase();
                        if video_extensions.contains(&ext_lower) {
                            processed_files += 1;

                            let progress = if total_files > 0 {
                                (processed_files as f32 / total_files as f32) * 100.0
                            } else {
                                0.0
                            };

                            if let Some(cb) = progress_callback.as_mut() {
                                cb(crate::ingest::ImportProgress {
                                    stage: crate::types::ImportStage::Processing,
                                    progress,
                                    message: format!("Processing: {}", entry.path().display()),
                                    clustering_stage: None,
                                });
                            }

                            let video_file = VideoFile {
                                path: entry.path().to_path_buf(),
                                name: entry.file_name().to_string_lossy().to_string(),
                                size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                                relative_path: entry
                                    .path()
                                    .strip_prefix(&root_path)
                                    .unwrap_or(entry.path())
                                    .to_path_buf(),
                            };

                            video_files.push(video_file);
                        }
                    }
                }

                // Small delay to allow cancellation
                if batch_size < usize::MAX {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }

            if video_files.is_empty() {
                Err(ImportError::NoContent)
            } else {
                Ok(video_files)
            }
        })
        .await
        .unwrap_or_else(|e| Err(ImportError::FileSystem(format!("Join error: {e}"))))
    }

    /// Checks if a file is a video based on its extension
    fn is_video_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            self.video_extensions.contains(&ext)
        } else {
            false
        }
    }
}

/// Represents a video file discovered during scanning
#[derive(Debug, Clone)]
pub struct VideoFile {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub relative_path: PathBuf,
}

/// Legacy function for backward compatibility
pub fn scan_directory(path: &Path) -> Result<Vec<PathBuf>, ImportError> {
    log::info!("Using legacy scan_directory (non-recursive)");
    let ingest = EnhancedLocalIngest::new();
    let video_files = ingest.scan_directory_recursive(path, None)?;
    Ok(video_files.into_iter().map(|vf| vf.path).collect())
}

/// Import video titles and durations from a local folder containing video files with sequential detection
///
/// # Arguments
/// * `path` - The directory path to scan for video files
///
/// # Returns
/// * `Ok(LocalImportResult)` - Import result with sections and content analysis
/// * `Err(ImportError)` - Error if import fails
pub fn import_from_local_folder_with_analysis(
    path: &Path,
) -> Result<LocalImportResult, ImportError> {
    log::info!("Starting local folder analysis for: {}", path.display());

    // Validate that the path exists and is a directory
    if !path.exists() {
        return Err(ImportError::FileSystem(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }

    if !path.is_dir() {
        return Err(ImportError::FileSystem(format!(
            "Path is not a directory: {}",
            path.display()
        )));
    }

    // Use recursive scanning like the validation does
    let mut video_files = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                log::warn!("Error accessing path during analysis: {err}");
                None
            }
        })
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();

        // Skip hidden files and system files
        if is_hidden_or_system_file(&file_path) {
            continue;
        }

        // Check if it's a video file
        if is_video_file(&file_path) {
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    // If we can't get metadata, still include the file but use current time
                    video_files.push(VideoFileInfo {
                        path: file_path.to_path_buf(),
                        created: SystemTime::now(),
                    });
                    continue;
                }
            };

            let created = metadata
                .created()
                .unwrap_or_else(|_| metadata.modified().unwrap_or_else(|_| SystemTime::now()));

            video_files.push(VideoFileInfo {
                path: file_path.to_path_buf(),
                created,
            });
        }
    }

    // Check if we found any video files
    if video_files.is_empty() {
        return Err(ImportError::NoContent);
    }

    // Extract titles first for sequential pattern detection
    let raw_titles: Vec<String> = video_files
        .iter()
        .map(|file_info| {
            extract_title_from_path(&file_info.path).unwrap_or_else(|| {
                file_info
                    .path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
        })
        .collect();

    // Perform content type analysis on raw titles
    let content_analysis = detect_sequential_patterns(&raw_titles);

    log::info!(
        "Local folder content analysis: type={:?}, confidence={:.2}, recommendation={:?}",
        content_analysis.content_type,
        content_analysis.confidence_score,
        content_analysis.recommendation
    );

    // Determine sorting method based on content analysis
    let sorting_method = determine_sorting_method(&content_analysis);

    // Apply appropriate sorting based on content analysis
    match sorting_method {
        SortingMethod::PreservedOrder => {
            // Keep original order when sequential patterns detected
            log::info!("Preserving original file order due to sequential pattern detection");
            // video_files already in directory order, no sorting needed
        }
        SortingMethod::Natural => {
            // Apply natural sorting (default behavior)
            video_files.sort_by(|a, b| natural_sort_compare(&a.path, &b.path));
        }
        SortingMethod::Alphabetical => {
            // Apply alphabetical sorting
            video_files.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
        }
        SortingMethod::CreationTime => {
            // Sort by creation time
            video_files.sort_by(|a, b| a.created.cmp(&b.created));
        }
    }

    // Extract titles and durations from video files with preserved indices
    let mut sections = Vec::new();
    for (index, file_info) in video_files.iter().enumerate() {
        let title = extract_title_from_path(&file_info.path).unwrap_or_else(|| {
            file_info
                .path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });
        let duration = probe_video_duration(&file_info.path)
            .unwrap_or_else(|| std::time::Duration::from_secs(0));
        sections.push(LocalVideoSection {
            title,
            duration,
            file_path: Some(file_info.path.to_string_lossy().to_string()),
            original_index: index, // Preserve order for sequential content
        });
    }

    if sections.is_empty() {
        return Err(ImportError::NoContent);
    }

    Ok(LocalImportResult {
        sections,
        content_analysis,
        sorting_applied: sorting_method,
    })
}

/// Information about a video file for sorting purposes
#[derive(Debug, Clone)]
struct VideoFileInfo {
    path: std::path::PathBuf,
    created: SystemTime,
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

/// Check if a file has a video extension
fn is_video_file(path: &Path) -> bool {
    const VIDEO_EXTENSIONS: &[&str] = &[
        "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "f4v",
        "asf", "rm", "rmvb", "vob", "ogv", "drc", "gif", "gifv", "mng", "qt", "yuv", "mp2", "mpe",
        "mpv", "m2v", "svi", "3g2", "mxf", "roq", "nsv",
    ];

    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return VIDEO_EXTENSIONS.contains(&ext_str.to_lowercase().as_str());
        }
    }
    false
}

/// Check if a file is hidden or a system file
fn is_hidden_or_system_file(path: &Path) -> bool {
    if let Some(filename) = path.file_name() {
        if let Some(filename_str) = filename.to_str() {
            // Skip hidden files (starting with .)
            if filename_str.starts_with('.') {
                return true;
            }

            // Skip common system files
            let system_files = ["Thumbs.db", "Desktop.ini", ".DS_Store"];
            if system_files.contains(&filename_str) {
                return true;
            }
        }
    }
    false
}

/// Extract a meaningful title from a file path
fn extract_title_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(clean_filename_title)
        .filter(|title| !title.is_empty())
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

/// Natural sorting comparison that handles numbers correctly
/// e.g., "video2.mp4" comes before "video10.mp4"
fn natural_sort_compare(a: &Path, b: &Path) -> std::cmp::Ordering {
    let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");

    natural_sort_string_compare(a_name, b_name)
}

/// Compare two strings using natural sorting
fn natural_sort_string_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(a_char), Some(b_char)) => {
                if a_char.is_ascii_digit() && b_char.is_ascii_digit() {
                    // Extract and compare numbers
                    let a_num = extract_number(&mut a_chars);
                    let b_num = extract_number(&mut b_chars);

                    match a_num.cmp(&b_num) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    // Compare characters normally
                    let a_char = a_chars.next().unwrap();
                    let b_char = b_chars.next().unwrap();

                    match a_char.to_lowercase().cmp(b_char.to_lowercase()) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                }
            }
        }
    }
}

/// Extract a number from the character iterator
fn extract_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> u64 {
    let mut number_str = String::new();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            number_str.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    number_str.parse().unwrap_or(0)
}

/// Determine appropriate sorting method based on content analysis
fn determine_sorting_method(analysis: &ContentTypeAnalysis) -> SortingMethod {
    match analysis.recommendation {
        ProcessingRecommendation::PreserveOrder => {
            // High confidence sequential content should preserve order
            if analysis.confidence_score > 0.7 {
                SortingMethod::PreservedOrder
            } else {
                // Medium confidence, use natural sorting as compromise
                SortingMethod::Natural
            }
        }
        ProcessingRecommendation::ApplyClustering => {
            // Thematic content can use natural sorting
            SortingMethod::Natural
        }
        ProcessingRecommendation::UserChoice => {
            // When ambiguous, default to natural sorting
            SortingMethod::Natural
        }
        ProcessingRecommendation::FallbackProcessing => {
            // Fallback to natural sorting
            SortingMethod::Natural
        }
    }
}

/// Import video titles and durations from a local folder containing video files
///
/// Legacy function for backward compatibility - returns only sections
///
/// # Arguments
/// * `path` - The directory path to scan for video files
///
/// # Returns
/// * `Ok(Vec<LocalVideoSection>)` - Vector of video sections (title, duration) in playlist order
/// * `Err(ImportError)` - Error if import fails
pub fn import_from_local_folder(path: &Path) -> Result<Vec<LocalVideoSection>, ImportError> {
    let result = import_from_local_folder_with_analysis(path)?;
    Ok(result.sections)
}

/// Get alternative sorting options for the folder
pub fn get_sorting_options(path: &Path) -> Result<Vec<SortingOption>, ImportError> {
    let entries = fs::read_dir(path)
        .map_err(|e| ImportError::FileSystem(format!("Failed to read directory: {e}")))?;

    let mut video_files = Vec::new();
    for entry in entries.flatten() {
        let file_path = entry.path();
        if is_video_file(&file_path) && !is_hidden_or_system_file(&file_path) {
            if let Ok(metadata) = entry.metadata() {
                let created = metadata
                    .created()
                    .unwrap_or_else(|_| metadata.modified().unwrap_or_else(|_| SystemTime::now()));

                video_files.push(VideoFileInfo {
                    path: file_path,
                    created,
                });
            }
        }
    }

    let mut options = Vec::new();

    // Alphabetical sorting
    let mut alphabetical = video_files.clone();
    alphabetical.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
    options.push(SortingOption {
        name: "Alphabetical".to_string(),
        titles: alphabetical
            .into_iter()
            .filter_map(|f| extract_title_from_path(&f.path))
            .collect(),
    });

    // Natural sorting (default)
    let mut natural = video_files.clone();
    natural.sort_by(|a, b| natural_sort_compare(&a.path, &b.path));
    options.push(SortingOption {
        name: "Natural (Default)".to_string(),
        titles: natural
            .into_iter()
            .filter_map(|f| extract_title_from_path(&f.path))
            .collect(),
    });

    // Creation time sorting
    let mut by_creation = video_files;
    by_creation.sort_by(|a, b| a.created.cmp(&b.created));
    options.push(SortingOption {
        name: "Creation Time".to_string(),
        titles: by_creation
            .into_iter()
            .filter_map(|f| extract_title_from_path(&f.path))
            .collect(),
    });

    Ok(options)
}

/// Import course from local folder with content type analysis and save to database
/// This function integrates with the enhanced sequential detection functionality
pub fn import_from_folder(
    db: &Database,
    folder_path: &Path,
    course_title: &str,
) -> Result<Course, ImportError> {
    log::info!(
        "Starting import from folder: {} with title: {}",
        folder_path.display(),
        course_title
    );

    // Use the enhanced import with content analysis
    let import_result = import_from_local_folder_with_analysis(folder_path)?;

    log::info!(
        "Import analysis completed with {} sections",
        import_result.sections.len()
    );

    if import_result.sections.is_empty() {
        return Err(ImportError::NoContent);
    }

    // Extract raw titles and create sections
    let raw_titles: Vec<String> = import_result
        .sections
        .iter()
        .map(|s| s.title.clone())
        .collect();
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
    let total_duration: std::time::Duration = sections.iter().map(|section| section.duration).sum();

    // Determine content type and processing strategy from analysis
    let (content_type_detected, processing_strategy, original_order_preserved) =
        match import_result.content_analysis.content_type {
            ContentType::Sequential => {
                log::info!(
                    "Local content detected as Sequential with confidence {:.2}",
                    import_result.content_analysis.confidence_score
                );
                ("Sequential".to_string(), "PreserveOrder".to_string(), true)
            }
            ContentType::Thematic => {
                log::info!(
                    "Local content detected as Thematic with confidence {:.2}",
                    import_result.content_analysis.confidence_score
                );
                ("Thematic".to_string(), "ApplyClustering".to_string(), false)
            }
            ContentType::Mixed => {
                log::info!(
                    "Local content detected as Mixed with confidence {:.2}",
                    import_result.content_analysis.confidence_score
                );
                (
                    "Mixed".to_string(),
                    "UserChoice".to_string(),
                    import_result.sorting_applied == SortingMethod::PreservedOrder,
                )
            }
            ContentType::Ambiguous => {
                log::info!(
                    "Local content type ambiguous with confidence {:.2}",
                    import_result.content_analysis.confidence_score
                );
                (
                    "Ambiguous".to_string(),
                    "FallbackProcessing".to_string(),
                    import_result.sorting_applied == SortingMethod::PreservedOrder,
                )
            }
        };

    // Create a single module containing all videos
    let module = crate::types::Module {
        title: "Course Content".to_string(),
        sections,
        total_duration,
        similarity_score: None,
        topic_keywords: Vec::new(),
        difficulty_level: None,
    };

    // Create course structure with content analysis metadata
    let structure = crate::types::CourseStructure {
        modules: vec![module],
        metadata: crate::types::StructureMetadata {
            total_videos: import_result.sections.len(),
            total_duration,
            estimated_duration_hours: Some(total_duration.as_secs_f32() / 3600.0),
            difficulty_level: Some("Beginner".to_string()),
            structure_quality_score: None,
            content_coherence_score: None,
            content_type_detected: Some(content_type_detected),
            original_order_preserved: Some(original_order_preserved),
            processing_strategy_used: Some(processing_strategy),
        },
        clustering_metadata: None,
    };

    // Create video metadata from local sections with preserved indices
    let videos: Vec<crate::types::VideoMetadata> = import_result
        .sections
        .iter()
        .map(|local_section| {
            let mut video_metadata = crate::types::VideoMetadata::new_local_with_index(
                local_section.title.clone(),
                local_section.file_path.clone().unwrap_or_default(),
                local_section.original_index,
            );
            video_metadata.duration_seconds = Some(local_section.duration.as_secs_f64());
            video_metadata
        })
        .collect();

    // Create course
    let course = Course {
        id: Uuid::new_v4(),
        name: course_title.to_string(),
        created_at: Utc::now(),
        raw_titles,
        videos,
        structure: Some(structure),
    };

    // Save course to database
    storage::save_course(db, &course)
        .map_err(|e| ImportError::Database(format!("Failed to save course: {e}")))?;

    log::info!(
        "Successfully imported course '{}' with {} videos from {} (content type: {:?}, sorting: {:?})",
        course_title,
        import_result.sections.len(),
        folder_path.display(),
        import_result.content_analysis.content_type,
        import_result.sorting_applied
    );

    Ok(course)
}

/// Represents different ways to sort the video files
#[derive(Debug, Clone)]
pub struct SortingOption {
    pub name: String,
    pub titles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_video_file_detection() {
        assert!(is_video_file(Path::new("test.mp4")));
        assert!(is_video_file(Path::new("movie.avi")));
        assert!(is_video_file(Path::new("VIDEO.MP4"))); // case insensitive
        assert!(!is_video_file(Path::new("document.pdf")));
        assert!(!is_video_file(Path::new("image.jpg")));
    }

    #[test]
    fn test_hidden_file_detection() {
        assert!(is_hidden_or_system_file(Path::new(".hidden_file")));
        assert!(is_hidden_or_system_file(Path::new("Thumbs.db")));
        assert!(is_hidden_or_system_file(Path::new(".DS_Store")));
        assert!(!is_hidden_or_system_file(Path::new("normal_file.mp4")));
    }

    #[test]
    fn test_title_cleaning() {
        assert_eq!(
            clean_filename_title("My_Video-Title.1080p"),
            "My Video Title"
        );
        assert_eq!(
            clean_filename_title("Lecture 01 - Introduction.mp4"),
            "Lecture 01 Introduction"
        );
        assert_eq!(clean_filename_title("Chapter_2_Part_1"), "Chapter 2 Part 1");
    }

    #[test]
    fn test_natural_sorting() {
        let files = vec!["video1.mp4", "video10.mp4", "video2.mp4", "video20.mp4"];
        let mut sorted_files = files.clone();
        sorted_files.sort_by(|a, b| natural_sort_string_compare(a, b));

        assert_eq!(
            sorted_files,
            vec!["video1.mp4", "video2.mp4", "video10.mp4", "video20.mp4"]
        );
    }

    #[test]
    fn test_number_extraction() {
        let mut chars = "123abc".chars().peekable();
        assert_eq!(extract_number(&mut chars), 123);

        let remaining: String = chars.collect();
        assert_eq!(remaining, "abc");
    }

    #[test]
    fn test_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let result = import_from_local_folder(temp_dir.path());
        assert!(matches!(result, Err(ImportError::NoContent)));
        Ok(())
    }

    #[test]
    fn test_directory_with_videos() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create some test video files
        File::create(temp_dir.path().join("video1.mp4"))?;
        File::create(temp_dir.path().join("video2.avi"))?;
        File::create(temp_dir.path().join("not_a_video.txt"))?;

        let result = import_from_local_folder(temp_dir.path())?;
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|s| s.title == "video1"));
        assert!(result.iter().any(|s| s.title == "video2"));

        Ok(())
    }

    #[test]
    fn test_sequential_pattern_detection() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create sequential video files
        File::create(temp_dir.path().join("Lesson 1 - Introduction.mp4"))?;
        File::create(temp_dir.path().join("Lesson 2 - Basics.mp4"))?;
        File::create(temp_dir.path().join("Lesson 3 - Advanced.mp4"))?;

        let result = import_from_local_folder_with_analysis(temp_dir.path())?;

        // Should detect sequential pattern
        assert_eq!(
            result.content_analysis.content_type,
            ContentType::Sequential
        );
        assert!(result.content_analysis.confidence_score > 0.5);
        assert_eq!(result.sorting_applied, SortingMethod::PreservedOrder);

        // Should have 3 videos with preserved indices
        assert_eq!(result.sections.len(), 3);
        assert!(
            result
                .sections
                .iter()
                .any(|s| s.title.contains("Introduction"))
        );
        assert!(result.sections.iter().any(|s| s.title.contains("Basics")));
        assert!(result.sections.iter().any(|s| s.title.contains("Advanced")));

        Ok(())
    }

    #[test]
    fn test_non_sequential_content() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create non-sequential video files
        File::create(temp_dir.path().join("JavaScript Fundamentals.mp4"))?;
        File::create(temp_dir.path().join("CSS Grid Layout.mp4"))?;
        File::create(temp_dir.path().join("React Components.mp4"))?;

        let result = import_from_local_folder_with_analysis(temp_dir.path())?;

        // Should not detect strong sequential pattern
        assert_ne!(
            result.content_analysis.content_type,
            ContentType::Sequential
        );
        assert_eq!(result.sorting_applied, SortingMethod::Natural);

        Ok(())
    }

    #[test]
    fn test_nonexistent_directory() {
        let result = import_from_local_folder(Path::new("/nonexistent/path"));
        assert!(matches!(result, Err(ImportError::FileSystem(_))));
    }
}
