//! YouTube video player embed

use dioxus::prelude::*;

use crate::ui::state::AppState;

/// YouTube IFrame player with fallback for webkit2gtk.
/// webkit2gtk has issues with referrer headers causing Error 153.
/// We provide both an embed attempt and a fallback "Watch on YouTube" button.
#[component]
pub fn YouTubePlayer(video_id: String) -> Element {
    let mut show_fallback = use_signal(|| false);
    let state = use_context::<AppState>();
    let video_id_clone = video_id.clone();

    // Direct YouTube watch URL for fallback
    let youtube_url = format!("https://www.youtube.com/watch?v={}", video_id);

    // Embed URL with all recommended parameters
    let embed_url = match state.youtube_embed_relay_url.read().as_ref() {
        Some(base_url) => format!("{}/embed?v={}", base_url, video_id_clone),
        None => format!(
            "https://www.youtube-nocookie.com/embed/{}?rel=0&modestbranding=1&playsinline=1",
            video_id_clone
        ),
    };

    rsx! {
        div {
            class: "aspect-video w-full bg-black rounded-lg overflow-hidden relative",

            // Try the iframe embed first
            if !show_fallback() {
                iframe {
                    class: "w-full h-full",
                    src: "{embed_url}",
                    allow: "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share",
                    allowfullscreen: true,
                    referrerpolicy: "strict-origin-when-cross-origin",
                }
            }

            if !show_fallback() {
                button {
                    class: "absolute bottom-4 right-4 btn btn-ghost btn-sm",
                    onclick: move |_| show_fallback.set(true),
                    "Having trouble?"
                }
            }

            // Fallback overlay with "Watch on YouTube" button
            if show_fallback() {
                div {
                    class: "absolute inset-0 flex flex-col items-center justify-center bg-base-300/90",

                    // YouTube logo/icon placeholder
                    div {
                        class: "text-6xl mb-4",
                        "▶"
                    }

                    p {
                        class: "text-lg mb-4 text-center px-4",
                        "Video playback may not work in this app due to browser restrictions."
                    }

                    a {
                        href: "{youtube_url}",
                        target: "_blank",
                        class: "btn btn-primary btn-lg gap-2",
                        "🔗 Watch on YouTube"
                    }
                }
            }
        }
    }
}
