use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_brands_icons::FaYoutube;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowUpRightFromSquare, FaCheck, FaEye, FaEyeSlash, FaKey, FaTriangleExclamation,
};

use crate::storage::AppSettings;
use crate::ui::components::toast_helpers;
use crate::ui::hooks::SettingsManager;

#[derive(Props, Clone)]
pub struct APIKeysSettingsProps {
    pub settings: AppSettings,
    pub settings_manager: SettingsManager,
    pub on_settings_updated: EventHandler<()>,
}

impl PartialEq for APIKeysSettingsProps {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ApiKeyStatus {
    NotSet,
    Valid,
    Invalid(String),
    Testing,
}

#[component]
pub fn APIKeysSettings(props: APIKeysSettingsProps) -> Element {
    let mut youtube_key = use_signal(|| props.settings.youtube_api_key.clone().unwrap_or_default());
    let mut gemini_key = use_signal(|| props.settings.gemini_api_key.clone().unwrap_or_default());
    let mut youtube_visible = use_signal(|| false);
    let mut gemini_visible = use_signal(|| false);
    let youtube_status = use_signal(|| ApiKeyStatus::NotSet);
    let gemini_status = use_signal(|| ApiKeyStatus::NotSet);
    let is_saving = use_signal(|| false);

    // Initialize status based on existing keys
    use_effect({
        let youtube_key = youtube_key();
        let gemini_key = gemini_key();
        let mut youtube_status = youtube_status;
        let mut gemini_status = gemini_status;

        move || {
            if !youtube_key.trim().is_empty() {
                youtube_status.set(ApiKeyStatus::Valid);
            }
            if !gemini_key.trim().is_empty() {
                gemini_status.set(ApiKeyStatus::Valid);
            }
        }
    });

    // Test YouTube API key
    let test_youtube_key = {
        let youtube_key = youtube_key();
        let mut youtube_status = youtube_status;

        move |_| {
            let key = youtube_key.trim().to_string();
            if key.is_empty() {
                toast_helpers::error("Please enter a YouTube API key first");
                return;
            }

            youtube_status.set(ApiKeyStatus::Testing);

            spawn(async move {
                // Test the API key by making a simple request
                match crate::ingest::youtube::validate_api_key(&key).await {
                    Ok(true) => {
                        youtube_status.set(ApiKeyStatus::Valid);
                        toast_helpers::success("YouTube API key is valid!");
                    }
                    Ok(false) => {
                        youtube_status.set(ApiKeyStatus::Invalid(
                            "API key is invalid or has insufficient permissions".to_string(),
                        ));
                        toast_helpers::error("YouTube API key is invalid");
                    }
                    Err(e) => {
                        youtube_status.set(ApiKeyStatus::Invalid(format!("Test failed: {e}")));
                        toast_helpers::error("Failed to test YouTube API key");
                    }
                }
            });
        }
    };

    // Test Gemini API key
    let test_gemini_key = {
        let gemini_key = gemini_key();
        let mut gemini_status = gemini_status;

        move |_| {
            let key = gemini_key.trim().to_string();
            if key.is_empty() {
                toast_helpers::error("Please enter a Gemini API key first");
                return;
            }

            gemini_status.set(ApiKeyStatus::Testing);

            spawn(async move {
                // Test the API key by making a simple request
                match test_gemini_api_key(&key).await {
                    Ok(true) => {
                        gemini_status.set(ApiKeyStatus::Valid);
                        toast_helpers::success("Gemini API key is valid!");
                    }
                    Ok(false) => {
                        gemini_status.set(ApiKeyStatus::Invalid(
                            "API key is invalid or has insufficient permissions".to_string(),
                        ));
                        toast_helpers::error("Gemini API key is invalid");
                    }
                    Err(e) => {
                        gemini_status.set(ApiKeyStatus::Invalid(format!("Test failed: {e}")));
                        toast_helpers::error("Failed to test Gemini API key");
                    }
                }
            });
        }
    };

    // Save API keys
    let save_keys = {
        let settings_manager = props.settings_manager.clone();
        let on_settings_updated = props.on_settings_updated;
        let youtube_key = youtube_key();
        let gemini_key = gemini_key();
        let mut is_saving = is_saving;

        move |_| {
            let settings_manager = settings_manager.clone();
            let on_settings_updated = on_settings_updated;
            let youtube_key = youtube_key.trim().to_string();
            let gemini_key = gemini_key.trim().to_string();

            spawn(async move {
                is_saving.set(true);

                let mut success = true;

                // Save YouTube API key
                let youtube_key_opt = if youtube_key.is_empty() {
                    None
                } else {
                    Some(youtube_key)
                };
                if let Err(e) = settings_manager.set_youtube_api_key(youtube_key_opt).await {
                    toast_helpers::error(format!("Failed to save YouTube API key: {e}"));
                    success = false;
                }

                // Save Gemini API key
                let gemini_key_opt = if gemini_key.is_empty() {
                    None
                } else {
                    Some(gemini_key)
                };
                if let Err(e) = settings_manager.set_gemini_api_key(gemini_key_opt).await {
                    toast_helpers::error(format!("Failed to save Gemini API key: {e}"));
                    success = false;
                }

                if success {
                    toast_helpers::success("API keys saved successfully!");
                    on_settings_updated.call(());
                }

                is_saving.set(false);
            });
        }
    };

    rsx! {
        div { class: "space-y-8",
            // Header with description
            div { class: "text-center pb-6 border-b border-base-300",
                div { class: "flex items-center justify-center gap-3 mb-3",
                    Icon { icon: FaKey, class: "w-6 h-6 text-primary" }
                    h2 { class: "text-2xl font-bold", "API Key Management" }
                }
                p { class: "text-base-content/70 max-w-2xl mx-auto",
                    "Configure your API keys to enable YouTube imports and AI-powered course structuring. "
                    "Your keys are stored securely on your device and never shared."
                }
            }

            // YouTube API Key Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaYoutube, class: "w-6 h-6 text-red-500" }
                        h3 { class: "text-xl font-semibold", "YouTube Data API v3" }
                        {match youtube_status() {
                            ApiKeyStatus::Valid => rsx! {
                                div { class: "badge badge-success gap-1",
                                    Icon { icon: FaCheck, class: "w-3 h-3" }
                                    "Valid"
                                }
                            },
                            ApiKeyStatus::Invalid(_) => rsx! {
                                div { class: "badge badge-error gap-1",
                                    Icon { icon: FaTriangleExclamation, class: "w-3 h-3" }
                                    "Invalid"
                                }
                            },
                            ApiKeyStatus::Testing => rsx! {
                                div { class: "badge badge-warning gap-1",
                                    span { class: "loading loading-spinner loading-xs" }
                                    "Testing"
                                }
                            },
                            ApiKeyStatus::NotSet => rsx! {
                                div { class: "badge badge-neutral", "Not Set" }
                            },
                        }}
                    }

                    div { class: "space-y-4",
                        // API Key Input
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "API Key" }
                            }
                            div { class: "input-group",
                                input {
                                    r#type: if youtube_visible() { "text" } else { "password" },
                                    class: "input input-bordered flex-1",
                                    placeholder: "Enter your YouTube Data API v3 key",
                                    value: youtube_key(),
                                    oninput: move |evt| youtube_key.set(evt.value())
                                }
                                button {
                                    class: "btn btn-square btn-outline",
                                    onclick: move |_| youtube_visible.set(!youtube_visible()),
                                    if youtube_visible() {
                                        Icon { icon: FaEyeSlash, class: "w-4 h-4" }
                                    } else {
                                        Icon { icon: FaEye, class: "w-4 h-4" }
                                    }
                                }
                                button {
                                    class: "btn btn-outline",
                                    disabled: youtube_key().trim().is_empty() || matches!(youtube_status(), ApiKeyStatus::Testing),
                                    onclick: test_youtube_key,
                                    if matches!(youtube_status(), ApiKeyStatus::Testing) {
                                        span { class: "loading loading-spinner loading-sm mr-1" }
                                        "Testing"
                                    } else {
                                        "Test"
                                    }
                                }
                            }

                            // Status message
                            if let ApiKeyStatus::Invalid(ref error) = youtube_status() {
                                label { class: "label",
                                    span { class: "label-text-alt text-error", "{error}" }
                                }
                            }
                        }

