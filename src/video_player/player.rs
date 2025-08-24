use dioxus::prelude::*;
use dioxus_desktop::use_window;
use uuid::Uuid;

use crate::video_player::{
    PlaybackState, VideoControls, VideoPlayerError, VideoSource, use_video_player,
};

#[derive(Props, PartialEq, Clone)]
pub struct VideoPlayerProps {
    pub source: Option<VideoSource>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub show_controls: Option<bool>,
    pub autoplay: Option<bool>,
    pub on_progress: Option<EventHandler<f64>>,
    pub on_complete: Option<EventHandler<()>>,
    pub on_error: Option<EventHandler<String>>,
}

/// Unified video player component that handles both local and YouTube videos
#[component]
pub fn VideoPlayer(props: VideoPlayerProps) -> Element {
    let state = use_video_player();
    let window = use_window();
    // Clone the current video for use in closures
    let current_video = state.current_video.clone();
    let player_id = use_signal(|| format!("cp-video-{}", Uuid::new_v4().simple()));
    let youtube_player_id = use_signal(|| format!("youtube-{}", Uuid::new_v4().simple()));

    // Load video when source changes
    use_effect({
        let source = props.source.clone();
        let mut state = state.clone();
        move || {
            if let Some(video_source) = source.clone() {
                log::info!("VideoPlayer: Loading video source: {:?}", video_source);
                state.load_video(video_source);
            } else {
                log::info!("VideoPlayer: No video source provided");
                state.current_video.set(None);
                state.playback_state.set(PlaybackState::Stopped);
            }
        }
    });

    // JavaScript bridge for state synchronization
    let execute_script = use_callback({
        let window = window.clone();
        move |script: String| {
            let window = window.clone();
            spawn(async move {
                if let Err(e) = window.webview.evaluate_script(&script) {
                    log::error!("Failed to execute JavaScript: {}", e);
                }
            });
        }
    });

    // Initialize global state object for JavaScript communication
    use_effect({
        let execute_script = execute_script.clone();
        let id = player_id();
        let window = window.clone();
        move || {
            // Ensure per-player state
            let init_script = crate::video_player::ipc::global::init_cp_state(&id);
            execute_script(init_script);

            // Ensure global keyboard action dispatcher is available
            let _ = window
                .webview
                .evaluate_script(&crate::video_player::ipc::global::keyboard_action_handler());
        }
    });

    // set active player and dispatch global keyboard keys to the active player
    use_effect({
        let window = window.clone();
        let local_id = player_id();
        let yt_id = youtube_player_id();
        let state = state.clone();
        move || {
            // Set the active player context based on current source
            let ap_script = match &*state.current_video.read() {
                Some(VideoSource::Local { .. }) => {
                    crate::video_player::ipc::global::set_active_player("local", &local_id)
                }
                Some(VideoSource::YouTube { .. }) => {
                    crate::video_player::ipc::global::set_active_player("youtube", &yt_id)
                }
                None => "window.cpActivePlayer=null;".to_string(),
            };
            let _ = window.webview.evaluate_script(&ap_script);

            // Poll keystrokes from global buffer and dispatch via cpHandleVideoKey
            spawn({
                let window = window.clone();
                async move {
                    let mut interval =
                        tokio::time::interval(tokio::time::Duration::from_millis(100));
                    loop {
                        interval.tick().await;
                        let script = r#"
                                (function() {
                                    if (window.lastVideoPlayerKey && window.cpHandleVideoKey) {
                                        const k = window.lastVideoPlayerKey;
                                        window.lastVideoPlayerKey = null;
                                        try { window.cpHandleVideoKey(k); } catch(e) {}
                                    }
                                })();
                            "#;
                        let _ = window.webview.evaluate_script(script);
                    }
                }
            });
        }
    });

    // Control handlers (currently unused but ready for future implementation)
    let _handle_play_pause = use_callback({
        let execute_script = execute_script.clone();
        let id = player_id();
        move |_: ()| {
            let script = crate::video_player::ipc::local::play_pause(&id);
            execute_script(script);
        }
    });

    let _handle_seek = use_callback({
        let execute_script = execute_script.clone();
        let id = player_id();
        let state = state.clone();
        move |position: f64| {
            let pos = position.max(0.0);
            let script = crate::video_player::ipc::local::seek_to(&id, pos);
            execute_script(script);
            let mut state = state.clone();
            state.position.set(pos);
        }
    });

    let _handle_volume_change = use_callback({
        let execute_script = execute_script.clone();
        let id = player_id();
        let state = state.clone();
        move |new_volume: f64| {
            let vol = new_volume.clamp(0.0, 1.0);
            let script = crate::video_player::ipc::local::set_volume(&id, vol);
            execute_script(script);
            let mut state = state.clone();
            state.set_volume(vol);
        }
    });

    // Container styling
    let container_classes = if *state.is_fullscreen.read() {
        "fixed inset-0 z-50 bg-black flex flex-col"
    } else {
        "relative bg-black rounded-lg overflow-hidden"
    };

    let video_area_style = format!(
        "width: {}; height: {};",
        props.width.as_deref().unwrap_or("100%"),
        props.height.as_deref().unwrap_or("400px")
    );

    // Check for errors outside the rsx! macro
    let has_error = {
        let error_ref = state.error.read();
        error_ref.is_some()
    };

    rsx! {
        div {
            class: "{container_classes}",
            style: if !*state.is_fullscreen.read() { video_area_style } else { String::new() },

            // Render appropriate player based on video source
            match &*state.current_video.read() {
                Some(VideoSource::Local { path, .. }) => rsx! {
                    LocalVideoPlayer {
                        player_id: player_id(),
                        path: path.clone(),
                        autoplay: props.autoplay.unwrap_or(false),
                        on_play: {
                            let mut playback_state = state.playback_state.clone();
                            move |_| {
                                playback_state.set(PlaybackState::Playing)
                            }
                        },
                        on_pause: {
                            let mut playback_state = state.playback_state.clone();
                            move |_| {
                                playback_state.set(PlaybackState::Paused)
                            }
                        },
                        on_ended: {
                            let state = state.clone();
                            let on_complete = props.on_complete.clone();
                            move |_| {
                                let mut state = state.clone();
                                state.stop();
                                if let Some(on_complete) = &on_complete {
                                    on_complete.call(());
                                }
                            }
                        },
                        on_timeupdate: {
                            let state = state.clone();
                            let on_progress = props.on_progress.clone();
                            move |_| {
                                // Position updates handled by sync interval
                                if let Some(on_progress) = &on_progress {
                                    on_progress.call(*state.position.read());
                                }
                            }
                        },
                        on_loadedmetadata: {
                            let mut loading = state.loading.clone();
                            move |_| {
                                loading.set(false);
                            }
                        },
                        on_error: {
                            let state = state.clone();
                            move |error: String| {
                                let mut state = state.clone();
                                state.set_error(Some(VideoPlayerError::PlaybackError(error)));
                            }
                        },
                    }
                },
                Some(VideoSource::YouTube { video_id, playlist_id, .. }) => rsx! {
                    YouTubeVideoPlayer {
                        player_id: youtube_player_id(),
                        video_id: video_id.clone(),
                        playlist_id: playlist_id.clone(),
                        on_state_change: {
                            let mut playback_state = state.playback_state.clone();
                            move |new_playback_state| {
                                // Update the state with the new playback state
                                playback_state.set(new_playback_state);
                            }
                        },
                        on_progress: {
                            let mut state_position = state.position.clone();
                            let on_progress = props.on_progress.clone();
                            move |position| {
                                state_position.set(position);
                                if let Some(on_progress) = &on_progress {
                                    on_progress.call(position);
                                }
                            }
                        },
                        on_duration: {
                            let mut duration = state.duration.clone();
                            move |duration_value| {
                                duration.set(duration_value);
                            }
                        },
                        on_error: {
                            let state = state.clone();
                            move |error: String| {
                                let mut state = state.clone();
                                state.set_error(Some(VideoPlayerError::PlaybackError(error)));
                            }
                        },
                    }
                },
                None => rsx! {
                    // No video loaded
                    div {
                        class: "flex-1 bg-gray-900 flex items-center justify-center",
                        div {
                            class: "text-gray-500 text-center",
                            "No video loaded"
                        }
                    }
                }
            }

            // Video controls overlay
            if props.show_controls.unwrap_or(true) && state.has_video() && !has_error {
                VideoControls {
                    player_id: player_id(),
                    show_playlist_controls: {
                        let current_video_ref = current_video.read();
                        matches!(&*current_video_ref, Some(VideoSource::YouTube { playlist_id: Some(_), .. }))
                    },
                    on_play_pause: {
                        let state = state.clone();
                        let current_video = current_video.clone();
                        let execute_script = execute_script.clone();
                        move |_| {
                            // Handle play/pause for the appropriate player type
                            match &*current_video.read() {
                                Some(VideoSource::Local { .. }) => {
                                    // Mark active and toggle HTML5 <video>
                                    let set_active = crate::video_player::ipc::global::set_active_player("local", &player_id());
                                    execute_script(set_active);
                                    let toggle = crate::video_player::ipc::local::play_pause(&player_id());
                                    execute_script(toggle);

                                    // Update state
                                    let mut state = state.clone();
                                    state.toggle_play_pause();
                                }
                                Some(VideoSource::YouTube { .. }) => {
                                    // Mark active and toggle via keyboard action handler (k)
                                    let set_active = crate::video_player::ipc::global::set_active_player("youtube", &youtube_player_id());
                                    execute_script(set_active);
                                    execute_script("window.cpHandleVideoKey && window.cpHandleVideoKey('k');".to_string());
                                }
                                None => {}
                            }
                        }
                    },
                        on_seek: {
                            let state = state.clone();
                            let current_video = current_video.clone();
                            let execute_script = execute_script.clone();
                            move |position| {
                                match &*current_video.read() {
                                    Some(VideoSource::Local { .. }) => {
                                        let mut state = state.clone();
                                        state.seek_to(position);

                                        // Reflect in DOM
                                        let script = crate::video_player::ipc::local::seek_to(&player_id(), position);
                                        execute_script(script);
                                    }
                                    Some(VideoSource::YouTube { .. }) => {
                                        // Seek YouTube embedded player
                                        let script = crate::video_player::ipc::youtube::seek_to(&youtube_player_id(), position);
                                        execute_script(script);
                                    }
                                    None => {}
                                }
                            }
                        },
                        on_volume_change: {
                            let state = state.clone();
                            let current_video = current_video.clone();
                            let execute_script = execute_script.clone();
                            move |volume| {
                                match &*current_video.read() {
                                    Some(VideoSource::Local { .. }) => {
                                        let mut state = state.clone();
                                        state.set_volume(volume);

                                        // Reflect in DOM
                                        let script = crate::video_player::ipc::local::set_volume(&player_id(), volume);
                                        execute_script(script);
                                    }
                                    Some(VideoSource::YouTube { .. }) => {
                                        // Set volume on YouTube embedded player (0..1 mapped in helper)
                                        let script = crate::video_player::ipc::youtube::set_volume(&youtube_player_id(), volume);
                                        execute_script(script);
                                    }
                                    None => {}
                                }
                            }
                        },
                        on_fullscreen_toggle: {
                            let state = state.clone();
                            move |_| {
                                let mut state = state.clone();
                                state.toggle_fullscreen();
                            }
                        },
                        on_stop: None,
                    }
                }

            // Error overlay
            {
                let error_ref = state.error.read();
                if let Some(error) = &*error_ref {
                    rsx! {
                        VideoErrorDisplay {
                            error: error.clone(),
                            on_retry: {
                                let state = state.clone();
                                let current_video = current_video.clone();
                                move |_| {
                                    let mut state = state.clone();
                                    state.error.set(None);
                                    if let Some(source) = &*current_video.read() {
                                        state.load_video(source.clone());
                                    }
                                }
                            },
                        }
                    }
                } else {
                    rsx! { div {} }
                }
            }

            // Loading overlay
            if *state.loading.read() {
                div {
                    class: "absolute inset-0 bg-black/50 flex items-center justify-center",
                    div {
                        class: "text-white text-center",
                        div { class: "loading loading-spinner loading-lg mb-2" }
                        div { "Loading video..." }
                    }
                }
            }
        }
    }
}

