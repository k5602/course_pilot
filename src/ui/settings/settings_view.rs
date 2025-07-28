use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaBookOpen, FaDownload, FaGear, FaKey};

use super::{APIKeysSettings, CourseDefaultSettings, GeneralSettings, ImportSettings};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    ApiKeys,
    CourseDefaults,
    ImportSettings,
}

impl SettingsTab {
    pub fn all() -> Vec<Self> {
        vec![
            Self::General,
            Self::ApiKeys,
            Self::CourseDefaults,
            Self::ImportSettings,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::ApiKeys => "API Keys",
            Self::CourseDefaults => "Course Defaults",
            Self::ImportSettings => "Import Settings",
        }
    }

    pub fn icon(&self) -> fn() -> Element {
        match self {
            Self::General => || rsx! { Icon { icon: FaGear, class: "w-4 h-4" } },
            Self::ApiKeys => || rsx! { Icon { icon: FaKey, class: "w-4 h-4" } },
            Self::CourseDefaults => || rsx! { Icon { icon: FaBookOpen, class: "w-4 h-4" } },
            Self::ImportSettings => || rsx! { Icon { icon: FaDownload, class: "w-4 h-4" } },
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::General => "Theme, notifications, and general preferences",
            Self::ApiKeys => "Manage YouTube and Gemini API keys",
            Self::CourseDefaults => "Default settings for new courses",
            Self::ImportSettings => "Configure import behavior and preferences",
        }
    }
}

#[component]
pub fn SettingsView() -> Element {
    let mut active_tab = use_signal(|| SettingsTab::General);
    let settings_manager = crate::ui::hooks::use_settings_manager();
    let mut settings_resource = crate::ui::hooks::use_settings_resource();

    // Handle settings loading state
    let settings_content = match settings_resource.read().as_ref() {
        Some(Ok(settings)) => {
            let tabs = SettingsTab::all();

            rsx! {
                div { class: "space-y-6",
                    // Header section
                    div { class: "text-center pb-6 border-b border-base-300",
                        h1 { class: "text-3xl font-bold text-base-content mb-2", "Settings" }
                        p { class: "text-base-content/70",
                            "Configure Course Pilot to match your learning preferences and workflow"
                        }
                    }

                    // Tab navigation
                    div { class: "w-full",
                        div { class: "tabs tabs-boxed tabs-lg w-full justify-center bg-base-200 p-1",
                            for tab in tabs {
                                {
                                    let is_selected = active_tab() == tab;
                                    let icon_fn = tab.icon();

                                    rsx! {
                                        button {
                                            key: "{tab.label()}",
                                            class: format!(
                                                "tab tab-lg flex-1 gap-2 {}",
                                                if is_selected { "tab-active" } else { "" }
                                            ),
                                            onclick: move |_| active_tab.set(tab),

                                            {icon_fn()}
                                            span { "{tab.label()}" }
                                        }
                                    }
                                }
                            }
                        }

                        // Tab descriptions
                        div { class: "text-center mt-2 mb-4",
                            p { class: "text-sm text-base-content/60",
                                "{active_tab().description()}"
                            }
                        }
                    }

                    // Tab content
                    div { class: "min-h-[400px] bg-base-50 rounded-lg p-6",
                        match active_tab() {
                            SettingsTab::General => rsx! {
                                GeneralSettings {
                                    settings: settings.clone(),
                                    settings_manager: settings_manager.clone(),
                                    on_settings_updated: move |_| {
                                        settings_resource.restart();
                                    }
                                }
                            },
                            SettingsTab::ApiKeys => rsx! {
                                APIKeysSettings {
                                    settings: settings.clone(),
                                    settings_manager: settings_manager.clone(),
                                    on_settings_updated: move |_| {
                                        settings_resource.restart();
                                    }
                                }
                            },
                            SettingsTab::CourseDefaults => rsx! {
                                CourseDefaultSettings {
                                    settings: settings.clone(),
                                    settings_manager: settings_manager.clone(),
                                    on_settings_updated: move |_| {
                                        settings_resource.restart();
                                    }
                                }
                            },
                            SettingsTab::ImportSettings => rsx! {
                                ImportSettings {
                                    settings: settings.clone(),
                                    settings_manager: settings_manager.clone(),
                                    on_settings_updated: move |_| {
                                        settings_resource.restart();
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
        Some(Err(e)) => {
            rsx! {
                div { class: "min-h-screen flex items-center justify-center",
                    div { class: "text-center p-8",
                        div { class: "text-6xl mb-4", "⚠️" }
                        h1 { class: "text-2xl font-bold text-error mb-2", "Settings Error" }
                        p { class: "text-base-content/70 mb-4",
                            "Failed to load settings: {e}"
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| settings_resource.restart(),
                            "Retry"
                        }
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "min-h-screen flex items-center justify-center",
                    div { class: "text-center p-8",
                        div { class: "loading loading-spinner loading-lg text-primary mb-4" }
                        p { class: "text-base-content/70", "Loading settings..." }
                    }
                }
            }
        }
    };

    rsx! {
        div { class: "container mx-auto max-w-4xl p-6",
            {settings_content}
        }
    }
}
