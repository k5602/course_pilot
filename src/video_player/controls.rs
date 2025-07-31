use anyhow::Result;
use std::sync::{Arc, Mutex};

use crate::video_player::{PlaybackState, VideoPlayer};

/// Video player control interface
pub struct VideoPlayerControls<T: VideoPlayer> {
    player: Arc<Mutex<T>>,
}

impl<T: VideoPlayer> VideoPlayerControls<T> {
    /// Create new video player controls
    pub fn new(player: Arc<Mutex<T>>) -> Self {
        Self { player }
    }

    /// Play or resume playback
    pub fn play(&self) -> Result<()> {
        let mut player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        match player.get_state() {
            PlaybackState::Stopped | PlaybackState::Paused => player.resume(),
            PlaybackState::Playing => Ok(()),   // Already playing
            PlaybackState::Buffering => Ok(()), // Let it continue buffering
            PlaybackState::Error => {
                // Try to resume from error state
                player.resume()
            }
        }
    }

    /// Pause playback
    pub fn pause(&self) -> Result<()> {
        let mut player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        if player.get_state().can_pause() {
            player.pause()
        } else {
            Ok(()) // Can't pause in current state
        }
    }

    /// Stop playback
    pub fn stop(&self) -> Result<()> {
        let mut player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        player.stop()
    }

    /// Toggle play/pause
    pub fn toggle_play_pause(&self) -> Result<()> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        match player.get_state() {
            PlaybackState::Playing => {
                drop(player);
                self.pause()
            }
            PlaybackState::Paused | PlaybackState::Stopped => {
                drop(player);
                self.play()
            }
            _ => Ok(()), // Don't toggle in other states
        }
    }

    /// Seek to position (in seconds)
    pub fn seek(&self, position_seconds: f64) -> Result<()> {
        let mut player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        // Clamp position to valid range
        let duration = player.get_duration().unwrap_or(0.0);
        let clamped_position = position_seconds.clamp(0.0, duration);

        player.seek(clamped_position)
    }

    /// Seek forward by seconds
    pub fn seek_forward(&self, seconds: f64) -> Result<()> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        let current_position = player.get_position().unwrap_or(0.0);
        drop(player);

        self.seek(current_position + seconds)
    }

    /// Seek backward by seconds
    pub fn seek_backward(&self, seconds: f64) -> Result<()> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        let current_position = player.get_position().unwrap_or(0.0);
        drop(player);

        self.seek(current_position - seconds)
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&self, volume: f64) -> Result<()> {
        let mut player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        player.set_volume(volume)
    }

    /// Increase volume by amount
    pub fn volume_up(&self, amount: f64) -> Result<()> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        // Get current volume (assume 1.0 if we can't get it)
        let current_volume = 1.0; // TODO: Add get_volume to VideoPlayer trait
        drop(player);

        let new_volume = (current_volume + amount).clamp(0.0, 1.0);
        self.set_volume(new_volume)
    }

    /// Decrease volume by amount
    pub fn volume_down(&self, amount: f64) -> Result<()> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        // Get current volume (assume 1.0 if we can't get it)
        let current_volume = 1.0; // TODO: Add get_volume to VideoPlayer trait
        drop(player);

        let new_volume = (current_volume - amount).clamp(0.0, 1.0);
        self.set_volume(new_volume)
    }

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen(&self) -> Result<()> {
        let mut player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;

        let is_fullscreen = player.is_fullscreen();
        player.set_fullscreen(!is_fullscreen)
    }

    /// Set fullscreen mode
    pub fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
        let mut player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        player.set_fullscreen(fullscreen)
    }

    /// Get current playback position
    pub fn get_position(&self) -> Result<f64> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        player.get_position()
    }

    /// Get total duration
    pub fn get_duration(&self) -> Result<f64> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        player.get_duration()
    }

    /// Get current playback state
    pub fn get_state(&self) -> Result<PlaybackState> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        Ok(player.get_state())
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> Result<bool> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        Ok(player.is_playing())
    }

    /// Check if in fullscreen mode
    pub fn is_fullscreen(&self) -> Result<bool> {
        let player = self
            .player
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock player"))?;
        Ok(player.is_fullscreen())
    }
}