/// Local video player using HTML5 video element
#[component]
fn LocalVideoPlayer(
    player_id: String,
    path: std::path::PathBuf,
    autoplay: bool,
    on_play: EventHandler<()>,
    on_pause: EventHandler<()>,
    on_ended: EventHandler<()>,
    on_timeupdate: EventHandler<()>,
    on_loadedmetadata: EventHandler<()>,
    on_error: EventHandler<String>,
) -> Element {
    // Convert file path to custom protocol URL
    let video_url = format!("local-video://file/{}", path.display());

    rsx! {
        div { class: "flex-1 bg-black relative",
            video {
                id: "{player_id}",
                class: "w-full h-full object-contain",
                autoplay: autoplay,
                controls: true,
                preload: "metadata",
                src: "{video_url}",

                // Event handlers
                onplay: move |_| on_play.call(()),
                onpause: move |_| on_pause.call(()),
                onended: move |_| on_ended.call(()),
                ontimeupdate: move |_| on_timeupdate.call(()),
                onloadedmetadata: move |_| on_loadedmetadata.call(()),
                onerror: move |_| on_error.call("Video playback error".to_string()),
                oncanplay: move |_| {
                    // Video is ready to play
                },
                onwaiting: move |_| {
                    // Video is buffering
                },
            }

            // Native controls are now enabled, no overlay needed
        }
    }
}

