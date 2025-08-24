//! Video protocol module for handling local video file serving with HTTP Range support
//!
//! This module provides efficient video streaming by supporting HTTP Range requests,
//! allowing for seeking and partial content delivery without loading entire files into memory.

use anyhow::{Result, anyhow};
use dioxus_desktop::wry::http::{HeaderMap, Response, StatusCode, header};
use log::{debug, error, info, warn};
use std::borrow::Cow;

pub mod mime_types;
pub mod range;

pub use mime_types::*;
pub use range::*;

/// Handle local video protocol requests with HTTP Range support
pub fn handle_video_request(uri: &str, headers: &HeaderMap) -> Response<Cow<'static, [u8]>> {
    debug!("Processing video request: {}", uri);

    // Extract file path from URI (format: local-video://file/path/to/video.mp4)
    let file_path = match extract_file_path(uri) {
        Ok(path) => path,
        Err(e) => {
            warn!("Invalid URI format: {} - Error: {}", uri, e);
            return error_response(StatusCode::BAD_REQUEST, "Invalid URI format");
        }
    };

    // Verify file exists and is readable
    if !file_path.exists() {
        warn!("Video file not found: {}", file_path.display());
        return error_response(StatusCode::NOT_FOUND, "Video file not found");
    }

    // Get file metadata
    let metadata = match file_path.metadata() {
        Ok(meta) => meta,
        Err(e) => {
            error!(
                "Failed to read file metadata for {}: {}",
                file_path.display(),
                e
            );
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read file metadata",
            );
        }
    };

    let file_size = metadata.len();
    let mime_type = get_video_mime_type(&file_path);

    info!(
        "Serving video: {} ({} bytes, type: {})",
        file_path.display(),
        file_size,
        mime_type
    );

    // Check for Range header to support partial content requests
    let range_header = headers.get(header::RANGE).and_then(|h| h.to_str().ok());

    // Use range support for efficient streaming
    match range::handle_range_request(&file_path, file_size, &mime_type, range_header) {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to serve video file {}: {}", file_path.display(), e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serve video file",
            )
        }
    }
}

/// Extract file path from the custom protocol URI
fn extract_file_path(uri: &str) -> Result<std::path::PathBuf> {
    if let Some(path_part) = uri.strip_prefix("local-video://file/") {
        Ok(std::path::PathBuf::from(path_part))
    } else {
        Err(anyhow!("URI does not match expected format: {}", uri))
    }
}

/// Create an error response with the given status and message
fn error_response(status: StatusCode, message: &str) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Cow::Owned(message.as_bytes().to_vec()))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_file_path() {
        let uri = "local-video://file/path/to/video.mp4";
        let result = extract_file_path(uri).unwrap();
        assert_eq!(result, PathBuf::from("path/to/video.mp4"));
    }

    #[test]
    fn test_extract_file_path_invalid() {
        let uri = "invalid://file/path";
        assert!(extract_file_path(uri).is_err());
    }

    #[test]
    fn test_get_video_mime_type() {
        assert_eq!(get_video_mime_type(&PathBuf::from("test.mp4")), "video/mp4");
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.webm")),
            "video/webm"
        );
        assert_eq!(
            get_video_mime_type(&PathBuf::from("test.unknown")),
            "application/octet-stream"
        );
    }
}
