use crate::ui::components::video_player::VideoPlayerComponent;
use crate::ui::hooks::use_modal_manager;
use crate::video_player::{PlaybackState, VideoInfo, VideoSource};
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

/// Video player modal component that integrates the video player system
#[component]
pub fn VideoPlayerModal(props: VideoPlayerModalProps) -> Element {
    let mut video_info = use_signal(|| None::<VideoInfo>);
    let mut playback_state = use_signal(|| PlaybackState::Stopped);
    let mut error_message = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    // Initialize video when source changes
    let video_source_clone = props.video_source.clone();
    use_effect(move || {
        log::info!("VideoPlayerModal: Source changed to: {:?}", video_source_clone);
        if let Some(source) = &video_source_clone {
            if props.is_open {
                log::info!(
                    "VideoPlayerModal: Modal is open, initializing video: {}",
                    source.title()
                );
                is_loading.set(true);
                error_message.set(None);

                // Create video info from source
                let info = VideoInfo::new(source.clone());
                video_info.set(Some(info));
                playback_state.set(PlaybackState::Buffering);

                // Simulate loading completion (in real implementation, this would be based on player events)
                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    is_loading.set(false);
                    playback_state.set(PlaybackState::Playing);
                });
            } else {
                log::info!("VideoPlayerModal: Modal is closed, not initializing video");
            }
        } else {
            log::info!("VideoPlayerModal: No video source provided");
        }
    });

    // Player controls
    let play_pause_handler = move |_| {
        let current_state = playback_state();
        match current_state {
            PlaybackState::Playing => {
                playback_state.set(PlaybackState::Paused);
                log::info!("Video paused");
            },
            PlaybackState::Paused | PlaybackState::Stopped => {
                playback_state.set(PlaybackState::Playing);
                log::info!("Video playing");
            },
            _ => {},
        }
    };

    let stop_handler = move |_| {
        playback_state.set(PlaybackState::Stopped);
        if let Some(mut info) = video_info() {
            info.position_seconds = 0.0;
            video_info.set(Some(info));
        }
        log::info!("Video stopped");
    };

    let close_handler = move |_| {
        playback_state.set(PlaybackState::Stopped);
        video_info.set(None);
        error_message.set(None);
        props.on_close.call(());
    };

    let modal_title = props
        .title
        .clone()
        .or_else(|| props.video_source.as_ref().map(|s| s.title().to_string()))
        .unwrap_or_else(|| "Video Player".to_string());

    if !props.is_open {
        return rsx! {};
    }

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            onclick: close_handler,

            // Modal content
            div {
                class: "bg-base-100 rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden mx-4",
                onclick: move |evt| evt.stop_propagation(),

                // Modal header
                div {
                    class: "flex justify-between items-center p-6 border-b border-base-300",

                    h3 {
                        class: "text-lg font-semibold",
                        "{modal_title}"
                    }

                    button {
                        class: "btn btn-sm btn-circle btn-ghost",
                        onclick: close_handler,
                        "✕"
                    }
                }

                // Modal body
                div {
                    class: "p-6 space-y-4",

                    // Video player area
                    div {
                        class: "bg-black rounded-lg aspect-video flex items-center justify-center relative overflow-hidden",

                        // Use the VideoPlayerComponent which provides VideoPlayerProvider context
                        VideoPlayerComponent {
                            video_source: props.video_source.clone(),
                            width: Some("100%".to_string()),
                            height: Some("100%".to_string()),
                            show_controls: Some(true),
                            autoplay: Some(false), // Don't autoplay to avoid browser blocking
                            on_position_change: move |position| {
                                if let Some(mut info) = video_info() {
                                    info.position_seconds = position;
                                    video_info.set(Some(info));
                                }
                            },
                            on_state_change: move |state| {
                                playback_state.set(state);
                            },
                            on_error: move |error| {
                                error_message.set(Some(error));
                                playback_state.set(PlaybackState::Error);
                                is_loading.set(false);
                                log::error!("Video playback error: {}", error_message().unwrap_or_default());
                            },
                        }

                        // Loading overlay
                        if is_loading() {
                            div {
                                class: "absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center",
                                div { class: "loading loading-spinner loading-lg text-white" }
                            }
                        }

                        // Error overlay
                        if let Some(error) = error_message() {
                            div {
                                class: "absolute inset-0 bg-black bg-opacity-75 flex items-center justify-center p-4",
                                div {
                                    class: "text-white text-center",
                                    div { class: "text-4xl mb-2", "⚠️" }
                                    p { class: "text-lg mb-2", "Playback Error" }
                                    p { class: "text-sm opacity-75", "{error}" }
                                }
                            }
                        }
                    }

                    // Video info
                    if let Some(info) = video_info() {
                        div {
                            class: "bg-base-200 rounded-lg p-4",
                            div {
                                class: "grid grid-cols-2 gap-4 text-sm",
                                div {
                                    span { class: "font-semibold", "Title: " }
                                    span { "{info.title()}" }
                                }
                                div {
                                    span { class: "font-semibold", "Status: " }
                                    span {
                                        class: match playback_state() {
                                            PlaybackState::Playing => "text-success",
                                            PlaybackState::Paused => "text-warning",
                                            PlaybackState::Error => "text-error",
                                            _ => "text-base-content",
                                        },
                                        "{playback_state():?}"
                                    }
                                }
                                if let Some(_duration) = info.duration_seconds {
                                    div {
                                        span { class: "font-semibold", "Duration: " }
                                        span { "{info.format_duration()}" }
                                    }
                                    div {
                                        span { class: "font-semibold", "Progress: " }
                                        span { "{info.progress_percentage():.1}%" }
                                    }
                                }
                                if let Some(resolution) = info.resolution_string() {
                                    div {
                                        span { class: "font-semibold", "Resolution: " }
                                        span { "{resolution}" }
                                    }
                                }
                            }
                        }
                    }

                    // Control buttons (for local videos - YouTube handles its own controls)
                    if let Some(VideoSource::Local { .. }) = &props.video_source {
                        div {
                            class: "flex justify-center space-x-2",
                            button {
                                class: "btn btn-outline btn-sm",
                                onclick: play_pause_handler,
                                if playback_state() == PlaybackState::Playing {
                                    "⏸️ Pause"
                                } else {
                                    "▶️ Play"
                                }
                            }
                            button {
                                class: "btn btn-outline btn-sm",
                                onclick: stop_handler,
                                "⏹️ Stop"
                            }
                        }
                    }
                }

                // Modal footer
                div {
                    class: "flex justify-end p-6 border-t border-base-300",
                    button {
                        class: "btn btn-outline",
                        onclick: close_handler,
                        "Close"
                    }
                }
            }
        }
    }
}

/// Hook for managing video player modal state
pub fn use_video_player_modal() -> VideoPlayerModalManager {
    let modal_manager = use_modal_manager(false);
    let current_video = use_signal(|| None::<VideoSource>);
    let current_title = use_signal(|| None::<String>);

    let open_video = use_callback({
        let mut current_video = current_video;
        let mut current_title = current_title;
        let modal_manager = modal_manager.clone();
        move |(source, title): (VideoSource, Option<String>)| {
            log::info!(
                "VideoPlayerModalManager: Opening video: {:?} with title: {:?}",
                source,
                title
            );
            current_video.set(Some(source));
            current_title.set(title);
            modal_manager.open.call(());
        }
    });

    let close = use_callback({
        let modal_manager = modal_manager.clone();
        move |_| {
            log::info!("VideoPlayerModalManager: Closing video player modal");
            modal_manager.close.call(());
            // Keep video source until next video to prevent flicker
        }
    });

    VideoPlayerModalManager {
        is_open: modal_manager.is_open,
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
