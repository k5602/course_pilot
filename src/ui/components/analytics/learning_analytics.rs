use dioxus::prelude::*;
use crate::storage::{get_clustering_analytics, ClusteringAnalytics, Database};
use crate::DatabaseError;
use std::sync::Arc;

#[component]
pub fn LearningAnalytics() -> Element {
    let db = use_context::<Arc<Database>>();
    
    let analytics_resource = use_resource(move || {
        let db = db.clone();
        async move {
            tokio::task::spawn_blocking(move || {
                get_clustering_analytics(&db)
            }).await.unwrap_or_else(|_| Err(DatabaseError::NotFound("Failed to load analytics".to_string())))
        }
    });

    match &*analytics_resource.read_unchecked() {
        Some(Ok(analytics)) => rsx! {
            div { class: "space-y-4",
                LearningVelocityChart { analytics: analytics.clone() }
                CognitiveLoadDistribution { analytics: analytics.clone() }
                DifficultyProgressionTracker { analytics: analytics.clone() }
            }
        },
        Some(Err(e)) => rsx! {
            div { class: "alert alert-error",
                "Failed to load analytics: {e:?}"
            }
        },
        None => rsx! {
            div { class: "space-y-4",
                div { class: "skeleton h-32 w-full" }
                div { class: "skeleton h-24 w-full" }
                div { class: "skeleton h-20 w-full" }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct LearningVelocityChartProps {
    analytics: ClusteringAnalytics,
}

#[component]
fn LearningVelocityChart(props: LearningVelocityChartProps) -> Element {
    let analytics = &props.analytics;
    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg", "Learning Velocity" }
                div { class: "grid grid-cols-2 gap-4 mt-4",
                    div { class: "stat",
                        div { class: "stat-title", "Total Courses" }
                        div { class: "stat-value text-primary", "{analytics.total_courses}" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Structured Courses" }
                        div { class: "stat-value text-secondary", "{analytics.clustered_courses}" }
                    }
                }
                div { class: "mt-4",
                    div { class: "text-sm text-base-content/70",
                        "Completion Rate: {((analytics.clustered_courses as f32 / analytics.total_courses.max(1) as f32) * 100.0):.1}%"
                    }
                    progress { 
                        class: "progress progress-primary w-full mt-2",
                        value: analytics.clustered_courses,
                        max: analytics.total_courses.max(1)
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct CognitiveLoadDistributionProps {
    analytics: ClusteringAnalytics,
}

#[component]
fn CognitiveLoadDistribution(props: CognitiveLoadDistributionProps) -> Element {
    let analytics = &props.analytics;
    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg", "Cognitive Load Distribution" }
                div { class: "grid grid-cols-4 gap-2 mt-4",
                    div { class: "text-center",
                        div { class: "text-2xl font-bold text-success", "{analytics.quality_distribution.excellent}" }
                        div { class: "text-xs text-base-content/70", "Excellent" }
                        div { class: "text-xs text-base-content/50", "0.8+" }
                    }
                    div { class: "text-center",
                        div { class: "text-2xl font-bold text-info", "{analytics.quality_distribution.good}" }
                        div { class: "text-xs text-base-content/70", "Good" }
                        div { class: "text-xs text-base-content/50", "0.6-0.8" }
                    }
                    div { class: "text-center",
                        div { class: "text-2xl font-bold text-warning", "{analytics.quality_distribution.fair}" }
                        div { class: "text-xs text-base-content/70", "Fair" }
                        div { class: "text-xs text-base-content/50", "0.4-0.6" }
                    }
                    div { class: "text-center",
                        div { class: "text-2xl font-bold text-error", "{analytics.quality_distribution.poor}" }
                        div { class: "text-xs text-base-content/70", "Poor" }
                        div { class: "text-xs text-base-content/50", "<0.4" }
                    }
                }
                div { class: "mt-4",
                    div { class: "text-sm text-base-content/70",
                        "Average Quality Score: {analytics.average_quality_score:.2}"
                    }
                    progress { 
                        class: "progress progress-accent w-full mt-2",
                        value: (analytics.average_quality_score * 100.0) as i32,
                        max: 100
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct DifficultyProgressionTrackerProps {
    analytics: ClusteringAnalytics,
}

#[component]
fn DifficultyProgressionTracker(props: DifficultyProgressionTrackerProps) -> Element {
    let analytics = &props.analytics;
    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg", "Processing Performance" }
                div { class: "grid grid-cols-2 gap-4 mt-4",
                    div { class: "stat",
                        div { class: "stat-title", "Avg Processing Time" }
                        div { class: "stat-value text-accent", "{analytics.processing_time_stats.average_ms:.0}ms" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Median Time" }
                        div { class: "stat-value text-neutral", "{analytics.processing_time_stats.median_ms:.0}ms" }
                    }
                }
                div { class: "mt-4 text-sm text-base-content/70",
                    "Range: {analytics.processing_time_stats.min_ms}ms - {analytics.processing_time_stats.max_ms}ms"
                }
            }
        }
    }
}