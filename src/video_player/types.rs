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

    /// Get the YouTube video ID for YouTube videos
    pub fn youtube_video_id(&self) -> Option<&str> {
        match self {
            VideoSource::YouTube { video_id, .. } => Some(video_id),
            _ => None,
        }
    }

    /// Get the YouTube playlist ID for YouTube videos
    pub fn youtube_playlist_id(&self) -> Option<&str> {
        match self {
            VideoSource::YouTube { playlist_id, .. } => playlist_id.as_deref(),
            _ => None,
        }
    }
}

/// Current playback state of the video player
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// Player is stopped
    Stopped,
    /// Player is playing
    Playing,
    /// Player is paused
    Paused,
    /// Player is buffering
    Buffering,
    /// Player encountered an error
    Error,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState::Stopped
    }
}

impl PlaybackState {
    /// Check if the player is in an active state (playing or buffering)
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
    /// The video source
    pub source: VideoSource,
    /// Duration in seconds (if known)
    pub duration_seconds: Option<f64>,
    /// Current playback position in seconds
    pub position_seconds: f64,
    /// Current volume (0.0 to 1.0)
    pub volume: f64,
    /// Whether the player is in fullscreen mode
    pub is_fullscreen: bool,
    /// Video resolution width (if known)
    pub width: Option<u32>,
    /// Video resolution height (if known)
    pub height: Option<u32>,
    /// Video bitrate in kbps (if known)
    pub bitrate_kbps: Option<u32>,
    /// Video frame rate (if known)
    pub frame_rate: Option<f32>,
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
            width: None,
            height: None,
            bitrate_kbps: None,
            frame_rate: None,
        }
    }

    /// Get the video title
    pub fn title(&self) -> &str {
        self.source.title()
    }

    /// Get progress as a percentage (0.0 to 100.0)
    pub fn progress_percentage(&self) -> f64 {
        if let Some(duration) = self.duration_seconds {
            if duration > 0.0 {
                (self.position_seconds / duration * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> Option<f64> {
        self.duration_seconds.map(|duration| (duration - self.position_seconds).max(0.0))
    }

    /// Format duration as MM:SS or HH:MM:SS
    pub fn format_duration(&self) -> String {
        if let Some(duration) = self.duration_seconds {
            format_seconds(duration)
        } else {
            "Unknown".to_string()
        }
    }

    /// Format current position as MM:SS or HH:MM:SS
    pub fn format_position(&self) -> String {
        format_seconds(self.position_seconds)
    }

    /// Format remaining time as MM:SS or HH:MM:SS
    pub fn format_remaining(&self) -> String {
        if let Some(remaining) = self.remaining_seconds() {
            format_seconds(remaining)
        } else {
            "Unknown".to_string()
        }
    }

    /// Get video resolution as a string (e.g., "1920x1080")
    pub fn resolution_string(&self) -> Option<String> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Some(format!("{w}x{h}")),
            _ => None,
        }
    }

    /// Check if this is a high-definition video
    pub fn is_hd(&self) -> bool {
        match (self.width, self.height) {
            (Some(w), Some(h)) => w >= 1280 && h >= 720,
            _ => false,
        }
    }

    /// Check if this is a 4K video
    pub fn is_4k(&self) -> bool {
        match (self.width, self.height) {
            (Some(w), Some(h)) => w >= 3840 && h >= 2160,
            _ => false,
        }
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

/// Video player capabilities
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerCapabilities {
    /// Supported video formats
    pub supported_formats: Vec<String>,
    /// Whether the player supports seeking
    pub supports_seeking: bool,
    /// Whether the player supports volume control
    pub supports_volume_control: bool,
    /// Whether the player supports fullscreen
    pub supports_fullscreen: bool,
    /// Whether the player supports playback rate control
    pub supports_playback_rate: bool,
    /// Available playback rates
    pub available_playback_rates: Vec<f64>,
}

impl Default for PlayerCapabilities {
    fn default() -> Self {
        Self {
            supported_formats: vec![
                "mp4".to_string(),
                "avi".to_string(),
                "mov".to_string(),
                "mkv".to_string(),
                "webm".to_string(),
            ],
            supports_seeking: true,
            supports_volume_control: true,
            supports_fullscreen: true,
            supports_playback_rate: true,
            available_playback_rates: vec![0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0],
        }
    }
}

/// Video player error types
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPlayerError {
    // File-related errors
    FileNotFound(std::path::PathBuf),
    FileAccessDenied(std::path::PathBuf),
    UnsupportedFormat(String),

    // Network-related errors
    NetworkError(String),
    YouTubeApiError { code: i32, message: String },

    // Player-related errors
    InitializationFailed(String),
    PlaybackError(String),
    WebViewError(String),

    // Validation errors
    InvalidVideoId(String),
    InvalidSource(String),
}

impl std::fmt::Display for VideoPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoPlayerError::FileNotFound(path) => {
                write!(f, "Video file not found: {}", path.display())
            },
            VideoPlayerError::FileAccessDenied(path) => {
                write!(f, "Cannot access video file: {}", path.display())
            },
            VideoPlayerError::UnsupportedFormat(format) => {
                write!(f, "Unsupported video format: {format}")
            },
            VideoPlayerError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            VideoPlayerError::YouTubeApiError { code, message } => {
                write!(f, "YouTube API error {code}: {message}")
            },
            VideoPlayerError::InitializationFailed(msg) => {
                write!(f, "Player initialization failed: {msg}")
            },
            VideoPlayerError::PlaybackError(msg) => write!(f, "Playback error: {msg}"),
            VideoPlayerError::WebViewError(msg) => write!(f, "WebView error: {msg}"),
            VideoPlayerError::InvalidVideoId(id) => write!(f, "Invalid video ID: {id}"),
            VideoPlayerError::InvalidSource(msg) => write!(f, "Invalid video source: {msg}"),
        }
    }
}

