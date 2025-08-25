//! MIME type utilities for video files
//!
//! Provides comprehensive MIME type detection for various video formats
//! using the mime_guess crate with fallbacks for common video types.

use mime_guess::MimeGuess;
use std::path::Path;

/// Get the MIME type for a video file based on its extension
pub fn get_video_mime_type(file_path: &Path) -> String {
    // First try mime_guess for comprehensive detection
    if let Some(mime) = MimeGuess::from_path(file_path).first() {
        let mime_str = mime.to_string();
        // Only return if it's a video type
        if mime_str.starts_with("video/") {
            return mime_str;
        }
    }

    // Fallback to manual mapping for common video formats
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("mp4") | Some("m4v") => "video/mp4",
        Some("webm") => "video/webm",
        Some("ogg") | Some("ogv") => "video/ogg",
        Some("mov") | Some("qt") => "video/quicktime",
        Some("avi") => "video/x-msvideo",
        Some("mkv") => "video/x-matroska",
        Some("3gp") => "video/3gpp",
        Some("3g2") => "video/3gpp2",
        Some("flv") => "video/x-flv",
        Some("wmv") => "video/x-ms-wmv",
        Some("asf") => "video/x-ms-asf",
        Some("rm") | Some("rmvb") => "video/vnd.rn-realvideo",
        Some("mpg") | Some("mpeg") | Some("mpe") => "video/mpeg",
        Some("ts") => "video/mp2t",
        Some("m2v") => "video/mpeg",
        Some("divx") => "video/x-msvideo",
        Some("xvid") => "video/x-msvideo",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Check if a file extension is a supported video format
pub fn is_video_file(file_path: &Path) -> bool {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => is_supported_video_extension(ext),
        None => false,
    }
}

/// Check if an extension string represents a supported video format
pub fn is_supported_video_extension(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "mp4"
            | "m4v"
            | "webm"
            | "ogg"
            | "ogv"
            | "mov"
            | "qt"
            | "avi"
            | "mkv"
            | "3gp"
            | "3g2"
            | "flv"
            | "wmv"
            | "asf"
            | "rm"
            | "rmvb"
            | "mpg"
            | "mpeg"
            | "mpe"
            | "ts"
            | "m2v"
            | "divx"
            | "xvid"
    )
}

/// Get a list of all supported video file extensions
pub fn supported_video_extensions() -> Vec<&'static str> {
    vec![
        // Common and previously supported
        "mp4", "m4v", "webm", "ogg", "ogv", "mov", "qt", "avi", "mkv", "3gp", "3g2", "flv", "wmv",
        "asf", "rm", "rmvb", "mpg", "mpeg", "mpe", "ts", "m2v", "divx", "xvid",
        // Unified with ingest module coverage
        "mp2", "mpv", "mts", "m2ts", "yuv", "drc", "svi", "mxf", "roq", "nsv", "f4v", "f4p", "f4a",
        "f4b", "vob",
    ]
}

/// Get a user-friendly description of supported formats
pub fn supported_formats_description() -> String {
    "MP4, WebM, OGG, MOV, AVI, MKV, FLV, WMV, MPEG, and other common video formats".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_video_mime_type_common_formats() {
        assert_eq!(get_video_mime_type(&PathBuf::from("test.mp4")), "video/mp4");
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.webm")),
            "video/webm"
        );
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.avi")),
            "video/x-msvideo"
        );
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.mkv")),
            "video/x-matroska"
        );
    }

    #[test]
    fn test_get_video_mime_type_case_insensitive() {
        assert_eq!(get_video_mime_type(&PathBuf::from("test.MP4")), "video/mp4");
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.WEBM")),
            "video/webm"
        );
    }

    #[test]
    fn test_get_video_mime_type_unknown() {
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.unknown")),
            "application/octet-stream"
        );
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.txt")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_is_video_file() {
        assert!(is_video_file(&PathBuf::from("test.mp4")));
        assert!(is_video_file(&PathBuf::from("test.webm")));
        assert!(is_video_file(&PathBuf::from("test.MP4")));
        assert!(!is_video_file(&PathBuf::from("test.txt")));
        assert!(!is_video_file(&PathBuf::from("test")));
    }

    #[test]
    fn test_is_supported_video_extension() {
        assert!(is_supported_video_extension("mp4"));
        assert!(is_supported_video_extension("MP4"));
        assert!(is_supported_video_extension("webm"));
        assert!(is_supported_video_extension("avi"));
        assert!(!is_supported_video_extension("txt"));
        assert!(!is_supported_video_extension(""));
    }

    #[test]
    fn test_supported_video_extensions() {
        let extensions = supported_video_extensions();
        assert!(extensions.contains(&"mp4"));
        assert!(extensions.contains(&"webm"));
        assert!(extensions.contains(&"avi"));
        assert!(extensions.len() > 10);
    }

    #[test]
    fn test_supported_formats_description() {
        let description = supported_formats_description();
        assert!(description.contains("MP4"));
        assert!(description.contains("WebM"));
        assert!(description.contains("common video formats"));
    }
}
