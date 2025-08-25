use dioxus::prelude::*;
use dioxus_desktop::use_window;

use crate::video_player::{VideoPlayerContext, use_video_player};

/// Keyboard shortcut mappings and handlers for the video player
pub struct KeyboardShortcuts;

impl KeyboardShortcuts {
    /// Get default keyboard shortcuts mapping
    pub fn get_default_shortcuts() -> Vec<(String, String)> {
        vec![
            ("Space".to_string(), "Toggle Play/Pause".to_string()),
            ("k".to_string(), "Toggle Play/Pause".to_string()),
            ("j".to_string(), "Seek Backward 10s".to_string()),
            ("l".to_string(), "Seek Forward 10s".to_string()),
            ("ArrowLeft".to_string(), "Seek Backward 5s".to_string()),
            ("ArrowRight".to_string(), "Seek Forward 5s".to_string()),
            ("ArrowUp".to_string(), "Volume Up".to_string()),
            ("ArrowDown".to_string(), "Volume Down".to_string()),
            ("m".to_string(), "Toggle Mute".to_string()),
            ("f".to_string(), "Toggle Fullscreen".to_string()),
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

    /// Handle a keyboard shortcut and mutate the player state
    pub fn handle_shortcut(
        key: &str,
        state: &mut VideoPlayerContext,
        on_handled: Option<&dyn Fn()>,
    ) -> bool {
        // Only handle shortcuts if video is loaded
        if !state.has_video() {
            return false;
        }

        let handled = match key.to_lowercase().as_str() {
            " " | "space" | "k" => {
                state.toggle_play_pause();
                true
            },
            "j" => {
                state.seek_relative(-10.0);
                true
            },
            "l" => {
                state.seek_relative(10.0);
                true
            },
            "arrowleft" => {
                state.seek_relative(-5.0);
                true
            },
            "arrowright" => {
                state.seek_relative(5.0);
                true
            },
            "arrowup" => {
                let new_volume = (*state.volume.read() + 0.1).clamp(0.0, 1.0);
                state.set_volume(new_volume);
                true
            },
            "arrowdown" => {
                let new_volume = (*state.volume.read() - 0.1).clamp(0.0, 1.0);
                state.set_volume(new_volume);
                true
            },
            "m" => {
                state.toggle_mute();
                true
            },
            "f" => {
                state.toggle_fullscreen();
                true
            },
            "escape" => {
                if *state.is_fullscreen.read() {
                    state.set_fullscreen(false);
                    true
                } else {
                    false
                }
            },
            "0" => {
                state.seek_to(0.0);
                true
            },
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                if let Ok(digit) = key.parse::<f64>() {
                    let percentage = digit / 10.0;
                    state.seek_to_percentage(percentage);
                    true
                } else {
                    false
                }
            },
            _ => false,
        };

        if handled {
            if let Some(callback) = on_handled {
                callback();
            }
        }

        handled
    }
}

/// Manager returned by `use_videoplayer()` following the hooks-driven state pattern.
/// Provides the reactive state and callable operations.
#[derive(Clone)]
pub struct VideoPlayerManager {
    pub state: VideoPlayerContext,
}

impl VideoPlayerManager {
    // Playback controls
    pub fn play(&mut self) {
        self.state.play();
    }
    pub fn pause(&mut self) {
        self.state.pause();
    }
    pub fn toggle_play_pause(&mut self) {
        self.state.toggle_play_pause();
    }
    pub fn stop(&mut self) {
        self.state.stop();
    }

    // Seeking
    pub fn seek_to(&mut self, seconds: f64) {
        self.state.seek_to(seconds);
    }
    pub fn seek_relative(&mut self, delta: f64) {
        self.state.seek_relative(delta);
    }
    pub fn seek_to_percentage(&mut self, pct_0_to_1: f64) {
        self.state.seek_to_percentage(pct_0_to_1);
    }

    // Volume
    pub fn set_volume(&mut self, vol_0_to_1: f64) {
        self.state.set_volume(vol_0_to_1);
    }
    pub fn toggle_mute(&mut self) {
        self.state.toggle_mute();
    }