/// Keyboard shortcuts for video player controls
pub struct VideoPlayerKeyboardShortcuts;

impl VideoPlayerKeyboardShortcuts {
    /// Get default keyboard shortcuts mapping
    pub fn get_default_shortcuts() -> Vec<(String, String)> {
        vec![
            ("Space".to_string(), "Toggle Play/Pause".to_string()),
            ("K".to_string(), "Toggle Play/Pause".to_string()),
            ("J".to_string(), "Seek Backward 10s".to_string()),
            ("L".to_string(), "Seek Forward 10s".to_string()),
            ("ArrowLeft".to_string(), "Seek Backward 5s".to_string()),
            ("ArrowRight".to_string(), "Seek Forward 5s".to_string()),
            ("ArrowUp".to_string(), "Volume Up".to_string()),
            ("ArrowDown".to_string(), "Volume Down".to_string()),
            ("M".to_string(), "Toggle Mute".to_string()),
            ("F".to_string(), "Toggle Fullscreen".to_string()),
            ("Escape".to_string(), "Exit Fullscreen".to_string()),
            ("0".to_string(), "Seek to Start".to_string()),
            ("1".to_string(), "Seek to 10%".to_string()),
            ("2".to_string(), "Seek to 20%".to_string()),
            ("3".to_string(), "Seek to 30%".to_string()),
            ("4".to_string(), "Seek to 40%".to_string()),
            ("5".to_string(), "Seek to 50%".to_string()),
            ("6".to_string(), "Seek to 60%".to_string()),
            ("7".to_string(), "Seek to 70%".to_string()),
            ("8".to_string(), "Seek to 80%".to_string()),
            ("9".to_string(), "Seek to 90%".to_string()),
        ]
    }

    /// Handle keyboard shortcut
    pub fn handle_shortcut<T: VideoPlayer>(
        key: &str,
        controls: &VideoPlayerControls<T>,
    ) -> Result<bool> {
        match key {
            "Space" | "K" => {
                controls.toggle_play_pause()?;
                Ok(true)
            }
            "J" => {
                controls.seek_backward(10.0)?;
                Ok(true)
            }
            "L" => {
                controls.seek_forward(10.0)?;
                Ok(true)
            }
            "ArrowLeft" => {
                controls.seek_backward(5.0)?;
                Ok(true)
            }
            "ArrowRight" => {
                controls.seek_forward(5.0)?;
                Ok(true)
            }
            "ArrowUp" => {
                controls.volume_up(0.1)?;
                Ok(true)
            }
            "ArrowDown" => {
                controls.volume_down(0.1)?;
                Ok(true)
            }
            "M" => {
                // Toggle mute (set volume to 0 or restore)
                // TODO: Implement mute state tracking
                controls.set_volume(0.0)?;
                Ok(true)
            }
            "F" => {
                controls.toggle_fullscreen()?;
                Ok(true)
            }
            "Escape" => {
                controls.set_fullscreen(false)?;
                Ok(true)
            }
            "0" => {
                controls.seek(0.0)?;
                Ok(true)
            }
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let percentage = key.parse::<f64>().unwrap_or(0.0) / 10.0;
                let duration = controls.get_duration().unwrap_or(0.0);
                controls.seek(duration * percentage)?;
                Ok(true)
            }
            _ => Ok(false), // Shortcut not handled
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shortcuts() {
        let shortcuts = VideoPlayerKeyboardShortcuts::get_default_shortcuts();
        assert!(!shortcuts.is_empty());

        // Check that common shortcuts are present
        let shortcut_keys: Vec<&String> = shortcuts.iter().map(|(key, _)| key).collect();
        assert!(shortcut_keys.contains(&&"Space".to_string()));
        assert!(shortcut_keys.contains(&&"F".to_string()));
        assert!(shortcut_keys.contains(&&"K".to_string()));
    }
}
