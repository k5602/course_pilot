use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCalendarDays, FaCheck, FaClock, FaGear, FaRotateRight, FaSpinner, FaTriangleExclamation,
};
use dioxus_motion::prelude::*;

use crate::types::{
    AdvancedSchedulerSettings, DifficultyLevel, DistributionStrategy, Plan, PlanSettings,
    RegenerationStatus,
};
use crate::ui::toast_helpers;
use crate::ui::use_backend;

#[derive(Props, PartialEq, Clone)]
pub struct SessionControlPanelProps {
    pub plan: Plan,
    pub on_settings_change: EventHandler<PlanSettings>,
    pub on_plan_regenerated: EventHandler<Plan>,
}

/// Session control panel for plan customization and advanced scheduling
#[component]
pub fn SessionControlPanel(props: SessionControlPanelProps) -> Element {
    let mut is_expanded = use_signal(|| false);
    let mut sessions_per_week = use_signal(|| props.plan.settings.sessions_per_week);
    let mut session_length = use_signal(|| props.plan.settings.session_length_minutes);
    let mut include_weekends = use_signal(|| props.plan.settings.include_weekends);

    // Advanced scheduler settings
    let mut advanced_settings = use_signal(|| {
        props
            .plan
            .settings
            .advanced_settings
            .clone()
            .unwrap_or_default()
    });
    let mut selected_strategy = use_signal(|| advanced_settings().strategy);
    let mut user_experience_level = use_signal(|| advanced_settings().user_experience_level);
    let mut difficulty_adaptation = use_signal(|| advanced_settings().difficulty_adaptation);
    let mut spaced_repetition_enabled =
        use_signal(|| advanced_settings().spaced_repetition_enabled);
    let mut cognitive_load_balancing = use_signal(|| advanced_settings().cognitive_load_balancing);
    // Auto-apply debounce version counter (increments on any control change)
    let mut change_version = use_signal(|| 0usize);

    // Form validation and regeneration state
    let mut form_errors = use_signal(Vec::<String>::new);
    let mut regeneration_status = use_signal(|| RegenerationStatus::Idle);
    let mut show_advanced = use_signal(|| false);

    // Backend hook
    let backend = use_backend();

    // Animation for panel expansion
    let mut panel_height = use_motion(0.0f32);
    let mut panel_opacity = use_motion(0.0f32);

    use_effect(move || {
        if is_expanded() {
            panel_height.animate_to(
                if show_advanced() { 600.0 } else { 300.0 },
                AnimationConfig::new(AnimationMode::Spring(Spring::default())),
            );
            panel_opacity.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Tween(Tween::default())),
            );
        } else {
            panel_height.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(Spring::default())),
            );
            panel_opacity.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Tween(Tween::default())),
            );
        }
    });

    let panel_style = use_memo(move || {
        format!(
            "max-height: {}px; opacity: {}; overflow: hidden;",
            panel_height.get_value(),
            panel_opacity.get_value()
        )
    });

    // Auto-apply Session Controls with debounce when any control changes
    use_effect({
        // Capture dependencies/signals
        let backend = backend.clone();
        let plan_id = props.plan.id;

        let selected_strategy = selected_strategy;
        let user_experience_level = user_experience_level;
        let difficulty_adaptation = difficulty_adaptation;
        let spaced_repetition_enabled = spaced_repetition_enabled;
        let cognitive_load_balancing = cognitive_load_balancing;

        let sessions_per_week = sessions_per_week;
        let session_length = session_length;
        let include_weekends = include_weekends;

        let mut regeneration_status = regeneration_status;
        let on_plan_regenerated = props.on_plan_regenerated;

        let change_version = change_version;

        move || {
            // Read signals for dependency tracking
            let ver = change_version();
            if ver == 0 {
                // No changes yet; skip
                return;
            }

            // Don't start a new regeneration while one is already in progress
            if matches!(regeneration_status(), RegenerationStatus::InProgress { .. }) {
                return;
            }

            spawn({
                let backend = backend.clone();
                let change_version = change_version;

                let selected_strategy = selected_strategy;
                let user_experience_level = user_experience_level;
                let difficulty_adaptation = difficulty_adaptation;
                let spaced_repetition_enabled = spaced_repetition_enabled;
                let cognitive_load_balancing = cognitive_load_balancing;

                let sessions_per_week = sessions_per_week;
                let session_length = session_length;
                let include_weekends = include_weekends;

                let mut regeneration_status = regeneration_status;
                let on_plan_regenerated = on_plan_regenerated;

                async move {
                    // Debounce window
                    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

                    // If another change happened during debounce, skip this run
                    if change_version() != ver {
                        return;
                    }

                    // Build new settings from current controls
                    let new_advanced_settings = AdvancedSchedulerSettings {
                        strategy: selected_strategy(),
                        difficulty_adaptation: difficulty_adaptation(),
                        spaced_repetition_enabled: spaced_repetition_enabled(),
                        cognitive_load_balancing: cognitive_load_balancing(),
                        user_experience_level: user_experience_level(),
                        custom_intervals: None,
                        max_session_duration_minutes: None,
                        min_break_between_sessions_hours: None,
                        prioritize_difficult_content: false,
                        adaptive_pacing: true,
                    };

                    let new_settings = PlanSettings {
                        start_date: props.plan.settings.start_date,
                        sessions_per_week: sessions_per_week(),
                        session_length_minutes: session_length(),
                        include_weekends: include_weekends(),
                        advanced_settings: Some(new_advanced_settings),
                    };

                    regeneration_status.set(RegenerationStatus::InProgress {
                        progress: 0.0,
                        message: "Applying session controls...".to_string(),
                    });

                    match backend.regenerate_plan(plan_id, new_settings).await {
                        Ok(new_plan) => {
                            regeneration_status.set(RegenerationStatus::Completed);
                            on_plan_regenerated.call(new_plan);
                            // briefly show completion state
                            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                            regeneration_status.set(RegenerationStatus::Idle);
                        }
                        Err(e) => {
                            regeneration_status.set(RegenerationStatus::Failed {
                                error: e.to_string(),
                            });
                        }
                    }
                }
            });
        }
    });

    let toggle_panel = move |_| {
        is_expanded.set(!is_expanded());
    };

    let toggle_advanced = move |_| {
        show_advanced.set(!show_advanced());
    };

    // Validate form inputs
    let validate_form = move || -> Vec<String> {
        let mut errors = Vec::new();

        if sessions_per_week() == 0 {
            errors.push("Sessions per week must be greater than 0".to_string());
        }
        if sessions_per_week() > 14 {
            errors.push("Sessions per week cannot exceed 14".to_string());
        }
        if session_length() < 15 {
            errors.push("Session length must be at least 15 minutes".to_string());
        }
        if session_length() > 180 {
            errors.push("Session length cannot exceed 180 minutes".to_string());
        }

        // Advanced settings validation
        if selected_strategy() == DistributionStrategy::SpacedRepetition
            && !spaced_repetition_enabled()
        {
            errors.push(
                "Spaced repetition must be enabled for SpacedRepetition strategy".to_string(),
            );
        }

        errors
    };

    let apply_settings = move |_| {
        // Validate form
        let errors = validate_form();
        form_errors.set(errors.clone());

        if !errors.is_empty() {
            spawn(async move {
                toast_helpers::error("Please fix validation errors before applying settings");
            });
            return;
        }

        // Create new advanced settings
        let new_advanced_settings = AdvancedSchedulerSettings {
            strategy: selected_strategy(),
            difficulty_adaptation: difficulty_adaptation(),
            spaced_repetition_enabled: spaced_repetition_enabled(),
            cognitive_load_balancing: cognitive_load_balancing(),
            user_experience_level: user_experience_level(),
            custom_intervals: None, // Could be enhanced later
            max_session_duration_minutes: None,
            min_break_between_sessions_hours: None,
            prioritize_difficult_content: false,
            adaptive_pacing: true,
        };

        let new_settings = PlanSettings {
            start_date: props.plan.settings.start_date,
            sessions_per_week: sessions_per_week(),
            session_length_minutes: session_length(),
            include_weekends: include_weekends(),
            advanced_settings: Some(new_advanced_settings),
        };

        // Start regeneration process
        regeneration_status.set(RegenerationStatus::InProgress {
            progress: 0.0,
            message: "Starting plan regeneration...".to_string(),
        });

        let plan_id = props.plan.id;
        let backend_clone = backend.clone();
        let on_plan_regenerated = props.on_plan_regenerated;

        spawn(async move {
            // Use the simpler regenerate_plan method without progress callback
            // since Dioxus signals can't be moved into Send + Sync closures
            match backend_clone.regenerate_plan(plan_id, new_settings).await {
                Ok(new_plan) => {
                    regeneration_status.set(RegenerationStatus::Completed);
                    on_plan_regenerated.call(new_plan);
                    toast_helpers::success("Plan regenerated successfully!");

                    // Reset status after a delay
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    regeneration_status.set(RegenerationStatus::Idle);
                }
                Err(e) => {
                    regeneration_status.set(RegenerationStatus::Failed {
                        error: e.to_string(),
                    });
                    toast_helpers::error(format!("Failed to regenerate plan: {e}"));
                }
            }
        });
    };

    let reset_settings = move |_| {
        sessions_per_week.set(props.plan.settings.sessions_per_week);
        session_length.set(props.plan.settings.session_length_minutes);
        include_weekends.set(props.plan.settings.include_weekends);

        // Reset advanced settings
        let current_advanced = props
            .plan
            .settings
            .advanced_settings
            .clone()
            .unwrap_or_default();
        advanced_settings.set(current_advanced.clone());
        selected_strategy.set(current_advanced.strategy);
        user_experience_level.set(current_advanced.user_experience_level);
        difficulty_adaptation.set(current_advanced.difficulty_adaptation);
        spaced_repetition_enabled.set(current_advanced.spaced_repetition_enabled);
        cognitive_load_balancing.set(current_advanced.cognitive_load_balancing);

        form_errors.set(Vec::new());

        spawn(async move {
            toast_helpers::info("Settings reset to current plan values");
        });
    };

    rsx! {
        div { class: "card bg-base-100 border border-base-300 mb-6 shadow-sm",
            div { class: "card-body p-4",
                // Header with toggle button
                div {
                    class: "flex items-center justify-between cursor-pointer hover:bg-base-200/50 -m-2 p-2 rounded-lg transition-colors",
                    onclick: toggle_panel,

                    div { class: "flex items-center gap-3",
                        Icon { icon: FaGear, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Session Controls" }
                        div { class: "badge badge-outline badge-sm",
                            "{selected_strategy().display_name()}"
                        }
                    }

                    div { class: "flex items-center gap-2 text-sm text-base-content/60",
                        span { "{sessions_per_week()} sessions/week" }
                        span { "•" }
                        span { "{session_length()} min each" }

                        button {
                            class: "btn btn-ghost btn-sm btn-circle ml-2",
                            span {
                                class: "w-4 h-4 flex items-center justify-center transition-transform",
                                style: if is_expanded() { "transform: rotate(180deg)" } else { "" },
                                "▼"
                            }
                        }
                    }
                }

                // Form validation errors
                if !form_errors().is_empty() {
                    div { class: "alert alert-error mt-4",
                        Icon { icon: FaTriangleExclamation, class: "w-5 h-5" }
                        div {
                            for error in form_errors() {
                                div { class: "text-sm", "{error}" }
                            }
                        }
                    }
                }

                // Regeneration status
                match regeneration_status() {
                    RegenerationStatus::InProgress { progress, message } => rsx! {
                        div { class: "alert alert-info mt-4",
                            Icon { icon: FaSpinner, class: "w-5 h-5 animate-spin" }
                            div { class: "flex-1",
                                div { class: "text-sm font-medium", "{message}" }
                                div { class: "w-full bg-base-300 rounded-full h-2 mt-2",
                                    div {
                                        class: "bg-info h-2 rounded-full transition-all duration-300",
                                        style: "width: {progress}%"
                                    }
                                }
                            }
                        }
                    },
                    RegenerationStatus::Completed => rsx! {
                        div { class: "alert alert-success mt-4",
                            Icon { icon: FaCheck, class: "w-5 h-5" }
                            span { "Plan regeneration completed successfully!" }
                        }
                    },
                    RegenerationStatus::Failed { error } => rsx! {
                        div { class: "alert alert-error mt-4",
                            Icon { icon: FaTriangleExclamation, class: "w-5 h-5" }
                            span { "Regeneration failed: {error}" }
                        }
                    },
                    RegenerationStatus::Idle => rsx! { div {} }
                }

                // Expandable controls panel
                div {
                    class: "transition-all duration-300",
                    style: "{panel_style}",

                    div { class: "pt-4 space-y-6",
                        // Basic Settings Section
                        div { class: "space-y-4",
                            h4 { class: "text-md font-semibold text-base-content/80 border-b border-base-300 pb-2",
                                "Basic Settings"
                            }

                            // Sessions per week control
                            div { class: "form-control",
                                label { class: "label",
                                    span { class: "label-text flex items-center gap-2",
                                        Icon { icon: FaCalendarDays, class: "w-4 h-4" }
                                        "Sessions per Week"
                                    }
                                }

                                div { class: "flex items-center gap-4",
                                    input {
                                        type: "range",
                                        class: "range range-primary flex-1",
                                        min: "1",
                                        max: "14",
                                        value: "{sessions_per_week()}",
                                        step: "1",
                                        oninput: move |evt| {
                                            if let Ok(value) = evt.value().parse::<u8>() {
                                                sessions_per_week.set(value);
                                                // mark change for debounce
                                                change_version.set(change_version() + 1);
                                            }
                                        }
                                    }

                                    div { class: "badge badge-primary badge-lg font-mono min-w-12",
                                        "{sessions_per_week()}"
                                    }
                                }

                                div { class: "flex justify-between text-xs text-base-content/60 mt-1 px-1",
                                    span { "1" }
                                    span { "7" }
                                    span { "14" }
                                }
                            }

                            // Session length control
                            div { class: "form-control",
                                label { class: "label",
                                    span { class: "label-text flex items-center gap-2",
                                        Icon { icon: FaClock, class: "w-4 h-4" }
                                        "Session Length (minutes)"
                                    }
                                }

                                div { class: "flex items-center gap-4",
                                    input {
                                        type: "range",
                                        class: "range range-secondary flex-1",
                                        min: "15",
                                        max: "180",
                                        value: "{session_length()}",
                                        step: "15",
                                        oninput: move |evt| {
                                            if let Ok(value) = evt.value().parse::<u32>() {
                                                session_length.set(value);
                                                // mark change for debounce
                                                change_version.set(change_version() + 1);
                                            }
                                        }
                                    }

                                    div { class: "badge badge-secondary badge-lg font-mono min-w-16",
                                        "{session_length()}m"
                                    }
                                }

                                div { class: "flex justify-between text-xs text-base-content/60 mt-1 px-1",
                                    span { "15m" }
                                    span { "90m" }
                                    span { "180m" }
                                }
                            }

                            // Weekend inclusion toggle
                            div { class: "form-control",
                                label { class: "label cursor-pointer justify-start gap-3",
                                    input {
                                        type: "checkbox",
                                        class: "toggle toggle-accent",
                                        checked: include_weekends(),
                                        onchange: move |evt| {
                                            include_weekends.set(evt.checked());
                                            // mark change for debounce
                                            change_version.set(change_version() + 1);
                                        }
                                    }
                                    span { class: "label-text", "Include weekends in schedule" }
                                }
                            }
                        }

                        // Advanced Settings Toggle
                        div { class: "divider",
                            button {
                                class: "btn btn-ghost btn-sm",
                                onclick: toggle_advanced,
                                "Advanced Settings"
                                span {
                                    class: "ml-2 transition-transform",
                                    style: if show_advanced() { "transform: rotate(180deg)" } else { "" },
                                    "▼"
                                }
                            }
                        }

                        // Advanced Settings Section
                        if show_advanced() {
                            div { class: "space-y-4 bg-base-200/30 p-4 rounded-lg",
                                h4 { class: "text-md font-semibold text-base-content/80 mb-4",
                                    "Advanced Scheduling Options"
                                }

                                // Distribution Strategy Selection
                                div { class: "form-control",
                                    label { class: "label",
                                        span { class: "label-text font-medium", "Distribution Strategy" }
                                    }
                                    select {
                                        class: "select select-bordered w-full",
                                        value: "{selected_strategy() as u8}",
                                        onchange: move |evt| {
                                            if let Ok(value) = evt.value().parse::<u8>() {
                                                let strategies = DistributionStrategy::all();
                                                if let Some(strategy) = strategies.get(value as usize) {
                                                    selected_strategy.set(strategy.clone());
                                                    // mark change for debounce
                                                    change_version.set(change_version() + 1);
                                                }
                                            }
                                        },
                                        for (index, strategy) in DistributionStrategy::all().iter().enumerate() {
                                            option { value: "{index}", "{strategy.display_name()}" }
                                        }
                                    }
                                    label { class: "label",
                                        span { class: "label-text-alt text-xs", "{selected_strategy().description()}" }
                                    }
                                }

                                // User Experience Level
                                div { class: "form-control",
                                    label { class: "label",
                                        span { class: "label-text font-medium", "Your Experience Level" }
                                    }
                                    div { class: "join w-full",
                                        for (_index, level) in DifficultyLevel::all().iter().enumerate() {
                                            button {
                                                type: "button",
                                                class: format!("join-item btn btn-outline btn-sm flex-1 {}",
                                                    if user_experience_level() == *level { "btn-active" } else { "" }
                                                ),
                                                onclick: {
                                                    let level = *level;
                                                    move |_| {
                                                        user_experience_level.set(level);
                                                        // mark change for debounce
                                                        change_version.set(change_version() + 1);
                                                    }
                                                },
                                                "{level.display_name()}"
                                            }
                                        }
                                    }
                                }

                                // Feature Toggles
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div { class: "form-control",
                                        label { class: "label cursor-pointer justify-start gap-3",
                                            input {
                                                type: "checkbox",
                                                class: "checkbox checkbox-primary",
                                                checked: difficulty_adaptation(),
                                                onchange: move |evt| {
                                                    difficulty_adaptation.set(evt.checked());
                                                    // mark change for debounce
                                                    change_version.set(change_version() + 1);
                                                }
                                            }
                                            div {
                                                div { class: "label-text font-medium", "Difficulty Adaptation" }
                                                div { class: "label-text-alt text-xs", "Adjust pacing based on content complexity" }
                                            }
                                        }
                                    }

                                    div { class: "form-control",
                                        label { class: "label cursor-pointer justify-start gap-3",
                                            input {
                                                type: "checkbox",
                                                class: "checkbox checkbox-secondary",
                                                checked: spaced_repetition_enabled(),
                                                onchange: move |evt| {
                                                    spaced_repetition_enabled.set(evt.checked());
                                                    // mark change for debounce
                                                    change_version.set(change_version() + 1);
                                                }
                                            }
                                            div {
                                                div { class: "label-text font-medium", "Spaced Repetition" }
                                                div { class: "label-text-alt text-xs", "Add review sessions for better retention" }
                                            }
                                        }
                                    }

                                    div { class: "form-control md:col-span-2",
                                        label { class: "label cursor-pointer justify-start gap-3",
                                            input {
                                                type: "checkbox",
                                                class: "checkbox checkbox-accent",
                                                checked: cognitive_load_balancing(),
                                                onchange: move |evt| {
                                                    cognitive_load_balancing.set(evt.checked());
                                                    // mark change for debounce
                                                    change_version.set(change_version() + 1);
                                                }
                                            }
                                            div {
                                                div { class: "label-text font-medium", "Cognitive Load Balancing" }
                                                div { class: "label-text-alt text-xs", "Balance difficult and easy content within sessions" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Action buttons
                        div { class: "flex flex-col sm:flex-row gap-2 pt-4 border-t border-base-300",
                            button {
                                class: "btn btn-primary flex-1",
                                disabled: matches!(regeneration_status(), RegenerationStatus::InProgress { .. }),
                                onclick: apply_settings,
                                if matches!(regeneration_status(), RegenerationStatus::InProgress { .. }) {
                                    Icon { icon: FaSpinner, class: "w-4 h-4 animate-spin" }
                                } else {
                                    Icon { icon: FaRotateRight, class: "w-4 h-4" }
                                }
                                "Regenerate Plan"
                            }

                            button {
                                class: "btn btn-ghost",
                                disabled: matches!(regeneration_status(), RegenerationStatus::InProgress { .. }),
                                onclick: reset_settings,
                                "Reset to Current"
                            }
                        }
                    }
                }
            }
        }
    }
}
