use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};

use crate::video_player::{
    PlaybackState, VideoInfo, VideoPlayer, VideoSource, controls::VideoPlayerControls,
    local_player::LocalVideoPlayer, webview_youtube_player::WebViewYouTubePlayer,
};

/// Cross-platform video player manager that handles both local and YouTube videos
pub struct VideoPlayerManager {
    local_player: Arc<Mutex<LocalVideoPlayer>>,
    youtube_player: Arc<Mutex<WebViewYouTubePlayer>>,
    current_player_type: Arc<Mutex<Option<PlayerType>>>,
    current_video_info: Arc<Mutex<Option<VideoInfo>>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerType {
    Local,
    YouTube,
}

impl VideoPlayerManager {
    /// Create a new cross-platform video player manager
    pub fn new() -> Result<Self> {
        // Initialize GStreamer
        crate::video_player::init()?;

        let local_player = Arc::new(Mutex::new(LocalVideoPlayer::new()?));
    let youtube_player = Arc::new(Mutex::new(WebViewYouTubePlayer::new()?));

        Ok(Self {
            local_player,
            youtube_player,
            current_player_type: Arc::new(Mutex::new(None)),
            current_video_info: Arc::new(Mutex::new(None)),
        })
    }

    /// Play a video from the given source
    pub fn play_video(&mut self, source: VideoSource) -> Result<()> {
        // Stop any currently playing video
        self.stop_current_video()?;

        // Determine which player to use based on source type
        let player_type = match &source {
            VideoSource::Local { .. } => PlayerType::Local,
            VideoSource::YouTube { .. } => PlayerType::YouTube,
        };

        // Update current player type
        {
            let mut current_type = self
                .current_player_type
                .lock()
                .map_err(|_| anyhow!("Failed to lock current player type"))?;
            *current_type = Some(player_type);
        }

        // Create video info
        let video_info = VideoInfo::new(source.clone());

        // Update current video info
        {
            let mut current_info = self
                .current_video_info
                .lock()
                .map_err(|_| anyhow!("Failed to lock current video info"))?;
            *current_info = Some(video_info);
        }

        // Load and play the video using the appropriate player
        match player_type {
            PlayerType::Local => {
                let mut player = self
                    .local_player
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock local player"))?;
                player.load_and_play(source)?;
            }
            PlayerType::YouTube => {
                let mut player = self
                    .youtube_player
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock YouTube player"))?;
                player.load_and_play(source)?;
            }
        }

        log::info!("Started playing video using {player_type:?} player");
        Ok(())
    }

    /// Stop the currently playing video
    pub fn stop_current_video(&mut self) -> Result<()> {
        let current_type = {
            let current_type = self
                .current_player_type
                .lock()
                .map_err(|_| anyhow!("Failed to lock current player type"))?;
            *current_type
        };

        if let Some(player_type) = current_type {
            match player_type {
                PlayerType::Local => {
                    let mut player = self
                        .local_player
                        .lock()
                        .map_err(|_| anyhow!("Failed to lock local player"))?;
                    player.stop()?;
                }
                PlayerType::YouTube => {
                    let mut player = self
                        .youtube_player
                        .lock()
                        .map_err(|_| anyhow!("Failed to lock YouTube player"))?;
                    player.stop()?;
                }
            }

            // Clear current player type and video info
            {
                let mut current_type = self
                    .current_player_type
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock current player type"))?;
                *current_type = None;
            }

            {
                let mut current_info = self
                    .current_video_info
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock current video info"))?;
                *current_info = None;
            }
        }

        Ok(())
    }

