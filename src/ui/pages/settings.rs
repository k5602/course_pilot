//! Settings page - API keys and preferences with save functionality

use dioxus::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::UpdatePreferencesInput;
use crate::domain::ports::SecretStore;
use crate::ui::custom::PresenceHealth;
use crate::ui::state::AppState;

/// Settings for API keys and app preferences.
#[component]
pub fn Settings() -> Element {
    let mut state = use_context::<AppState>();

    {
        let mut state = state.clone();
        use_effect(move || {
            state.current_video_id.set(None);
        });
    }

    let mut active_tab = use_signal(|| "integrations".to_string());

    let mut gemini_key = use_signal(String::new);
    let mut ml_boundary_enabled = use_signal(|| false);
    let mut cognitive_limit = use_signal(|| 45u32);
    let mut right_panel_visible = use_signal(|| true);

    let mut save_status = use_signal(|| None::<(bool, String)>);

    // Clone backend for closures
    let backend_load = state.backend.clone();
    let backend_save = state.backend.clone();
    let backend_prefs = state.backend.clone();

    // Load current values on mount
    use_effect(move || {
        if let Some(ref ctx) = backend_load {
            // Show masked indicator if key exists
            if ctx.has_llm() {
                gemini_key.set("••••••••••••••••".to_string());
            }

            let use_case = ServiceFactory::preferences(ctx);
            match use_case.load() {
                Ok(prefs) => {
                    ml_boundary_enabled.set(prefs.ml_boundary_enabled());
                    cognitive_limit.set(prefs.cognitive_limit_minutes());
                    right_panel_visible.set(prefs.right_panel_visible());
                    state.right_panel_width.set(prefs.right_panel_width() as f64);
                    state.onboarding_completed.set(prefs.onboarding_completed());
                },
                Err(e) => {
                    save_status.set(Some((false, format!("Failed to load preferences: {}", e))));
                },
            }
        }
    });

    let handle_save_integrations = move |_| {
        let gem_key = gemini_key.read().clone();

        // Only save if not masked placeholder
        if let Some(ref ctx) = backend_save {
            let mut success = true;
            let mut errors = Vec::new();

            // Save Gemini key
            if !gem_key.is_empty() && !gem_key.starts_with("••") {
                if let Err(e) = ctx.keystore.store("gemini_api_key", &gem_key) {
                    success = false;
                    errors.push(format!("Gemini key: {}", e));
                }
            }

            if success {
                save_status.set(Some((true, "Integrations saved.".to_string())));
            } else {
                save_status.set(Some((false, errors.join(", "))));
            }
        } else {
            save_status.set(Some((false, "Backend not available".to_string())));
        }
    };

    let handle_save_preferences = move |_| {
        if let Some(ref ctx) = backend_prefs {
            let use_case = ServiceFactory::preferences(ctx);
            let input = UpdatePreferencesInput {
                ml_boundary_enabled: *ml_boundary_enabled.read(),
                cognitive_limit_minutes: *cognitive_limit.read(),
                right_panel_visible: *right_panel_visible.read(),
                right_panel_width: state.right_panel_width.read().round() as u32,
                onboarding_completed: *state.onboarding_completed.read(),
            };

            match use_case.update(input) {
                Ok(_) => {
                    save_status.set(Some((true, "Preferences saved.".to_string())));
                },
                Err(e) => {
                    save_status.set(Some((false, format!("Failed to save preferences: {}", e))));
                },
            }
        } else {
            save_status.set(Some((false, "Backend not available".to_string())));
        }
    };

    rsx! {
        div { class: "p-6 max-w-3xl",

            h1 { class: "text-2xl font-bold mb-6", "Settings" }

            div { class: "tabs tabs-boxed mb-6",
                button {
                    class: if *active_tab.read() == "integrations" { "tab tab-active" } else { "tab" },
                    onclick: move |_| active_tab.set("integrations".to_string()),
                    "Integrations"
                }
                button {
                    class: if *active_tab.read() == "preferences" { "tab tab-active" } else { "tab" },
                    onclick: move |_| active_tab.set("preferences".to_string()),
                    "Preferences"
                }
                button {
                    class: if *active_tab.read() == "about" { "tab tab-active" } else { "tab" },
                    onclick: move |_| active_tab.set("about".to_string()),
                    "About"
                }
            }

            // Save status alert
            if let Some((is_success, ref msg)) = *save_status.read() {
                div { class: if is_success { "alert alert-success mb-6" } else { "alert alert-error mb-6" },
                    "{msg}"
                }
            }

            if *active_tab.read() == "integrations" {
                section { class: "mb-8",
                    h2 { class: "text-lg font-semibold mb-4", "API Keys" }

                    div { class: "space-y-4",

                        // Gemini API Key
                        div {
                            label { class: "label", "Gemini API Key" }
                            div { class: "flex gap-2",
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
                                    span { class: "badge badge-success self-center",
                                        "Active"
                                    }
                                }
                            }
                            p { class: "text-sm text-base-content/60 mt-1",
                                "Required for AI Companion, quiz generation, and video summaries. "
                                a {
                                    href: "https://aistudio.google.com/apikey",
                                    class: "link link-primary",
                                    target: "_blank",
                                    "Get from AI Studio →"
                                }
                            }
                        }
                    }

                    div { class: "divider" }

                    h2 { class: "text-lg font-semibold mb-4", "Discord Rich Presence" }

                    div { class: "space-y-4",
                        div { class: "bg-base-200 rounded-lg p-4 flex items-center justify-between",
                            div {
                                h3 { class: "font-semibold", "Connection Status" }
                                p { class: "text-sm text-base-content/60",
                                    "Show your current course and video on your Discord profile."
                                }
                            }
                            PresenceHealth {}
                        }
                    }

                    button {
                        class: "btn btn-primary mt-6",
                        onclick: handle_save_integrations,
                        "Save Integrations"
                    }
                }
            } else if *active_tab.read() == "preferences" {
                section { class: "mb-8",
                    h2 { class: "text-lg font-semibold mb-4", "Preferences" }

                    div { class: "space-y-6",

                        // Cognitive limit
                        div {
                            label { class: "label", "Daily study time: {cognitive_limit} minutes" }
                            input {
                                r#type: "range",
                                class: "range range-primary w-full",
                                min: "15",
                                max: "120",
                                step: "5",
                                value: "{cognitive_limit}",
                                oninput: move |e| {
                                    if let Ok(val) = e.value().parse::<u32>() {
                                        cognitive_limit.set(val);
                                    }
                                },
                            }
                            p { class: "text-sm text-base-content/60 mt-1",
                                "Used to plan study sessions across modules."
                            }
                        }
                    }

                    button {
                        class: "btn btn-primary mt-6",
                        onclick: handle_save_preferences,
                        "Save Preferences"
                    }
                }
            } else {
                section { class: "mb-8 space-y-4",
                    h2 { class: "text-lg font-semibold", "About" }
                    p { class: "text-sm text-base-content/70",
                        "Course Pilot helps you transform YouTube playlists into structured study plans."
                    }
                    p { class: "text-sm text-base-content/70", "Window title: Course Pilot" }
                    p { class: "text-sm text-base-content/70",
                        "Version: {env!(\"CARGO_PKG_VERSION\")}"
                    }
                    p { class: "text-sm text-base-content/70", "Author: Made with love by Khaled" }
                }
            }
        }
    }
}
