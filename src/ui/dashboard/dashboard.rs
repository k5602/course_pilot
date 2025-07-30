use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::ui::components::base::BaseCard;
use crate::ui::components::{
    AIRecommendationsPanel, LastAccessedCourse, LearningAnalytics, PomodoroTimer, TodaysSessions,
};

/// Analytics-focused dashboard component
#[component]
pub fn Dashboard() -> Element {
    // Animation for dashboard entrance
    let mut dashboard_opacity = use_motion(0.0f32);
    let mut dashboard_y = use_motion(-24.0f32);

    use_effect(move || {
        dashboard_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        dashboard_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let dashboard_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px);",
            dashboard_opacity.get_value(),
            dashboard_y.get_value()
        )
    });

    rsx! {
        section {
            class: "w-full max-w-7xl mx-auto px-4 py-8",
            style: "{dashboard_style}",

            // Header
            div {
                class: "flex items-center justify-between mb-8",
                h1 { class: "text-3xl font-bold", "Learning Dashboard" }
                div { class: "text-sm text-base-content/70",
                    "Track your progress and optimize your learning"
                }
            }

            // Analytics Grid Layout
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Learning Analytics Section (spans 2 columns on large screens)
                BaseCard {
                    title: "Learning Analytics",
                    class: "lg:col-span-2",
                    children: rsx! {
                        LearningAnalytics {}
                    }
                }

                BaseCard {
                    title: "AI Recommendations",
                    children: rsx! {
                        AIRecommendationsPanel {}
                    }
                }

                // Today's Sessions
                BaseCard {
                    title: "Today's Sessions",
                    children: rsx! {
                        TodaysSessions {}
                    }
                }

                // Continue Learning
                BaseCard {
                    title: "Continue Learning",
                    children: rsx! {
                        LastAccessedCourse {}
                    }
                }

                // Focus Timer
                BaseCard {
                    title: "Focus Timer",
                    children: rsx! {
                        PomodoroTimer {}
                    }
                }
            }
        }
    }
}
