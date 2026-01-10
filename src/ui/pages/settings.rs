//! Settings page - API keys and preferences

use dioxus::prelude::*;

/// Settings for API keys and app preferences.
#[component]
pub fn Settings() -> Element {
    let mut youtube_key = use_signal(String::new);
    let mut gemini_key = use_signal(String::new);
    let mut ml_enabled = use_signal(|| false);

    rsx! {
        div {
            class: "p-6 max-w-2xl",

            h1 { class: "text-2xl font-bold mb-6", "Settings" }

            // API Keys section
            section {
                class: "mb-8",
                h2 { class: "text-lg font-semibold mb-4", "API Keys" }

                div {
                    class: "space-y-4",

                    // YouTube API Key
                    div {
                        label { class: "label", "YouTube API Key" }
                        input {
                            class: "input input-bordered w-full",
                            r#type: "password",
                            placeholder: "Enter your YouTube Data API v3 key",
                            value: "{youtube_key}",
                            oninput: move |e| youtube_key.set(e.value()),
                        }
                        p {
                            class: "text-sm text-base-content/60 mt-1",
                            "Required for playlist import. Get one from Google Cloud Console."
                        }
                    }

                    // Gemini API Key
                    div {
                        label { class: "label", "Gemini API Key (Optional)" }
                        input {
                            class: "input input-bordered w-full",
                            r#type: "password",
                            placeholder: "Enter your Gemini API key",
                            value: "{gemini_key}",
                            oninput: move |e| gemini_key.set(e.value()),
                        }
                        p {
                            class: "text-sm text-base-content/60 mt-1",
                            "Enables AI Companion and quiz generation. Get one from AI Studio."
                        }
                    }
                }
            }

            // Preferences section
            section {
                class: "mb-8",
                h2 { class: "text-lg font-semibold mb-4", "Preferences" }

                div {
                    class: "flex items-center justify-between p-4 bg-base-200 rounded-lg",
                    div {
                        p { class: "font-medium", "ML Boundary Detection" }
                        p {
                            class: "text-sm text-base-content/60",
                            "Use AI to automatically detect module boundaries in playlists"
                        }
                    }
                    input {
                        r#type: "checkbox",
                        class: "toggle toggle-primary",
                        checked: *ml_enabled.read(),
                        onchange: move |e| ml_enabled.set(e.checked()),
                    }
                }
            }

            // Save button
            button {
                class: "btn btn-primary",
                "Save Settings"
            }
        }
    }
}
