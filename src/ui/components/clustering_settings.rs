//! Clustering settings and preference adjustment UI components
//!
//! This module provides UI components for users to adjust clustering parameters,
//! provide feedback, and interact with the preference learning system.

use crate::nlp::clustering::{
    ABTestConfig, ABTestResult, ABTestVariant, AdjustmentType, ClusteringFeedback,
    ClusteringPreferences, FeedbackType, ManualAdjustment,
};
use crate::types::{ClusteringAlgorithm, ClusteringStrategy, DifficultyLevel};
use dioxus::prelude::*;
use uuid::Uuid;

/// Props for the clustering settings component
#[derive(Props, Clone, PartialEq)]
pub struct ClusteringSettingsProps {
    pub preferences: ClusteringPreferences,
    pub on_preferences_change: EventHandler<ClusteringPreferences>,
    pub on_feedback_submit: EventHandler<ClusteringFeedback>,
    pub show_advanced: Option<bool>,
}

/// Main clustering settings component
#[component]
pub fn ClusteringSettings(props: ClusteringSettingsProps) -> Element {
    let mut show_advanced = use_signal(|| props.show_advanced.unwrap_or(false));
    let mut preferences = use_signal(|| props.preferences.clone());
    let mut show_feedback_modal = use_signal(|| false);

    let handle_preference_change = move |new_prefs: ClusteringPreferences| {
        preferences.set(new_prefs.clone());
        props.on_preferences_change.call(new_prefs);
    };

    rsx! {
        div { class: "space-y-6",
            // Header
            div { class: "flex items-center justify-between",
                h3 { class: "text-lg font-semibold text-gray-900 dark:text-gray-100",
                    "Clustering Preferences"
                }
                button {
                    class: "btn btn-sm btn-outline",
                    onclick: move |_| show_advanced.set(!show_advanced()),
                    if show_advanced() { "Hide Advanced" } else { "Show Advanced" }
                }
            }

            // Basic Settings
            BasicClusteringSettings {
                preferences: preferences(),
                on_change: handle_preference_change,
            }

            // Advanced Settings (conditionally shown)
            if show_advanced() {
                AdvancedClusteringSettings {
                    preferences: preferences(),
                    on_change: handle_preference_change,
                }
            }

            // Action Buttons
            div { class: "flex gap-3 pt-4 border-t border-gray-200 dark:border-gray-700",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        // Apply current preferences
                        props.on_preferences_change.call(preferences());
                    },
                    "Apply Settings"
                }
                button {
                    class: "btn btn-outline",
                    onclick: move |_| show_feedback_modal.set(true),
                    "Provide Feedback"
                }
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| {
                        // Reset to defaults
                        let default_prefs = ClusteringPreferences::default();
                        preferences.set(default_prefs.clone());
                        props.on_preferences_change.call(default_prefs);
                    },
                    "Reset to Defaults"
                }
            }

            // Feedback Modal
            if show_feedback_modal() {
                FeedbackModal {
                    preferences: preferences(),
                    on_submit: move |feedback| {
                        props.on_feedback_submit.call(feedback);
                        show_feedback_modal.set(false);
                    },
                    on_close: move |_| show_feedback_modal.set(false),
                }
            }
        }
    }
}