impl std::error::Error for VideoPlayerError {}

/// YouTube player state information from the YouTube API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YouTubePlayerState {
    /// Current playback time in seconds
    pub current_time: f64,
    /// Total video duration in seconds
    pub duration: f64,
    /// Current volume (0.0 to 1.0)
    pub volume: f64,
    /// Whether the player is muted
    pub is_muted: bool,
    /// Playback rate (e.g., 1.0 for normal speed)
    pub playback_rate: f64,
    /// YouTube player state number (-1: unstarted, 0: ended, 1: playing, 2: paused, 3: buffering, 5: video cued)
    pub player_state: i32,
    /// Video quality (e.g., "hd720", "large", "medium", "small")
    pub quality: Option<String>,
}

impl YouTubePlayerState {
    /// Create a new YouTube player state with default values
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            duration: 0.0,
            volume: 1.0,
            is_muted: false,
            playback_rate: 1.0,
            player_state: -1, // Unstarted
            quality: None,
        }
    }

    /// Check if the player is playing
    pub fn is_playing(&self) -> bool {
        self.player_state == 1
    }

    /// Check if the player is paused
    pub fn is_paused(&self) -> bool {
        self.player_state == 2
    }

    /// Check if the player is buffering
    pub fn is_buffering(&self) -> bool {
        self.player_state == 3
    }

    /// Check if the video has ended
    pub fn has_ended(&self) -> bool {
        self.player_state == 0
    }

    /// Get a human-readable state description
    pub fn state_description(&self) -> &'static str {
        match self.player_state {
            -1 => "Unstarted",
            0 => "Ended",
            1 => "Playing",
            2 => "Paused",
            3 => "Buffering",
            5 => "Video Cued",
            _ => "Unknown",
        }
    }
}

impl Default for YouTubePlayerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Video metadata extracted from files or URLs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoMetadata {
    /// Video title
    pub title: String,
    /// Duration in seconds
    pub duration_seconds: f64,
    /// Video width in pixels
    pub width: u32,
    /// Video height in pixels
    pub height: u32,
    /// Video bitrate in kbps
    pub bitrate_kbps: u32,
    /// Frame rate
    pub frame_rate: f32,
    /// Video codec
    pub codec: Option<String>,
    /// Audio codec
    pub audio_codec: Option<String>,
    /// File size in bytes (for local files)
    pub file_size_bytes: Option<u64>,
    /// Thumbnail URL or path
    pub thumbnail: Option<String>,
    /// Video description
    pub description: Option<String>,
    /// Upload date (for online videos)
    pub upload_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Channel or author name
    pub author: Option<String>,
    /// Video tags
    pub tags: Vec<String>,
}

