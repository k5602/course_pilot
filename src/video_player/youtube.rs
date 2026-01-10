//! YouTube video player component
//!
//! Simple YouTube IFrame API integration inspired by Yudoku.
//! Uses onStateChange callback for progress tracking.

use crate::video_player::{PlaybackState, VideoSource};
use dioxus::prelude::*;

/// YouTube player component props
#[derive(Props, Clone, PartialEq)]
pub struct YouTubePlayerProps {
    /// YouTube video ID
    pub video_id: String,
    /// Optional playlist ID
    #[props(optional)]
    pub playlist_id: Option<String>,
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

/// Simple YouTube player using iframe embed
///
/// This is a distraction-free player that:
/// - Embeds YouTube via iframe
/// - Tracks playback state via YouTube IFrame API
/// - Reports progress for persistence
#[component]
pub fn YouTubePlayer(props: YouTubePlayerProps) -> Element {
    let player_id = use_signal(|| format!("yt-player-{}", uuid::Uuid::new_v4()));

    // Build the embed URL with best practices for course viewing
    let embed_url = {
        let mut url = format!(
            "https://www.youtube.com/embed/{}?enablejsapi=1&rel=0&modestbranding=1&origin={}",
            props.video_id, "tauri://localhost"
        );

        if let Some(playlist) = &props.playlist_id {
            url.push_str(&format!("&list={playlist}"));
        }

        if props.start_time > 0.0 {
            url.push_str(&format!("&start={}", props.start_time as u64));
        }

        url
    };

    let class = props.class.clone().unwrap_or_default();

    rsx! {
        div {
            class: "youtube-player-container {class}",
            style: "position: relative; width: 100%; padding-bottom: 56.25%; /* 16:9 aspect ratio */",

            iframe {
                id: "{player_id}",
                src: "{embed_url}",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: none;",
                allow: "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture",
                allowfullscreen: true,
                title: "YouTube video player",
            }
        }
    }
}

/// Generate JavaScript to initialize YouTube IFrame API
pub fn youtube_api_init_script(
    player_div_id: &str,
    video_id: &str,
    playlist_id: Option<&str>,
) -> String {
    let playlist_param = playlist_id.map(|p| format!(", list: '{p}'")).unwrap_or_default();

    format!(
        r#"
        if (!window.YT) {{
            var tag = document.createElement('script');
            tag.src = "https://www.youtube.com/iframe_api";
            var firstScriptTag = document.getElementsByTagName('script')[0];
            firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
        }}
        
        function onYouTubeIframeAPIReady() {{
            window.ytPlayer_{player_div_id} = new YT.Player('{player_div_id}', {{
                videoId: '{video_id}'{playlist_param},
                events: {{
                    'onStateChange': function(event) {{
                        window.ytPlayerState_{player_div_id} = event.data;
                    }},
                    'onReady': function(event) {{
                        window.ytPlayerReady_{player_div_id} = true;
                    }}
                }}
            }});
        }}
        
        if (window.YT && window.YT.Player) {{
            onYouTubeIframeAPIReady();
        }}
    "#
    )
}

/// JavaScript to get current playback state
pub fn youtube_get_state_script(player_div_id: &str) -> String {
    format!(
        r#"
        (function() {{
            var player = window.ytPlayer_{player_div_id};
            if (!player || !player.getCurrentTime) return null;
            return JSON.stringify({{
                currentTime: player.getCurrentTime(),
                duration: player.getDuration(),
                volume: player.getVolume() / 100,
                isMuted: player.isMuted(),
                playbackRate: player.getPlaybackRate(),
                playerState: player.getPlayerState()
            }});
        }})()
    "#
    )
}

/// JavaScript to control playback
pub fn youtube_play_script(player_div_id: &str) -> String {
    format!("window.ytPlayer_{player_div_id}?.playVideo()")
}

pub fn youtube_pause_script(player_div_id: &str) -> String {
    format!("window.ytPlayer_{player_div_id}?.pauseVideo()")
}

pub fn youtube_seek_script(player_div_id: &str, seconds: f64) -> String {
    format!("window.ytPlayer_{player_div_id}?.seekTo({seconds}, true)")
}

pub fn youtube_set_volume_script(player_div_id: &str, volume: f64) -> String {
    let vol_percent = (volume * 100.0).clamp(0.0, 100.0);
    format!("window.ytPlayer_{player_div_id}?.setVolume({vol_percent})")
}
