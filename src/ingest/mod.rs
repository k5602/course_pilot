//! Data ingestion module for Course Pilot
//!
//! This module provides functionality for importing course content from various sources
//! including YouTube playlists and local video folders.

pub mod local_folder;
pub mod youtube;

// Re-export main import functions
pub use local_folder::import_from_local_folder;
pub use youtube::import_from_youtube;

// Re-export error types
pub use crate::ImportError;

// Common validation utilities
use std::path::Path;

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