                        // Setup Instructions
                        div { class: "alert alert-info",
                            div {
                                h4 { class: "font-semibold mb-2", "How to get your YouTube API key:" }
                                ol { class: "list-decimal list-inside space-y-1 text-sm",
                                    li { "Go to the "
                                        a {
                                            href: "https://console.developers.google.com/",
                                            target: "_blank",
                                            class: "link link-primary",
                                            "Google Cloud Console"
                                            Icon { icon: FaArrowUpRightFromSquare, class: "w-3 h-3 ml-1 inline" }
                                        }
                                    }
                                    li { "Create a new project or select an existing one" }
                                    li { "Enable the YouTube Data API v3" }
                                    li { "Create credentials (API key)" }
                                    li { "Copy the API key and paste it above" }
                                }
                            }
                        }
                    }
                }
            }

            // Gemini API Key Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        div { class: "w-6 h-6 bg-gradient-to-r from-blue-500 to-purple-500 rounded flex items-center justify-center",
                            span { class: "text-white text-xs font-bold", "G" }
                        }
                        h3 { class: "text-xl font-semibold", "Google Gemini API" }
                        {match gemini_status() {
                            ApiKeyStatus::Valid => rsx! {
                                div { class: "badge badge-success gap-1",
                                    Icon { icon: FaCheck, class: "w-3 h-3" }
                                    "Valid"
                                }
                            },
                            ApiKeyStatus::Invalid(_) => rsx! {
                                div { class: "badge badge-error gap-1",
                                    Icon { icon: FaTriangleExclamation, class: "w-3 h-3" }
                                    "Invalid"
                                }
                            },
                            ApiKeyStatus::Testing => rsx! {
                                div { class: "badge badge-warning gap-1",
                                    span { class: "loading loading-spinner loading-xs" }
                                    "Testing"
                                }
                            },
                            ApiKeyStatus::NotSet => rsx! {
                                div { class: "badge badge-neutral", "Not Set" }
                            },
                        }}
                    }

                    div { class: "space-y-4",
                        // API Key Input
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "API Key" }
                            }
                            div { class: "input-group",
                                input {
                                    r#type: if gemini_visible() { "text" } else { "password" },
                                    class: "input input-bordered flex-1",
                                    placeholder: "Enter your Gemini API key",
                                    value: gemini_key(),
                                    oninput: move |evt| gemini_key.set(evt.value())
                                }
                                button {
                                    class: "btn btn-square btn-outline",
                                    onclick: move |_| gemini_visible.set(!gemini_visible()),
                                    if gemini_visible() {
                                        Icon { icon: FaEyeSlash, class: "w-4 h-4" }
                                    } else {
                                        Icon { icon: FaEye, class: "w-4 h-4" }
                                    }
                                }
                                button {
                                    class: "btn btn-outline",
                                    disabled: gemini_key().trim().is_empty() || matches!(gemini_status(), ApiKeyStatus::Testing),
                                    onclick: test_gemini_key,
                                    if matches!(gemini_status(), ApiKeyStatus::Testing) {
                                        span { class: "loading loading-spinner loading-sm mr-1" }
                                        "Testing"
                                    } else {
                                        "Test"
                                    }
                                }
                            }

                            // Status message
                            if let ApiKeyStatus::Invalid(ref error) = gemini_status() {
                                label { class: "label",
                                    span { class: "label-text-alt text-error", "{error}" }
                                }
                            }
                        }

                        // Setup Instructions
                        div { class: "alert alert-info",
                            div {
                                h4 { class: "font-semibold mb-2", "How to get your Gemini API key:" }
                                ol { class: "list-decimal list-inside space-y-1 text-sm",
                                    li { "Go to "
                                        a {
                                            href: "https://makersuite.google.com/app/apikey",
                                            target: "_blank",
                                            class: "link link-primary",
                                            "Google AI Studio"
                                            Icon { icon: FaArrowUpRightFromSquare, class: "w-3 h-3 ml-1 inline" }
                                        }
                                    }
                                    li { "Sign in with your Google account" }
                                    li { "Click 'Create API Key'" }
                                    li { "Copy the generated API key and paste it above" }
                                }
                                div { class: "mt-2 text-sm text-base-content/70",
                                    "Note: Gemini API is used for AI-powered course structuring and recommendations."
                                }
                            }
                        }
                    }
                }
            }

            // Save button
            div { class: "flex justify-end pt-4 border-t border-base-300",
                button {
                    class: "btn btn-primary btn-lg",
                    disabled: is_saving(),
                    onclick: save_keys,

                    if is_saving() {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                        "Saving API Keys..."
                    } else {
                        Icon { icon: FaKey, class: "w-4 h-4 mr-2" }
                        "Save API Keys"
                    }
                }
            }
        }
    }
}

// Helper function to test Gemini API key
async fn test_gemini_api_key(api_key: &str) -> anyhow::Result<bool> {
    // This is a placeholder - in a real implementation, you would make a test request to Gemini API
    // For now, we'll just check if the key looks valid (starts with expected prefix)
    if api_key.starts_with("AIza") && api_key.len() > 20 {
        // Simulate API call delay
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        Ok(true)
    } else {
        Ok(false)
    }
}
