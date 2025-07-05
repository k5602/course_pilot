//! Local folder import functionality
//!
//! This module provides functionality to scan local directories for video files
//! and extract their titles for course creation.

use crate::ImportError;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Import video titles from a local folder containing video files
///
/// # Arguments
/// * `path` - The directory path to scan for video files
///
/// # Returns
/// * `Ok(Vec<String>)` - Vector of video titles derived from filenames
/// * `Err(ImportError)` - Error if import fails
///
/// # Errors
/// * `ImportError::FileSystem` - If the directory cannot be read or doesn't exist
/// * `ImportError::NoContent` - If no video files are found in the directory
pub fn import_from_local_folder(path: &Path) -> Result<Vec<String>, ImportError> {
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

    // Read directory entries
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(ImportError::FileSystem(format!(
                "Failed to read directory {}: {}",
                path.display(),
                e
            )));
        }
    };

    // Collect video files with metadata for sorting
    let mut video_files = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Warning: Failed to read directory entry: {}", e);
                continue;
            }
        };

        let file_path = entry.path();

        // Skip directories, hidden files, and system files
        if file_path.is_dir() || is_hidden_or_system_file(&file_path) {
            continue;
        }

        // Check if it's a video file
        if is_video_file(&file_path) {
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => {
                    // If we can't get metadata, still include the file but use current time
                    video_files.push(VideoFileInfo {
                        path: file_path,
                        created: SystemTime::now(),
                    });
                    continue;
                }
            };

            let created = metadata
                .created()
                .unwrap_or_else(|_| metadata.modified().unwrap_or_else(|_| SystemTime::now()));

            video_files.push(VideoFileInfo {
                path: file_path,
                created,
            });
        }
    }

    // Check if we found any video files
    if video_files.is_empty() {
        return Err(ImportError::NoContent);
    }

    // Sort files using natural sorting (handles numbers properly)
    video_files.sort_by(|a, b| natural_sort_compare(&a.path, &b.path));

    // Extract titles from filenames
    let titles: Vec<String> = video_files
        .into_iter()
        .filter_map(|file_info| extract_title_from_path(&file_info.path))
        .collect();

    if titles.is_empty() {
        return Err(ImportError::NoContent);
    }

    Ok(titles)
}

/// Information about a video file for sorting purposes
#[derive(Debug, Clone)]
struct VideoFileInfo {
    path: std::path::PathBuf,
    created: SystemTime,
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
        .map(|title| clean_filename_title(title))
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

/// Get alternative sorting options for the folder
pub fn get_sorting_options(path: &Path) -> Result<Vec<SortingOption>, ImportError> {
    let entries = fs::read_dir(path)
        .map_err(|e| ImportError::FileSystem(format!("Failed to read directory: {}", e)))?;

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
            clean_filename_title("Lecture 01 - Introduction [HD]"),
            "Lecture 01"
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
        assert!(result.contains(&"video1".to_string()));
        assert!(result.contains(&"video2".to_string()));

        Ok(())
    }

    #[test]
    fn test_nonexistent_directory() {
        let result = import_from_local_folder(Path::new("/nonexistent/path"));
        assert!(matches!(result, Err(ImportError::FileSystem(_))));
    }
}
