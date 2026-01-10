//! Local video player component
//!
//! Simple HTML5 video player for local files.
//! Uses native browser controls for simplicity.

use crate::video_player::PlaybackState;
use dioxus::prelude::*;
use std::path::Path;

/// Local video player component props
#[derive(Props, Clone, PartialEq)]
pub struct LocalPlayerProps {
    /// Path to the local video file
    pub path: String,
    /// Video title for display
    pub title: String,
    /// Callback when playback state changes
    #[props(optional)]
    pub on_state_change: Option<EventHandler<PlaybackState>>,
    /// Callback when video progress updates (position in seconds)
    #[props(optional)]
    pub on_progress: Option<EventHandler<f64>>,
    /// Callback when video ends
    #[props(optional)]
    pub on_ended: Option<EventHandler<()>>,
    /// Start time in seconds
    #[props(default = 0.0)]
    pub start_time: f64,
    /// Custom CSS class
    #[props(optional)]
    pub class: Option<String>,
}

/// Simple local video player using HTML5 video element
///
/// Uses the `local-video://` protocol registered in main.rs
/// to serve local video files to the webview.
#[component]
pub fn LocalPlayer(props: LocalPlayerProps) -> Element {
    let player_id = use_signal(|| format!("local-player-{}", uuid::Uuid::new_v4()));

    // Convert file path to the custom protocol URL
    let video_url = format!("local-video://{}", props.path.replace('\\', "/"));

    let class = props.class.clone().unwrap_or_default();
    let on_ended = props.on_ended.clone();
    let on_progress = props.on_progress.clone();
    let on_state_change = props.on_state_change.clone();

    rsx! {
        div {
            class: "local-player-container {class}",
            style: "position: relative; width: 100%; aspect-ratio: 16/9; background: #000;",

            video {
                id: "{player_id}",
                src: "{video_url}",
                style: "width: 100%; height: 100%; object-fit: contain;",
                controls: true,
                preload: "metadata",

                // Handle video events
                onplay: move |_| {
                    if let Some(handler) = &on_state_change {
                        handler.call(PlaybackState::Playing);
                    }
                },
                onpause: move |_| {
                    if let Some(handler) = &on_state_change {
                        handler.call(PlaybackState::Paused);
                    }
                },
                onended: move |_| {
                    if let Some(handler) = &on_ended {
                        handler.call(());
                    }
                    if let Some(handler) = &on_state_change {
                        handler.call(PlaybackState::Stopped);
                    }
                },
                ontimeupdate: move |_| {
                    // Progress updates are handled via JavaScript polling
                    // to avoid excessive re-renders
                },
            }
        }
    }
}

/// Get the MIME type for a video file extension
pub fn get_video_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref() {
        Some("mp4") | Some("m4v") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("wmv") => "video/x-ms-wmv",
        Some("flv") => "video/x-flv",
        Some("ogv") => "video/ogg",
        Some("3gp") => "video/3gpp",
        Some("ts") | Some("mts") | Some("m2ts") => "video/mp2t",
        _ => "video/mp4", // Default fallback
    }
}

/// JavaScript to get current playback state from local video
pub fn local_get_state_script(player_id: &str) -> String {
    format!(
        r#"
        (function() {{
            var video = document.getElementById('{player_id}');
            if (!video) return null;
            return JSON.stringify({{
                currentTime: video.currentTime,
                duration: video.duration || 0,
                volume: video.volume,
                isMuted: video.muted,
                paused: video.paused,
                ended: video.ended
            }});
        }})()
    "#
    )
}

/// JavaScript to control playback
pub fn local_play_script(player_id: &str) -> String {
    format!("document.getElementById('{player_id}')?.play()")
}

pub fn local_pause_script(player_id: &str) -> String {
    format!("document.getElementById('{player_id}')?.pause()")
}

pub fn local_seek_script(player_id: &str, seconds: f64) -> String {
    format!("document.getElementById('{player_id}').currentTime = {seconds}")
}

pub fn local_set_volume_script(player_id: &str, volume: f64) -> String {
    let vol = volume.clamp(0.0, 1.0);
    format!("document.getElementById('{player_id}').volume = {vol}")
}