/// YouTube embedded player (YouTube IFrame API)
#[component]
fn YouTubeVideoPlayer(
    player_id: String,
    video_id: String,
    playlist_id: Option<String>,
    on_state_change: EventHandler<PlaybackState>,
    on_progress: EventHandler<f64>,
    on_duration: EventHandler<f64>,
    on_error: EventHandler<String>,
) -> Element {
    let window = use_window();
    // use_eval removed (no JSON polling)
    let _api_ready = use_signal(|| false);

    // Execute JavaScript in webview
    let execute_script = use_callback({
        let window = window.clone();
        move |script: String| {
            let window = window.clone();
            spawn(async move {
                if let Err(e) = window.webview.evaluate_script(&script) {
                    log::error!("Failed to execute YouTube script: {}", e);
                }
            });
        }
    });

    // Initialize YouTube API
    use_effect({
        let execute_script = execute_script.clone();
        move || {
            let init_script = crate::video_player::ipc::youtube::load_api();
            execute_script(init_script);
        }
    });

    // Create YouTube player
    use_effect({
        let execute_script = execute_script.clone();
        let player_id_val = player_id.clone();
        let video_id_val = video_id.clone();
        let playlist_opt = playlist_id.clone();
        move || {
            let create_script = crate::video_player::ipc::youtube::create_player(
                &player_id_val,
                &video_id_val,
                playlist_opt.as_deref(),
            );
            execute_script(create_script);
        }
    });

    // YouTube polling removed (handled via future IPC bridge)

    rsx! {
        div { class: "flex-1 bg-gray-900 relative",
            // Embedded iframe will be inserted here via the YouTube IFrame API (no HTML5 <video>)
            div {
                id: "{player_id}",
                class: "w-full h-full",
                style: "min-height: 200px;",
            }
        }
    }
}

