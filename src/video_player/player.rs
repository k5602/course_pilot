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
        move || {
            let init_script = format!(
                r#"
                if (!window.cpVideoState) {{
                    window.cpVideoState = {{}};
                }}
                window.cpVideoState['{}'] = {{
                    currentTime: 0,
                    duration: 0,
                    volume: 1.0,
                    muted: false,
                    paused: true
                }};
            "#,
                id
            );
            execute_script(init_script);
        }
    });

    // Sync state from JavaScript to Rust periodically
    use_effect(move || {
        let execute_script = execute_script.clone();
        let id = player_id();
        let _on_progress = props.on_progress.clone();
        // Set up periodic sync using tokio interval
        spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
            loop {
                interval.tick().await;
                let sync_script = format!(
                    r#"
                        (function() {{
                            const videoState = window.cpVideoState['{}'];
                            if (!videoState) return null;

                            const el = document.getElementById('cp-video-{}');
                            if (el) {{
                                videoState.currentTime = el.currentTime || 0;
                                videoState.duration = isFinite(el.duration) ? (el.duration || 0) : 0;
                                videoState.volume = (typeof el.volume === 'number') ? el.volume : 1.0;
                                videoState.muted = !!el.muted;
                                videoState.paused = !!el.paused;
                            }}

                            return JSON.stringify(videoState);
                        }})()
                    "#,
                    id, id
                );

                execute_script(sync_script);
            }
        });
    });

    // Control handlers (currently unused but ready for future implementation)
    let _handle_play_pause = use_callback({
        let execute_script = execute_script.clone();
        let id = player_id();
        move |_: ()| {
            let script = format!(
                r#"
                (function() {{
                    const el = document.getElementById('cp-video-{}');
                    if (!el) return;
                    if (el.paused) {{
                        el.play().catch(e => console.error('Play failed:', e));
                    }} else {{
                        el.pause();
                    }}
                }})()
            "#,
                id
            );
            execute_script(script);
        }
    });

    let _handle_seek = use_callback({
        let execute_script = execute_script.clone();
        let id = player_id();
        let state = state.clone();
        move |position: f64| {
            let pos = position.max(0.0);
            let script = format!(
                r#"
                (function() {{
                    const el = document.getElementById('cp-video-{}');
                    if (!el) return;
                    el.currentTime = {};
                }})()
            "#,
                id, pos
            );
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
            let script = format!(
                r#"
                (function() {{
                    const el = document.getElementById('cp-video-{}');
                    if (!el) return;
                    el.volume = {};
                    el.muted = ({} <= 0);
                }})()
            "#,
                id, vol, vol
            );
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
                            move |_| {
                                // Handle play/pause for the appropriate player type
                                match &*current_video.read() {
                                    Some(VideoSource::Local { .. }) => {
                                        // Local player toggle
                                        let mut state = state.clone();
                                        state.toggle_play_pause();
                                    }
                                    Some(VideoSource::YouTube { .. }) => {
                                        // YouTube controls will be handled by the YouTube component
                                    }
                                    None => {}
                                }
                            }
                        },
                        on_seek: {
                            let state = state.clone();
                            let current_video = current_video.clone();
                            move |position| {
                                match &*current_video.read() {
                                    Some(VideoSource::Local { .. }) => {
                                        let mut state = state.clone();
                                        state.seek_to(position);
                                    }
                                    Some(VideoSource::YouTube { .. }) => {
                                        // YouTube seek will be handled by the YouTube component
                                    }
                                    None => {}
                                }
                            }
                        },
                        on_volume_change: {
                            let state = state.clone();
                            let current_video = current_video.clone();
                            move |volume| {
                                match &*current_video.read() {
                                    Some(VideoSource::Local { .. }) => {
                                        let mut state = state.clone();
                                        state.set_volume(volume);
                                    }
                                    Some(VideoSource::YouTube { .. }) => {
                                        // YouTube volume will be handled by the YouTube component
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

/// YouTube video player using iframe
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
            let init_script = r#"
                // Load YouTube IFrame Player API if not already loaded
                if (!window.YT) {
                    console.log('Loading YouTube IFrame Player API...');
                    const tag = document.createElement('script');
                    tag.src = 'https://www.youtube.com/iframe_api';
                    const firstScriptTag = document.getElementsByTagName('script')[0];
                    firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);

                    // Set up global callback
                    window.onYouTubeIframeAPIReady = function() {
                        console.log('YouTube IFrame Player API ready');
                        window.ytAPIReady = true;
                    };
                } else {
                    console.log('YouTube IFrame Player API already loaded');
                    window.ytAPIReady = true;
                }
            "#
            .to_string();

            execute_script(init_script);
        }
    });

    // Create YouTube player
    use_effect({
        let execute_script = execute_script.clone();
        let player_id_val = player_id.clone();
        let video_id_val = video_id.clone();
        let playlist_param = playlist_id
            .as_ref()
            .map(|p| format!(", list: '{}'", p))
            .unwrap_or_default();

        move || {
            let create_script = format!(
                r#"
                function createYouTubePlayer() {{
                    if (window.YT && window.YT.Player) {{
                        console.log('Creating YouTube player for video: {}');

                        try {{
                            window.ytPlayer_{} = new YT.Player('{}', {{
                                height: '100%',
                                width: '100%',
                                videoId: '{}'{},
                                playerVars: {{
                                    'enablejsapi': 1,
                                    'origin': window.location.origin,
                                    'playsinline': 1,
                                    'rel': 0,
                                    'modestbranding': 1,
                                    'controls': 0,
                                    'disablekb': 1,
                                    'fs': 0,
                                    'iv_load_policy': 3
                                }},
                                events: {{
                                    'onReady': function(event) {{
                                        console.log('YouTube player ready: {}');
                                        window.ytPlayerReady_{} = true;
                                        window.ytPlayerInstance_{} = event.target;
                                    }},
                                    'onStateChange': function(event) {{
                                        console.log('YouTube player state changed: {}', event.data);
                                        window.ytPlayerState_{} = event.data;

                                        // Update position and duration
                                        if (event.target) {{
                                            try {{
                                                window.ytPlayerPosition_{} = event.target.getCurrentTime() || 0;
                                                window.ytPlayerDuration_{} = event.target.getDuration() || 0;
                                            }} catch (e) {{
                                                console.warn('Error getting player time info:', e);
                                            }}
                                        }}
                                    }},
                                    'onError': function(event) {{
                                        console.error('YouTube player error: {}', event.data);
                                        window.ytPlayerError_{} = event.data;
                                    }}
                                }}
                            }});
                        }} catch (e) {{
                            console.error('Error creating YouTube player:', e);
                            window.ytPlayerError_{} = 'Player creation failed';
                        }}
                    }} else {{
                        console.log('YouTube API not ready yet, retrying...');
                        setTimeout(createYouTubePlayer, 100);
                    }}
                }}

                // Wait for API to be ready, then create the player
                if (window.ytAPIReady) {{
                    createYouTubePlayer();
                }} else {{
                    const checkAPI = setInterval(function() {{
                        if (window.ytAPIReady) {{
                            clearInterval(checkAPI);
                            createYouTubePlayer();
                        }}
                    }}, 100);

                    // Timeout after 10 seconds
                    setTimeout(function() {{
                        if (!window.ytAPIReady) {{
                            clearInterval(checkAPI);
                            console.error('YouTube API failed to load within timeout');
                            window.ytPlayerError_{} = 'YouTube API load timeout';
                        }}
                    }}, 10000);
                }}
            "#,
                video_id_val,
                player_id_val,
                player_id_val,
                video_id_val,
                playlist_param,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val,
                player_id_val
            );

            execute_script(create_script);
        }
    });

    rsx! {
        div { class: "flex-1 bg-gray-900 relative",
            // YouTube iframe will be inserted here by JavaScript
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
