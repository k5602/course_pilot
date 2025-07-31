use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Video source types supported by the player
#[derive(Debug, Clone, PartialEq)]
pub enum VideoSource {
    /// Local video file
    Local { path: PathBuf, title: String },
    /// YouTube video
    YouTube {
        video_id: String,
        playlist_id: Option<String>,
        title: String,
    },
}

/// Playback state of the video player
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
    Error,
}

/// Video player controls interface
pub struct VideoPlayerControls {
    pub play: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    pub pause: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    pub stop: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    pub seek: Box<dyn Fn(f64) -> anyhow::Result<()> + Send + Sync>,
    pub set_volume: Box<dyn Fn(f64) -> anyhow::Result<()> + Send + Sync>,
    pub toggle_fullscreen: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
}

/// Video information for display
#[derive(Debug, Clone, PartialEq)]
pub struct VideoInfo {
    pub source: VideoSource,
    pub duration_seconds: Option<f64>,
    pub current_position: f64,
    pub state: PlaybackState,
    pub volume: f64,
    pub is_fullscreen: bool,
}

impl VideoInfo {
    pub fn new(source: VideoSource) -> Self {
        Self {
            source,
            duration_seconds: None,
            current_position: 0.0,
            state: PlaybackState::Stopped,
            volume: 1.0,
            is_fullscreen: false,
        }
    }

    pub fn title(&self) -> &str {
        match &self.source {
            VideoSource::Local { title, .. } => title,
            VideoSource::YouTube { title, .. } => title,
        }
    }
}

/// Video player events
#[derive(Debug, Clone)]
pub enum VideoPlayerEvent {
    StateChanged(PlaybackState),
    PositionChanged(f64),
    DurationChanged(f64),
    VolumeChanged(f64),
    FullscreenChanged(bool),
    Error(String),
}

/// Video format support information
#[derive(Debug, Clone)]
pub struct VideoFormatSupport {
    pub mp4: bool,
    pub avi: bool,
    pub mov: bool,
    pub mkv: bool,
    pub webm: bool,
}

impl Default for VideoFormatSupport {
    fn default() -> Self {
        Self {
            mp4: true,
            avi: true,
            mov: true,
            mkv: true,
            webm: true,
        }
    }
}

impl PlaybackState {
    pub fn is_active(&self) -> bool {
        matches!(self, PlaybackState::Playing | PlaybackState::Buffering)
    }

    pub fn can_pause(&self) -> bool {
        matches!(self, PlaybackState::Playing)
    }

    pub fn can_play(&self) -> bool {
        matches!(self, PlaybackState::Stopped | PlaybackState::Paused)
    }
}
