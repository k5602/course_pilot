use crate::planner::PlanAnalysis;

use crate::types::{AdvancedSchedulerSettings, Course, DifficultyLevel};
use crate::ui::hooks::{use_analytics_manager, use_backend};
use dioxus::prelude::*;

use uuid::Uuid;

#[derive(Props, PartialEq, Clone)]
pub struct AIRecommendationsPanelProps {
    #[props(optional)]
    pub course_id: Option<Uuid>,
    #[props(default = DifficultyLevel::Intermediate)]
    pub user_experience: DifficultyLevel,
}

#[component]
pub fn AIRecommendationsPanel(props: AIRecommendationsPanelProps) -> Element {
    let analytics_manager = use_analytics_manager();
    let backend = use_backend();

    let recommendations_resource = use_resource(move || {
        let analytics_manager = analytics_manager.clone();
        let backend = backend.clone();
        let course_id = props.course_id;
        let user_experience = props.user_experience;

        async move {
            // Get comprehensive recommendations based on course analysis
            if let Some(course_id) = course_id {
                // Get course data and plan analysis
                let course_result = tokio::task::spawn_blocking({
                    let _backend = backend.clone(); // backend captured outside; direct call used below
                    move || Ok::<Option<Course>, anyhow::Error>(None) // placeholder; replaced by async call below
                })
                .await
                .unwrap_or(Ok(None));

                let plan_analysis_result = analytics_manager.get_plan_analysis(course_id).await;
                let settings_result = analytics_manager
                    .get_recommended_advanced_settings(course_id, user_experience)
                    .await;

                match (course_result, plan_analysis_result, settings_result) {
                    (Ok(Some(course)), Ok(plan_analysis), Ok(settings)) => {
                        Ok(AIRecommendationData {
                            course: Some(course),
                            plan_analysis: Some(plan_analysis),
                            settings,
                            user_experience,
                        })
                    },
                    (Ok(Some(course)), Err(_), Ok(settings)) => {
                        // Course exists but no plan yet
                        Ok(AIRecommendationData {
                            course: Some(course),
                            plan_analysis: None,
                            settings,
                            user_experience,
                        })
                    },
                    (_, _, Ok(settings)) => {
                        // Fallback to basic settings
                        Ok(AIRecommendationData {
                            course: None,
                            plan_analysis: None,
                            settings,
                            user_experience,
                        })
                    },
                    (_, _, Err(e)) => Err(e),
                }
            } else {
                // General recommendations without specific course
                let learning_analytics = analytics_manager.get_learning_analytics().await?;
                let default_settings = AdvancedSchedulerSettings::default();

                Ok(AIRecommendationData {
                    course: None,
                    plan_analysis: learning_analytics.first().cloned(),
                    settings: default_settings,
                    user_experience,
                })
            }
        }
    });

    match &*recommendations_resource.read() {
        Some(Ok(data)) => rsx! {
            div { class: "space-y-4",
                // Strategy recommendations
                StrategyRecommendations { data: data.clone() }

                // Study optimization suggestions
                StudyOptimizationSuggestions { data: data.clone() }

                // Adaptive pacing recommendations
                AdaptivePacingRecommendations { data: data.clone() }
            }
        },
        Some(Err(e)) => rsx! {
            div { class: "alert alert-warning",
                div { class: "flex items-center gap-2",
                    span { "⚠️" }
                    span { "Unable to load AI recommendations: {e}" }
                }
            }
        },
        None => rsx! {
            div { class: "space-y-3",
                div { class: "skeleton h-20 w-full" }
                div { class: "skeleton h-16 w-full" }
                div { class: "skeleton h-16 w-full" }
            }
        },
    }
}

#[derive(Clone, PartialEq)]
struct AIRecommendationData {
    course: Option<Course>,
    plan_analysis: Option<PlanAnalysis>,
    settings: AdvancedSchedulerSettings,
    user_experience: DifficultyLevel,
}