impl VideoMetadata {
    /// Create basic metadata with just title and duration
    pub fn basic(title: String, duration_seconds: f64) -> Self {
        Self {
            title,
            duration_seconds,
            width: 0,
            height: 0,
            bitrate_kbps: 0,
            frame_rate: 0.0,
            codec: None,
            audio_codec: None,
            file_size_bytes: None,
            thumbnail: None,
            description: None,
            upload_date: None,
            author: None,
            tags: Vec::new(),
        }
    }

    /// Get resolution as a string
    pub fn resolution_string(&self) -> String {
        if self.width > 0 && self.height > 0 {
            format!("{}x{}", self.width, self.height)
        } else {
            "Unknown".to_string()
        }
    }

    /// Check if this is HD video
    pub fn is_hd(&self) -> bool {
        self.width >= 1280 && self.height >= 720
    }

    /// Check if this is 4K video
    pub fn is_4k(&self) -> bool {
        self.width >= 3840 && self.height >= 2160
    }

    /// Format duration as human-readable string
    pub fn format_duration(&self) -> String {
        format_seconds(self.duration_seconds)
    }

    /// Format file size as human-readable string
    pub fn format_file_size(&self) -> String {
        if let Some(size) = self.file_size_bytes {
            format_bytes(size)
        } else {
            "Unknown".to_string()
        }
    }
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_source_creation() {
        let local = VideoSource::Local {
            path: PathBuf::from("/path/to/video.mp4"),
            title: "Test Video".to_string(),
        };
        assert!(local.is_local());
        assert!(!local.is_youtube());
        assert_eq!(local.title(), "Test Video");

        let youtube = VideoSource::YouTube {
            video_id: "dQw4w9WgXcQ".to_string(),
            playlist_id: None,
            title: "YouTube Video".to_string(),
        };
        assert!(!youtube.is_local());
        assert!(youtube.is_youtube());
        assert_eq!(youtube.title(), "YouTube Video");
    }

    #[test]
    fn test_playback_state() {
        assert!(PlaybackState::Playing.is_active());
        assert!(PlaybackState::Buffering.is_active());
        assert!(!PlaybackState::Stopped.is_active());
        assert!(!PlaybackState::Paused.is_active());

        assert!(PlaybackState::Paused.can_resume());
        assert!(PlaybackState::Stopped.can_resume());
        assert!(!PlaybackState::Playing.can_resume());

        assert!(PlaybackState::Playing.can_pause());
        assert!(!PlaybackState::Paused.can_pause());
    }

    #[test]
    fn test_video_info() {
        let source =
            VideoSource::Local { path: PathBuf::from("/test.mp4"), title: "Test".to_string() };
        let mut info = VideoInfo::new(source);
        info.duration_seconds = Some(120.0);
        info.position_seconds = 60.0;

        assert_eq!(info.progress_percentage(), 50.0);
        assert_eq!(info.remaining_seconds(), Some(60.0));
        assert_eq!(info.format_position(), "01:00");
        assert_eq!(info.format_duration(), "02:00");
    }

    #[test]
    fn test_format_seconds() {
        assert_eq!(format_seconds(30.0), "00:30");
        assert_eq!(format_seconds(90.0), "01:30");
        assert_eq!(format_seconds(3661.0), "01:01:01");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_video_metadata() {
        let metadata = VideoMetadata::basic("Test Video".to_string(), 120.0);
        assert_eq!(metadata.title, "Test Video");
        assert_eq!(metadata.duration_seconds, 120.0);
        assert_eq!(metadata.format_duration(), "02:00");
        assert!(!metadata.is_hd());
        assert!(!metadata.is_4k());

        let mut hd_metadata = metadata.clone();
        hd_metadata.width = 1920;
        hd_metadata.height = 1080;
        assert!(hd_metadata.is_hd());
        assert!(!hd_metadata.is_4k());
        assert_eq!(hd_metadata.resolution_string(), "1920x1080");
    }
}
