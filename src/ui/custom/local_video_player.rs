//! Local video player component.
//!
//! Uses the local media relay server to stream video files in the desktop WebView.

use dioxus::prelude::*;

use crate::ui::state::AppState;

/// Local video player for file-backed videos.
#[component]
pub fn LocalVideoPlayer(path: String) -> Element {
    let state = use_context::<AppState>();
    let relay_url = state.local_media_relay_url.read().clone();

    let src = relay_url.as_ref().map(|base| {
        let encoded = url_encode(&path);
        format!("{}/media?path={}", base, encoded)
    });

    rsx! {
        div { class: "aspect-video w-full bg-black rounded-lg overflow-hidden relative",
            if let Some(src) = src {
                video {
                    class: "w-full h-full",
                    controls: true,
                    preload: "metadata",
                    src: "{src}",
                }
            } else {
                div { class: "absolute inset-0 flex flex-col items-center justify-center bg-base-300/90",
                    div { class: "text-6xl mb-4", "🎬" }
                    p { class: "text-lg mb-2 text-center px-4", "Local media relay is not available." }
                    p { class: "text-sm text-base-content/70 text-center px-4",
                        "Restart the app or check logs for relay server errors."
                    }
                }
            }
        }
    }
}

fn url_encode(input: &str) -> String {
    input
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                (b as char).to_string()
            },
            _ => format!("%{:02X}", b),
        })
        .collect()
}
