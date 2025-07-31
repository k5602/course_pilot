use anyhow::Result;

#[cfg(feature = "ffmpeg")]
use ffmpeg_next;

pub mod controls;
pub mod cross_platform;
pub mod local_player;
pub mod types;
pub mod youtube_player;
pub mod webview_youtube_player;

pub use cross_platform::VideoPlayerManager;
pub use types::*;
pub use webview_youtube_player::WebViewYouTubePlayer;

/// Initialize the video player subsystem
pub fn init() -> Result<()> {
    #[cfg(feature = "ffmpeg")]
    {
        ffmpeg_next::init().map_err(|e| anyhow::anyhow!("Failed to initialize FFmpeg: {}", e))?;
        log::info!("FFmpeg video player subsystem initialized");
    }

    #[cfg(not(feature = "ffmpeg"))]
    {
        log::warn!("Video player compiled without FFmpeg support");
    }

    Ok(())
}

/// Video player trait for unified interface across platforms
pub trait VideoPlayer {
    /// Load and play a video from the given source
    fn load_and_play(&mut self, source: VideoSource) -> Result<()>;

    /// Pause the current video
    fn pause(&mut self) -> Result<()>;

    /// Resume playback
    fn resume(&mut self) -> Result<()>;

    /// Stop playback and reset
    fn stop(&mut self) -> Result<()>;

    /// Seek to a specific position (in seconds)
    fn seek(&mut self, position_seconds: f64) -> Result<()>;

    /// Set volume (0.0 to 1.0)
    fn set_volume(&mut self, volume: f64) -> Result<()>;

    /// Get current playback position in seconds
    fn get_position(&self) -> Result<f64>;

    /// Get total duration in seconds
    fn get_duration(&self) -> Result<f64>;

    /// Check if currently playing
    fn is_playing(&self) -> bool;

    /// Get current playback state
    fn get_state(&self) -> PlaybackState;

    /// Set fullscreen mode
    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()>;

    /// Check if in fullscreen mode
    fn is_fullscreen(&self) -> bool;
}