/// Error display component
#[component]
fn VideoErrorDisplay(error: VideoPlayerError, on_retry: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "absolute inset-0 bg-red-900/50 flex items-center justify-center",
            div {
                class: "text-white text-center p-6 max-w-md",
                div { class: "text-4xl mb-4", "❌" }
                div {
                    class: "text-lg font-semibold mb-2",
                    "Video Error"
                }
                div {
                    class: "text-sm text-gray-300 mb-4",
                    "{error.user_message()}"
                }

                // Recovery suggestions
                div { class: "text-xs text-gray-400 mb-4",
                    for suggestion in error.recovery_suggestions() {
                        div { "• {suggestion}" }
                    }
                }

                // Retry button
                button {
                    class: "btn btn-primary btn-sm",
                    onclick: move |_| on_retry.call(()),
                    "Try Again"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_player_props() {
        // Test that props can be created and cloned
        let props = VideoPlayerProps {
            source: None,
            width: Some("800px".to_string()),
            height: Some("600px".to_string()),
            show_controls: Some(true),
            autoplay: Some(false),
            on_progress: None,
            on_complete: None,
            on_error: None,
        };

        let cloned_props = props.clone();
        assert_eq!(props.width, cloned_props.width);
        assert_eq!(props.height, cloned_props.height);
    }
}
