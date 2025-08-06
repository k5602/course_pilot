
use crate::planner::scheduler::{PlanAnalysis, VelocityCategory};
use crate::storage::{ClusteringAnalytics, Database, get_clustering_analytics};
use crate::ui::hooks::use_analytics_manager;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn LearningAnalytics() -> Element {
    let analytics_manager = use_analytics_manager();
    let db = use_context::<Arc<Database>>();

    let learning_analytics_resource = use_resource(move || {
        let analytics_manager = analytics_manager.clone();
        async move { analytics_manager.get_learning_analytics().await }
    });

    let clustering_analytics_resource = use_resource(move || {
        let db_clone = db.clone();
        async move {
            tokio::task::spawn_blocking(move || get_clustering_analytics(&db_clone))
                .await
                .unwrap_or_else(|_| {
                    Err(anyhow::anyhow!("Failed to load analytics"))
                })
        }
    });

    match (
        &*learning_analytics_resource.read_unchecked(),
        &*clustering_analytics_resource.read_unchecked(),
    ) {
        (Some(Ok(plan_analyses)), Some(Ok(clustering_analytics))) => rsx! {
            div { class: "space-y-6",
                // Learning velocity trends across all plans
                LearningVelocityChart { plan_analyses: plan_analyses.clone() }

                // Cognitive load distribution visualization
                CognitiveLoadDistribution { plan_analyses: plan_analyses.clone() }

                // Difficulty progression tracker with recommendations
                DifficultyProgressionTracker {
                    plan_analyses: plan_analyses.clone(),
                    clustering_analytics: clustering_analytics.clone()
                }
            }
        },
        (Some(Err(e)), _) => rsx! {
            div { class: "alert alert-error",
                "Failed to load learning analytics: {e}"
            }
        },
        (_, Some(Err(e))) => rsx! {
            div { class: "alert alert-error",
                "Failed to load clustering analytics: {e:?}"
            }
        },
        _ => rsx! {
            div { class: "space-y-4",
                div { class: "skeleton h-32 w-full" }
                div { class: "skeleton h-24 w-full" }
                div { class: "skeleton h-20 w-full" }
            }
        },
    }
}

#[derive(Props, PartialEq, Clone)]
struct LearningVelocityChartProps {
    plan_analyses: Vec<PlanAnalysis>,
}