/// Basic clustering settings component
#[component]
pub fn BasicClusteringSettings(
    preferences: ClusteringPreferences,
    on_change: EventHandler<ClusteringPreferences>,
) -> Element {
    let mut local_prefs = use_signal(|| preferences.clone());

    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
            // Similarity Threshold
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "Content Similarity Threshold" }
                    span { class: "label-text-alt text-xs", "{(local_prefs().similarity_threshold * 100.0) as u32}%" }
                }
                input {
                    r#type: "range",
                    class: "range range-primary",
                    min: "30",
                    max: "90",
                    value: "{(local_prefs().similarity_threshold * 100.0) as u32}",
                    oninput: move |evt| {
                        if let Ok(value) = evt.value().parse::<f32>() {
                            let mut prefs = local_prefs();
                            prefs.similarity_threshold = value / 100.0;
                            local_prefs.set(prefs.clone());
                            on_change.call(prefs);
                        }
                    },
                }
                div { class: "text-xs text-gray-500 mt-1",
                    "Higher values create fewer, more similar groups. Lower values create more, diverse groups."
                }
            }

            // Preferred Algorithm
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "Clustering Algorithm" }
                }
                select {
                    class: "select select-bordered w-full",
                    value: format!("{:?}", local_prefs().preferred_algorithm),
                    onchange: move |evt| {
                        let algorithm = evt.value();
                        let algo = match algorithm.as_str() {
                            "TfIdf" => ClusteringAlgorithm::TfIdf,
                            "KMeans" => ClusteringAlgorithm::KMeans,
                            "Hierarchical" => ClusteringAlgorithm::Hierarchical,
                            "Lda" => ClusteringAlgorithm::Lda,
                            "Hybrid" => ClusteringAlgorithm::Hybrid,
                            _ => ClusteringAlgorithm::Hybrid,
                        };
                        let mut prefs = local_prefs();
                        prefs.preferred_algorithm = algo;
                        local_prefs.set(prefs.clone());
                        on_change.call(prefs);
                    },
                    option { value: "TfIdf", "TF-IDF (Content Analysis)" }
                    option { value: "KMeans", "K-Means (Similarity Grouping)" }
                    option { value: "Hierarchical", "Hierarchical (Tree Structure)" }
                    option { value: "Lda", "LDA (Topic Modeling)" }
                    option { value: "Hybrid", "Hybrid (Automatic Selection)" }
                }
            }

            // Max Clusters
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "Maximum Modules" }
                    span { class: "label-text-alt text-xs", "{local_prefs().max_clusters}" }
                }
                input {
                    r#type: "range",
                    class: "range range-secondary",
                    min: "3",
                    max: "15",
                    value: "{local_prefs().max_clusters}",
                    oninput: move |evt| {
                        if let Ok(value) = evt.value().parse::<usize>() {
                            let mut prefs = local_prefs();
                            prefs.max_clusters = value;
                            local_prefs.set(prefs.clone());
                            on_change.call(prefs);
                        }
                    },
                }
            }

            // User Experience Level
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "Your Experience Level" }
                }
                select {
                    class: "select select-bordered w-full",
                    value: format!("{:?}", local_prefs().user_experience_level),
                    onchange: move |evt| {
                        let level_str = evt.value();
                        let level = match level_str.as_str() {
                            "Beginner" => DifficultyLevel::Beginner,
                            "Intermediate" => DifficultyLevel::Intermediate,
                            "Advanced" => DifficultyLevel::Advanced,
                            "Expert" => DifficultyLevel::Expert,
                            _ => DifficultyLevel::Intermediate,
                        };
                        let mut prefs = local_prefs();
                        prefs.user_experience_level = level;
                        local_prefs.set(prefs.clone());
                        on_change.call(prefs);
                    },
                    option { value: "Beginner", "Beginner" }
                    option { value: "Intermediate", "Intermediate" }
                    option { value: "Advanced", "Advanced" }
                    option { value: "Expert", "Expert" }
                }
            }
        }
    }
}

