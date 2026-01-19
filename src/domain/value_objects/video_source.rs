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