    /// Get controls for the currently active player
    pub fn get_current_controls(&self) -> Result<Option<Box<dyn VideoPlayerControlsInterface>>> {
        let current_type = {
            let current_type = self
                .current_player_type
                .lock()
                .map_err(|_| anyhow!("Failed to lock current player type"))?;
            *current_type
        };

        match current_type {
            Some(PlayerType::Local) => {
                let controls = VideoPlayerControls::new(Arc::clone(&self.local_player));
                Ok(Some(Box::new(LocalPlayerControls { controls })))
            }
            Some(PlayerType::YouTube) => {
                let controls = VideoPlayerControls::new(Arc::clone(&self.youtube_player));
                Ok(Some(Box::new(YouTubePlayerControls { controls })))
            }
            None => Ok(None),
        }
    }

    /// Get current video information
    pub fn get_current_video_info(&self) -> Result<Option<VideoInfo>> {
        let current_info = self
            .current_video_info
            .lock()
            .map_err(|_| anyhow!("Failed to lock current video info"))?;
        Ok(current_info.clone())
    }

    /// Check if any video is currently playing
    pub fn is_playing(&self) -> Result<bool> {
        let current_type = {
            let current_type = self
                .current_player_type
                .lock()
                .map_err(|_| anyhow!("Failed to lock current player type"))?;
            *current_type
        };

        match current_type {
            Some(PlayerType::Local) => {
                let player = self
                    .local_player
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock local player"))?;
                Ok(player.is_playing())
            }
            Some(PlayerType::YouTube) => {
                let player = self
                    .youtube_player
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock YouTube player"))?;
                Ok(player.is_playing())
            }
            None => Ok(false),
        }
    }

    /// Get current playback state
    pub fn get_current_state(&self) -> Result<PlaybackState> {
        let current_type = {
            let current_type = self
                .current_player_type
                .lock()
                .map_err(|_| anyhow!("Failed to lock current player type"))?;
            *current_type
        };

        match current_type {
            Some(PlayerType::Local) => {
                let player = self
                    .local_player
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock local player"))?;
                Ok(player.get_state())
            }
            Some(PlayerType::YouTube) => {
                let player = self
                    .youtube_player
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock YouTube player"))?;
                Ok(player.get_state())
            }
            None => Ok(PlaybackState::Stopped),
        }
    }

    /// Get supported local video formats
    pub fn get_supported_local_formats() -> Vec<&'static str> {
        LocalVideoPlayer::get_supported_formats()
    }

    /// Check if a local video format is supported
    pub fn is_local_format_supported(extension: &str) -> bool {
        LocalVideoPlayer::is_format_supported(extension)
    }
}

/// Trait for unified video player controls interface
pub trait VideoPlayerControlsInterface {
    fn play(&self) -> Result<()>;
    fn pause(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn toggle_play_pause(&self) -> Result<()>;
    fn seek(&self, position_seconds: f64) -> Result<()>;
    fn seek_forward(&self, seconds: f64) -> Result<()>;
    fn seek_backward(&self, seconds: f64) -> Result<()>;
    fn set_volume(&self, volume: f64) -> Result<()>;
    fn volume_up(&self, amount: f64) -> Result<()>;
    fn volume_down(&self, amount: f64) -> Result<()>;
    fn toggle_fullscreen(&self) -> Result<()>;
    fn set_fullscreen(&self, fullscreen: bool) -> Result<()>;
    fn get_position(&self) -> Result<f64>;
    fn get_duration(&self) -> Result<f64>;
    fn get_state(&self) -> Result<PlaybackState>;
    fn is_playing(&self) -> Result<bool>;
    fn is_fullscreen(&self) -> Result<bool>;
}

/// Local player controls wrapper
struct LocalPlayerControls {
    controls: VideoPlayerControls<LocalVideoPlayer>,
}

impl VideoPlayerControlsInterface for LocalPlayerControls {
    fn play(&self) -> Result<()> {
        self.controls.play()
    }
    fn pause(&self) -> Result<()> {
        self.controls.pause()
    }
    fn stop(&self) -> Result<()> {
        self.controls.stop()
    }
    fn toggle_play_pause(&self) -> Result<()> {
        self.controls.toggle_play_pause()
    }
    fn seek(&self, position_seconds: f64) -> Result<()> {
        self.controls.seek(position_seconds)
    }
    fn seek_forward(&self, seconds: f64) -> Result<()> {
        self.controls.seek_forward(seconds)
    }
    fn seek_backward(&self, seconds: f64) -> Result<()> {
        self.controls.seek_backward(seconds)
    }
    fn set_volume(&self, volume: f64) -> Result<()> {
        self.controls.set_volume(volume)
    }
    fn volume_up(&self, amount: f64) -> Result<()> {
        self.controls.volume_up(amount)
    }
    fn volume_down(&self, amount: f64) -> Result<()> {
        self.controls.volume_down(amount)
    }
    fn toggle_fullscreen(&self) -> Result<()> {
        self.controls.toggle_fullscreen()
    }
    fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
        self.controls.set_fullscreen(fullscreen)
    }
    fn get_position(&self) -> Result<f64> {
        self.controls.get_position()
    }
    fn get_duration(&self) -> Result<f64> {
        self.controls.get_duration()
    }
    fn get_state(&self) -> Result<PlaybackState> {
        self.controls.get_state()
    }
    fn is_playing(&self) -> Result<bool> {
        self.controls.is_playing()
    }
    fn is_fullscreen(&self) -> Result<bool> {
        self.controls.is_fullscreen()
    }
}