/// Advanced clustering settings component
#[component]
pub fn AdvancedClusteringSettings(
    preferences: ClusteringPreferences,
    on_change: EventHandler<ClusteringPreferences>,
) -> Element {
    let mut local_prefs = use_signal(|| preferences.clone());

    rsx! {
        div { class: "space-y-4 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg",
            h4 { class: "font-medium text-gray-900 dark:text-gray-100 mb-3",
                "Advanced Settings"
            }

            // Content vs Duration Weight
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "Content vs Duration Balance" }
                    span { class: "label-text-alt text-xs",
                        "Content: {(local_prefs().content_vs_duration_weight * 100.0) as u32}%"
                    }
                }
                input {
                    r#type: "range",
                    class: "range range-accent",
                    min: "10",
                    max: "90",
                    value: "{(local_prefs().content_vs_duration_weight * 100.0) as u32}",
                    oninput: move |evt| {
                        if let Ok(value) = evt.value().parse::<f32>() {
                            let mut prefs = local_prefs();
                            prefs.content_vs_duration_weight = value / 100.0;
                            local_prefs.set(prefs.clone());
                            on_change.call(prefs);
                        }
                    },
                }
            }

            // Min Cluster Size
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "Minimum Videos per Module" }
                    span { class: "label-text-alt text-xs", "{local_prefs().min_cluster_size}" }
                }
                input {
                    r#type: "range",
                    class: "range range-warning",
                    min: "1",
                    max: "10",
                    value: "{local_prefs().min_cluster_size}",
                    oninput: move |evt| {
                        if let Ok(value) = evt.value().parse::<usize>() {
                            let mut prefs = local_prefs();
                            prefs.min_cluster_size = value;
                            local_prefs.set(prefs.clone());
                            on_change.call(prefs);
                        }
                    },
                }
            }

            // Duration Balancing Toggle
            div { class: "form-control",
                label { class: "label cursor-pointer",
                    span { class: "label-text font-medium", "Enable Duration Balancing" }
                    input {
                        r#type: "checkbox",
                        class: "toggle toggle-primary",
                        checked: local_prefs().enable_duration_balancing,
                        onchange: move |evt| {
                            let mut prefs = local_prefs();
                            prefs.enable_duration_balancing = evt.checked();
                            local_prefs.set(prefs.clone());
                            on_change.call(prefs);
                        },
                    }
                }
            }

            // Performance Info
            div { class: "stats stats-horizontal w-full mt-4",
                div { class: "stat",
                    div { class: "stat-title", "Usage Count" }
                    div { class: "stat-value text-sm", "{local_prefs().usage_count}" }
                }
                div { class: "stat",
                    div { class: "stat-title", "Satisfaction Score" }
                    div { class: "stat-value text-sm", "{(local_prefs().satisfaction_score * 100.0) as u32}%" }
                }
            }
        }
    }
}

