//! Video player type definitions
//!
//! Core types for video playback that are used across the application.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Video source types supported by the player
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoSource {
    /// Local video file
    Local { path: PathBuf, title: String },
    /// YouTube video
    YouTube { video_id: String, playlist_id: Option<String>, title: String },
}

impl VideoSource {
    /// Get the display title for this video source
    pub fn title(&self) -> &str {
        match self {
            VideoSource::Local { title, .. } => title,
            VideoSource::YouTube { title, .. } => title,
        }
    }

    /// Check if this is a local video source
    pub fn is_local(&self) -> bool {
        matches!(self, VideoSource::Local { .. })
    }

    /// Check if this is a YouTube video source
    pub fn is_youtube(&self) -> bool {
        matches!(self, VideoSource::YouTube { .. })
    }

    /// Get the file path for local videos
    pub fn local_path(&self) -> Option<&PathBuf> {
        match self {
            VideoSource::Local { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Get the YouTube video ID
    pub fn youtube_video_id(&self) -> Option<&str> {
        match self {
            VideoSource::YouTube { video_id, .. } => Some(video_id),
            _ => None,
        }
    }

    /// Get the YouTube playlist ID
    pub fn youtube_playlist_id(&self) -> Option<&str> {
        match self {
            VideoSource::YouTube { playlist_id, .. } => playlist_id.as_deref(),
            _ => None,
        }
    }
}

/// Current playback state of the video player
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Buffering,
    Error,
}

impl PlaybackState {
    /// Check if the player is in an active state
    pub fn is_active(&self) -> bool {
        matches!(self, PlaybackState::Playing | PlaybackState::Buffering)
    }

    /// Check if the player can be resumed
    pub fn can_resume(&self) -> bool {
        matches!(self, PlaybackState::Paused | PlaybackState::Stopped)
    }

    /// Check if the player can be paused
    pub fn can_pause(&self) -> bool {
        matches!(self, PlaybackState::Playing)
    }
}

/// Information about a video being played
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoInfo {
    pub source: VideoSource,
    pub duration_seconds: Option<f64>,
    pub position_seconds: f64,
    pub volume: f64,
    pub is_fullscreen: bool,
}

impl VideoInfo {
    /// Create new video info from a source
    pub fn new(source: VideoSource) -> Self {
        Self {
            source,
            duration_seconds: None,
            position_seconds: 0.0,
            volume: 1.0,
            is_fullscreen: false,
        }
    }

    /// Get the video title
    pub fn title(&self) -> &str {
        self.source.title()
    }

    /// Get progress as a percentage (0.0 to 100.0)
    pub fn progress_percentage(&self) -> f64 {
        self.duration_seconds
            .filter(|&d| d > 0.0)
            .map(|d| (self.position_seconds / d * 100.0).clamp(0.0, 100.0))
            .unwrap_or(0.0)
    }

    /// Format duration as MM:SS or HH:MM:SS
    pub fn format_duration(&self) -> String {
        self.duration_seconds.map(format_seconds).unwrap_or_else(|| "Unknown".to_string())
    }

    /// Format current position as MM:SS or HH:MM:SS
    pub fn format_position(&self) -> String {
        format_seconds(self.position_seconds)
    }
}

/// Format seconds as a human-readable time string
pub fn format_seconds(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes:02}:{secs:02}")
    }
}

/// Video player error types
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPlayerError {
    FileNotFound(PathBuf),
    UnsupportedFormat(String),
    NetworkError(String),
    YouTubeApiError { code: i32, message: String },
    PlaybackError(String),
    InvalidSource(String),
}

impl std::fmt::Display for VideoPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoPlayerError::FileNotFound(path) => {
                write!(f, "Video file not found: {}", path.display())
            },
            VideoPlayerError::UnsupportedFormat(format) => {
                write!(f, "Unsupported format: {format}")
            },
            VideoPlayerError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            VideoPlayerError::YouTubeApiError { code, message } => {
                write!(f, "YouTube error {code}: {message}")
            },
            VideoPlayerError::PlaybackError(msg) => write!(f, "Playback error: {msg}"),
            VideoPlayerError::InvalidSource(msg) => write!(f, "Invalid source: {msg}"),
        }
    }
}

impl std::error::Error for VideoPlayerError {}

/// YouTube player state from the YouTube IFrame API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct YouTubePlayerState {
    pub current_time: f64,
    pub duration: f64,
    pub volume: f64,
    pub is_muted: bool,
    pub playback_rate: f64,
    /// -1: unstarted, 0: ended, 1: playing, 2: paused, 3: buffering, 5: cued
    pub player_state: i32,
}

impl YouTubePlayerState {
    pub fn is_playing(&self) -> bool {
        self.player_state == 1
    }

    pub fn is_paused(&self) -> bool {
        self.player_state == 2
    }

    pub fn is_buffering(&self) -> bool {
        self.player_state == 3
    }

    pub fn has_ended(&self) -> bool {
        self.player_state == 0
    }
}

/// Video metadata (simplified from ingest)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoMetadata {
    pub title: String,
    pub duration_seconds: f64,
}

impl VideoMetadata {
    pub fn new(title: String, duration_seconds: f64) -> Self {
        Self { title, duration_seconds }
    }

    pub fn format_duration(&self) -> String {
        format_seconds(self.duration_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_source() {
        let local =
            VideoSource::Local { path: PathBuf::from("/test.mp4"), title: "Test".to_string() };
        assert!(local.is_local());
        assert!(!local.is_youtube());

        let youtube = VideoSource::YouTube {
            video_id: "abc123".to_string(),
            playlist_id: None,
            title: "YouTube".to_string(),
        };
        assert!(!youtube.is_local());
        assert!(youtube.is_youtube());
    }

    #[test]
    fn test_format_seconds() {
        assert_eq!(format_seconds(30.0), "00:30");
        assert_eq!(format_seconds(90.0), "01:30");
        assert_eq!(format_seconds(3661.0), "01:01:01");
    }
}
