use crate::video_player::{
    PlaybackState, VideoMetadata, VideoPlayerError, VideoSource, ipc::global,
};
use dioxus::prelude::*;
use dioxus_desktop::use_window;

/// Modern hooks-driven video player manager that replaces legacy context providers
#[derive(Clone)]
pub struct VideoPlayerManager {
    // Reactive state
    pub current_video: Signal<Option<VideoSource>>,
    pub playback_state: Signal<PlaybackState>,
    pub position: Signal<f64>,
    pub duration: Signal<f64>,
    pub volume: Signal<f64>,
    pub is_muted: Signal<bool>,
    pub is_fullscreen: Signal<bool>,
    pub loading: Signal<bool>,
    pub error: Signal<Option<VideoPlayerError>>,
    pub metadata: Signal<Option<VideoMetadata>>,

    // Internal state
    window: dioxus_desktop::DesktopContext,
}

impl VideoPlayerManager {
    /// Load a video source
    pub fn load_video(&mut self, source: VideoSource) {
        log::info!("Loading video: {:?}", source);
        self.current_video.set(Some(source));
        self.playback_state.set(PlaybackState::Stopped);
        self.position.set(0.0);
        self.duration.set(0.0);
        self.loading.set(true);
        self.error.set(None);
    }

    /// Play the current video
    pub fn play(&mut self) {
        if self.current_video().is_some() {
            self.playback_state.set(PlaybackState::Playing);
            self.execute_script(global::set_active_player("local", "cp-video-player"));
            self.execute_script(crate::video_player::ipc::local::play("cp-video-player"));
        }
    }

    /// Pause the current video
    pub fn pause(&mut self) {
        self.playback_state.set(PlaybackState::Paused);
        self.execute_script(global::set_active_player("local", "cp-video-player"));
        self.execute_script(crate::video_player::ipc::local::pause("cp-video-player"));
    }

    /// Toggle play/pause
    pub fn toggle_play_pause(&mut self) {
        match self.playback_state() {
            PlaybackState::Playing => self.pause(),
            _ => self.play(),
        }
    }

    /// Seek to a specific position
    pub fn seek_to(&mut self, position: f64) {
        let clamped_position = position.max(0.0).min(self.duration());
        self.position.set(clamped_position);
        self.execute_script(global::set_active_player("local", "cp-video-player"));
        self.execute_script(crate::video_player::ipc::local::seek_to(
            "cp-video-player",
            clamped_position,
        ));
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f64) {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.volume.set(clamped_volume);
        self.execute_script(global::set_active_player("local", "cp-video-player"));
        self.execute_script(crate::video_player::ipc::local::set_volume(
            "cp-video-player",
            clamped_volume,
        ));
    }

    // Getters for reactive state
    pub fn current_video(&self) -> Option<VideoSource> {
        self.current_video.read().clone()
    }

    pub fn playback_state(&self) -> PlaybackState {
        *self.playback_state.read()
    }

    pub fn position(&self) -> f64 {
        *self.position.read()
    }

    pub fn duration(&self) -> f64 {
        *self.duration.read()
    }

    pub fn volume(&self) -> f64 {
        *self.volume.read()
    }

    pub fn has_video(&self) -> bool {
        self.current_video().is_some()
    }

    pub fn is_fullscreen(&self) -> bool {
        *self.is_fullscreen.read()
    }

    pub fn is_loading(&self) -> bool {
        *self.loading.read()
    }

    pub fn error(&self) -> Option<VideoPlayerError> {
        self.error.read().clone()
    }

    pub fn metadata(&self) -> Option<VideoMetadata> {
        self.metadata.read().clone()
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.duration() > 0.0 { (self.position() / self.duration()) * 100.0 } else { 0.0 }
    }

    pub fn remaining_time(&self) -> f64 {
        self.duration() - self.position()
    }

    pub fn is_ready(&self) -> bool {
        self.has_video() && !self.is_loading() && self.error().is_none()
    }

    /// Update position (called by video element events)
    pub fn update_position(&mut self, position: f64) {
        self.position.set(position);
    }

    /// Update duration (called by video element events)
    pub fn update_duration(&mut self, duration: f64) {
        self.duration.set(duration);
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.loading.set(loading);
    }

    /// Set error state
    pub fn set_error(&mut self, error: Option<VideoPlayerError>) {
        let has_error = error.is_some();
        self.error.set(error);
        if has_error {
            self.playback_state.set(PlaybackState::Error);
            self.loading.set(false);
        }
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: Option<VideoMetadata>) {
        self.metadata.set(metadata);
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.playback_state.set(PlaybackState::Stopped);
        self.position.set(0.0);
        self.execute_script(global::set_active_player("local", "cp-video-player"));
        self.execute_script(crate::video_player::ipc::local::pause("cp-video-player"));
    }

    /// Seek relative to current position
    pub fn seek_relative(&mut self, delta: f64) {
        let new_position = self.position() + delta;
        self.seek_to(new_position);
    }

    /// Seek to percentage (0.0 to 1.0)
    pub fn seek_to_percentage(&mut self, percentage: f64) {
        let clamped_percentage = percentage.clamp(0.0, 1.0);
        let position = self.duration() * clamped_percentage;
        self.seek_to(position);
    }

    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        let new_muted = !self.is_muted();
        self.is_muted.set(new_muted);
        self.execute_script(global::set_active_player("local", "cp-video-player"));
        self.execute_script(crate::video_player::ipc::local::set_muted(
            "cp-video-player",
            new_muted,
        ));
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        *self.is_muted.read()
    }

    /// Toggle fullscreen
    pub fn toggle_fullscreen(&mut self) {
        let new_fullscreen = !self.is_fullscreen();
        self.is_fullscreen.set(new_fullscreen);
    }

    /// Set fullscreen state
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.is_fullscreen.set(fullscreen);
    }

    /// Execute JavaScript in the webview
    fn execute_script(&self, script: String) {
        if let Err(e) = self.window.webview.evaluate_script(&script) {
            log::error!("Failed to execute JavaScript: {}", e);
        }
    }
}

/// Primary hook for video player management
pub fn use_video_player_manager() -> VideoPlayerManager {
    let window = use_window();

    // Reactive state signals
    let current_video = use_signal(|| None::<VideoSource>);
    let playback_state = use_signal(|| PlaybackState::Stopped);
    let position = use_signal(|| 0.0f64);
    let duration = use_signal(|| 0.0f64);
    let volume = use_signal(|| 1.0f64);
    let is_muted = use_signal(|| false);
    let is_fullscreen = use_signal(|| false);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<VideoPlayerError>);
    let metadata = use_signal(|| None::<VideoMetadata>);

    // Create manager instance
    let manager = VideoPlayerManager {
        current_video: current_video.clone(),
        playback_state: playback_state.clone(),
        position: position.clone(),
        duration: duration.clone(),
        volume: volume.clone(),
        is_muted: is_muted.clone(),
        is_fullscreen: is_fullscreen.clone(),
        loading: loading.clone(),
        error: error.clone(),
        metadata: metadata.clone(),
        window: window.clone(),
    };

    // Initialize global keyboard handler
    use_effect({
        let window = window.clone();
        move || {
            let setup_script = global::attach_keyboard_handler();
            if let Err(e) = window.webview.evaluate_script(&setup_script) {
                log::error!("Failed to set up keyboard shortcuts: {}", e);
            }
        }
    });

    manager
}
