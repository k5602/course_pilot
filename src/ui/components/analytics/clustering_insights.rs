use crate::storage::{
    ClusteringAnalytics, Database, get_clustering_analytics, get_courses_by_clustering_quality,
};
use crate::types::{ClusteringAlgorithm, ClusteringStrategy, Course, TopicInfo};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[component]
pub fn ClusteringInsights() -> Element {
    let db = use_context::<Arc<Database>>();
    let db_for_analytics = db.clone();
    let db_for_courses = db.clone();

    let clustering_analytics_resource = use_resource(move || {
        let db_clone = db_for_analytics.clone();
        async move {
            tokio::task::spawn_blocking(move || get_clustering_analytics(&db_clone))
                .await
                .unwrap_or_else(|_| {
                    Err(anyhow::anyhow!("Failed to load clustering analytics"))
                })
        }
    });

    let high_quality_courses_resource = use_resource(move || {
        let db_clone = db_for_courses.clone();
        async move {
            tokio::task::spawn_blocking(move || get_courses_by_clustering_quality(&db_clone, 0.8))
                .await
                .unwrap_or_else(|_| {
                    Err(anyhow::anyhow!("Failed to load high quality courses"))
                })
        }
    });

    match (
        &*clustering_analytics_resource.read_unchecked(),
        &*high_quality_courses_resource.read_unchecked(),
    ) {
        (Some(Ok(analytics)), Some(Ok(high_quality_courses))) => rsx! {
            div { class: "space-y-6",
                // Clustering quality overview
                ClusteringQualityOverview { analytics: analytics.clone() }

                // Algorithm performance comparison
                AlgorithmPerformanceComparison { analytics: analytics.clone() }

                // Interactive similarity matrix (simplified visualization)
                SimilarityMatrixVisualization { high_quality_courses: high_quality_courses.clone() }

                // Topic analysis and keyword clouds
                TopicAnalysisVisualization { high_quality_courses: high_quality_courses.clone() }

                // Performance metrics
                ClusteringPerformanceMetrics { analytics: analytics.clone() }
            }
        },
        (Some(Err(e)), _) | (_, Some(Err(e))) => rsx! {
            div { class: "alert alert-error",
                "Failed to load clustering insights: {e:?}"
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
struct ClusteringQualityOverviewProps {
    analytics: ClusteringAnalytics,
}

#[component]
fn ClusteringQualityOverview(props: ClusteringQualityOverviewProps) -> Element {
    let analytics = &props.analytics;

    // Calculate overall clustering health
    let clustering_health = if analytics.total_courses == 0 {
        ("No Data", "text-base-content/50", 0.0)
    } else {
        let structured_percentage =
            (analytics.clustered_courses as f32 / analytics.total_courses as f32) * 100.0;
        let quality_score = analytics.average_quality_score;

        match (structured_percentage, quality_score) {
            (p, q) if p >= 80.0 && q >= 0.8 => ("Excellent", "text-success", q),
            (p, q) if p >= 60.0 && q >= 0.6 => ("Good", "text-info", q),
            (p, q) if p >= 40.0 && q >= 0.4 => ("Fair", "text-warning", q),
            (_, q) => ("Needs Improvement", "text-error", q),
        }
    };

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "🎯" }
                    "Clustering Quality Overview"
                }

                div { class: "grid grid-cols-3 gap-4 mt-4",
                    div { class: "stat",
                        div { class: "stat-title", "Overall Health" }
                        div { class: "stat-value {clustering_health.1}", "{clustering_health.0}" }
                        div { class: "stat-desc", "Quality: {clustering_health.2:.2}" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Structured Courses" }
                        div { class: "stat-value text-primary", "{analytics.clustered_courses}" }
                        div { class: "stat-desc", "of {analytics.total_courses} total" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Avg Quality Score" }
                        div { class: "stat-value text-secondary", "{analytics.average_quality_score:.2}" }
                        div { class: "stat-desc", "0.0 - 1.0 scale" }
                    }
                }

                // Quality distribution visualization
                div { class: "mt-6",
                    h4 { class: "font-semibold mb-3", "Quality Score Distribution" }
                    div { class: "grid grid-cols-4 gap-2",
                        QualityDistributionCard {
                            label: "Excellent",
                            count: analytics.quality_distribution.excellent,
                            color: "text-success",
                            range: "0.8 - 1.0",
                            percentage: if analytics.clustered_courses > 0 {
                                (analytics.quality_distribution.excellent as f32 / analytics.clustered_courses as f32) * 100.0
                            } else { 0.0 }
                        }
                        QualityDistributionCard {
                            label: "Good",
                            count: analytics.quality_distribution.good,
                            color: "text-info",
                            range: "0.6 - 0.8",
                            percentage: if analytics.clustered_courses > 0 {
                                (analytics.quality_distribution.good as f32 / analytics.clustered_courses as f32) * 100.0
                            } else { 0.0 }
                        }
                        QualityDistributionCard {
                            label: "Fair",
                            count: analytics.quality_distribution.fair,
                            color: "text-warning",
                            range: "0.4 - 0.6",
                            percentage: if analytics.clustered_courses > 0 {
                                (analytics.quality_distribution.fair as f32 / analytics.clustered_courses as f32) * 100.0
                            } else { 0.0 }
                        }
                        QualityDistributionCard {
                            label: "Poor",
                            count: analytics.quality_distribution.poor,
                            color: "text-error",
                            range: "< 0.4",
                            percentage: if analytics.clustered_courses > 0 {
                                (analytics.quality_distribution.poor as f32 / analytics.clustered_courses as f32) * 100.0
                            } else { 0.0 }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct AlgorithmPerformanceComparisonProps {
    analytics: ClusteringAnalytics,
}

#[component]
fn AlgorithmPerformanceComparison(props: AlgorithmPerformanceComparisonProps) -> Element {
    let analytics = &props.analytics;

    // Convert algorithm distribution to sorted vector for display
    let mut algorithm_stats: Vec<(ClusteringAlgorithm, usize)> = analytics
        .algorithm_distribution
        .iter()
        .map(|(alg, count)| (alg.clone(), *count))
        .collect();
    algorithm_stats.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by usage count descending

    // Convert strategy distribution to sorted vector
    let mut strategy_stats: Vec<(ClusteringStrategy, usize)> = analytics
        .strategy_distribution
        .iter()
        .map(|(strat, count)| (strat.clone(), *count))
        .collect();
    strategy_stats.sort_by(|a, b| b.1.cmp(&a.1));

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "⚙️" }
                    "Algorithm Performance Comparison"
                }

                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6 mt-4",
                    // Algorithm usage
                    div {
                        h4 { class: "font-semibold mb-3", "Algorithm Usage" }
                        if algorithm_stats.is_empty() {
                            div { class: "text-center py-4 text-base-content/60",
                                "No algorithm data available"
                            }
                        } else {
                            div { class: "space-y-2",
                                {algorithm_stats.iter().map(|(algorithm, count)| {
                                    let percentage = (*count as f32 / analytics.clustered_courses as f32) * 100.0;
                                    rsx! {
                                        AlgorithmUsageBar {
                                            key: "{algorithm:?}",
                                            algorithm: algorithm.clone(),
                                            count: *count,
                                            percentage
                                        }
                                    }
                                })}
                            }
                        }
                    }

                    // Strategy selection
                    div {
                        h4 { class: "font-semibold mb-3", "Strategy Selection" }
                        if strategy_stats.is_empty() {
                            div { class: "text-center py-4 text-base-content/60",
                                "No strategy data available"
                            }
                        } else {
                            div { class: "space-y-2",
                                {strategy_stats.iter().map(|(strategy, count)| {
                                    let percentage = (*count as f32 / analytics.clustered_courses as f32) * 100.0;
                                    rsx! {
                                        StrategyUsageBar {
                                            key: "{strategy:?}",
                                            strategy: strategy.clone(),
                                            count: *count,
                                            percentage
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }

                // Algorithm selection rationale
                div { class: "mt-6 p-4 bg-base-200 rounded-lg",
                    h4 { class: "font-semibold mb-2", "Algorithm Selection Reasoning" }
                    div { class: "text-sm text-base-content/80 space-y-1",
                        p { "• TF-IDF: Best for content-based similarity analysis" }
                        p { "• K-Means: Effective for balanced cluster sizes" }
                        p { "• Hierarchical: Preserves natural content structure" }
                        p { "• LDA: Discovers hidden topic patterns" }
                        p { "• Hybrid: Combines multiple approaches for robustness" }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SimilarityMatrixVisualizationProps {
    high_quality_courses: Vec<Course>,
}

#[component]
fn SimilarityMatrixVisualization(props: SimilarityMatrixVisualizationProps) -> Element {
    let courses = &props.high_quality_courses;

    if courses.is_empty() {
        return rsx! {
            div { class: "card bg-base-100 shadow-sm border border-base-300",
                div { class: "card-body text-center py-8",
                    div { class: "text-4xl mb-2", "🔗" }
                    h3 { class: "card-title text-lg justify-center", "Similarity Matrix" }
                    p { class: "text-base-content/60", "No high-quality courses available" }
                    p { class: "text-sm text-base-content/50", "Structure some courses to see similarity analysis" }
                }
            }
        };
    }

    // Take first 6 courses for matrix visualization (to keep it manageable)
    let display_courses: Vec<Course> = courses.iter().take(6).cloned().collect();

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "🔗" }
                    "Interactive Similarity Matrix"
                }

                if display_courses.len() < 2 {
                    div { class: "text-center py-6 text-base-content/60",
                        p { "Need at least 2 structured courses for similarity analysis" }
                    }
                } else {
                    div { class: "mt-4",
                        // Course labels
                        div { class: "mb-4",
                            h4 { class: "font-semibold mb-2", "Courses in Analysis:" }
                            div { class: "flex flex-wrap gap-2",
                                {display_courses.iter().enumerate().map(|(index, course)| rsx! {
                                    span {
                                        key: "{course.id}",
                                        class: "badge badge-outline badge-sm",
                                        "{index + 1}. {course.name}"
                                    }
                                })}
                            }
                        }

                        // Simplified similarity grid
                        SimilarityGrid { courses: display_courses.clone() }

                        // Legend
                        div { class: "mt-4 flex items-center gap-4 text-xs",
                            span { class: "flex items-center gap-1",
                                div { class: "w-3 h-3 bg-success rounded" }
                                "High (0.8+)"
                            }
                            span { class: "flex items-center gap-1",
                                div { class: "w-3 h-3 bg-info rounded" }
                                "Medium (0.6-0.8)"
                            }
                            span { class: "flex items-center gap-1",
                                div { class: "w-3 h-3 bg-warning rounded" }
                                "Low (0.4-0.6)"
                            }
                            span { class: "flex items-center gap-1",
                                div { class: "w-3 h-3 bg-error rounded" }
                                "Very Low (<0.4)"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct TopicAnalysisVisualizationProps {
    high_quality_courses: Vec<Course>,
}

#[component]
fn TopicAnalysisVisualization(props: TopicAnalysisVisualizationProps) -> Element {
    let courses = &props.high_quality_courses;

    // Extract all topics from courses
    let mut all_topics: Vec<TopicInfo> = Vec::new();
    for course in courses {
        if let Some(structure) = &course.structure {
            if let Some(clustering_metadata) = &structure.clustering_metadata {
                all_topics.extend(clustering_metadata.content_topics.clone());
            }
        }
    }

    // Aggregate topics by keyword
    let mut topic_aggregation: HashMap<String, (f32, usize)> = HashMap::new();
    for topic in &all_topics {
        let entry = topic_aggregation
            .entry(topic.keyword.clone())
            .or_insert((0.0, 0));
        entry.0 += topic.relevance_score;
        entry.1 += topic.video_count;
    }

    // Convert to sorted vector
    let mut aggregated_topics: Vec<(String, f32, usize)> = topic_aggregation
        .into_iter()
        .map(|(keyword, (total_relevance, total_videos))| (keyword, total_relevance, total_videos))
        .collect();
    aggregated_topics.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top 20 topics
    aggregated_topics.truncate(20);

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "☁️" }
                    "Topic Analysis & Content Keywords"
                }

                if aggregated_topics.is_empty() {
                    div { class: "text-center py-8 text-base-content/60",
                        div { class: "text-4xl mb-2", "📝" }
                        p { "No topic data available" }
                        p { class: "text-sm text-base-content/50", "Topics will appear after course clustering" }
                    }
                } else {
                    div { class: "mt-4",
                        // Topic cloud visualization
                        div { class: "mb-6",
                            h4 { class: "font-semibold mb-3", "Topic Keyword Cloud" }
                            div { class: "flex flex-wrap gap-2",
                                {aggregated_topics.iter().take(15).map(|(keyword, relevance, video_count)| {
                                    let size_class = match relevance {
                                        r if *r >= 3.0 => "text-lg",
                                        r if *r >= 2.0 => "text-base",
                                        _ => "text-sm",
                                    };
                                    let color_class = match relevance {
                                        r if *r >= 3.0 => "badge-primary",
                                        r if *r >= 2.0 => "badge-secondary",
                                        _ => "badge-accent",
                                    };

                                    rsx! {
                                        span {
                                            key: "{keyword}",
                                            class: "badge {color_class} {size_class}",
                                            title: "Relevance: {relevance:.1}, Videos: {video_count}",
                                            "{keyword}"
                                        }
                                    }
                                })}
                            }
                        }

                        // Top topics table
                        div {
                            h4 { class: "font-semibold mb-3", "Top Content Topics" }
                            div { class: "overflow-x-auto",
                                table { class: "table table-sm",
                                    thead {
                                        tr {
                                            th { "Keyword" }
                                            th { "Relevance Score" }
                                            th { "Video Count" }
                                            th { "Strength" }
                                        }
                                    }
                                    tbody {
                                        {aggregated_topics.iter().take(10).map(|(keyword, relevance, video_count)| {
                                            let strength = match relevance {
                                                r if *r >= 3.0 => ("High", "text-success"),
                                                r if *r >= 2.0 => ("Medium", "text-info"),
                                                _ => ("Low", "text-warning"),
                                            };

                                            rsx! {
                                                tr { key: "{keyword}",
                                                    td { class: "font-medium", "{keyword}" }
                                                    td { "{relevance:.2}" }
                                                    td { "{video_count}" }
                                                    td {
                                                        span { class: "badge badge-outline badge-xs {strength.1}", "{strength.0}" }
                                                    }
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct ClusteringPerformanceMetricsProps {
    analytics: ClusteringAnalytics,
}

#[component]
fn ClusteringPerformanceMetrics(props: ClusteringPerformanceMetricsProps) -> Element {
    let analytics = &props.analytics;
    let stats = &analytics.processing_time_stats;

    // Performance assessment
    let performance_assessment = match stats.average_ms {
        avg if avg < 1000.0 => ("Excellent", "text-success", "< 1s average"),
        avg if avg < 5000.0 => ("Good", "text-info", "< 5s average"),
        avg if avg < 15000.0 => ("Fair", "text-warning", "< 15s average"),
        _ => ("Slow", "text-error", "> 15s average"),
    };

    rsx! {
        div { class: "card bg-base-100 shadow-sm border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    span { "⚡" }
                    "Clustering Performance Metrics"
                }

                div { class: "grid grid-cols-2 lg:grid-cols-4 gap-4 mt-4",
                    div { class: "stat",
                        div { class: "stat-title", "Performance" }
                        div { class: "stat-value {performance_assessment.1}", "{performance_assessment.0}" }
                        div { class: "stat-desc", "{performance_assessment.2}" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Average Time" }
                        div { class: "stat-value text-primary", "{stats.average_ms:.0}ms" }
                        div { class: "stat-desc", "Per clustering operation" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Median Time" }
                        div { class: "stat-value text-secondary", "{stats.median_ms:.0}ms" }
                        div { class: "stat-desc", "Typical processing time" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Time Range" }
                        div { class: "stat-value text-accent text-sm", "{stats.min_ms}-{stats.max_ms}ms" }
                        div { class: "stat-desc", "Min - Max processing" }
                    }
                }

                // Performance insights
                div { class: "mt-6 p-4 bg-base-200 rounded-lg",
                    h4 { class: "font-semibold mb-2", "Performance Insights" }
                    div { class: "text-sm text-base-content/80 space-y-1",
                        if stats.average_ms < 1000.0 {
                            p { "✅ Clustering performance is excellent - operations complete quickly" }
                        } else if stats.average_ms < 5000.0 {
                            p { "✅ Good clustering performance - reasonable processing times" }
                        } else {
                            p { "⚠️ Clustering may be slow for large courses - consider optimization" }
                        }

                        if (stats.max_ms as f64 - stats.min_ms as f64) > stats.average_ms * 2.0 {
                            p { "📊 High variance in processing times - performance depends on course complexity" }
                        } else {
                            p { "📊 Consistent processing times across different course sizes" }
                        }

                        p { "💡 Processing time scales with course size and content complexity" }
                    }
                }
            }
        }
    }
}

// Helper components
#[derive(Props, PartialEq, Clone)]
struct QualityDistributionCardProps {
    label: String,
    count: usize,
    color: String,
    range: String,
    percentage: f32,
}

#[component]
fn QualityDistributionCard(props: QualityDistributionCardProps) -> Element {
    rsx! {
        div { class: "text-center p-3 bg-base-200 rounded-lg",
            div { class: "text-2xl font-bold {props.color}", "{props.count}" }
            div { class: "text-xs font-medium", "{props.label}" }
            div { class: "text-xs text-base-content/50 mt-1", "{props.range}" }
            div { class: "text-xs text-base-content/60 mt-1", "{props.percentage:.1}%" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct AlgorithmUsageBarProps {
    algorithm: ClusteringAlgorithm,
    count: usize,
    percentage: f32,
}

#[component]
fn AlgorithmUsageBar(props: AlgorithmUsageBarProps) -> Element {
    let algorithm_name = match props.algorithm {
        ClusteringAlgorithm::TfIdf => "TF-IDF",
        ClusteringAlgorithm::KMeans => "K-Means",
        ClusteringAlgorithm::Hierarchical => "Hierarchical",
        ClusteringAlgorithm::Lda => "LDA",
        ClusteringAlgorithm::Hybrid => "Hybrid",
        ClusteringAlgorithm::Fallback => "Fallback",
    };

    rsx! {
        div { class: "flex items-center gap-3",
            div { class: "w-20 text-sm font-medium", "{algorithm_name}" }
            div { class: "flex-1 bg-base-300 rounded-full h-2",
                div {
                    class: "bg-primary h-2 rounded-full transition-all duration-300",
                    style: "width: {props.percentage}%"
                }
            }
            div { class: "text-sm text-base-content/70", "{props.count} ({props.percentage:.1}%)" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct StrategyUsageBarProps {
    strategy: ClusteringStrategy,
    count: usize,
    percentage: f32,
}

#[component]
fn StrategyUsageBar(props: StrategyUsageBarProps) -> Element {
    let strategy_name = match props.strategy {
        ClusteringStrategy::ContentBased => "Content-Based",
        ClusteringStrategy::DurationBased => "Duration-Based",
        ClusteringStrategy::Hierarchical => "Hierarchical",
        ClusteringStrategy::Lda => "LDA",
        ClusteringStrategy::Hybrid => "Hybrid",
        ClusteringStrategy::Fallback => "Fallback",
    };

    rsx! {
        div { class: "flex items-center gap-3",
            div { class: "w-24 text-sm font-medium", "{strategy_name}" }
            div { class: "flex-1 bg-base-300 rounded-full h-2",
                div {
                    class: "bg-secondary h-2 rounded-full transition-all duration-300",
                    style: "width: {props.percentage}%"
                }
            }
            div { class: "text-sm text-base-content/70", "{props.count} ({props.percentage:.1}%)" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SimilarityGridProps {
    courses: Vec<Course>,
}

#[component]
fn SimilarityGrid(props: SimilarityGridProps) -> Element {
    let courses = &props.courses;
    let course_count = courses.len();

    // Pre-calculate all similarity values and elements
    let grid_data: Vec<(usize, usize, f32, String)> = (0..course_count)
        .flat_map(|i| {
            (0..course_count).map(move |j| {
                let similarity = calculate_course_similarity(&courses[i], &courses[j]);
                let color_class = match similarity {
                    s if s >= 0.8 => "bg-success text-success-content".to_string(),
                    s if s >= 0.6 => "bg-info text-info-content".to_string(),
                    s if s >= 0.4 => "bg-warning text-warning-content".to_string(),
                    _ => "bg-error text-error-content".to_string(),
                };
                (i, j, similarity, color_class)
            })
        })
        .collect();

    rsx! {
        div { class: "grid gap-2",
            style: "grid-template-columns: repeat({course_count}, 1fr);",
            {grid_data.iter().map(|(i, j, similarity, color_class)| rsx! {
                div {
                    key: "{i}-{j}",
                    class: "aspect-square flex items-center justify-center text-xs font-bold rounded {color_class}",
                    title: "Similarity between course {i+1} and {j+1}: {similarity:.2}",
                    "{similarity:.2}"
                }
            })}
        }
    }
}

// Helper function to calculate course similarity (simplified)
fn calculate_course_similarity(course1: &Course, course2: &Course) -> f32 {
    if course1.id == course2.id {
        return 1.0;
    }

    // Simple similarity based on course name and video count
    let name_similarity =
        crate::nlp::text_similarity(&course1.name.to_lowercase(), &course2.name.to_lowercase());
    let video_count_similarity = {
        let count1 = course1.video_count() as f32;
        let count2 = course2.video_count() as f32;
        if count1 == 0.0 && count2 == 0.0 {
            1.0
        } else {
            1.0 - ((count1 - count2).abs() / (count1 + count2).max(1.0))
        }
    };

    // Weighted average
    (name_similarity * 0.7 + video_count_similarity * 0.3)
        .max(0.0)
        .min(1.0)
}
