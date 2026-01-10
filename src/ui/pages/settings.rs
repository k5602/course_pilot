//! Settings page - API keys and preferences with save functionality

use dioxus::prelude::*;

use crate::domain::ports::SecretStore;
use crate::ui::state::AppState;

/// Settings for API keys and app preferences.
#[component]
pub fn Settings() -> Element {
    let state = use_context::<AppState>();

    let mut youtube_key = use_signal(String::new);
    let mut gemini_key = use_signal(String::new);
    let mut ml_enabled = use_signal(|| false);
    let mut save_status = use_signal(|| None::<(bool, String)>);

    // Clone backend for closures
    let backend_load = state.backend.clone();
    let backend_save = state.backend.clone();

    // Load current values on mount
    use_effect(move || {
        if let Some(ref ctx) = backend_load {
            // Show masked indicator if keys exist
            if ctx.has_youtube() {
                youtube_key.set("••••••••••••••••".to_string());
            }
            if ctx.has_llm() {
                gemini_key.set("••••••••••••••••".to_string());
            }
        }
    });

    let handle_save = move |_| {
        let yt_key = youtube_key.read().clone();
        let gem_key = gemini_key.read().clone();

        // Only save if not masked placeholder
        if let Some(ref ctx) = backend_save {
            let mut success = true;
            let mut errors = Vec::new();

            // Save YouTube key
            if !yt_key.is_empty() && !yt_key.starts_with("••") {
                if let Err(e) = ctx.keystore.store("youtube_api_key", &yt_key) {
                    success = false;
                    errors.push(format!("YouTube key: {}", e));
                }
            }

            // Save Gemini key
            if !gem_key.is_empty() && !gem_key.starts_with("••") {
                if let Err(e) = ctx.keystore.store("gemini_api_key", &gem_key) {
                    success = false;
                    errors.push(format!("Gemini key: {}", e));
                }
            }

            if success {
                save_status
                    .set(Some((true, "Settings saved. Restart app to apply changes.".to_string())));
            } else {
                save_status.set(Some((false, errors.join(", "))));
            }
        } else {
            save_status.set(Some((false, "Backend not available".to_string())));
        }
    };

    rsx! {
        div {
            class: "p-6 max-w-2xl",

            h1 { class: "text-2xl font-bold mb-6", "Settings" }

            // Save status alert
            if let Some((is_success, ref msg)) = *save_status.read() {
                div {
                    class: if is_success { "alert alert-success mb-6" } else { "alert alert-error mb-6" },
                    "{msg}"
                }
            }

            // API Keys section
            section {
                class: "mb-8",
                h2 { class: "text-lg font-semibold mb-4", "API Keys" }

                div {
                    class: "space-y-4",

                    // YouTube API Key
                    div {
                        label { class: "label", "YouTube API Key" }
                        div {
                            class: "flex gap-2",
                            input {
                                class: "input input-bordered flex-1",
                                r#type: "password",
                                placeholder: "Enter your YouTube Data API v3 key",
                                value: "{youtube_key}",
                                oninput: move |e| youtube_key.set(e.value()),
                                onfocus: move |_| {
                                    if youtube_key.read().starts_with("••") {
                                        youtube_key.set(String::new());
                                    }
                                },
                            }
                            if state.has_youtube() {
                                span { class: "badge badge-success self-center", "Active" }
                            }
                        }
                        p {
                            class: "text-sm text-base-content/60 mt-1",
                            "Required for playlist import. "
                            a {
                                href: "https://console.cloud.google.com/apis/credentials",
                                class: "link link-primary",
                                target: "_blank",
                                "Get from Google Cloud Console →"
                            }
                        }
                    }

                    // Gemini API Key
                    div {
                        label { class: "label", "Gemini API Key (Optional)" }
                        div {
                            class: "flex gap-2",
                            input {
                                class: "input input-bordered flex-1",
                                r#type: "password",
                                placeholder: "Enter your Gemini API key",
                                value: "{gemini_key}",
                                oninput: move |e| gemini_key.set(e.value()),
                                onfocus: move |_| {
                                    if gemini_key.read().starts_with("••") {
                                        gemini_key.set(String::new());
                                    }
                                },
                            }
                            if state.has_gemini() {
                                span { class: "badge badge-success self-center", "Active" }
                            }
                        }
                        p {
                            class: "text-sm text-base-content/60 mt-1",
                            "Enables AI Companion and quiz generation. "
                            a {
                                href: "https://aistudio.google.com/apikey",
                                class: "link link-primary",
                                target: "_blank",
                                "Get from AI Studio →"
                            }
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
                onclick: handle_save,
                "Save Settings"
            }
        }
    }
}
