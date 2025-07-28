use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaDownload, FaFilter, FaGear, FaRobot, FaVideo};

use crate::storage::{AppSettings, CourseNamingPattern, VideoQualityPreference};
use crate::ui::components::toast_helpers;
use crate::ui::hooks::SettingsManager;

#[derive(Props, Clone)]
pub struct ImportSettingsProps {
    pub settings: AppSettings,
    pub settings_manager: SettingsManager,
    pub on_settings_updated: EventHandler<()>,
}

impl PartialEq for ImportSettingsProps {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings
        // Note: SettingsManager and EventHandler don't implement PartialEq,
        // so we only compare the settings data
    }
}

#[component]
pub fn ImportSettings(props: ImportSettingsProps) -> Element {
    let mut import_prefs = use_signal(|| props.settings.import_preferences.clone());
    let is_saving = use_signal(|| false);

    // Save import preferences
    let save_preferences = {
        let settings_manager = props.settings_manager.clone();
        let on_settings_updated = props.on_settings_updated;
        let import_prefs = import_prefs();
        let mut is_saving = is_saving;

        move |_| {
            let settings_manager = settings_manager.clone();
            let on_settings_updated = on_settings_updated;
            let import_prefs = import_prefs.clone();

            spawn(async move {
                is_saving.set(true);

                match settings_manager.set_import_preferences(import_prefs).await {
                    Ok(_) => {
                        toast_helpers::success("Import preferences saved successfully!");
                        on_settings_updated.call(());
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to save import preferences: {e}"));
                    }
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
                    Icon { icon: FaDownload, class: "w-6 h-6 text-primary" }
                    h2 { class: "text-2xl font-bold", "Import Preferences" }
                }
                p { class: "text-base-content/70 max-w-2xl mx-auto",
                    "Configure default settings for importing courses from YouTube and local folders. "
                    "These preferences will be applied to all new imports."
                }
            }

            // Course Naming Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaVideo, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Course Naming" }
                    }

                    div { class: "space-y-4",
                        // Course naming pattern
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Naming Pattern" }
                            }
                            select {
                                class: "select select-bordered w-full",
                                value: match import_prefs().course_naming_pattern {
                                    CourseNamingPattern::PlaylistTitle => "playlist_title",
                                    CourseNamingPattern::PrefixPlusTitle => "prefix_plus_title",
                                    CourseNamingPattern::DatePlusTitle => "date_plus_title",
                                    CourseNamingPattern::CustomPattern(_) => "custom",
                                },
                                onchange: move |evt| {
                                    let mut prefs = import_prefs();
                                    prefs.course_naming_pattern = match evt.value().as_str() {
                                        "playlist_title" => CourseNamingPattern::PlaylistTitle,
                                        "prefix_plus_title" => CourseNamingPattern::PrefixPlusTitle,
                                        "date_plus_title" => CourseNamingPattern::DatePlusTitle,
                                        "custom" => CourseNamingPattern::CustomPattern("{prefix} - {title}".to_string()),
                                        _ => CourseNamingPattern::PlaylistTitle,
                                    };
                                    import_prefs.set(prefs);
                                },
                                option { value: "playlist_title", "Use Playlist Title" }
                                option { value: "prefix_plus_title", "Prefix + Title" }
                                option { value: "date_plus_title", "Date + Title" }
                                option { value: "custom", "Custom Pattern" }
                            }
                        }

                        // Default course prefix
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Default Course Prefix" }
                            }
                            input {
                                r#type: "text",
                                class: "input input-bordered",
                                placeholder: "e.g., Course, Tutorial, Learning",
                                value: import_prefs().default_course_prefix.unwrap_or_default(),
                                oninput: move |evt| {
                                    let mut prefs = import_prefs();
                                    prefs.default_course_prefix = if evt.value().trim().is_empty() {
                                        None
                                    } else {
                                        Some(evt.value().trim().to_string())
                                    };
                                    import_prefs.set(prefs);
                                }
                            }
                            label { class: "label",
                                span { class: "label-text-alt", "Optional prefix to add to course names" }
                            }
                        }

                        // Use playlist title toggle
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Use original playlist title" }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-primary",
                                    checked: import_prefs().use_playlist_title,
                                    onchange: move |evt| {
                                        let mut prefs = import_prefs();
                                        prefs.use_playlist_title = evt.value().parse().unwrap_or(false);
                                        import_prefs.set(prefs);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Video Filtering Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaFilter, class: "w-5 h-5 text-secondary" }
                        h3 { class: "text-lg font-semibold", "Video Filtering" }
                    }

                    div { class: "space-y-4",
                        // Duration filtering
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            // Min duration
                            div { class: "form-control",
                                label { class: "label cursor-pointer",
                                    span { class: "label-text", "Skip short videos" }
                                    input {
                                        r#type: "checkbox",
                                        class: "toggle toggle-secondary",
                                        checked: import_prefs().skip_short_videos,
                                        onchange: move |evt| {
                                            let mut prefs = import_prefs();
                                            prefs.skip_short_videos = evt.value().parse().unwrap_or(false);
                                            import_prefs.set(prefs);
                                        }
                                    }
                                }
                                if import_prefs().skip_short_videos {
                                    div { class: "mt-2",
                                        label { class: "label",
                                            span { class: "label-text-alt", "Minimum duration (seconds)" }
                                        }
                                        input {
                                            r#type: "number",
                                            class: "input input-bordered input-sm",
                                            min: "1",
                                            max: "3600",
                                            value: import_prefs().min_video_duration_seconds.to_string(),
                                            oninput: move |evt| {
                                                let mut prefs = import_prefs();
                                                prefs.min_video_duration_seconds = evt.value().parse().unwrap_or(30);
                                                import_prefs.set(prefs);
                                            }
                                        }
                                    }
                                }
                            }

                            // Max duration
                            div { class: "form-control",
                                label { class: "label cursor-pointer",
                                    span { class: "label-text", "Skip long videos" }
                                    input {
                                        r#type: "checkbox",
                                        class: "toggle toggle-secondary",
                                        checked: import_prefs().skip_long_videos,
                                        onchange: move |evt| {
                                            let mut prefs = import_prefs();
                                            prefs.skip_long_videos = evt.value().parse().unwrap_or(false);
                                            import_prefs.set(prefs);
                                        }
                                    }
                                }
                                if import_prefs().skip_long_videos {
                                    div { class: "mt-2",
                                        label { class: "label",
                                            span { class: "label-text-alt", "Maximum duration (seconds)" }
                                        }
                                        input {
                                            r#type: "number",
                                            class: "input input-bordered input-sm",
                                            min: "60",
                                            max: "14400",
                                            value: import_prefs().max_video_duration_seconds.to_string(),
                                            oninput: move |evt| {
                                                let mut prefs = import_prefs();
                                                prefs.max_video_duration_seconds = evt.value().parse().unwrap_or(3600);
                                                import_prefs.set(prefs);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Quality preference
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Video Quality Preference" }
                            }
                            select {
                                class: "select select-bordered w-full",
                                value: match import_prefs().quality_preference {
                                    VideoQualityPreference::Any => "any",
                                    VideoQualityPreference::PreferHD => "prefer_hd",
                                    VideoQualityPreference::RequireHD => "require_hd",
                                    VideoQualityPreference::PreferSD => "prefer_sd",
                                },
                                onchange: move |evt| {
                                    let mut prefs = import_prefs();
                                    prefs.quality_preference = match evt.value().as_str() {
                                        "any" => VideoQualityPreference::Any,
                                        "prefer_hd" => VideoQualityPreference::PreferHD,
                                        "require_hd" => VideoQualityPreference::RequireHD,
                                        "prefer_sd" => VideoQualityPreference::PreferSD,
                                        _ => VideoQualityPreference::Any,
                                    };
                                    import_prefs.set(prefs);
                                },
                                option { value: "any", "Any Quality" }
                                option { value: "prefer_hd", "Prefer HD (720p+)" }
                                option { value: "require_hd", "Require HD (720p+)" }
                                option { value: "prefer_sd", "Prefer SD (480p)" }
                            }
                            label { class: "label",
                                span { class: "label-text-alt", "Quality preference for video imports" }
                            }
                        }
                    }
                }
            }

            // Auto-Processing Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaRobot, class: "w-5 h-5 text-accent" }
                        h3 { class: "text-lg font-semibold", "Auto-Processing" }
                    }

                    div { class: "space-y-4",
                        // Auto-create plan
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Automatically create study plan after import" }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-accent",
                                    checked: import_prefs().auto_create_plan,
                                    onchange: move |evt| {
                                        let mut prefs = import_prefs();
                                        prefs.auto_create_plan = evt.value().parse().unwrap_or(false);
                                        import_prefs.set(prefs);
                                    }
                                }
                            }
                        }

                        // Auto-structure course
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Automatically structure course into modules" }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-accent",
                                    checked: import_prefs().auto_structure_course,
                                    onchange: move |evt| {
                                        let mut prefs = import_prefs();
                                        prefs.auto_structure_course = evt.value().parse().unwrap_or(false);
                                        import_prefs.set(prefs);
                                    }
                                }
                            }
                        }

                        // Enable AI clustering
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Enable AI-powered content clustering" }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-accent",
                                    checked: import_prefs().enable_ai_clustering,
                                    onchange: move |evt| {
                                        let mut prefs = import_prefs();
                                        prefs.enable_ai_clustering = evt.value().parse().unwrap_or(false);
                                        import_prefs.set(prefs);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Advanced Options Section
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaGear, class: "w-5 h-5 text-info" }
                        h3 { class: "text-lg font-semibold", "Advanced Options" }
                    }

                    div { class: "space-y-4",
                        // Preserve playlist order
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Preserve original playlist order" }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-info",
                                    checked: import_prefs().preserve_playlist_order,
                                    onchange: move |evt| {
                                        let mut prefs = import_prefs();
                                        prefs.preserve_playlist_order = evt.value().parse().unwrap_or(false);
                                        import_prefs.set(prefs);
                                    }
                                }
                            }
                        }

                        // Extract timestamps
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Extract video timestamps and chapters" }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-info",
                                    checked: import_prefs().extract_timestamps,
                                    onchange: move |evt| {
                                        let mut prefs = import_prefs();
                                        prefs.extract_timestamps = evt.value().parse().unwrap_or(false);
                                        import_prefs.set(prefs);
                                    }
                                }
                            }
                        }

                        // Download thumbnails
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Download video thumbnails" }
                                input {
                                    r#type: "checkbox",
                                    class: "toggle toggle-info",
                                    checked: import_prefs().download_thumbnails,
                                    onchange: move |evt| {
                                        let mut prefs = import_prefs();
                                        prefs.download_thumbnails = evt.value().parse().unwrap_or(false);
                                        import_prefs.set(prefs);
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
                    class: "btn btn-primary btn-lg",
                    disabled: is_saving(),
                    onclick: save_preferences,

                    if is_saving() {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                        "Saving Preferences..."
                    } else {
                        Icon { icon: FaDownload, class: "w-4 h-4 mr-2" }
                        "Save Import Preferences"
                    }
                }
            }
        }
    }
}