/// YouTube player controls wrapper
struct YouTubePlayerControls {
    controls: VideoPlayerControls<WebViewYouTubePlayer>,
}

impl VideoPlayerControlsInterface for YouTubePlayerControls {
    fn play(&self) -> Result<()> {
        self.controls.play()
    }
    fn pause(&self) -> Result<()> {
        self.controls.pause()
    }
    fn stop(&self) -> Result<()> {
        self.controls.stop()
    }
    fn toggle_play_pause(&self) -> Result<()> {
        self.controls.toggle_play_pause()
    }
    fn seek(&self, position_seconds: f64) -> Result<()> {
        self.controls.seek(position_seconds)
    }
    fn seek_forward(&self, seconds: f64) -> Result<()> {
        self.controls.seek_forward(seconds)
    }
    fn seek_backward(&self, seconds: f64) -> Result<()> {
        self.controls.seek_backward(seconds)
    }
    fn set_volume(&self, volume: f64) -> Result<()> {
        self.controls.set_volume(volume)
    }
    fn volume_up(&self, amount: f64) -> Result<()> {
        self.controls.volume_up(amount)
    }
    fn volume_down(&self, amount: f64) -> Result<()> {
        self.controls.volume_down(amount)
    }
    fn toggle_fullscreen(&self) -> Result<()> {
        self.controls.toggle_fullscreen()
    }
    fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
        self.controls.set_fullscreen(fullscreen)
    }
    fn get_position(&self) -> Result<f64> {
        self.controls.get_position()
    }
    fn get_duration(&self) -> Result<f64> {
        self.controls.get_duration()
    }
    fn get_state(&self) -> Result<PlaybackState> {
        self.controls.get_state()
    }
    fn is_playing(&self) -> Result<bool> {
        self.controls.is_playing()
    }
    fn is_fullscreen(&self) -> Result<bool> {
        self.controls.is_fullscreen()
    }
}

impl Default for VideoPlayerManager {
    fn default() -> Self {
        Self::new().expect("Failed to create VideoPlayerManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_formats() {
        let formats = VideoPlayerManager::get_supported_local_formats();
        assert!(formats.contains(&"mp4"));
        assert!(formats.contains(&"avi"));
        assert!(formats.contains(&"mov"));
    }

    #[test]
    fn test_format_support_check() {
        assert!(VideoPlayerManager::is_local_format_supported("mp4"));
        assert!(VideoPlayerManager::is_local_format_supported("MP4"));
        assert!(!VideoPlayerManager::is_local_format_supported("txt"));
    }
}
