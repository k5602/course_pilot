//! Video source value object.
//!
//! Supports multiple origin types (YouTube, local filesystem).

use std::path::Path;

use crate::domain::value_objects::YouTubeVideoId;

/// Error when constructing a video source.
#[derive(Debug, thiserror::Error)]
pub enum VideoSourceError {
    #[error("Invalid local path: {0}")]
    InvalidLocalPath(String),
}

/// Video source for a video entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VideoSource {
    /// YouTube-backed video.
    YouTube(YouTubeVideoId),
    /// Local file path (absolute).
    LocalPath(String),
}

impl VideoSource {
    /// Creates a YouTube-backed source.
    pub fn youtube(id: YouTubeVideoId) -> Self {
        Self::YouTube(id)
    }

    /// Creates a local path source.
    pub fn local_path(path: impl AsRef<str>) -> Result<Self, VideoSourceError> {
        let raw = path.as_ref().trim();
        if raw.is_empty() {
            return Err(VideoSourceError::InvalidLocalPath("path is empty".to_string()));
        }

        let path = Path::new(raw);
        if !path.is_absolute() {
            return Err(VideoSourceError::InvalidLocalPath("path must be absolute".to_string()));
        }

        Ok(Self::LocalPath(raw.to_string()))
    }

    /// Returns the YouTube ID if the source is YouTube-backed.
    pub fn youtube_id(&self) -> Option<&YouTubeVideoId> {
        match self {
            Self::YouTube(id) => Some(id),
            _ => None,
        }
    }

    /// Returns the local path if the source is file-backed.
    pub fn local_path_str(&self) -> Option<&str> {
        match self {
            Self::LocalPath(path) => Some(path.as_str()),
            _ => None,
        }
    }

    /// Returns the source type label for persistence.
    pub fn source_type(&self) -> &'static str {
        match self {
            Self::YouTube(_) => "youtube",
            Self::LocalPath(_) => "local",
        }
    }

    /// Returns the source reference for persistence.
    pub fn source_ref(&self) -> &str {
        match self {
            Self::YouTube(id) => id.as_str(),
            Self::LocalPath(path) => path.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_path_rejects_empty() {
        assert!(VideoSource::local_path("").is_err());
    }

    #[test]
    fn local_path_rejects_whitespace_only() {
        assert!(VideoSource::local_path("   ").is_err());
    }

    #[test]
    fn local_path_rejects_relative() {
        assert!(VideoSource::local_path("videos/course.mp4").is_err());
    }

    #[test]
    fn local_path_accepts_absolute() {
        let src = VideoSource::local_path("/home/user/videos/course.mp4").unwrap();
        assert_eq!(src.local_path_str(), Some("/home/user/videos/course.mp4"));
    }

    #[test]
    fn youtube_source_returns_id() {
        let yt_id = YouTubeVideoId::new("dQw4w9WgXcQ").unwrap();
        let src = VideoSource::youtube(yt_id);
        assert!(src.youtube_id().is_some());
        assert!(src.local_path_str().is_none());
        assert_eq!(src.source_type(), "youtube");
    }

    #[test]
    fn local_source_type_and_ref() {
        let src = VideoSource::local_path("/tmp/test.mp4").unwrap();
        assert_eq!(src.source_type(), "local");
        assert_eq!(src.source_ref(), "/tmp/test.mp4");
    }
}