/// Feedback modal component
#[component]
pub fn FeedbackModal(
    preferences: ClusteringPreferences,
    on_submit: EventHandler<ClusteringFeedback>,
    on_close: EventHandler<()>,
) -> Element {
    let mut rating = use_signal(|| 3.0);
    let mut comments = use_signal(|| String::new());
    let mut feedback_type = use_signal(|| FeedbackType::ExplicitRating);

    let handle_submit = move |_| {
        let feedback = ClusteringFeedback {
            id: Uuid::new_v4(),
            course_id: Uuid::new_v4(), // This should be passed from parent
            clustering_parameters: preferences.clone(),
            feedback_type: feedback_type(),
            rating: rating() / 5.0, // Convert 1-5 scale to 0-1
            comments: if comments().trim().is_empty() {
                None
            } else {
                Some(comments())
            },
            manual_adjustments: Vec::new(), // This would be populated from actual adjustments
            created_at: chrono::Utc::now(),
        };
        on_submit.call(feedback);
    };

    rsx! {
        div { class: "modal modal-open",
            div { class: "modal-box",
                h3 { class: "font-bold text-lg mb-4", "Clustering Feedback" }

                div { class: "space-y-4",
                    // Rating
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "How satisfied are you with the clustering?" }
                        }
                        div { class: "rating rating-lg",
                            for i in 1..=5 {
                                input {
                                    r#type: "radio",
                                    name: "rating",
                                    class: "mask mask-star-2 bg-orange-400",
                                    checked: rating() as u32 == i,
                                    onchange: move |_| rating.set(i as f32),
                                }
                            }
                        }
                        div { class: "text-sm text-gray-500 mt-1",
                            match rating() as u32 {
                                1 => "Very Poor - Clustering made no sense",
                                2 => "Poor - Many videos were grouped incorrectly",
                                3 => "Average - Some good groups, some poor ones",
                                4 => "Good - Most groups made sense with minor issues",
                                5 => "Excellent - Perfect grouping, very logical",
                                _ => "Please rate your experience",
                            }
                        }
                    }

                    // Comments
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Additional Comments (Optional)" }
                        }
                        textarea {
                            class: "textarea textarea-bordered h-24",
                            placeholder: "What worked well? What could be improved?",
                            value: "{comments()}",
                            oninput: move |evt| comments.set(evt.value()),
                        }
                    }

                    // Feedback Type
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Feedback Type" }
                        }
                        select {
                            class: "select select-bordered w-full",
                            value: format!("{:?}", feedback_type()),
                            onchange: move |evt| {
                                let ft = match evt.value().as_str() {
                                    "ExplicitRating" => FeedbackType::ExplicitRating,
                                    "ManualAdjustment" => FeedbackType::ManualAdjustment,
                                    "ParameterChange" => FeedbackType::ParameterChange,
                                    "ImplicitAcceptance" => FeedbackType::ImplicitAcceptance,
                                    "Rejection" => FeedbackType::Rejection,
                                    _ => FeedbackType::ExplicitRating,
                                };
                                feedback_type.set(ft);
                            },
                            option { value: "ExplicitRating", "General Rating" }
                            option { value: "ManualAdjustment", "After Manual Changes" }
                            option { value: "ParameterChange", "After Parameter Adjustment" }
                            option { value: "ImplicitAcceptance", "Accepted As-Is" }
                            option { value: "Rejection", "Rejected Clustering" }
                        }
                    }
                }

                div { class: "modal-action",
                    button {
                        class: "btn btn-primary",
                        onclick: handle_submit,
                        "Submit Feedback"
                    }
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}

