use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaBell, FaChartLine, FaClock, FaPalette};

use crate::storage::AppSettings;
use crate::ui::components::toast_helpers;
use crate::ui::hooks::SettingsManager;

#[derive(Props, Clone)]
pub struct GeneralSettingsProps {
    pub settings: AppSettings,
    pub settings_manager: SettingsManager,
    pub on_settings_updated: EventHandler<()>,
}

impl PartialEq for GeneralSettingsProps {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings

    }
}

#[component]
pub fn GeneralSettings(props: GeneralSettingsProps) -> Element {
    let is_saving = use_signal(|| false);
    let local_settings = use_signal(|| props.settings.clone());

    // Available themes (DaisyUI themes)
    let themes = vec![
        ("light", "Light"),
        ("dark", "Dark"),
        ("corporate", "Corporate"),
        ("synthwave", "Synthwave"),
        ("retro", "Retro"),
        ("cyberpunk", "Cyberpunk"),
        ("valentine", "Valentine"),
        ("halloween", "Halloween"),
        ("garden", "Garden"),
        ("forest", "Forest"),
        ("aqua", "Aqua"),
        ("lofi", "Lo-Fi"),
        ("pastel", "Pastel"),
        ("fantasy", "Fantasy"),
        ("wireframe", "Wireframe"),
        ("black", "Black"),
        ("luxury", "Luxury"),
        ("dracula", "Dracula"),
        ("cmyk", "CMYK"),
        ("autumn", "Autumn"),
        ("business", "Business"),
        ("acid", "Acid"),
        ("lemonade", "Lemonade"),
        ("night", "Night"),
        ("coffee", "Coffee"),
        ("winter", "Winter"),
    ];

    // Save settings function
    let save_settings = {
        let settings_manager = props.settings_manager.clone();
        let on_settings_updated = props.on_settings_updated;
        let mut is_saving = is_saving;
        let local_settings = local_settings;

        move |_| {
            let settings_manager = settings_manager.clone();
            let on_settings_updated = on_settings_updated;
            let settings = local_settings();

            spawn(async move {
                is_saving.set(true);

                match settings_manager.save_settings(settings).await {
                    Ok(_) => {
                        toast_helpers::success("Settings saved successfully!");
                        on_settings_updated.call(());
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to save settings: {e}"));
                    }
                }

                is_saving.set(false);
            });
        }
    };

    // Handle theme change
    let mut handle_theme_change = {
        let mut local_settings = local_settings;
        move |theme: String| {
            let mut settings = local_settings();
            settings.theme = Some(theme);
            local_settings.set(settings);
        }
    };

    // Handle boolean setting changes
    let mut handle_auto_structure_change = {
        let mut local_settings = local_settings;
        move |enabled: bool| {
            let mut settings = local_settings();
            settings.auto_structure = enabled;
            local_settings.set(settings);
        }
    };

    let mut handle_notifications_change = {
        let mut local_settings = local_settings;
        move |enabled: bool| {
            let mut settings = local_settings();
            settings.notifications_enabled = enabled;
            local_settings.set(settings);
        }
    };

    let mut handle_analytics_change = {
        let mut local_settings = local_settings;
        move |enabled: bool| {
            let mut settings = local_settings();
            settings.analytics_enabled = enabled;
            local_settings.set(settings);
        }
    };

    let mut handle_track_study_time_change = {
        let mut local_settings = local_settings;
        move |enabled: bool| {
            let mut settings = local_settings();
            settings.track_study_time = enabled;
            local_settings.set(settings);
        }
    };

    let current_settings = local_settings();

    rsx! {
        div { class: "space-y-8",
            // Theme Settings
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaPalette, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Theme & Appearance" }
                    }

                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Theme" }
                        }
                        select {
                            class: "select select-bordered w-full max-w-xs",
                            value: current_settings.theme.as_deref().unwrap_or("corporate"),
                            onchange: move |evt| {
                                handle_theme_change(evt.value());
                            },

                            {themes.iter().map(|(value, label)| {
                                rsx! {
                                    option {
                                        key: "{value}",
                                        value: "{value}",
                                        "{label}"
                                    }
                                }
                            })}
                        }
                        label { class: "label",
                            span { class: "label-text-alt text-base-content/60",
                                "Choose your preferred color theme"
                            }
                        }
                    }
                }
            }

            // Notifications Settings
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaBell, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Notifications" }
                    }

                    div { class: "space-y-4",
                        div { class: "form-control",
                            label { class: "label cursor-pointer justify-start gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: current_settings.notifications_enabled,
                                    onchange: move |evt| {
                                        handle_notifications_change(evt.checked());
                                    }
                                }
                                div {
                                    span { class: "label-text font-medium", "Enable Notifications" }
                                    div { class: "text-sm text-base-content/60",
                                        "Receive notifications for import completion, session reminders, and updates"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Course Processing Settings
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaChartLine, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Course Processing" }
                    }

                    div { class: "space-y-4",
                        div { class: "form-control",
                            label { class: "label cursor-pointer justify-start gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: current_settings.auto_structure,
                                    onchange: move |evt| {
                                        handle_auto_structure_change(evt.checked());
                                    }
                                }
                                div {
                                    span { class: "label-text font-medium", "Auto-structure Courses" }
                                    div { class: "text-sm text-base-content/60",
                                        "Automatically analyze and structure imported courses using AI clustering"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Analytics Settings
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaClock, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Analytics & Tracking" }
                    }

                    div { class: "space-y-4",
                        div { class: "form-control",
                            label { class: "label cursor-pointer justify-start gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: current_settings.analytics_enabled,
                                    onchange: move |evt| {
                                        handle_analytics_change(evt.checked());
                                    }
                                }
                                div {
                                    span { class: "label-text font-medium", "Enable Learning Analytics" }
                                    div { class: "text-sm text-base-content/60",
                                        "Track learning progress and generate insights for the dashboard"
                                    }
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label cursor-pointer justify-start gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: current_settings.track_study_time,
                                    onchange: move |evt| {
                                        handle_track_study_time_change(evt.checked());
                                    }
                                }
                                div {
                                    span { class: "label-text font-medium", "Track Study Time" }
                                    div { class: "text-sm text-base-content/60",
                                        "Monitor time spent on sessions for productivity insights"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Save button
            div { class: "flex justify-end pt-4 border-t border-base-300",
                button {
                    class: "btn btn-primary",
                    disabled: is_saving(),
                    onclick: save_settings,

                    if is_saving() {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                        "Saving..."
                    } else {
                        "Save Settings"
                    }
                }
            }
        }
    }
}
