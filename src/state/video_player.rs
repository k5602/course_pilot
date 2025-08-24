#![allow(clippy::module_name_repetitions)]
//! State SoT wrapper for the video player subsystem.
//!
//! This module exposes the video player state and operations through the global
//! `state` namespace, aligning it with the app's hooks-driven SoT pattern used
//! across `state::{courses, notes, plans, imports, ui}`.
//!
//! Design:
//! - Re-exports core video player types for ergonomics
//! - Provides a provider component under the `state` namespace
//! - Adds reactive hook wrapper and a set of convenience operations with a
//!   consistent "*_reactive" naming convention
//!
//! Notes:
//! - Internally delegates to `crate::video_player::state` (the authoritative
//!   context implementation). Signals are shared handles, so cloning the
//!   context and mutating via its methods will update the same underlying state.

use dioxus::prelude::*;

pub use crate::video_player::{PlaybackState, VideoMetadata, VideoPlayerError, VideoSource};

/// Provider component under the state namespace (SoT entrypoint).
///
/// Wraps the existing video player provider to integrate with the state module tree.
#[component]
pub fn VideoPlayerStateProvider(children: Element) -> Element {
    rsx! {
        crate::video_player::VideoPlayerProvider {
            {children}
        }
    }
}

/// Reactive access to the video player context (SoT hook).
///
/// This mirrors other state modules' `*_reactive` hook naming.
pub fn use_video_player_reactive() -> crate::video_player::VideoPlayerContext {
    crate::video_player::use_video_player()
}

// --------------- Convenience reactive operations ---------------

/// Load a video source and initialize state (validates source and resets position/metadata).
pub fn load_video_reactive(source: VideoSource) {
    let mut ctx = use_video_player_reactive();
    ctx.load_video(source);
}

/// Toggle between play and pause (no-op for non-applicable states).
pub fn toggle_play_pause_reactive() {
    let mut ctx = use_video_player_reactive();
    ctx.toggle_play_pause();
}

/// Start playback if possible.
pub fn play_reactive() {
    let mut ctx = use_video_player_reactive();
    ctx.play();
}

/// Pause playback if possible.
pub fn pause_reactive() {
    let mut ctx = use_video_player_reactive();
    ctx.pause();
}

/// Stop playback and reset position.
pub fn stop_reactive() {
    let mut ctx = use_video_player_reactive();
    ctx.stop();
}

/// Seek to an absolute position in seconds (clamped to [0, duration]).
pub fn seek_to_reactive(position_seconds: f64) {
    let mut ctx = use_video_player_reactive();
    ctx.seek_to(position_seconds);
}

/// Seek relative to the current position (delta seconds).
pub fn seek_relative_reactive(delta_seconds: f64) {
    let mut ctx = use_video_player_reactive();
    ctx.seek_relative(delta_seconds);
}

/// Seek to a percentage of the total duration (0.0 to 1.0).
pub fn seek_to_percentage_reactive(percentage: f64) {
    let mut ctx = use_video_player_reactive();
    ctx.seek_to_percentage(percentage);
}

/// Set volume (0.0 to 1.0) and update mute flag accordingly.
pub fn set_volume_reactive(volume_0_to_1: f64) {
    let mut ctx = use_video_player_reactive();
    ctx.set_volume(volume_0_to_1);
}

/// Toggle mute/unmute.
pub fn toggle_mute_reactive() {
    let mut ctx = use_video_player_reactive();
    ctx.toggle_mute();
}

/// Toggle fullscreen mode.
pub fn toggle_fullscreen_reactive() {
    let mut ctx = use_video_player_reactive();
    ctx.toggle_fullscreen();
}

/// Explicitly set fullscreen mode.
pub fn set_fullscreen_reactive(fullscreen: bool) {
    let mut ctx = use_video_player_reactive();
    ctx.set_fullscreen(fullscreen);
}

/// Update the playback position (typically called from player implementations).
pub fn update_position_reactive(position_seconds: f64) {
    let mut ctx = use_video_player_reactive();
    ctx.update_position(position_seconds);
}

/// Update the media duration in seconds (typically called from player implementations).
pub fn update_duration_reactive(duration_seconds: f64) {
    let mut ctx = use_video_player_reactive();
    ctx.update_duration(duration_seconds);
}

/// Set loading spinner state.
pub fn set_loading_reactive(loading: bool) {
    let mut ctx = use_video_player_reactive();
    ctx.set_loading(loading);
}

/// Set (or clear) a player error and synchronize playback/loading accordingly.
pub fn set_error_reactive(error: Option<VideoPlayerError>) {
    let mut ctx = use_video_player_reactive();
    ctx.set_error(error);
}

/// Update metadata (duration, title, etc.) discovered by the player backend.
pub fn set_metadata_reactive(metadata: Option<VideoMetadata>) {
    let mut ctx = use_video_player_reactive();
    ctx.set_metadata(metadata);
}

/// Synchronize the internal state from a YouTube IFrame bridge update.
pub fn sync_youtube_state_reactive(state: crate::video_player::types::YouTubePlayerState) {
    let mut ctx = use_video_player_reactive();
    ctx.sync_youtube_state(state);
}

// --------------- Convenience getters (computed/read-only) ---------------

/// Get current playback progress percentage (0.0..=100.0).
pub fn progress_percentage_reactive() -> f64 {
    let ctx = use_video_player_reactive();
    ctx.progress_percentage()
}

/// Get remaining time in seconds (>= 0.0).
pub fn remaining_time_reactive() -> f64 {
    let ctx = use_video_player_reactive();
    ctx.remaining_time()
}

/// Returns true if a video is currently loaded.
pub fn has_video_reactive() -> bool {
    let ctx = use_video_player_reactive();
    ctx.has_video()
}

/// Returns true if a video is loaded, not loading, and there is no error.
pub fn is_ready_reactive() -> bool {
    let ctx = use_video_player_reactive();
    ctx.is_ready()
}
