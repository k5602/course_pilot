//! HTTP Range request handling for video streaming
//!
//! This module implements HTTP Range support for efficient video streaming,
//! allowing clients to request specific byte ranges of video files for seeking
//! and progressive download without loading entire files into memory.

use anyhow::{Result, anyhow};
use dioxus_desktop::wry::http::{Response, StatusCode, header};
use http_range::HttpRange;
use log::{debug, warn};
use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Maximum chunk size for range requests (1MB)
const MAX_CHUNK_SIZE: u64 = 1024 * 1024;

/// Handle HTTP Range requests for video files
pub fn handle_range_request(
    file_path: &Path,
    file_size: u64,
    mime_type: &str,
    range_header: Option<&str>,
) -> Result<Response<Cow<'static, [u8]>>> {
    match range_header {
        Some(range_str) => serve_range(file_path, file_size, mime_type, range_str),
        None => serve_entire_file(file_path, file_size, mime_type),
    }
}

/// Parse and serve a specific byte range
fn serve_range(
    file_path: &Path,
    file_size: u64,
    mime_type: &str,
    range_str: &str,
) -> Result<Response<Cow<'static, [u8]>>> {
    debug!(
        "Processing range request: {} for file: {}",
        range_str,
        file_path.display()
    );

    // Parse the Range header
    let ranges = match HttpRange::parse(range_str, file_size) {
        Ok(ranges) => ranges,
        Err(e) => {
            warn!("Invalid range header '{}': {:?}", range_str, e);
            return Err(anyhow!("Invalid range header: {:?}", e));
        }
    };

    // We only support single range requests for now
    if ranges.len() != 1 {
        warn!(
            "Multiple ranges not supported: {} ranges requested",
            ranges.len()
        );
        return serve_entire_file(file_path, file_size, mime_type);
    }

    let range = &ranges[0];
    let start = range.start;
    let end = range.start + range.length - 1;
    let content_length = range.length;

    // Validate range bounds
    if start >= file_size || end >= file_size {
        warn!(
            "Range out of bounds: {}-{} for file size {}",
            start, end, file_size
        );
        return Err(anyhow!("Range out of bounds"));
    }

    // Limit chunk size for memory efficiency
    let actual_length = if content_length > MAX_CHUNK_SIZE {
        warn!(
            "Range too large ({}), limiting to {}",
            content_length, MAX_CHUNK_SIZE
        );
        MAX_CHUNK_SIZE
    } else {
        content_length
    };

    debug!(
        "Serving range {}-{} ({} bytes) from {}",
        start,
        start + actual_length - 1,
        actual_length,
        file_path.display()
    );

    // Read the requested range
    let content = read_file_range(file_path, start, actual_length)?;

    // Build the response
    Ok(Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_LENGTH, actual_length.to_string())
        .header(
            header::CONTENT_RANGE,
            format!(
                "bytes {}-{}/{}",
                start,
                start + actual_length - 1,
                file_size
            ),
        )
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Cow::Owned(content))
        .unwrap())
}

/// Serve the entire file when no range is requested
fn serve_entire_file(
    file_path: &Path,
    file_size: u64,
    mime_type: &str,
) -> Result<Response<Cow<'static, [u8]>>> {
    debug!(
        "Serving entire file: {} ({} bytes)",
        file_path.display(),
        file_size
    );

    // For large files, we might want to stream instead of loading everything
    if file_size > MAX_CHUNK_SIZE * 10 {
        warn!(
            "Large file ({} bytes), consider implementing streaming",
            file_size
        );
    }

    let content = read_entire_file(file_path)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Cow::Owned(content))
        .unwrap())
}

/// Read a specific range of bytes from a file
fn read_file_range(file_path: &Path, start: u64, length: u64) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;

    // Seek to the start position
    file.seek(SeekFrom::Start(start))?;

    // Read the requested number of bytes
    let mut buffer = vec![0u8; length as usize];
    let bytes_read = file.read(&mut buffer)?;

    // Trim buffer to actual bytes read
    buffer.truncate(bytes_read);

    Ok(buffer)
}

/// Read the entire file into memory
fn read_entire_file(file_path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

/// Check if a range request is satisfiable for the given file size
pub fn is_range_satisfiable(range_str: &str, file_size: u64) -> bool {
    match HttpRange::parse(range_str, file_size) {
        Ok(ranges) => !ranges.is_empty(),
        Err(_) => false,
    }
}

/// Get the total length that would be served for a range request
pub fn get_range_length(range_str: &str, file_size: u64) -> Option<u64> {
    match HttpRange::parse(range_str, file_size) {
        Ok(ranges) if ranges.len() == 1 => {
            let range = &ranges[0];
            Some(std::cmp::min(range.length, MAX_CHUNK_SIZE))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_read_file_range() {
        let content = b"Hello, World! This is a test file.";
        let file = create_test_file(content);

        // Test reading a range
        let result = read_file_range(file.path(), 7, 5).unwrap();
        assert_eq!(result, b"World");

        // Test reading from the beginning
        let result = read_file_range(file.path(), 0, 5).unwrap();
        assert_eq!(result, b"Hello");

        // Test reading to the end
        let result = read_file_range(file.path(), 29, 5).unwrap();
        assert_eq!(result, b"file.");
    }

    #[test]
    fn test_read_entire_file() {
        let content = b"Complete file content";
        let file = create_test_file(content);

        let result = read_entire_file(file.path()).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_is_range_satisfiable() {
        assert!(is_range_satisfiable("bytes=0-499", 1000));
        assert!(is_range_satisfiable("bytes=500-999", 1000));
        assert!(is_range_satisfiable("bytes=-500", 1000));
        assert!(!is_range_satisfiable("bytes=1000-1499", 1000));
        assert!(!is_range_satisfiable("invalid", 1000));
    }

    #[test]
    fn test_get_range_length() {
        assert_eq!(get_range_length("bytes=0-499", 1000), Some(500));
        assert_eq!(get_range_length("bytes=500-999", 1000), Some(500));
        assert_eq!(get_range_length("bytes=-100", 1000), Some(100));
        assert_eq!(get_range_length("invalid", 1000), None);

        // Test max chunk size limiting
        let large_size = MAX_CHUNK_SIZE * 2;
        let result = get_range_length(&format!("bytes=0-{}", large_size - 1), large_size);
        assert_eq!(result, Some(MAX_CHUNK_SIZE));
    }

    #[test]
    fn test_serve_range_basic() {
        let content = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let file = create_test_file(content);

        let response = serve_range(
            file.path(),
            content.len() as u64,
            "video/mp4",
            "bytes=10-19",
        )
        .unwrap();

        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(response.body().as_ref(), b"abcdefghij");
    }

    #[test]
    fn test_serve_entire_file_response() {
        let content = b"Small test file";
        let file = create_test_file(content);

        let response = serve_entire_file(file.path(), content.len() as u64, "video/mp4").unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body().as_ref(), content);

        // Check headers
        let headers = response.headers();
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "video/mp4");
        assert_eq!(headers.get(header::ACCEPT_RANGES).unwrap(), "bytes");
    }
}
