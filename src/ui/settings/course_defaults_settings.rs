use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaBookOpen, FaBrain, FaCalendarWeek, FaClock, FaGear,
};

use crate::storage::AppSettings;
use crate::types::{AdvancedSchedulerSettings, DifficultyLevel, DistributionStrategy};
use crate::ui::components::toast_helpers;
use crate::ui::hooks::SettingsManager;

#[derive(Props, Clone)]
pub struct CourseDefaultSettingsProps {
    pub settings: AppSettings,
    pub settings_manager: SettingsManager,
    pub on_settings_updated: EventHandler<()>,
}

impl PartialEq for CourseDefaultSettingsProps {
    fn eq(&self, other: &Self) -> bool {
        self.settings == other.settings
    }
}

#[component]
pub fn CourseDefaultSettings(props: CourseDefaultSettingsProps) -> Element {
    let mut local_settings = use_signal(|| props.settings.clone());
    let is_saving = use_signal(|| false);
    let mut show_advanced = use_signal(|| false);

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
                        toast_helpers::success("Course defaults saved successfully!");
                        on_settings_updated.call(());
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to save course defaults: {e}"));
                    }
                }

                is_saving.set(false);
            });
        }
    };

    // Handle plan settings changes
    let mut handle_sessions_per_week_change = {
        let mut local_settings = local_settings;
        move |value: String| {
            if let Ok(sessions) = value.parse::<u8>() {
                if (1..=14).contains(&sessions) {
                    let mut settings = local_settings();
                    settings.default_plan_settings.sessions_per_week = sessions;
                    local_settings.set(settings);
                }
            }
        }
    };

    let mut handle_session_length_change = {
        let mut local_settings = local_settings;
        move |value: String| {
            if let Ok(length) = value.parse::<u32>() {
                if (15..=300).contains(&length) {
                    let mut settings = local_settings();
                    settings.default_plan_settings.session_length_minutes = length;
                    local_settings.set(settings);
                }
            }
        }
    };

    let mut handle_include_weekends_change = {
        let mut local_settings = local_settings;
        move |enabled: bool| {
            let mut settings = local_settings();
            settings.default_plan_settings.include_weekends = enabled;
            local_settings.set(settings);
        }
    };

    let mut handle_auto_create_plan_change = {
        let mut local_settings = local_settings;
        move |enabled: bool| {
            let mut settings = local_settings();
            settings.auto_create_plan = enabled;
            local_settings.set(settings);
        }
    };

    // Handle advanced scheduler settings
    let mut handle_strategy_change = {
        let mut local_settings = local_settings;
        move |strategy_str: String| {
            let strategy = match strategy_str.as_str() {
                "ModuleBased" => DistributionStrategy::ModuleBased,
                "TimeBased" => DistributionStrategy::TimeBased,
                "Hybrid" => DistributionStrategy::Hybrid,
                "DifficultyBased" => DistributionStrategy::DifficultyBased,
                "SpacedRepetition" => DistributionStrategy::SpacedRepetition,
                "Adaptive" => DistributionStrategy::Adaptive,
                _ => DistributionStrategy::Hybrid,
            };

            let mut settings = local_settings();
            let mut advanced = settings
                .default_plan_settings
                .advanced_settings
                .unwrap_or_else(AdvancedSchedulerSettings::default);
            advanced.strategy = strategy;
            settings.default_plan_settings.advanced_settings = Some(advanced);
            local_settings.set(settings);
        }
    };

    let mut handle_difficulty_adaptation_change = {
        let mut local_settings = local_settings;
        move |enabled: bool| {
            let mut settings = local_settings();
            let mut advanced = settings
                .default_plan_settings
                .advanced_settings
                .unwrap_or_else(AdvancedSchedulerSettings::default);
            advanced.difficulty_adaptation = enabled;
            settings.default_plan_settings.advanced_settings = Some(advanced);
            local_settings.set(settings);
        }
    };

    let mut handle_user_experience_change = {
        let mut local_settings = local_settings;
        move |level_str: String| {
            let level = match level_str.as_str() {
                "Beginner" => DifficultyLevel::Beginner,
                "Intermediate" => DifficultyLevel::Intermediate,
                "Advanced" => DifficultyLevel::Advanced,
                "Expert" => DifficultyLevel::Expert,
                _ => DifficultyLevel::Intermediate,
            };

            let mut settings = local_settings();
            let mut advanced = settings
                .default_plan_settings
                .advanced_settings
                .unwrap_or_else(AdvancedSchedulerSettings::default);
            advanced.user_experience_level = level;
            settings.default_plan_settings.advanced_settings = Some(advanced);
            local_settings.set(settings);
        }
    };

    // Handle clustering preferences
    let mut handle_clustering_algorithm_change = {
        let mut local_settings = local_settings;
        move |algorithm_str: String| {
            let algorithm = match algorithm_str.as_str() {
                "TfIdf" => crate::types::ClusteringAlgorithm::TfIdf,
                "KMeans" => crate::types::ClusteringAlgorithm::KMeans,
                "Hierarchical" => crate::types::ClusteringAlgorithm::Hierarchical,
                "Lda" => crate::types::ClusteringAlgorithm::Lda,
                "Hybrid" => crate::types::ClusteringAlgorithm::Hybrid,
                _ => crate::types::ClusteringAlgorithm::Hybrid,
            };

            let mut settings = local_settings();
            settings.clustering_preferences.preferred_algorithm = algorithm;
            local_settings.set(settings);
        }
    };

    let mut handle_similarity_threshold_change = {
        let mut local_settings = local_settings;
        move |value: String| {
            if let Ok(threshold) = value.parse::<f32>() {
                if (0.1..=1.0).contains(&threshold) {
                    let mut settings = local_settings();
                    settings.clustering_preferences.similarity_threshold = threshold;
                    local_settings.set(settings);
                }
            }
        }
    };

    let current_settings = local_settings();
    let default_advanced = AdvancedSchedulerSettings::default();
    let advanced_settings = current_settings
        .default_plan_settings
        .advanced_settings
        .as_ref()
        .unwrap_or(&default_advanced);

    rsx! {
        div { class: "space-y-8",
            // Header
            div { class: "text-center pb-6 border-b border-base-300",
                div { class: "flex items-center justify-center gap-3 mb-3",
                    Icon { icon: FaBookOpen, class: "w-6 h-6 text-primary" }
                    h2 { class: "text-2xl font-bold", "Course Defaults" }
                }
                p { class: "text-base-content/70 max-w-2xl mx-auto",
                    "Configure default settings that will be applied to all newly imported courses. "
                    "You can always customize these settings for individual courses later."
                }
            }

            // Basic Plan Settings
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaClock, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Default Plan Settings" }
                    }

                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                        // Sessions per week
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Sessions per Week" }
                            }
                            input {
                                r#type: "number",
                                class: "input input-bordered",
                                min: "1",
                                max: "14",
                                value: "{current_settings.default_plan_settings.sessions_per_week}",
                                oninput: move |evt| handle_sessions_per_week_change(evt.value())
                            }
                            label { class: "label",
                                span { class: "label-text-alt text-base-content/60",
                                    "How many study sessions per week (1-14)"
                                }
                            }
                        }

                        // Session length
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Session Length (minutes)" }
                            }
                            input {
                                r#type: "number",
                                class: "input input-bordered",
                                min: "15",
                                max: "300",
                                step: "15",
                                value: "{current_settings.default_plan_settings.session_length_minutes}",
                                oninput: move |evt| handle_session_length_change(evt.value())
                            }
                            label { class: "label",
                                span { class: "label-text-alt text-base-content/60",
                                    "Duration of each study session (15-300 minutes)"
                                }
                            }
                        }
                    }

                    div { class: "space-y-4 mt-6",
                        // Include weekends
                        div { class: "form-control",
                            label { class: "label cursor-pointer justify-start gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: current_settings.default_plan_settings.include_weekends,
                                    onchange: move |evt| handle_include_weekends_change(evt.checked())
                                }
                                div {
                                    span { class: "label-text font-medium", "Include Weekends" }
                                    div { class: "text-sm text-base-content/60",
                                        "Schedule study sessions on weekends"
                                    }
                                }
                            }
                        }

                        // Auto-create plan
                        div { class: "form-control",
                            label { class: "label cursor-pointer justify-start gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: current_settings.auto_create_plan,
                                    onchange: move |evt| handle_auto_create_plan_change(evt.checked())
                                }
                                div {
                                    span { class: "label-text font-medium", "Auto-create Study Plans" }
                                    div { class: "text-sm text-base-content/60",
                                        "Automatically generate study plans for new courses"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Advanced Scheduler Settings
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center justify-between mb-4",
                        div { class: "flex items-center gap-3",
                            Icon { icon: FaGear, class: "w-5 h-5 text-primary" }
                            h3 { class: "text-lg font-semibold", "Advanced Scheduler Settings" }
                        }
                        button {
                            class: "btn btn-sm btn-outline",
                            onclick: move |_| show_advanced.set(!show_advanced()),
                            if show_advanced() { "Hide Advanced" } else { "Show Advanced" }
                        }
                    }

                    if show_advanced() {
                        div { class: "space-y-6",
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                                // Distribution strategy
                                div { class: "form-control",
                                    label { class: "label",
                                        span { class: "label-text font-medium", "Distribution Strategy" }
                                    }
                                    select {
                                        class: "select select-bordered",
                                        value: format!("{:?}", advanced_settings.strategy),
                                        onchange: move |evt| handle_strategy_change(evt.value()),

                                        for strategy in DistributionStrategy::all() {
                                            option {
                                                key: "{strategy:?}",
                                                value: "{strategy:?}",
                                                "{strategy.display_name()}"
                                            }
                                        }
                                    }
                                    label { class: "label",
                                        span { class: "label-text-alt text-base-content/60",
                                            "{advanced_settings.strategy.description()}"
                                        }
                                    }
                                }

                                // User experience level
                                div { class: "form-control",
                                    label { class: "label",
                                        span { class: "label-text font-medium", "Experience Level" }
                                    }
                                    select {
                                        class: "select select-bordered",
                                        value: format!("{:?}", advanced_settings.user_experience_level),
                                        onchange: move |evt| handle_user_experience_change(evt.value()),

                                        for level in DifficultyLevel::all() {
                                            option {
                                                key: "{level:?}",
                                                value: "{level:?}",
                                                "{level.display_name()}"
                                            }
                                        }
                                    }
                                    label { class: "label",
                                        span { class: "label-text-alt text-base-content/60",
                                            "Your general experience level for learning new topics"
                                        }
                                    }
                                }
                            }

                            // Advanced options
                            div { class: "form-control",
                                label { class: "label cursor-pointer justify-start gap-3",
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox checkbox-primary",
                                        checked: advanced_settings.difficulty_adaptation,
                                        onchange: move |evt| handle_difficulty_adaptation_change(evt.checked())
                                    }
                                    div {
                                        span { class: "label-text font-medium", "Difficulty Adaptation" }
                                        div { class: "text-sm text-base-content/60",
                                            "Adjust pacing based on content difficulty analysis"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Clustering Preferences
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaBrain, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "AI Clustering Preferences" }
                    }

                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                        // Clustering algorithm
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Preferred Algorithm" }
                            }
                            select {
                                class: "select select-bordered",
                                value: format!("{:?}", current_settings.clustering_preferences.preferred_algorithm),
                                onchange: move |evt| handle_clustering_algorithm_change(evt.value()),

                                option { value: "TfIdf", "TF-IDF (Content-based)" }
                                option { value: "KMeans", "K-Means (Similarity-based)" }
                                option { value: "Hierarchical", "Hierarchical (Tree-based)" }
                                option { value: "Lda", "LDA (Topic-based)" }
                                option { value: "Hybrid", "Hybrid (Adaptive)" }
                            }
                            label { class: "label",
                                span { class: "label-text-alt text-base-content/60",
                                    "Algorithm used for course structure analysis"
                                }
                            }
                        }

                        // Similarity threshold
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Similarity Threshold" }
                            }
                            input {
                                r#type: "number",
                                class: "input input-bordered",
                                min: "0.1",
                                max: "1.0",
                                step: "0.1",
                                value: "{current_settings.clustering_preferences.similarity_threshold}",
                                oninput: move |evt| handle_similarity_threshold_change(evt.value())
                            }
                            label { class: "label",
                                span { class: "label-text-alt text-base-content/60",
                                    "Minimum similarity for grouping content (0.1-1.0)"
                                }
                            }
                        }
                    }
                }
            }

            // Import Preferences
            div { class: "card bg-base-100 shadow-sm",
                div { class: "card-body",
                    div { class: "flex items-center gap-3 mb-4",
                        Icon { icon: FaCalendarWeek, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Import Preferences" }
                    }

                    div { class: "space-y-4",
                        // Course prefix
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text font-medium", "Default Course Prefix" }
                            }
                            input {
                                r#type: "text",
                                class: "input input-bordered",
                                placeholder: "e.g., 'Course: ' or leave empty",
                                value: current_settings.import_preferences.default_course_prefix.as_deref().unwrap_or(""),
                                oninput: move |evt| {
                                    let mut settings = local_settings();
                                    let value = evt.value();
                                    let prefix = value.trim();
                                    settings.import_preferences.default_course_prefix =
                                        if prefix.is_empty() { None } else { Some(prefix.to_string()) };
                                    local_settings.set(settings);
                                }
                            }
                            label { class: "label",
                                span { class: "label-text-alt text-base-content/60",
                                    "Text to prepend to imported course names"
                                }
                            }
                        }

                        // Skip short videos
                        div { class: "form-control",
                            label { class: "label cursor-pointer justify-start gap-3",
                                input {
                                    r#type: "checkbox",
                                    class: "checkbox checkbox-primary",
                                    checked: current_settings.import_preferences.skip_short_videos,
                                    onchange: move |evt| {
                                        let mut settings = local_settings();
                                        settings.import_preferences.skip_short_videos = evt.checked();
                                        local_settings.set(settings);
                                    }
                                }
                                div {
                                    span { class: "label-text font-medium", "Skip Short Videos" }
                                    div { class: "text-sm text-base-content/60",
                                        "Exclude videos shorter than the minimum duration"
                                    }
                                }
                            }
                        }

                        // Minimum video duration
                        if current_settings.import_preferences.skip_short_videos {
                            div { class: "form-control ml-6",
                                label { class: "label",
                                    span { class: "label-text font-medium", "Minimum Duration (seconds)" }
                                }
                                input {
                                    r#type: "number",
                                    class: "input input-bordered w-32",
                                    min: "10",
                                    max: "3600",
                                    value: "{current_settings.import_preferences.min_video_duration_seconds}",
                                    oninput: move |evt| {
                                        if let Ok(duration) = evt.value().parse::<u64>() {
                                            let mut settings = local_settings();
                                            settings.import_preferences.min_video_duration_seconds = duration;
                                            local_settings.set(settings);
                                        }
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
                    onclick: save_settings,

                    if is_saving() {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                        "Saving Defaults..."
                    } else {
                        Icon { icon: FaBookOpen, class: "w-4 h-4 mr-2" }
                        "Save Course Defaults"
                    }
                }
            }
        }
    }
}
