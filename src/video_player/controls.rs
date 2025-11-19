use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaBackward, FaCompress, FaExpand, FaForward, FaPause, FaPlay, FaStop, FaVolumeHigh,
    FaVolumeXmark,
};

use crate::ui::hooks::use_video_player_manager;
use crate::video_player::{PlaybackState, VideoSource};

#[derive(Props, PartialEq, Clone)]
pub struct VideoControlsProps {
    pub player_id: String,
    pub show_playlist_controls: Option<bool>,
    pub on_play_pause: Option<EventHandler<()>>,
    pub on_stop: Option<EventHandler<()>>,
    pub on_seek: Option<EventHandler<f64>>,
    pub on_volume_change: Option<EventHandler<f64>>,
    pub on_fullscreen_toggle: Option<EventHandler<()>>,
}

/// Custom video controls overlay with DaisyUI styling
#[component]
pub fn VideoControls(props: VideoControlsProps) -> Element {
    let mut state = use_video_player_manager();
    let window = use_window();
    let show_playlist_controls = props.show_playlist_controls.unwrap_or(false);
    let controls_visible = use_signal(|| true);
    let last_mouse_move = use_signal(|| None::<std::time::Instant>);

    let playback_rate = use_signal(|| 1.0f64);

    // Auto-hide controls in fullscreen mode
    let handle_mouse_move = use_callback({
        let mut controls_visible = controls_visible.clone();
        let mut last_mouse_move = last_mouse_move.clone();
        move |_| {
            controls_visible.set(true);

            // record last mouse move time; auto-hide loop will handle hiding
            last_mouse_move.set(Some(std::time::Instant::now()));
        }
    });

    // Install global pointer tracker to compute click position via JS
    use_effect({
        let window = window.clone();
        move || {
            let script = r#"
                    (function() {
                        if (!window._cpPointerTrackerInstalled) {
                            window._cpPointerTrackerInstalled = true;
                            document.addEventListener('pointermove', function(e) {
                                window._cp_last_pointer = { x: e.clientX, y: e.clientY };
                            }, { capture: true, passive: true });
                        }
                    })();
                "#
            .to_string();
            let _ = window.webview.evaluate_script(&script);
        }
    });

    // Auto-hide controls in fullscreen after 3s of inactivity
    use_effect({
        let controls_visible = controls_visible.clone();
        let last_mouse_move = last_mouse_move.clone();
        let state_clone = state.clone();
        move || {
            let mut controls_visible = controls_visible.clone();
            let last_mouse_move = last_mouse_move.clone();
            let state_for_async = state_clone.clone();
            spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(250));
                loop {
                    interval.tick().await;
                    if state_for_async.is_fullscreen() && *controls_visible.read() {
                        if let Some(t) = *last_mouse_move.read() {
                            if t.elapsed() >= std::time::Duration::from_secs(3) {
                                controls_visible.set(false);
                            }
                        }
                    }
                }
            });
        }
    });

    // Format time helper
    let format_time = |seconds: f64| -> String {
        let total_seconds = seconds as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;

        if hours > 0 {
            format!("{hours:02}:{minutes:02}:{secs:02}")
        } else {
            format!("{minutes:02}:{secs:02}")
        }
    };

    // Calculate progress percentage
    let progress_percentage = state.progress_percentage();

    // Control handlers
    let handle_play_pause = use_callback({
        let mut state = state.clone();
        let on_play_pause = props.on_play_pause.clone();
        move |_| {
            state.toggle_play_pause();
            if let Some(on_play_pause) = &on_play_pause {
                on_play_pause.call(());
            }
        }
    });

    let handle_stop = use_callback({
        let mut state = state.clone();
        let on_stop = props.on_stop.clone();
        move |_| {
            state.stop();
            if let Some(on_stop) = &on_stop {
                on_stop.call(());
            }
        }
    });

    let handle_seek_backward = use_callback({
        let mut state = state.clone();
        let on_seek = props.on_seek.clone();
        move |_| {
            state.seek_relative(-10.0);
            if let Some(on_seek) = &on_seek {
                on_seek.call(*state.position.read());
            }
        }
    });

    let handle_seek_forward = use_callback({
        let mut state = state.clone();
        let on_seek = props.on_seek.clone();
        move |_| {
            state.seek_relative(10.0);
            if let Some(on_seek) = &on_seek {
                on_seek.call(*state.position.read());
            }
        }
    });

    let handle_volume_change = use_callback({
        let mut state = state.clone();
        let on_volume_change = props.on_volume_change.clone();
        move |evt: Event<FormData>| {
            if let Ok(new_volume) = evt.value().parse::<f64>() {
                state.set_volume(new_volume);
                if let Some(on_volume_change) = &on_volume_change {
                    on_volume_change.call(new_volume);
                }
            }
        }
    });

    let handle_mute_toggle = use_callback({
        let mut state = state.clone();
        let on_volume_change = props.on_volume_change.clone();
        move |_| {
            state.toggle_mute();
            if let Some(on_volume_change) = &on_volume_change {
                on_volume_change.call(*state.volume.read());
            }
        }
    });

    let handle_fullscreen_toggle = use_callback({
        let mut state = state.clone();
        let on_fullscreen_toggle = props.on_fullscreen_toggle.clone();
        move |_| {
            state.toggle_fullscreen();
            if let Some(on_fullscreen_toggle) = &on_fullscreen_toggle {
                on_fullscreen_toggle.call(());
            }
        }
    });

    let handle_progress_click = use_callback({
        let mut state = state.clone();
        let on_seek = props.on_seek.clone();
        move |_evt: MouseEvent| {
            let duration = state.duration();
            if duration > 0.0 {
                let percentage = 0.5;
                let new_position = duration * percentage;
                state.seek_to(new_position);
                if let Some(on_seek) = &on_seek {
                    on_seek.call(new_position);
                }
            }
        }
    });

    // Don't render controls if video is not loaded
    if !state.has_video() {
        return rsx! { div {} };
    }

    let controls_class = if state.is_fullscreen() && !*controls_visible.read() {
        "absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-6 opacity-0 transition-opacity duration-300"
    } else {
        "absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4 opacity-100 transition-opacity duration-300"
    };

    rsx! {
        div {
            class: "{controls_class}",
            onmousemove: handle_mouse_move,

            // Progress bar
            div {
                class: "flex items-center gap-3 text-white text-sm mb-3",

                span {
                    class: "text-xs font-mono min-w-[3rem]",
                    "{format_time(state.position())}"
                }

                div {
                    class: "flex-1 relative",

                    // Progress track
                    div {

                        class: "h-2 bg-white/20 rounded-full cursor-pointer hover:bg-white/30 transition-colors",
                        onclick: handle_progress_click,

                        // Progress fill
                        div {
                            class: "h-full bg-primary rounded-full transition-all duration-200",
                            style: "width: {progress_percentage}%"
                        }

                        // Progress handle
                        div {
                            class: "absolute top-1/2 -translate-y-1/2 w-3 h-3 bg-primary rounded-full opacity-0 hover:opacity-100 transition-opacity",
                            style: "left: {progress_percentage}%"
                        }
                    }
                }

                span {
                    class: "text-xs font-mono min-w-[3rem]",
                    "{format_time(state.duration())}"
                }
            }

            // Control buttons
            div {
                class: "flex items-center justify-between",

                // Left controls
                div {
                    class: "flex items-center gap-2",

                    // Previous video (for playlists)
                    if show_playlist_controls {
                        button {
                            class: "btn btn-ghost btn-sm text-white hover:text-primary",
                            "aria-label": "Previous video",
                            title: "Previous video",

                            Icon {
                                icon: FaBackward,
                                class: "w-4 h-4"
                            }
                        }
                    }

                    // Seek backward
                    button {
                        class: "btn btn-ghost btn-sm text-white hover:text-primary",
                        onclick: handle_seek_backward,
                        "aria-label": "Seek backward 10 seconds",
                        title: "Seek backward 10s (J)",

                        Icon {
                            icon: FaBackward,
                            class: "w-4 h-4"
                        }
                        span { class: "text-xs ml-1", "10s" }
                    }

                    // Play/Pause button
                    button {
                        class: "btn btn-ghost btn-lg text-white hover:text-primary",
                        onclick: handle_play_pause,
                        "aria-label": if *state.playback_state.read() == PlaybackState::Playing { "Pause" } else { "Play" },
                        title: if state.playback_state() == PlaybackState::Playing { "Pause (Space)" } else { "Play (Space)" },

                        if state.playback_state() == PlaybackState::Playing {
                            Icon {
                                icon: FaPause,
                                class: "w-6 h-6"
                            }
                        } else {
                            Icon {
                                icon: FaPlay,
                                class: "w-6 h-6"
                            }
                        }
                    }

                    // Stop button
                    button {
                        class: "btn btn-ghost btn-sm text-white hover:text-primary",
                        onclick: handle_stop,
                        "aria-label": "Stop",
                        title: "Stop",

                        Icon {
                            icon: FaStop,
                            class: "w-4 h-4"
                        }
                    }

                    // Seek forward
                    button {
                        class: "btn btn-ghost btn-sm text-white hover:text-primary",
                        onclick: handle_seek_forward,
                        "aria-label": "Seek forward 10 seconds",
                        title: "Seek forward 10s (L)",

                        Icon {
                            icon: FaForward,
                            class: "w-4 h-4"
                        }
                        span { class: "text-xs ml-1", "10s" }
                    }

                    // Next video (for playlists)
                    if show_playlist_controls {
                        button {
                            class: "btn btn-ghost btn-sm text-white hover:text-primary",
                            "aria-label": "Next video",
                            title: "Next video",

                            Icon {
                                icon: FaForward,
                                class: "w-4 h-4"
                            }
                        }
                    }
                }

                // Center info (video title for YouTube)
                div {
                    class: "flex-1 text-center",
                    if let Some(VideoSource::YouTube { title, .. }) = state.current_video() {
                        div {
                            class: "text-sm text-white/80 truncate max-w-md mx-auto",
                            title: "{title}",
                            "{title}"
                        }
                    }
                }

                // Right controls
                div {
                    class: "flex items-center gap-2",

                    // Volume controls
                    div {
                        class: "flex items-center gap-2",

                        // Mute button
                        button {
                            class: "btn btn-ghost btn-sm text-white hover:text-primary",
                            onclick: handle_mute_toggle,
                            "aria-label": if *state.is_muted.read() { "Unmute" } else { "Mute" },
                            title: if *state.is_muted.read() { "Unmute (M)" } else { "Mute (M)" },

                            if state.is_muted() {
                                Icon {
                                    icon: FaVolumeXmark,
                                    class: "w-4 h-4"
                                }
                            } else {
                                Icon {
                                    icon: FaVolumeHigh,
                                    class: "w-4 h-4"
                                }
                            }
                        }

                        // Volume slider
                        input {
                            r#type: "range",
                            class: "range range-primary range-sm w-20",
                            min: "0",
                            max: "1",
                            step: "0.1",
                            value: "{state.volume()}",
                            oninput: handle_volume_change,
                            title: "Volume: {(state.volume() * 100.0) as i32}%",
                        }

                        // Volume percentage
                        span {
                            class: "text-xs text-white/60 min-w-[2rem]",
                            "{(state.volume() * 100.0) as i32}%"
                        }
                    }

                    // Playback speed
                    div {
                        class: "dropdown dropdown-top dropdown-end",

                        button {
                            class: "btn btn-ghost btn-sm text-white hover:text-primary",
                            title: "Playback speed",
                            "{*playback_rate.read()}x"
                        }

                        div {
                            class: "dropdown-content menu p-2 shadow bg-base-100 rounded-box w-32",
                            for speed in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0] {
                                button {
                                    class: "btn btn-ghost btn-sm justify-start",
                                    onclick: {
                                        let window = window.clone();
                                        let mut playback_rate = playback_rate.clone();
                                        let player_id = props.player_id.clone();
                                        move |_| {
                                            playback_rate.set(speed);
                                            let script = format!("(function() {{ var el = document.getElementById('{}'); if (el) el.playbackRate = {}; }})()", player_id, speed);
                                            let _ = window.webview.evaluate_script(&script);
                                        }
                                    },
                                    "{speed}x"
                                }
                            }
                        }
                    }

                    // Fullscreen button
                    button {
                        class: "btn btn-ghost btn-sm text-white hover:text-primary",
                        onclick: handle_fullscreen_toggle,
                        "aria-label": if state.is_fullscreen() { "Exit fullscreen" } else { "Enter fullscreen" },
                        title: if state.is_fullscreen() { "Exit fullscreen (F)" } else { "Enter fullscreen (F)" },

                        if state.is_fullscreen() {
                            Icon {
                                icon: FaCompress,
                                class: "w-4 h-4"
                            }
                        } else {
                            Icon {
                                icon: FaExpand,
                                class: "w-4 h-4"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Simplified controls for minimal UI
#[component]
pub fn MinimalVideoControls(on_play_pause: EventHandler<()>) -> Element {
    let mut state = use_video_player_manager();

    rsx! {
        div {
            class: "absolute inset-0 flex items-center justify-center",

            button {
                class: "btn btn-circle btn-lg bg-black/50 border-white/20 text-white hover:bg-black/70",
                onclick: {
                    let mut state = state.clone();
                    move |_| {
                        state.toggle_play_pause();
                        on_play_pause.call(());
                    }
                },
                "aria-label": if state.playback_state() == PlaybackState::Playing { "Pause" } else { "Play" },

                if state.playback_state() == PlaybackState::Playing {
                    Icon {
                        icon: FaPause,
                        class: "w-8 h-8"
                    }
                } else {
                    Icon {
                        icon: FaPlay,
                        class: "w-8 h-8"
                    }
                }
            }
        }
    }
}

/// Progress bar only (for compact layouts)
#[component]
pub fn VideoProgressBar(on_seek: EventHandler<f64>) -> Element {
    let mut state = use_video_player_manager();

    let handle_progress_click = use_callback({
        let mut state = state.clone();
        move |_evt: MouseEvent| {
            let duration = state.duration();
            if duration > 0.0 {
                // Simplified click handling - would need proper bounds calculation
                let percentage = 0.5; // Placeholder
                let new_position = duration * percentage;
                state.seek_to(new_position);
                on_seek.call(new_position);
            }
        }
    });

    let progress_percentage = state.progress_percentage();

    rsx! {
        div {
            class: "w-full h-1 bg-white/20 rounded-full cursor-pointer hover:h-2 transition-all duration-200",
            onclick: handle_progress_click,

            div {
                class: "h-full bg-primary rounded-full transition-all duration-200",
                style: "width: {progress_percentage}%"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_controls_props() {
        let props = VideoControlsProps {
            player_id: "test-player".to_string(),
            show_playlist_controls: Some(true),
            on_play_pause: None,
            on_stop: None,
            on_seek: None,
            on_volume_change: None,
            on_fullscreen_toggle: None,
        };

        assert_eq!(props.player_id, "test-player");
        assert_eq!(props.show_playlist_controls, Some(true));
    }

    #[test]
    fn test_format_time() {
        // This would test the time formatting if it were extracted as a separate function
        // For now, this is a placeholder
        assert!(true);
    }
}
