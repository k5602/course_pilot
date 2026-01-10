//! YouTube video player embed

use dioxus::prelude::*;

/// YouTube IFrame player using privacy-enhanced mode.
#[component]
pub fn YouTubePlayer(video_id: String) -> Element {
    let embed_url =
        format!("https://www.youtube-nocookie.com/embed/{}?rel=0&modestbranding=1", video_id);

    rsx! {
        div {
            class: "aspect-video w-full bg-black rounded-lg overflow-hidden",
            iframe {
                class: "w-full h-full",
                src: "{embed_url}",
                allow: "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture",
                allowfullscreen: true,
            }
        }
    }
}