    // Fullscreen
    pub fn toggle_fullscreen(&mut self) {
        self.state.toggle_fullscreen();
    }
    pub fn set_fullscreen(&mut self, value: bool) {
        self.state.set_fullscreen(value);
    }
}

/// Primary hook for the Video Player SoT manager.
/// - Returns a `VideoPlayerManager`
/// - Attaches global keyboard handler via centralized IPC
/// - Starts a lightweight polling loop for key events bridge (non-blocking)
pub fn use_videoplayer() -> VideoPlayerManager {
    let state = use_video_player();
    let window = use_window();

    // Attach the global keyboard handler once
    use_effect({
        let window = window.clone();
        move || {
            let setup_script = crate::video_player::ipc::global::attach_keyboard_handler();
            if let Err(e) = window.webview.evaluate_script(&setup_script) {
                log::error!("Failed to set up keyboard shortcuts: {}", e);
            }
        }
    });

    // Periodically check for last captured key from the global bridge
    use_effect({
        let window = window.clone();
        let mut state = state.clone();
        move || {
            let window_clone = window.clone();
            let mut state_clone = state.clone();
            spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
                loop {
                    interval.tick().await;
                    // Pop last key from the global handler, if any, and apply mapping
                    let script = crate::video_player::ipc::global::pop_last_keyboard_key();
                    if let Err(e) = window_clone.webview.evaluate_script(&script) {
                        log::trace!("Keyboard poll eval failed: {}", e);
                        continue;
                    }
                    // Note:
                    // - The evaluate_script API returns (), so the bridge should route events
                    //   to window.cp.keyboardAction(key) which then invokes our Rust-side mapping
                    //   via IPC. This polling loop exists as a safety net and can be removed
                    //   when the event-driven bridge fully replaces it.
                    //
                    // - If/when a return channel is wired, the following can be enabled:
                    // if let Ok(Some(key)) = ... { KeyboardShortcuts::handle_shortcut(&key, &mut state, None); }
                    let _ = &mut state_clone; // keep state captured
                }
            });
        }
    });

    VideoPlayerManager { state }
}

/// Hook for keyboard shortcuts (migrated).
/// Prefer using `use_videoplayer()` which auto-initializes the global handler.
/// This remains for compatibility and explicit initialization.
pub fn use_video_keyboard_shortcuts() -> impl Fn() {
    let manager = use_videoplayer();

    // Return a function placeholder for explicit manual triggering if needed
    move || {
        let _ = &manager;
        // In case you want to trigger programmatic shortcuts, wire here.
    }
}

/// Hook for handling video player focus and blur events
pub fn use_video_focus() -> Signal<bool> {
    let is_focused = use_signal(|| false);
    is_focused
}

/// Video analytics data
#[derive(Debug, Clone, PartialEq)]
pub struct VideoAnalytics {
    pub total_watch_time: std::time::Duration,
    pub seek_count: u32,
    pub pause_count: u32,
    pub completion_percentage: f64,
}

impl VideoAnalytics {
    /// Check if video is considered "completed" (watched > 90%)
    pub fn is_completed(&self) -> bool {
        self.completion_percentage >= 90.0
    }

    /// Get engagement score (0.0 to 1.0)
    pub fn engagement_score(&self) -> f64 {
        // Simple engagement calculation based on completion and interaction
        let completion_score = (self.completion_percentage / 100.0).clamp(0.0, 1.0);
        let interaction_score = if self.seek_count + self.pause_count > 0 {
            0.1 // Bonus for interaction
        } else {
            0.0
        };

        (completion_score + interaction_score).clamp(0.0, 1.0)
    }
}

