//! Analytics components for the dashboard.

use dioxus::prelude::*;

use crate::domain::entities::AppAnalytics;

/// Component showing an overview of learning analytics.
#[component]
pub fn AnalyticsOverview(analytics: AppAnalytics) -> Element {
    let completion = analytics.completion_percent();
    let summary_coverage = analytics.summary_coverage_percent();

    rsx! {
        div {
            class: "space-y-6",

            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                StatCard {
                    title: "Courses",
                    value: analytics.total_courses().to_string(),
                    subtitle: None,
                }
                StatCard {
                    title: "Modules",
                    value: analytics.total_modules().to_string(),
                    subtitle: None,
                }
                StatCard {
                    title: "Videos",
                    value: analytics.total_videos().to_string(),
                    subtitle: None,
                }
                StatCard {
                    title: "Completed",
                    value: analytics.completed_videos().to_string(),
                    subtitle: Some(format!("{:.0}%", completion)),
                }
            }

            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                StatCard {
                    title: "Total Study Time",
                    value: format!("{} min", analytics.total_duration_minutes()),
                    subtitle: None,
                }
                StatCard {
                    title: "Completed Time",
                    value: format!("{} min", analytics.completed_duration_minutes()),
                    subtitle: Some(format!("{:.0}%", completion)),
                }
            }

            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                StatCard {
                    title: "Summaries",
                    value: analytics.videos_with_summary().to_string(),
                    subtitle: Some(format!("{:.0}%", summary_coverage)),
                }
                StatCard {
                    title: "Summary Coverage",
                    value: format!("{:.0}%", summary_coverage),
                    subtitle: None,
                }
            }
        }
    }
}

#[component]
fn StatCard(title: String, value: String, subtitle: Option<String>) -> Element {
    rsx! {
        div {
            class: "card bg-base-200 border border-base-300",
            div {
                class: "card-body p-4",
                p { class: "text-sm text-base-content/70", "{title}" }
                div {
                    class: "flex items-end justify-between gap-2",
                    h3 { class: "text-2xl font-semibold", "{value}" }
                    if let Some(sub) = subtitle {
                        span { class: "text-xs text-base-content/60", "{sub}" }
                    }
                }
            }
        }
    }
}
