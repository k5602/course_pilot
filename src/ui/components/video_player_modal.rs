//! Video player modal using new YouTube/Local player components

use crate::video_player::{LocalPlayer, PlaybackState, VideoSource, YouTubePlayer};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct VideoPlayerModalProps {
    /// Whether the modal is open
    pub is_open: bool,
    /// Callback to close the modal
    pub on_close: Callback<()>,
    /// Video source to play
    pub video_source: Option<VideoSource>,
    /// Title to display in modal header
    pub title: Option<String>,
}

/// video player modal using YouTube IFrame or HTML5 video
#[component]
pub fn VideoPlayerModal(props: VideoPlayerModalProps) -> Element {
    if !props.is_open || props.video_source.is_none() {
        return rsx! {};
    }

    let source = props.video_source.clone().unwrap();
    let modal_title = props.title.clone().unwrap_or_else(|| source.title().to_string());

    let close_handler = {
        let on_close = props.on_close.clone();
        move |_| on_close.call(())
    };

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
            onclick: close_handler,

            // Modal content
            div {
                class: "bg-base-100 rounded-lg shadow-2xl max-w-4xl w-full mx-4 overflow-hidden",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    class: "flex justify-between items-center p-4 border-b border-base-300",
                    h3 { class: "text-lg font-semibold truncate", "{modal_title}" }
                    button {
                        class: "btn btn-sm btn-circle btn-ghost",
                        onclick: close_handler,
                        "✕"
                    }
                }

                // Video player area
                div {
                    class: "bg-black aspect-video",
                    {render_player(&source)}
                }

                // Footer
                div {
                    class: "flex justify-end p-4 border-t border-base-300",
                    button {
                        class: "btn btn-outline btn-sm",
                        onclick: close_handler,
                        "Close"
                    }
                }
            }
        }
    }
}

/// Render the appropriate player based on video source
fn render_player(source: &VideoSource) -> Element {
    match source {
        VideoSource::YouTube { video_id, playlist_id, title } => {
            rsx! {
                YouTubePlayer {
                    video_id: video_id.clone(),
                    playlist_id: playlist_id.clone(),
                    class: "w-full h-full".to_string(),
                }
            }
        },
        VideoSource::Local { path, title } => {
            rsx! {
                LocalPlayer {
                    path: path.to_string_lossy().to_string(),
                    title: title.clone(),
                    class: "w-full h-full".to_string(),
                }
            }
        },
    }
}

/// Hook for managing video player modal state
pub fn use_video_player_modal() -> VideoPlayerModalManager {
    let mut is_open = use_signal(|| false);
    let mut current_video = use_signal(|| None::<VideoSource>);
    let mut current_title = use_signal(|| None::<String>);

    let open_video = use_callback({
        move |(source, title): (VideoSource, Option<String>)| {
            current_video.set(Some(source));
            current_title.set(title);
            is_open.set(true);
        }
    });

    let close = use_callback({
        move |_| {
            is_open.set(false);
        }
    });

    VideoPlayerModalManager {
        is_open: is_open(),
        video_source: current_video,
        title: current_title,
        open_video,
        close,
    }
}

#[derive(Clone)]
pub struct VideoPlayerModalManager {
    pub is_open: bool,
    pub video_source: Signal<Option<VideoSource>>,
    pub title: Signal<Option<String>>,
    pub open_video: Callback<(VideoSource, Option<String>)>,
    pub close: Callback<()>,
}
