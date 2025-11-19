use crate::ui::hooks::use_video_player_manager;
use crate::video_player::{PlaybackState, VideoSource};
use dioxus::prelude::*;

#[component]
pub fn VideoPlayerTest() -> Element {
    let mut manager = use_video_player_manager();

    // Test basic functionality
    let test_load_video = move |_| {
        let source = VideoSource::YouTube {
            video_id: "dQw4w9WgXcQ".to_string(),
            playlist_id: None,
            title: "Test Video".to_string(),
        };
        manager.load_video(source);
        println!("Video loaded successfully");
    };

    let test_play = move |_| {
        manager.play();
        println!("Play command sent");
    };

    let test_pause = move |_| {
        manager.pause();
        println!("Pause command sent");
    };

    rsx! {
        div {
            class: "p-4 space-y-4",
            h2 { class: "text-xl font-bold", "Video Player Test" }

            div { class: "space-x-2",
                button {
                    class: "btn btn-primary",
                    onclick: test_load_video,
                    "Load Test Video"
                }
                button {
                    class: "btn btn-secondary",
                    onclick: test_play,
                    "Play"
                }
                button {
                    class: "btn btn-secondary",
                    onclick: test_pause,
                    "Pause"
                }
            }

            div { class: "mt-4 p-4 bg-base-200 rounded",
                p { "Current State: {manager.playback_state():?}" }
                p { "Has Video: {manager.has_video()}" }
                p { "Position: {manager.position():.2}s" }
                p { "Duration: {manager.duration():.2}s" }
                p { "Volume: {manager.volume():.2}" }
                p { "Fullscreen: {manager.is_fullscreen()}" }
            }
        }
    }
}