/// Hook for video player analytics and metrics (migrated)
pub fn use_video_analytics() -> VideoAnalytics {
    let state = use_video_player();
    let watch_start_time = use_signal(|| None::<std::time::Instant>);
    let total_watch_time = use_signal(|| std::time::Duration::ZERO);
    let seek_count = use_signal(|| 0u32);
    let pause_count = use_signal(|| 0u32);

    // Track playback state changes
    use_effect({
        let mut watch_start_time = watch_start_time.clone();
        let mut total_watch_time = total_watch_time.clone();
        let mut pause_count = pause_count.clone();
        let state = state.clone();
        move || match *state.playback_state.read() {
            crate::video_player::PlaybackState::Playing => {
                if watch_start_time.read().is_none() {
                    watch_start_time.set(Some(std::time::Instant::now()));
                }
            },
            crate::video_player::PlaybackState::Paused => {
                let start_time_opt = *watch_start_time.read();
                if let Some(start_time) = start_time_opt {
                    let elapsed = start_time.elapsed();
                    let current_watch_time = *total_watch_time.read();
                    let current_pause_count = *pause_count.read();

                    total_watch_time.set(current_watch_time + elapsed);
                    watch_start_time.set(None);
                    pause_count.set(current_pause_count + 1);
                }
            },
            crate::video_player::PlaybackState::Stopped => {
                let start_time_opt = *watch_start_time.read();
                if let Some(start_time) = start_time_opt {
                    let elapsed = start_time.elapsed();
                    let current_watch_time = *total_watch_time.read();

                    total_watch_time.set(current_watch_time + elapsed);
                    watch_start_time.set(None);
                }
            },
            _ => {},
        }
    });

    // Track seek events
    let previous_position = use_signal(|| 0.0);
    use_effect({
        let mut seek_count = seek_count.clone();
        let mut previous_position = previous_position.clone();
        let state = state.clone();
        move || {
            let current_position = *state.position.read();
            let prev_pos = *previous_position.read();

            // Detect seeks (position jumps > 2 seconds)
            if (current_position - prev_pos).abs() > 2.0 {
                let current_seek_count = *seek_count.read();
                seek_count.set(current_seek_count + 1);
            }

            previous_position.set(current_position);
        }
    });

    VideoAnalytics {
        total_watch_time: *total_watch_time.read(),
        seek_count: *seek_count.read(),
        pause_count: *pause_count.read(),
        completion_percentage: state.progress_percentage(),
    }
}

/// Video performance metrics
#[derive(Debug, Clone, PartialEq)]
pub struct VideoPerformanceMetrics {
    pub load_duration: Option<std::time::Duration>,
    pub error_count: u32,
    pub is_loading: bool,
    pub has_error: bool,
}

impl VideoPerformanceMetrics {
    /// Check if performance is acceptable
    pub fn is_performance_good(&self) -> bool {
        if let Some(duration) = self.load_duration {
            duration.as_secs() < 5 && self.error_count == 0
        } else {
            self.error_count == 0
        }
    }
}

/// Hook for video player performance monitoring (migrated)
pub fn use_video_performance() -> VideoPerformanceMetrics {
    let load_start_time = use_signal(|| None::<std::time::Instant>);
    let load_duration = use_signal(|| None::<std::time::Duration>);
    let error_count = use_signal(|| 0u32);
    let state = use_video_player();

    // Track loading performance
    use_effect({
        let mut load_start_time = load_start_time.clone();
        let mut load_duration = load_duration.clone();
        let state = state.clone();
        move || {
            let is_loading = *state.loading.read();
            let start_time_opt = *load_start_time.read();

            if is_loading && start_time_opt.is_none() {
                load_start_time.set(Some(std::time::Instant::now()));
            } else if !is_loading && start_time_opt.is_some() {
                if let Some(start_time) = start_time_opt {
                    load_duration.set(Some(start_time.elapsed()));
                    load_start_time.set(None);
                }
            }
        }
    });

    // Track errors
    use_effect({
        let mut error_count = error_count.clone();
        let state = state.clone();
        move || {
            if state.error.read().is_some() {
                let current_count = *error_count.read();
                error_count.set(current_count + 1);
            }
        }
    });

    VideoPerformanceMetrics {
        load_duration: *load_duration.read(),
        error_count: *error_count.read(),
        is_loading: *state.loading.read(),
        has_error: state.error.read().is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_shortcuts_list() {
        let shortcuts = KeyboardShortcuts::get_default_shortcuts();
        assert!(!shortcuts.is_empty());

        // Check that common shortcuts are present
        let shortcut_keys: Vec<&String> = shortcuts.iter().map(|(key, _)| key).collect();
        assert!(shortcut_keys.contains(&&"Space".to_string()));
        assert!(shortcut_keys.contains(&&"f".to_string()));
        assert!(shortcut_keys.contains(&&"k".to_string()));
    }

    #[test]
    fn test_video_analytics_helpers() {
        let analytics = VideoAnalytics {
            total_watch_time: std::time::Duration::from_secs(300),
            seek_count: 5,
            pause_count: 2,
            completion_percentage: 95.0,
        };

        assert!(analytics.is_completed());
        assert!(analytics.engagement_score() > 0.9);
    }

    #[test]
    fn test_performance_metrics_helpers() {
        let metrics = VideoPerformanceMetrics {
            load_duration: Some(std::time::Duration::from_secs(2)),
            error_count: 0,
            is_loading: false,
            has_error: false,
        };

        assert!(metrics.is_performance_good());
    }
}
