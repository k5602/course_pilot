use dioxus::prelude::*;

use crate::ui::hooks::use_video_player_manager;
use crate::video_player::{PlaybackState, VideoPlayer, VideoSource};

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

#[component]
fn VideoPlayerShortcutsInit() -> Element {
    rsx! { div {} }
}

/// Cross-platform video player component wrapper
#[component]
pub fn VideoPlayerComponent(props: VideoPlayerProps) -> Element {
    // Initialize video player manager for keyboard shortcuts and state management
    let _manager = use_video_player_manager();

    rsx! {
        VideoPlayerShortcutsInit {}
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_format_time() {
        // This would test the time formatting function if it were extracted
        // For now, this is a placeholder test
        assert!(true);
    }
}
