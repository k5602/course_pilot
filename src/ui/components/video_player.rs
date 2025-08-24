use dioxus::prelude::*;

use crate::video_player::{
    PlaybackState, VideoPlayer, VideoPlayerProvider, VideoSource, use_video_keyboard_shortcuts,
};

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

/// Cross-platform video player component wrapper
#[component]
pub fn VideoPlayerComponent(props: VideoPlayerProps) -> Element {
    // Set up keyboard shortcuts
    let _shortcuts = use_video_keyboard_shortcuts();

    rsx! {
        VideoPlayerProvider {
            VideoPlayer {
                source: props.video_source,
                width: props.width,
                height: props.height,
                show_controls: props.show_controls,
                autoplay: props.autoplay,
                on_progress: props.on_position_change,
                on_complete: move |_| {
                    if let Some(on_state_change) = &props.on_state_change {
                        on_state_change.call(PlaybackState::Stopped);
                    }
                },
                on_error: props.on_error,
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
