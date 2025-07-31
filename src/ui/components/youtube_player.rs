use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaBackward, FaCompress, FaExpand, FaForward, FaPause, FaPlay, FaStop, FaVolumeHigh,
    FaVolumeXmark,
};
use uuid::Uuid;

use crate::video_player::{PlaybackState, VideoSource};

#[derive(Props, PartialEq, Clone)]
pub struct YouTubePlayerProps {
    pub video_source: Option<VideoSource>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub show_controls: Option<bool>,
    pub autoplay: Option<bool>,
    pub on_state_change: Option<EventHandler<PlaybackState>>,
    pub on_position_change: Option<EventHandler<f64>>,
    pub on_error: Option<EventHandler<String>>,
}

/// YouTube embedded player component using IFrame Player API
#[component]
pub fn YouTubePlayer(props: YouTubePlayerProps) -> Element {
    let mut current_state = use_signal(|| PlaybackState::Stopped);
    let mut current_position = use_signal(|| 0.0);
    let duration = use_signal(|| 0.0);
    let mut volume = use_signal(|| 1.0);
    let mut is_fullscreen = use_signal(|| false);
    let mut is_muted = use_signal(|| false);
    let show_controls = use_signal(|| props.show_controls.unwrap_or(true));
    let player_id = use_signal(|| format!("youtube-player-{}", Uuid::new_v4().simple()));
    let _is_api_ready = use_signal(|| false);
    let window = use_window();

    // Helper function to execute JavaScript
    let execute_script = use_callback({
        let window = window.clone();
        move |script: String| {
            let window = window.clone();
            spawn(async move {
                match window.webview.evaluate_script(&script) {
                    Ok(_) => {
                        log::debug!("Successfully executed JavaScript");
                    }
                    Err(e) => {
                        log::error!("Failed to execute JavaScript: {}", e);
                    }
                }
            });
        }
    });

    // Initialize YouTube API
    use_effect({
        let execute_script = execute_script.clone();
        move || {
            let script = r#"
                // Load YouTube IFrame Player API if not already loaded
                if (!window.YT) {
                    console.log('Loading YouTube IFrame Player API...');
                    var tag = document.createElement('script');
                    tag.src = 'https://www.youtube.com/iframe_api';
                    var firstScriptTag = document.getElementsByTagName('script')[0];
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
            "#.to_string();
            
            execute_script(script);
            log::info!("YouTube IFrame Player API initialization started");
        }
    });

    // Load video when source changes
    use_effect({
        let video_source = props.video_source.clone();
        let execute_script = execute_script.clone();
        let player_id_val = player_id();
        move || {
            if let Some(source) = video_source.clone() {
                match source {
                    VideoSource::YouTube { video_id, playlist_id, title } => {
                        let playlist_param = if let Some(playlist) = playlist_id {
                            format!(", list: '{playlist}'")
                        } else {
                            String::new()
                        };

                        let script = format!(r#"
                            // Function to create the player
                            function createYouTubePlayer() {{
                                if (window.YT && window.YT.Player) {{
                                    console.log('Creating YouTube player for video: {video_id}');
                                    
                                    window.ytPlayer_{player_id} = new YT.Player('{player_id}', {{
                                        height: '100%',
                                        width: '100%',
                                        videoId: '{video_id}'{playlist_param},
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
                                                console.log('YouTube player ready: {player_id}');
                                                window.ytPlayerReady_{player_id} = true;
                                                window.ytPlayerInstance_{player_id} = event.target;
                                            }},
                                            'onStateChange': function(event) {{
                                                console.log('YouTube player state changed: {player_id}', event.data);
                                                window.ytPlayerState_{player_id} = event.data;
                                                
                                                // Update position and duration
                                                if (event.target) {{
                                                    window.ytPlayerPosition_{player_id} = event.target.getCurrentTime() || 0;
                                                    window.ytPlayerDuration_{player_id} = event.target.getDuration() || 0;
                                                }}
                                            }},
                                            'onError': function(event) {{
                                                console.error('YouTube player error: {player_id}', event.data);
                                                window.ytPlayerError_{player_id} = event.data;
                                            }}
                                        }}
                                    }});
                                }} else {{
                                    console.log('YouTube API not ready yet, retrying...');
                                    setTimeout(createYouTubePlayer, 100);
                                }}
                            }}

                            // Wait for API to be ready, then create the player
                            if (window.ytAPIReady) {{
                                createYouTubePlayer();
                            }} else {{
                                var checkAPI = setInterval(function() {{
                                    if (window.ytAPIReady) {{
                                        clearInterval(checkAPI);
                                        createYouTubePlayer();
                                    }}
                                }}, 100);
                            }}
                        "#, 
                        player_id = player_id_val, 
                        video_id = video_id,
                        playlist_param = playlist_param
                        );

                        execute_script(script);
                        current_state.set(PlaybackState::Stopped);
                        log::info!("YouTube video loading initiated: {video_id} ({})", title);
                    }
                    VideoSource::Local { .. } => {
                        log::error!("Local videos not supported by YouTube player");
                        current_state.set(PlaybackState::Error);
                    }
                }
            }
        }
    });

    // Helper function to execute player methods
    let execute_player_method = use_callback({
        let execute_script = execute_script.clone();
        let player_id_val = player_id();
        move |(method, args): (String, String)| {
            let script = format!(
                r#"
                if (window.ytPlayerInstance_{} && window.ytPlayerInstance_{}.{}) {{
                    try {{
                        window.ytPlayerInstance_{}.{}({});
                        console.log('Executed YouTube player method: {}({})');
                    }} catch (e) {{
                        console.error('Error executing YouTube player method {}:', e);
                    }}
                }} else {{
                    console.warn('YouTube player not ready for method: {}');
                }}
                "#,
                player_id_val, player_id_val, method, player_id_val, method, args, method, args, method, method
            );
            execute_script(script);
        }
    });

    // Control handlers
    let handle_play_pause = use_callback({
        let execute_player_method = execute_player_method.clone();
        move |_| {
            match current_state() {
                PlaybackState::Playing => {
                    execute_player_method(("pauseVideo".to_string(), "".to_string()));
                    current_state.set(PlaybackState::Paused);
                }
                PlaybackState::Paused | PlaybackState::Stopped => {
                    execute_player_method(("playVideo".to_string(), "".to_string()));
                    current_state.set(PlaybackState::Playing);
                }
                _ => {}
            }
        }
    });

    let handle_stop = use_callback({
        let execute_player_method = execute_player_method.clone();
        move |_| {
            execute_player_method(("stopVideo".to_string(), "".to_string()));
            current_state.set(PlaybackState::Stopped);
            current_position.set(0.0);
        }
    });

    let handle_seek = use_callback({
        let execute_player_method = execute_player_method.clone();
        move |position: f64| {
            let clamped_position = position.max(0.0);
            execute_player_method(("seekTo".to_string(), format!("{}, true", clamped_position)));
            current_position.set(clamped_position);
        }
    });

    let handle_volume_change = use_callback({
        let execute_player_method = execute_player_method.clone();
        move |new_volume: f64| {
            let clamped_volume = new_volume.clamp(0.0, 1.0);
            let youtube_volume = (clamped_volume * 100.0) as i32; // YouTube API expects 0-100
            execute_player_method(("setVolume".to_string(), youtube_volume.to_string()));
            volume.set(clamped_volume);
            is_muted.set(clamped_volume == 0.0);
        }
    });

    let handle_mute_toggle = use_callback({
        let execute_player_method = execute_player_method.clone();
        move |_| {
            if is_muted() {
                execute_player_method(("unMute".to_string(), "".to_string()));
            } else {
                execute_player_method(("mute".to_string(), "".to_string()));
            }
            is_muted.set(!is_muted());
        }
    });

    let handle_fullscreen_toggle = use_callback({
        let execute_script = execute_script.clone();
        let player_id_val = player_id();
        move |_| {
            let new_fullscreen = !is_fullscreen();
            let script = if new_fullscreen {
                format!(
                    r#"
                    var iframe = document.getElementById('{}');
                    if (iframe && iframe.requestFullscreen) {{
                        iframe.requestFullscreen();
                    }} else if (iframe && iframe.webkitRequestFullscreen) {{
                        iframe.webkitRequestFullscreen();
                    }} else if (iframe && iframe.mozRequestFullScreen) {{
                        iframe.mozRequestFullScreen();
                    }}
                    "#,
                    player_id_val
                )
            } else {
                r#"
                if (document.exitFullscreen) {
                    document.exitFullscreen();
                } else if (document.webkitExitFullscreen) {
                    document.webkitExitFullscreen();
                } else if (document.mozCancelFullScreen) {
                    document.mozCancelFullScreen();
                }
                "#.to_string()
            };
            execute_script(script);
            is_fullscreen.set(new_fullscreen);
        }
    });

    let handle_next_video = use_callback({
        let execute_player_method = execute_player_method.clone();
        move |_| {
            execute_player_method(("nextVideo".to_string(), "".to_string()));
        }
    });

    let handle_previous_video = use_callback({
        let execute_player_method = execute_player_method.clone();
        move |_| {
            execute_player_method(("previousVideo".to_string(), "".to_string()));
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

    // Check if we have a YouTube video to display
    let has_youtube_video = matches!(props.video_source, Some(VideoSource::YouTube { .. }));

    rsx! {
        div {
            class: "{container_classes}",
            style: if !is_fullscreen() { video_area_style } else { String::new() },

            // YouTube iframe container
            div {
                class: "flex-1 bg-gray-900 relative",

                if has_youtube_video {
                    // YouTube iframe will be inserted here by JavaScript
                    div {
                        id: "{player_id()}",
                        class: "w-full h-full",
                        style: "min-height: 200px;",
                    }
                } else {
                    // Placeholder when no video is loaded
                    div {
                        class: "flex items-center justify-center h-full text-white text-center",
                        div {
                            class: "text-gray-500",
                            "No YouTube video loaded"
                        }
                    }
                }

                // Loading overlay
                if current_state() == PlaybackState::Buffering {
                    div {
                        class: "absolute inset-0 bg-black/50 flex items-center justify-center",
                        div {
                            class: "text-white text-center",
                            div { class: "loading loading-spinner loading-lg mb-2" }
                            div { "Loading..." }
                        }
                    }
                }

                // Error overlay
                if current_state() == PlaybackState::Error {
                    div {
                        class: "absolute inset-0 bg-red-900/50 flex items-center justify-center",
                        div {
                            class: "text-white text-center",
                            div { class: "text-4xl mb-2", "❌" }
                            div { "Error loading video" }
                        }
                    }
                }
            }

            // Controls
            if show_controls() && has_youtube_video {
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
                                onclick: move |_evt| {
                                    // Calculate click position and seek
                                    // This would need proper event handling to get click position
                                    let percentage = 0.5; // Placeholder
                                    let new_position = duration() * percentage;
                                    handle_seek(new_position);
                                },

                                // Progress fill
                                div {
                                    class: "h-full bg-red-600 rounded-full transition-all duration-200",
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

                            // Previous video (for playlists)
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-red-500",
                                onclick: handle_previous_video,
                                "aria-label": "Previous video",
                                title: "Previous video",

                                Icon {
                                    icon: FaBackward,
                                    class: "w-4 h-4"
                                }
                            }

                            // Play/Pause button
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-red-500",
                                onclick: handle_play_pause,
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
                                class: "btn btn-ghost btn-sm text-white hover:text-red-500",
                                onclick: handle_stop,
                                "aria-label": "Stop",

                                Icon {
                                    icon: FaStop,
                                    class: "w-4 h-4"
                                }
                            }

                            // Next video (for playlists)
                            button {
                                class: "btn btn-ghost btn-sm text-white hover:text-red-500",
                                onclick: handle_next_video,
                                "aria-label": "Next video",
                                title: "Next video",

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
                                    class: "btn btn-ghost btn-sm text-white hover:text-red-500",
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
                                    class: "range range-error range-sm w-20",
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
                                class: "btn btn-ghost btn-sm text-white hover:text-red-500",
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
    use super::*;

    #[test]
    fn test_format_time() {
        // Test time formatting
        assert_eq!("00:30", format!("{:02}:{:02}", 0, 30));
        assert_eq!("01:05", format!("{:02}:{:02}", 1, 5));
    }
}