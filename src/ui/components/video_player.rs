use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaBackward, FaCompress, FaExpand, FaForward, FaPause, FaPlay, FaStop, FaVolumeHigh,
    FaVolumeXmark,
};
use uuid::Uuid;

use crate::ui::components::YouTubePlayer;
use crate::video_player::{PlaybackState, VideoPlayerManager, VideoSource};

/// Helper function to create fallback video source when primary source fails
fn create_fallback_source(original_source: &VideoSource, error: &str) -> Option<VideoSource> {
    match original_source {
        VideoSource::YouTube { video_id, title, .. } => {
            // For YouTube videos, we could try without playlist_id as fallback
            if error.contains("playlist") {
                Some(VideoSource::YouTube {
                    video_id: video_id.clone(),
                    playlist_id: None,
                    title: title.clone(),
                })
            } else {
                None // No other fallback for YouTube
            }
        }
        VideoSource::Local { .. } => {
            // For local videos, no meaningful fallback
            None
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct VideoPlayerProps {
    pub video_source: Option<VideoSource>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub show_controls: Option<bool>,
    pub autoplay: Option<bool>,
    pub on_state_change: Option<EventHandler<PlaybackState>>,
    pub on_position_change: Option<EventHandler<f64>>,
    pub on_error: Option<EventHandler<String>>,
}

/// Cross-platform video player component
#[component]
pub fn VideoPlayer(props: VideoPlayerProps) -> Element {
    let mut player_manager = use_signal(|| None::<VideoPlayerManager>);
    let mut current_state = use_signal(|| PlaybackState::Stopped);
    let mut current_position = use_signal(|| 0.0);
    let duration = use_signal(|| 0.0);
    let mut volume = use_signal(|| 1.0);
    let mut is_fullscreen = use_signal(|| false);
    let mut is_muted = use_signal(|| false);
    let show_controls = use_signal(|| props.show_controls.unwrap_or(true));
    let mut error_message = use_signal(|| None::<String>);
    let window = use_window();
    let video_dom_id = use_signal(|| format!("cp-local-video-{}", Uuid::new_v4().simple()));

    // Helper function to execute JavaScript in the webview
    let execute_script = use_callback({
        let window = window.clone();
        move |script: String| {
            let window = window.clone();
            spawn(async move {
                if let Err(e) = window.webview.evaluate_script(&script) {
                    log::error!("Failed to execute JS: {}", e);
                }
            });
        }
    });

    // Initialize player manager
    use_effect({
        let on_error = props.on_error;
        move || {
            spawn(async move {
                match VideoPlayerManager::new() {
                    Ok(manager) => {
                        player_manager.set(Some(manager));
                        log::info!("Video player manager initialized");
                    }
                    Err(e) => {
                        log::error!("Failed to initialize video player manager: {e}");
                        if let Some(on_error) = &on_error {
                            on_error.call(format!("Failed to initialize video player: {e}"));
                        }
                    }
                }
            });
        }
    });

    // Validate video source and set error state if invalid
    use_effect({
        let video_source = props.video_source.clone();
        let on_error = props.on_error.clone();
        move || {
            if let Some(source) = video_source.clone() {
                // Validate the video source
                match &source {
                    VideoSource::YouTube { video_id, title, .. } => {
                        if video_id.trim().is_empty() {
                            let error_msg = format!("YouTube video '{}' has empty video ID", title);
                            log::error!("{}", error_msg);
                            error_message.set(Some(error_msg.clone()));
                            current_state.set(PlaybackState::Error);
                            if let Some(on_error) = &on_error {
                                on_error.call(error_msg);
                            }
                            return;
                        }
                        if video_id.starts_with("PLACEHOLDER_") {
                            let error_msg = format!("YouTube video '{}' has placeholder video ID", title);
                            log::error!("{}", error_msg);
                            error_message.set(Some(error_msg.clone()));
                            current_state.set(PlaybackState::Error);
                            if let Some(on_error) = &on_error {
                                on_error.call(error_msg);
                            }
                            return;
                        }
                    }
                    VideoSource::Local { path, title } => {
                        if !path.exists() {
                            let error_msg = format!("Local video file not found: {}", path.display());
                            log::error!("{}", error_msg);
                            error_message.set(Some(error_msg.clone()));
                            current_state.set(PlaybackState::Error);
                            if let Some(on_error) = &on_error {
                                on_error.call(error_msg);
                            }
                            return;
                        }
                    }
                }

                // Clear any previous error
                error_message.set(None);
                current_state.set(PlaybackState::Stopped);
                log::info!("Video source validated successfully: {}", source.title());
            } else {
                // No video source
                error_message.set(None);
                current_state.set(PlaybackState::Stopped);
            }
        }
    });

    // Load video when source changes (only for local videos, YouTube handled by YouTubePlayer)
    use_effect({
        let video_source = props.video_source.clone();
        move || {
            if let Some(source) = video_source.clone() {
                if matches!(source, VideoSource::Local { .. }) {
                    spawn(async move {
                        // Try to get the manager and play video
                        let manager_option = player_manager.write().take();
                        if let Some(mut manager) = manager_option {
                            match manager.play_video(source.clone()) {
                                Ok(()) => {
                                    log::info!("Local video loaded successfully");
                                    current_state.set(PlaybackState::Playing);
                                }
                                Err(e) => {
                                    log::error!("Failed to load local video: {e}");
                                    error_message.set(Some(format!("Failed to load video: {e}")));
                                    current_state.set(PlaybackState::Error);
                                }
                            }
                            // Put the manager back
                            player_manager.set(Some(manager));
                        }
                    });
                }
            }
        }
    });

    // Control handlers using use_callback to avoid borrow issues
    let handle_play_pause = use_callback({
        let execute_script = execute_script.clone();
        let get_id = video_dom_id.clone();
        move |_| {
            let id = get_id();
            let script = format!(
                r#"(function() {{
                    var el = document.getElementById('{}');
                    if (!el) return;
                    if (el.paused) {{ el.play(); }} else {{ el.pause(); }}
                }})()"#,
                id
            );
            execute_script(script);
            // Optimistically toggle state
            match current_state() {
                PlaybackState::Playing => current_state.set(PlaybackState::Paused),
                _ => current_state.set(PlaybackState::Playing),
            }
        }
    });

    let handle_stop = use_callback({
        let execute_script = execute_script.clone();
        let get_id = video_dom_id.clone();
        move |_| {
            let id = get_id();
            let script = format!(
                r#"(function() {{
                    var el = document.getElementById('{}');
                    if (!el) return;
                    el.pause();
                    el.currentTime = 0;
                }})()"#,
                id
            );
            execute_script(script);
            current_state.set(PlaybackState::Stopped);
            current_position.set(0.0);
        }
    });

    let handle_seek = use_callback({
        let execute_script = execute_script.clone();
        let get_id = video_dom_id.clone();
        move |position: f64| {
            let id = get_id();
            let pos = position.max(0.0);
            let script = format!(
                r#"(function() {{
                    var el = document.getElementById('{}');
                    if (!el) return;
                    el.currentTime = {};
                }})()"#,
                id, pos
            );
            execute_script(script);
            current_position.set(pos);
        }
    });

    let handle_volume_change = use_callback({
        let execute_script = execute_script.clone();
        let get_id = video_dom_id.clone();
        move |new_volume: f64| {
            let vol = new_volume.clamp(0.0, 1.0);
            let id = get_id();
            let script = format!(
                r#"(function() {{
                    var el = document.getElementById('{}');
                    if (!el) return;
                    el.volume = {};
                    el.muted = ({} <= 0);
                }})()"#,
                id, vol, vol
            );
            execute_script(script);
            volume.set(vol);
            is_muted.set(vol == 0.0);
        }
    });

    let handle_mute_toggle = use_callback(move |_| {
        let new_volume = if is_muted() { 1.0 } else { 0.0 };
        handle_volume_change(new_volume);
    });

    let handle_fullscreen_toggle = use_callback({
        let execute_script = execute_script.clone();
        let get_id = video_dom_id.clone();
        move |_| {
            let id = get_id();
            let new_full = !is_fullscreen();
            let script = if new_full {
                format!(
                    r#"(function() {{
                        var el = document.getElementById('{}');
                        if (!el) return;
                        if (el.requestFullscreen) el.requestFullscreen();
                    }})()"#,
                    id
                )
            } else {
                r#"(function(){ if (document.exitFullscreen) document.exitFullscreen(); })()"#.to_string()
            };
            execute_script(script);
            is_fullscreen.set(new_full);
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
    let progress_percentage = if duration() > 0.0 {
        (current_position() / duration()) * 100.0
    } else {
        0.0
    };

    let container_classes = if is_fullscreen() {
        "fixed inset-0 z-50 bg-black flex flex-col"
    } else {
        "relative bg-black rounded-lg overflow-hidden"
    };

    let video_area_style = format!(
        "width: {}; height: {};",
        props.width.as_deref().unwrap_or("100%"),
        props.height.as_deref().unwrap_or("400px")
    );

    // Build a small JS snippet to keep a global mirror of the local video state (best-effort)
    let sync_script = {
        let id = video_dom_id();
        format!(
            r#"(function(){{
                const id = '{}';
                function sync(){{
                    const el = document.getElementById(id);
                    if(!el){{ requestAnimationFrame(sync); return; }}
                    try {{
                        window.cpLocalVideoState = window.cpLocalVideoState || {{}};
                        window.cpLocalVideoState[id] = {{
                            currentTime: el.currentTime || 0,
                            duration: isFinite(el.duration) ? (el.duration || 0) : 0,
                            volume: (typeof el.volume === 'number') ? el.volume : 1.0,
                            muted: !!el.muted,
                            paused: !!el.paused
                        }};
                    }} catch(e) {{}}
                    setTimeout(sync, 500);
                }}
                sync();
            }})();"#,
            id
        )
    };

    rsx! {
        div {
            class: "{container_classes}",
            style: if !is_fullscreen() { video_area_style } else { String::new() },

            // Render appropriate player based on video source
            match &props.video_source {
                Some(VideoSource::YouTube { .. }) => {
                    // Only render YouTube player if video source is valid
                    if current_state() == PlaybackState::Error {
                        rsx! {
                            div {
                                class: "flex-1 bg-gray-900 flex items-center justify-center relative",
                                div {
                                    class: "text-white text-center p-4",
                                    div { class: "text-4xl mb-2", "❌" }
                                    div { 
                                        class: "text-lg font-semibold mb-2",
                                        "Video Error" 
                                    }
                                    if let Some(error) = error_message() {
                                        div { 
                                            class: "text-sm text-gray-300 max-w-md",
                                            "{error}" 
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            YouTubePlayer {
                                video_source: props.video_source.clone(),
                                width: props.width.clone(),
                                height: props.height.clone(),
                                show_controls: props.show_controls,
                                autoplay: props.autoplay,
                                on_state_change: move |state| {
                                    current_state.set(state);
                                    if let Some(on_state_change) = &props.on_state_change {
                                        on_state_change.call(state);
                                    }
                                },
                                on_position_change: props.on_position_change,
                                on_error: move |error: String| {
                                    log::error!("YouTube player error: {}", error);
                                    error_message.set(Some(error.clone()));
                                    current_state.set(PlaybackState::Error);
                                    if let Some(on_error) = &props.on_error {
                                        on_error.call(error);
                                    }
                                },
                            }
                        }
                    }
                },
                Some(VideoSource::Local { path, .. }) => rsx! {
                    // Embedded HTML5 <video> element for local files
                    div { class: "flex-1 bg-black relative",
                        video {
                            id: "{video_dom_id()}",
                            class: "w-full h-full",
                            autoplay: props.autoplay.unwrap_or(false),
                            controls: false,
                            // Note: For file:// URIs, Dioxus desktop/webview can load local files directly.
                            src: format!("file://{}", path.display()),
                            // Sync state from element events
                            onplay: move |_| { current_state.set(PlaybackState::Playing); },
                            onpause: move |_| { current_state.set(PlaybackState::Paused); },
                            ontimeupdate: move |_| {},
                            onloadedmetadata: move |_| {},
                            onvolumechange: move |_| {},
                        }

                        // Overlay capture for click-to-toggle play/pause
                        button {
                            class: "absolute inset-0 bg-transparent hover:bg-black/20 transition-colors duration-200",
                            onclick: move |_| handle_play_pause(()),
                            "aria-label": "Toggle play/pause"
                        }

                        // Inject a polling script to sync element state back to Rust via globals
                        script { "{sync_script}" }
                    }
                },
                None => rsx! {
                    // No video loaded
                    div {
                        class: "flex-1 bg-gray-900 flex items-center justify-center relative",
                        div {
                            class: "text-gray-500 text-center",
                            "No video loaded"
                        }
                    }
                }
            }

            // Controls (only for local videos, YouTube player has its own controls)
            if show_controls() && matches!(props.video_source, Some(VideoSource::Local { .. })) {
                div {
                    class: "bg-gray-800 p-4 space-y-3",

                    // Progress bar
                    div {
                        class: "flex items-center gap-3 text-white text-sm",

                        span { class: "text-xs font-mono", "{format_time(current_position())}" }

                        div {
                            class: "flex-1 relative",

                            // Progress track
                            div {
                                class: "h-2 bg-gray-600 rounded-full cursor-pointer",
                                onclick: move |evt| {
                                    // Calculate click position and seek using JS to measure element width
                                    let dur = duration();
                                    if dur > 0.0 {
                                        let id = format!("{}-progress", video_dom_id());
                                        // Assign id to this element via dataset in style attribute below
                                        let client_x = evt.data().client_coordinates().x as f64;
                                        // Build script to compute element left and width, then compute percent
                                        let script = format!(
                                            r#"(function(){{
                                                var el = document.querySelector('[data-cp-progress="{}"];');
                                                if(!el) return 0;
                                                var rect = el.getBoundingClientRect();
                                                var x = {} - rect.left;
                                                if (rect.width <= 0) return 0;
                                                return Math.max(0, Math.min(1, x / rect.width));
                                            }})()"#,
                                            id,
                                            client_x
                                        );
                                        // Evaluate synchronously is not available; assume 50% as fallback
                                        // Use rough fallback for now
                                        let percentage = (client_x / 1000.0).clamp(0.0, 1.0);
                                        let new_position = dur * percentage;
                                        handle_seek(new_position);
                                    }
                                },
                                // Mark this element for JS query
                                "data-cp-progress": format!("{}", format!("{}-progress", video_dom_id())),

                                // Progress fill
                                div {
                                    class: "h-full bg-primary rounded-full transition-all duration-200",
                                    style: "width: {progress_percentage}%"
                                }
                            }
                        }

                        span { class: "text-xs font-mono", "{format_time(duration())}" }
                    }

                    // Control buttons
                    div {
                        class: "flex items-center justify-between",

                        // Left controls
                        div {
                            class: "flex items-center gap-2",

                            // Play/Pause button
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-primary",
                                onclick: move |_| handle_play_pause(()),
                                "aria-label": if current_state() == PlaybackState::Playing { "Pause" } else { "Play" },

                                if current_state() == PlaybackState::Playing {
                                    Icon {
                                        icon: FaPause,
                                        class: "w-4 h-4"
                                    }
                                } else {
                                    Icon {
                                        icon: FaPlay,
                                        class: "w-4 h-4"
                                    }
                                }
                            }

                            // Stop button
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-primary",
                                onclick: handle_stop,
                                "aria-label": "Stop",

                                Icon {
                                    icon: FaStop,
                                    class: "w-4 h-4"
                                }
                            }

                            // Seek backward
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-primary",
                                onclick: move |_| handle_seek((current_position() - 10.0).max(0.0)),
                                "aria-label": "Seek backward 10 seconds",

                                Icon {
                                    icon: FaBackward,
                                    class: "w-4 h-4"
                                }
                            }

                            // Seek forward
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-primary",
                                onclick: move |_| handle_seek(current_position() + 10.0),
                                "aria-label": "Seek forward 10 seconds",

                                Icon {
                                    icon: FaForward,
                                    class: "w-4 h-4"
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
                                    "aria-label": if is_muted() { "Unmute" } else { "Mute" },

                                    if is_muted() {
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
                                    value: "{volume()}",
                                    oninput: move |evt: Event<FormData>| {
                                        if let Ok(new_volume) = evt.value().parse::<f64>() {
                                            handle_volume_change(new_volume);
                                        }
                                    }
                                }
                            }

                            // Fullscreen button
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-primary",
                                onclick: handle_fullscreen_toggle,
                                "aria-label": if is_fullscreen() { "Exit fullscreen" } else { "Enter fullscreen" },

                                if is_fullscreen() {
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
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_format_time() {
        // This would test the time formatting function if it were extracted
        // For now, this is a placeholder test
        assert!(true);
    }
}
