use dioxus::prelude::*;
use crate::types::{AdvancedSchedulerSettings, DifficultyLevel};
use crate::ui::hooks::use_analytics_manager;
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
    
    let recommendations_resource = use_resource(move || {
        let analytics_manager = analytics_manager.clone();
        let course_id = props.course_id;
        let user_experience = props.user_experience;
        
        async move {
            if let Some(course_id) = course_id {
                analytics_manager.get_recommended_advanced_settings(course_id, user_experience).await
            } else {
                // Return default recommendations when no course is selected
                Ok(AdvancedSchedulerSettings::default())
            }
        }
    });

    match &*recommendations_resource.read_unchecked() {
        Some(Ok(settings)) => rsx! {
            div { class: "space-y-4",
                RecommendationCard {
                    title: "Recommended Strategy",
                    content: settings.strategy.display_name(),
                    description: settings.strategy.description(),
                    icon: "🎯"
                }
                
                if settings.spaced_repetition_enabled {
                    RecommendationCard {
                        title: "Spaced Repetition",
                        content: "Enabled",
                        description: "Optimizes long-term retention with review sessions",
                        icon: "🔄"
                    }
                }
                
                if settings.cognitive_load_balancing {
                    RecommendationCard {
                        title: "Cognitive Load Balancing",
                        content: "Active",
                        description: "Balances difficulty across sessions",
                        icon: "⚖️"
                    }
                }
                
                if settings.adaptive_pacing {
                    RecommendationCard {
                        title: "Adaptive Pacing",
                        content: "Enabled",
                        description: "Adjusts session pacing based on progress",
                        icon: "📈"
                    }
                }
            }
        },
        Some(Err(e)) => rsx! {
            div { class: "alert alert-warning",
                "Unable to load AI recommendations: {e}"
            }
        },
        None => rsx! {
            div { class: "space-y-3",
                div { class: "skeleton h-16 w-full" }
                div { class: "skeleton h-16 w-full" }
                div { class: "skeleton h-16 w-full" }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct RecommendationCardProps {
    title: String,
    content: String,
    description: String,
    icon: String,
}

#[component]
fn RecommendationCard(props: RecommendationCardProps) -> Element {
    rsx! {
        div { class: "card bg-gradient-to-r from-primary/10 to-secondary/10 border border-primary/20",
            div { class: "card-body p-4",
                div { class: "flex items-start gap-3",
                    div { class: "text-2xl", "{props.icon}" }
                    div { class: "flex-1",
                        h4 { class: "font-semibold text-sm", "{props.title}" }
                        p { class: "text-primary font-medium", "{props.content}" }
                        p { class: "text-xs text-base-content/70 mt-1", "{props.description}" }
                    }
                }
            }
        }
    }
}