#[component]
fn LearningVelocityChart(props: LearningVelocityChartProps) -> Element {
    let plan_analyses = &props.plan_analyses;

    if plan_analyses.is_empty() {
        return rsx! {
            div { class: "card bg-base-100 shadow-sm border border-base-300",
                div { class: "card-body text-center py-8",
                    div { class: "text-4xl mb-2", "📊" }
                    h3 { class: "card-title text-lg justify-center", "Learning Velocity" }
                    p { class: "text-base-content/60", "No learning data available yet" }
                    p { class: "text-sm text-base-content/50", "Create some study plans to see your learning velocity trends" }
                }
            }
        };
    }

    // Calculate aggregate velocity metrics
    let total_videos: usize = plan_analyses
        .iter()
        .map(|analysis| {
            (analysis.velocity_analysis.videos_per_day
                * analysis.velocity_analysis.total_duration_days as f32) as usize
        })
        .sum();

    let avg_videos_per_day: f32 = plan_analyses
        .iter()
        .map(|analysis| analysis.velocity_analysis.videos_per_day)
        .sum::<f32>()
        / plan_analyses.len() as f32;

    let total_study_days: i64 = plan_analyses
        .iter()
        .map(|analysis| analysis.velocity_analysis.total_duration_days)
        .sum();

    // Categorize velocity distribution
    let velocity_counts = plan_analyses.iter().fold([0; 4], |mut acc, analysis| {
        match analysis.velocity_analysis.velocity_category {
            VelocityCategory::Slow => acc[0] += 1,
            VelocityCategory::Moderate => acc[1] += 1,
            VelocityCategory::Fast => acc[2] += 1,
            VelocityCategory::Intensive => acc[3] += 1,
        }
        acc
    });

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "📈" }
                    "Learning Velocity Trends"
                }

                div { class: "grid grid-cols-3 gap-4 mt-4",
                    div { class: "stat",
                        div { class: "stat-title", "Avg Videos/Day" }
                        div { class: "stat-value text-primary", "{avg_videos_per_day:.1}" }
                        div { class: "stat-desc", "Across all plans" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Total Videos" }
                        div { class: "stat-value text-secondary", "{total_videos}" }
                        div { class: "stat-desc", "Studied so far" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Study Days" }
                        div { class: "stat-value text-accent", "{total_study_days}" }
                        div { class: "stat-desc", "Total planned" }
                    }
                }

                // Velocity distribution visualization
                div { class: "mt-6",
                    h4 { class: "font-semibold mb-3", "Velocity Distribution" }
                    div { class: "grid grid-cols-4 gap-2",
                        VelocityCard {
                            label: "Slow",
                            count: velocity_counts[0],
                            color: "text-info",
                            description: "< 0.5 videos/day"
                        }
                        VelocityCard {
                            label: "Moderate",
                            count: velocity_counts[1],
                            color: "text-success",
                            description: "0.5-1.0 videos/day"
                        }
                        VelocityCard {
                            label: "Fast",
                            count: velocity_counts[2],
                            color: "text-warning",
                            description: "1.0-2.0 videos/day"
                        }
                        VelocityCard {
                            label: "Intensive",
                            count: velocity_counts[3],
                            color: "text-error",
                            description: "> 2.0 videos/day"
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct VelocityCardProps {
    label: String,
    count: usize,
    color: String,
    description: String,
}

#[component]
fn VelocityCard(props: VelocityCardProps) -> Element {
    rsx! {
        div { class: "text-center p-3 bg-base-200 rounded-lg",
            div { class: "text-2xl font-bold {props.color}", "{props.count}" }
            div { class: "text-xs font-medium", "{props.label}" }
            div { class: "text-xs text-base-content/50 mt-1", "{props.description}" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct CognitiveLoadDistributionProps {
    plan_analyses: Vec<PlanAnalysis>,
}

#[component]
fn CognitiveLoadDistribution(props: CognitiveLoadDistributionProps) -> Element {
    let plan_analyses = &props.plan_analyses;

    if plan_analyses.is_empty() {
        return rsx! {
            div { class: "card bg-base-100 shadow-sm border border-base-300",
                div { class: "card-body text-center py-8",
                    div { class: "text-4xl mb-2", "⚖️" }
                    h3 { class: "card-title text-lg justify-center", "Cognitive Load Distribution" }
                    p { class: "text-base-content/60", "No cognitive load data available" }
                    p { class: "text-sm text-base-content/50", "Study plans will show cognitive load balance" }
                }
            }
        };
    }

    // Calculate aggregate cognitive load metrics
    let avg_load: f32 = plan_analyses
        .iter()
        .map(|analysis| analysis.load_distribution.average_load)
        .sum::<f32>()
        / plan_analyses.len() as f32;

    let total_overloaded: usize = plan_analyses
        .iter()
        .map(|analysis| analysis.load_distribution.overloaded_sessions)
        .sum();

    let total_underloaded: usize = plan_analyses
        .iter()
        .map(|analysis| analysis.load_distribution.underloaded_sessions)
        .sum();

    let total_sessions: usize = plan_analyses
        .iter()
        .map(|analysis| {
            analysis.load_distribution.overloaded_sessions
                + analysis.load_distribution.underloaded_sessions
        })
        .sum();

    let avg_variance: f32 = plan_analyses
        .iter()
        .map(|analysis| analysis.load_distribution.load_variance)
        .sum::<f32>()
        / plan_analyses.len() as f32;

    // Categorize load balance quality
    let balance_quality = match avg_variance {
        v if v < 0.1 => ("Excellent", "text-success", "Very well balanced"),
        v if v < 0.2 => ("Good", "text-info", "Well balanced"),
        v if v < 0.4 => ("Fair", "text-warning", "Moderately balanced"),
        _ => ("Poor", "text-error", "Needs rebalancing"),
    };

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "⚖️" }
                    "Cognitive Load Distribution"
                }

                div { class: "grid grid-cols-2 gap-4 mt-4",
                    div { class: "stat",
                        div { class: "stat-title", "Average Load" }
                        div { class: "stat-value text-primary", "{avg_load:.2}" }
                        div { class: "stat-desc", "Across all sessions" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Load Variance" }
                        div { class: "stat-value {balance_quality.1}", "{avg_variance:.3}" }
                        div { class: "stat-desc", "{balance_quality.2}" }
                    }
                }

                // Load distribution visualization
                div { class: "mt-6",
                    h4 { class: "font-semibold mb-3", "Session Load Balance" }
                    div { class: "grid grid-cols-3 gap-4",
                        LoadBalanceCard {
                            label: "Overloaded",
                            count: total_overloaded,
                            color: "text-error",
                            description: "High cognitive load"
                        }
                        LoadBalanceCard {
                            label: "Balanced",
                            count: total_sessions.saturating_sub(total_overloaded + total_underloaded),
                            color: "text-success",
                            description: "Optimal load"
                        }
                        LoadBalanceCard {
                            label: "Underloaded",
                            count: total_underloaded,
                            color: "text-warning",
                            description: "Low cognitive load"
                        }
                    }
                }

                // Balance quality indicator
                div { class: "mt-4 p-3 bg-base-200 rounded-lg",
                    div { class: "flex items-center justify-between",
                        span { class: "font-medium", "Overall Balance Quality" }
                        span { class: "badge {balance_quality.1} badge-outline", "{balance_quality.0}" }
                    }
                    progress {
                        class: "progress progress-accent w-full mt-2",
                        value: ((1.0 - avg_variance.min(1.0)) * 100.0) as i32,
                        max: 100
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct LoadBalanceCardProps {
    label: String,
    count: usize,
    color: String,
    description: String,
}

#[component]
fn LoadBalanceCard(props: LoadBalanceCardProps) -> Element {
    rsx! {
        div { class: "text-center p-3 bg-base-200 rounded-lg",
            div { class: "text-2xl font-bold {props.color}", "{props.count}" }
            div { class: "text-xs font-medium", "{props.label}" }
            div { class: "text-xs text-base-content/50 mt-1", "{props.description}" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct DifficultyProgressionTrackerProps {
    plan_analyses: Vec<PlanAnalysis>,
    clustering_analytics: ClusteringAnalytics,
}

#[component]
fn DifficultyProgressionTracker(props: DifficultyProgressionTrackerProps) -> Element {
    let plan_analyses = &props.plan_analyses;
    let clustering_analytics = &props.clustering_analytics;

    if plan_analyses.is_empty() {
        return rsx! {
            div { class: "card bg-base-100 shadow-sm border border-base-300",
                div { class: "card-body text-center py-8",
                    div { class: "text-4xl mb-2", "📊" }
                    h3 { class: "card-title text-lg justify-center", "Difficulty Progression" }
                    p { class: "text-base-content/60", "No progression data available" }
                    p { class: "text-sm text-base-content/50", "Complete some sessions to track difficulty progression" }
                }
            }
        };
    }

    // Calculate temporal distribution metrics
    let avg_gap_days: f32 = plan_analyses
        .iter()
        .map(|analysis| analysis.temporal_distribution.average_gap_days)
        .sum::<f32>()
        / plan_analyses.len() as f32;

    let longest_gap: i64 = plan_analyses
        .iter()
        .map(|analysis| analysis.temporal_distribution.longest_gap_days)
        .max()
        .unwrap_or(0);

    let avg_consistency: f32 = plan_analyses
        .iter()
        .map(|analysis| analysis.temporal_distribution.consistency_score)
        .sum::<f32>()
        / plan_analyses.len() as f32;

    let avg_weekend_utilization: f32 = plan_analyses
        .iter()
        .map(|analysis| analysis.temporal_distribution.weekend_utilization)
        .sum::<f32>()
        / plan_analyses.len() as f32;

    // Generate recommendations based on analysis
    let mut recommendations = Vec::new();

    if avg_gap_days > 3.0 {
        recommendations
            .push("Consider increasing session frequency to maintain momentum".to_string());
    }

    if avg_consistency < 0.7 {
        recommendations.push("Try to maintain more consistent study intervals".to_string());
    }

    if avg_weekend_utilization < 0.3 {
        recommendations
            .push("Consider utilizing weekends for additional study sessions".to_string());
    }

    if clustering_analytics.average_quality_score < 0.6 {
        recommendations.push(
            "Review course structure quality - some courses may benefit from re-clustering"
                .to_string(),
        );
    }

    if recommendations.is_empty() {
        recommendations.push("Great job! Your study patterns are well optimized".to_string());
    }

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "📈" }
                    "Difficulty Progression & Recommendations"
                }

                div { class: "grid grid-cols-2 gap-4 mt-4",
                    div { class: "stat",
                        div { class: "stat-title", "Avg Session Gap" }
                        div { class: "stat-value text-primary", "{avg_gap_days:.1} days" }
                        div { class: "stat-desc", "Between study sessions" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Consistency Score" }
                        div { class: "stat-value text-secondary", "{(avg_consistency * 100.0):.0}%" }
                        div { class: "stat-desc", "Study rhythm stability" }
                    }
                }

                div { class: "grid grid-cols-2 gap-4 mt-2",
                    div { class: "stat",
                        div { class: "stat-title", "Longest Gap" }
                        div { class: "stat-value text-accent", "{longest_gap} days" }
                        div { class: "stat-desc", "Maximum break taken" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Weekend Usage" }
                        div { class: "stat-value text-neutral", "{(avg_weekend_utilization * 100.0):.0}%" }
                        div { class: "stat-desc", "Weekend study utilization" }
                    }
                }

                // Recommendations section
                div { class: "mt-6",
                    h4 { class: "font-semibold mb-3 flex items-center gap-2",
                        span { "💡" }
                        "AI Recommendations"
                    }
                    div { class: "space-y-2",
                        {recommendations.iter().enumerate().map(|(index, rec)| rsx! {
                            div {
                                key: "{index}",
                                class: "flex items-start gap-2 p-3 bg-gradient-to-r from-primary/5 to-secondary/5 rounded-lg border border-primary/10",
                                div { class: "text-primary font-bold text-sm", "{index + 1}." }
                                div { class: "text-sm text-base-content/80", "{rec}" }
                            }
                        })}
                    }
                }

                // Processing performance from clustering analytics
                div { class: "mt-6 p-3 bg-base-200 rounded-lg",
                    h4 { class: "font-semibold mb-2", "Processing Performance" }
                    div { class: "grid grid-cols-2 gap-4 text-sm",
                        div {
                            span { class: "text-base-content/70", "Avg Processing: " }
                            span { class: "font-medium", "{clustering_analytics.processing_time_stats.average_ms:.0}ms" }
                        }
                        div {
                            span { class: "text-base-content/70", "Quality Score: " }
                            span { class: "font-medium", "{clustering_analytics.average_quality_score:.2}" }
                        }
                    }
                }
            }
        }
    }
}