/// A/B Test results display component
#[component]
pub fn ABTestResults(test_config: ABTestConfig, results: Vec<ABTestResult>) -> Element {
    let variant_a_results: Vec<&ABTestResult> = results
        .iter()
        .filter(|r| matches!(r.variant, ABTestVariant::VariantA))
        .collect();

    let variant_b_results: Vec<&ABTestResult> = results
        .iter()
        .filter(|r| matches!(r.variant, ABTestVariant::VariantB))
        .collect();

    let avg_satisfaction_a = if !variant_a_results.is_empty() {
        variant_a_results
            .iter()
            .map(|r| r.user_satisfaction)
            .sum::<f32>()
            / variant_a_results.len() as f32
    } else {
        0.0
    };

    let avg_satisfaction_b = if !variant_b_results.is_empty() {
        variant_b_results
            .iter()
            .map(|r| r.user_satisfaction)
            .sum::<f32>()
            / variant_b_results.len() as f32
    } else {
        0.0
    };

    rsx! {
        div { class: "card bg-base-100 shadow-xl",
            div { class: "card-body",
                h2 { class: "card-title", "{test_config.name}" }
                p { class: "text-sm text-gray-600", "{test_config.description}" }

                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4 mt-4",
                    // Variant A Results
                    div { class: "stats shadow",
                        div { class: "stat",
                            div { class: "stat-title", "Variant A ({test_config.algorithm_a:?})" }
                            div { class: "stat-value text-primary", "{(avg_satisfaction_a * 100.0) as u32}%" }
                            div { class: "stat-desc", "{variant_a_results.len()} samples" }
                        }
                    }

                    // Variant B Results
                    div { class: "stats shadow",
                        div { class: "stat",
                            div { class: "stat-title", "Variant B ({test_config.algorithm_b:?})" }
                            div { class: "stat-value text-secondary", "{(avg_satisfaction_b * 100.0) as u32}%" }
                            div { class: "stat-desc", "{variant_b_results.len()} samples" }
                        }
                    }
                }

                // Progress
                div { class: "mt-4",
                    div { class: "flex justify-between text-sm mb-1",
                        span { "Progress" }
                        span { "{test_config.current_sample_size}/{test_config.target_sample_size}" }
                    }
                    progress {
                        class: "progress progress-primary w-full",
                        value: "{test_config.current_sample_size}",
                        max: "{test_config.target_sample_size}",
                    }
                }

                // Winner indication
                if test_config.current_sample_size >= test_config.target_sample_size {
                    div { class: "alert mt-4",
                        class: if avg_satisfaction_a > avg_satisfaction_b { "alert-success" } else { "alert-info" },
                        span {
                            if avg_satisfaction_a > avg_satisfaction_b {
                                "🏆 Variant A ({test_config.algorithm_a:?}) is the winner!"
                            } else if avg_satisfaction_b > avg_satisfaction_a {
                                "🏆 Variant B ({test_config.algorithm_b:?}) is the winner!"
                            } else {
                                "🤝 Results are too close to determine a clear winner."
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Manual adjustment interface component
#[component]
pub fn ManualAdjustmentInterface(
    course_id: Uuid,
    modules: Vec<crate::types::Module>,
    on_adjustment: EventHandler<ManualAdjustment>,
) -> Element {
    let mut selected_videos = use_signal(|| Vec::<usize>::new());
    let mut target_module = use_signal(|| 0usize);
    let mut adjustment_reason = use_signal(|| String::new());

    rsx! {
        div { class: "card bg-base-100 shadow-xl",
            div { class: "card-body",
                h3 { class: "card-title", "Manual Clustering Adjustments" }
                p { class: "text-sm text-gray-600 mb-4",
                    "Select videos to move between modules. Your adjustments help improve future clustering."
                }

                div { class: "space-y-4",
                    // Module selector
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Move to Module" }
                        }
                        select {
                            class: "select select-bordered w-full",
                            value: "{target_module()}",
                            onchange: move |evt| {
                                if let Ok(idx) = evt.value().parse::<usize>() {
                                    target_module.set(idx);
                                }
                            },
                            for (idx, module) in modules.iter().enumerate() {
                                option { value: "{idx}", "{module.title}" }
                            }
                        }
                    }

                    // Video selection (simplified - in real implementation would show all videos)
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Selected Videos" }
                        }
                        div { class: "text-sm text-gray-500",
                            "Selected: {selected_videos().len()} videos"
                        }
                    }

                    // Reason input
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Reason for Change (Optional)" }
                        }
                        textarea {
                            class: "textarea textarea-bordered",
                            placeholder: "Why are you moving these videos?",
                            value: "{adjustment_reason()}",
                            oninput: move |evt| adjustment_reason.set(evt.value()),
                        }
                    }

                    // Action buttons
                    div { class: "flex gap-2",
                        button {
                            class: "btn btn-primary",
                            disabled: selected_videos().is_empty(),
                            onclick: move |_| {
                                let adjustment = ManualAdjustment {
                                    adjustment_type: AdjustmentType::MoveVideos,
                                    from_module: 0, // Would be determined from selection
                                    to_module: target_module(),
                                    video_indices: selected_videos(),
                                    reason: if adjustment_reason().trim().is_empty() {
                                        None
                                    } else {
                                        Some(adjustment_reason())
                                    },
                                    timestamp: chrono::Utc::now(),
                                };
                                on_adjustment.call(adjustment);
                            },
                            "Apply Changes"
                        }
                        button {
                            class: "btn btn-ghost",
                            onclick: move |_| {
                                selected_videos.set(Vec::new());
                                adjustment_reason.set(String::new());
                            },
                            "Clear Selection"
                        }
                    }
                }
            }
        }
    }
}