#[derive(Props, PartialEq, Clone)]
struct StrategyRecommendationsProps {
    data: AIRecommendationData,
}

#[component]
fn StrategyRecommendations(props: StrategyRecommendationsProps) -> Element {
    let data = &props.data;
    let settings = &data.settings;

    // Generate strategy rationale based on course characteristics
    let strategy_rationale = if let Some(course) = &data.course {
        let video_count = course.video_count();
        let has_structure = course.structure.is_some();

        match (&settings.strategy, video_count, has_structure) {
            (crate::types::DistributionStrategy::SpacedRepetition, _, _) => {
                "Recommended for beginners to optimize long-term retention through spaced intervals"
            },
            (crate::types::DistributionStrategy::Adaptive, videos, _) if videos > 50 => {
                "Large course detected - adaptive scheduling will balance cognitive load dynamically"
            },
            (crate::types::DistributionStrategy::ModuleBased, _, true) => {
                "Well-structured course - module-based approach respects content organization"
            },
            (crate::types::DistributionStrategy::Hybrid, _, _) => {
                "Balanced approach combining multiple strategies for optimal learning"
            },
            _ => "Strategy selected based on course characteristics and user experience level",
        }
    } else {
        "General strategy recommendation based on user experience level"
    };

    rsx! {
        div { class: "card bg-gradient-to-r from-primary/5 to-secondary/5 border border-primary/20",
            div { class: "card-body p-4",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "🎯" }
                    "Recommended Strategy"
                }

                div { class: "mt-3",
                    div { class: "flex items-center justify-between mb-2",
                        span { class: "font-semibold text-primary", "{settings.strategy.display_name()}" }
                        span { class: "badge badge-primary badge-outline", "AI Selected" }
                    }
                    p { class: "text-sm text-base-content/80 mb-3", "{settings.strategy.description()}" }
                    p { class: "text-xs text-base-content/60 italic", "{strategy_rationale}" }
                }

                // Strategy features
                div { class: "mt-4 grid grid-cols-2 gap-2",
                    if settings.spaced_repetition_enabled {
                        FeatureBadge {
                            label: "Spaced Repetition",
                            enabled: true,
                            description: "Review sessions for retention"
                        }
                    }
                    if settings.cognitive_load_balancing {
                        FeatureBadge {
                            label: "Load Balancing",
                            enabled: true,
                            description: "Balanced difficulty distribution"
                        }
                    }
                    if settings.adaptive_pacing {
                        FeatureBadge {
                            label: "Adaptive Pacing",
                            enabled: true,
                            description: "Dynamic session adjustment"
                        }
                    }
                    if settings.difficulty_adaptation {
                        FeatureBadge {
                            label: "Difficulty Adaptation",
                            enabled: true,
                            description: "Progressive difficulty increase"
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct StudyOptimizationSuggestionsProps {
    data: AIRecommendationData,
}

#[component]
fn StudyOptimizationSuggestions(props: StudyOptimizationSuggestionsProps) -> Element {
    let data = &props.data;

    // Generate optimization suggestions based on plan analysis
    let suggestions = if let Some(analysis) = &data.plan_analysis {
        let mut suggestions = Vec::new();

        // Velocity-based suggestions
        if analysis.velocity_analysis.videos_per_day < 0.5 {
            suggestions.push(OptimizationSuggestion {
                title: "Increase Study Frequency".to_string(),
                description: "Consider adding more sessions per week to maintain momentum"
                    .to_string(),
                priority: "Medium".to_string(),
                icon: "📈".to_string(),
            });
        } else if analysis.velocity_analysis.videos_per_day > 2.0 {
            suggestions.push(OptimizationSuggestion {
                title: "Reduce Study Intensity".to_string(),
                description: "High velocity detected - consider longer sessions with fewer videos"
                    .to_string(),
                priority: "Low".to_string(),
                icon: "⚡".to_string(),
            });
        }

        // Load distribution suggestions
        if analysis.load_distribution.load_variance > 0.3 {
            suggestions.push(OptimizationSuggestion {
                title: "Improve Load Balance".to_string(),
                description: "Cognitive load varies significantly - enable load balancing"
                    .to_string(),
                priority: "High".to_string(),
                icon: "⚖️".to_string(),
            });
        }

        // Temporal distribution suggestions
        if analysis.temporal_distribution.consistency_score < 0.7 {
            suggestions.push(OptimizationSuggestion {
                title: "Maintain Consistency".to_string(),
                description: "Try to keep more regular intervals between study sessions"
                    .to_string(),
                priority: "Medium".to_string(),
                icon: "🎯".to_string(),
            });
        }

        if analysis.temporal_distribution.weekend_utilization < 0.3 {
            suggestions.push(OptimizationSuggestion {
                title: "Utilize Weekends".to_string(),
                description: "Consider adding weekend sessions for faster progress".to_string(),
                priority: "Low".to_string(),
                icon: "📅".to_string(),
            });
        }

        suggestions
    } else {
        // Default suggestions for new users
        vec![
            OptimizationSuggestion {
                title: "Start with Structured Learning".to_string(),
                description: "Create your first study plan to get personalized recommendations"
                    .to_string(),
                priority: "High".to_string(),
                icon: "🚀".to_string(),
            },
            OptimizationSuggestion {
                title: "Enable Analytics Tracking".to_string(),
                description: "Complete a few sessions to unlock detailed performance insights"
                    .to_string(),
                priority: "Medium".to_string(),
                icon: "📊".to_string(),
            },
        ]
    };

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body p-4",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "💡" }
                    "Study Optimization Suggestions"
                }

                if suggestions.is_empty() {
                    div { class: "text-center py-6 text-base-content/60",
                        div { class: "text-3xl mb-2", "✨" }
                        p { "Your study patterns are well optimized!" }
                        p { class: "text-sm mt-1", "Keep up the great work" }
                    }
                } else {
                    div { class: "space-y-3 mt-4",
                        {suggestions.iter().enumerate().map(|(index, suggestion)| rsx! {
                            SuggestionCard {
                                key: "{index}",
                                suggestion: suggestion.clone()
                            }
                        })}
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct AdaptivePacingRecommendationsProps {
    data: AIRecommendationData,
}

#[component]
fn AdaptivePacingRecommendations(props: AdaptivePacingRecommendationsProps) -> Element {
    let data = &props.data;

    // Generate pacing recommendations based on user experience and course characteristics
    let pacing_recommendations = match data.user_experience {
        DifficultyLevel::Beginner => vec![
            PacingRecommendation {
                title: "Start Slowly".to_string(),
                description: "Begin with 2-3 sessions per week, 30-45 minutes each".to_string(),
                rationale: "Builds sustainable learning habits without overwhelming".to_string(),
                icon: "🐌".to_string(),
            },
            PacingRecommendation {
                title: "Focus on Retention".to_string(),
                description: "Enable spaced repetition for better long-term memory".to_string(),
                rationale: "Beginners benefit most from repeated exposure to concepts".to_string(),
                icon: "🧠".to_string(),
            },
        ],
        DifficultyLevel::Intermediate => vec![
            PacingRecommendation {
                title: "Balanced Approach".to_string(),
                description: "3-4 sessions per week with moderate cognitive load".to_string(),
                rationale: "Optimal balance between progress and comprehension".to_string(),
                icon: "⚖️".to_string(),
            },
            PacingRecommendation {
                title: "Progressive Difficulty".to_string(),
                description: "Gradually increase session complexity over time".to_string(),
                rationale: "Builds confidence while maintaining appropriate challenge".to_string(),
                icon: "📈".to_string(),
            },
        ],
        DifficultyLevel::Advanced => vec![
            PacingRecommendation {
                title: "Intensive Learning".to_string(),
                description: "4-5 sessions per week with higher cognitive load".to_string(),
                rationale: "Advanced learners can handle more intensive schedules".to_string(),
                icon: "🚀".to_string(),
            },
            PacingRecommendation {
                title: "Challenge Prioritization".to_string(),
                description: "Focus on difficult content first when energy is highest".to_string(),
                rationale: "Maximizes learning efficiency for complex topics".to_string(),
                icon: "🎯".to_string(),
            },
        ],
        DifficultyLevel::Expert => vec![
            PacingRecommendation {
                title: "Efficient Mastery".to_string(),
                description: "Fewer but longer sessions with maximum content density".to_string(),
                rationale: "Experts can process information more efficiently".to_string(),
                icon: "⚡".to_string(),
            },
            PacingRecommendation {
                title: "Self-Directed Learning".to_string(),
                description: "Adaptive pacing based on real-time comprehension".to_string(),
                rationale: "Expert learners benefit from flexible, responsive scheduling"
                    .to_string(),
                icon: "🎛️".to_string(),
            },
        ],
    };

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body p-4",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "📊" }
                    "Adaptive Pacing for {data.user_experience.display_name()}s"
                }

                div { class: "space-y-3 mt-4",
                    {pacing_recommendations.iter().enumerate().map(|(index, rec)| rsx! {
                        PacingCard {
                            key: "{index}",
                            recommendation: rec.clone()
                        }
                    })}
                }
            }
        }
    }
}

// Helper components and structs
#[derive(Clone, PartialEq)]
struct OptimizationSuggestion {
    title: String,
    description: String,
    priority: String,
    icon: String,
}

#[derive(Clone, PartialEq)]
struct PacingRecommendation {
    title: String,
    description: String,
    rationale: String,
    icon: String,
}

#[derive(Props, PartialEq, Clone)]
struct FeatureBadgeProps {
    label: String,
    enabled: bool,
    description: String,
}

#[component]
fn FeatureBadge(props: FeatureBadgeProps) -> Element {
    rsx! {
        div {
            class: "tooltip tooltip-top",
            "data-tip": "{props.description}",
            span {
                class: if props.enabled { "badge badge-success badge-sm" } else { "badge badge-ghost badge-sm" },
                "{props.label}"
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SuggestionCardProps {
    suggestion: OptimizationSuggestion,
}

#[component]
fn SuggestionCard(props: SuggestionCardProps) -> Element {
    let suggestion = &props.suggestion;
    let priority_color = match suggestion.priority.as_str() {
        "High" => "border-l-error",
        "Medium" => "border-l-warning",
        "Low" => "border-l-info",
        _ => "border-l-primary",
    };

    rsx! {
        div { class: "flex items-start gap-3 p-3 bg-base-200 rounded-lg border-l-4 {priority_color}",
            div { class: "text-xl", "{suggestion.icon}" }
            div { class: "flex-1",
                h4 { class: "font-semibold text-sm", "{suggestion.title}" }
                p { class: "text-xs text-base-content/70 mt-1", "{suggestion.description}" }
                span { class: "badge badge-outline badge-xs mt-2", "{suggestion.priority} Priority" }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct PacingCardProps {
    recommendation: PacingRecommendation,
}

#[component]
fn PacingCard(props: PacingCardProps) -> Element {
    let rec = &props.recommendation;

    rsx! {
        div { class: "flex items-start gap-3 p-3 bg-gradient-to-r from-accent/5 to-secondary/5 rounded-lg border border-accent/20",
            div { class: "text-xl", "{rec.icon}" }
            div { class: "flex-1",
                h4 { class: "font-semibold text-sm", "{rec.title}" }
                p { class: "text-xs text-base-content/80 mt-1", "{rec.description}" }
                p { class: "text-xs text-base-content/60 mt-2 italic", "{rec.rationale}" }
            }
        }
    }
}
