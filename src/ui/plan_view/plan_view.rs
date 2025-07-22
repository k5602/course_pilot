use dioxus::prelude::*;
use dioxus_motion::prelude::*;
use uuid::Uuid;

use crate::ui::components::toast::toast;
use crate::ui::hooks::use_plan_resource;
use crate::types::{PlanExt, PlanSettings};
use super::{PlanHeader, PlanChecklist, SessionControlPanel};

#[derive(Props, PartialEq, Clone)]
pub struct PlanViewProps {
    pub course_id: Uuid,
}

/// Clean plan view component with proper separation of concerns
#[component]
pub fn PlanView(props: PlanViewProps) -> Element {
    let plan_resource = use_plan_resource(props.course_id);
    
    // Show loading toast only once when plan is None
    use_effect(use_reactive!(|plan_resource| {
        if plan_resource.read().is_none() {
            spawn(async move {
                toast::info("Loading study plan...");
            });
        }
    }));
    
    match &*plan_resource.read_unchecked() {
        None => render_loading_state(),
        Some(Err(err)) => render_error_state(err),
        Some(Ok(Some(plan))) => {
            let (completed_sections, total_sections, progress_percentage) = plan.calculate_progress();
            let progress = progress_percentage.round() as u8;
            
            rsx! {
                render_plan_content {
                    plan: plan.clone(),
                    progress: progress,
                    completed_sections: completed_sections,
                    total_sections: total_sections,
                }
            }
        },
        Some(Ok(None)) => render_no_plan_state(props.course_id),
    }
}

/// Render loading state
fn render_loading_state() -> Element {
    rsx! {
        section {
            class: "w-full max-w-3xl mx-auto px-4 py-8",
            h1 { class: "text-2xl font-bold mb-6", "Study Plan" }
            div { class: "h-4 w-1/2 bg-base-300 rounded mb-6 animate-pulse" }
            div { class: "h-3 w-full bg-base-300 rounded mb-6 animate-pulse" }
            div { class: "h-2 w-1/3 bg-base-300 rounded animate-pulse" }
        }
    }
}

/// Render error state
fn render_error_state(err: &anyhow::Error) -> Element {
    let error_msg = format!("Failed to load study plan: {err}");
    spawn(async move {
        toast::error(error_msg);
    });
    
    rsx! {
        section {
            class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
            div { class: "text-error", "Failed to load study plan." }
            button {
                class: "btn btn-outline btn-sm mt-4",
                onclick: move |_| {
                    toast::info("Please refresh the page to retry loading the plan");
                },
                "Retry"
            }
        }
    }
}

/// Render plan content with animation
#[component]
fn render_plan_content(
    plan: crate::types::Plan, 
    progress: u8, 
    completed_sections: usize, 
    total_sections: usize
) -> Element {
    let mut list_opacity = use_motion(0.0f32);
    let mut list_y = use_motion(-16.0f32);

    use_effect(move || {
        list_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        list_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let list_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px);",
            list_opacity.get_value(),
            list_y.get_value()
        )
    });

    let handle_settings_change = move |_new_settings: PlanSettings| {
        // TODO: Implement plan regeneration with new settings
        // This would typically involve calling a backend service to regenerate the plan
        // while preserving existing progress
        spawn(async move {
            toast::info("Plan regeneration with new settings is not yet implemented");
        });
    };

    rsx! {
        section {
            class: "w-full max-w-3xl mx-auto px-4 py-8",
            h1 { class: "text-2xl font-bold mb-6", "Study Plan" }
            
            PlanHeader {
                progress: progress,
                completed_sections: completed_sections,
                total_sections: total_sections,
            }
            
            SessionControlPanel {
                plan: plan.clone(),
                on_settings_change: handle_settings_change,
            }
            
            div {
                style: "{list_style}",
                PlanChecklist { plan: plan.clone() }
            }
        }
    }
}

/// Render no plan state
fn render_no_plan_state(_course_id: Uuid) -> Element {
    rsx! {
        section {
            class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
            h1 { class: "text-2xl font-bold mb-6", "Study Plan" }
            div { 
                class: "text-base-content/60 text-center", 
                "No study plan found for this course."
                br {}
                "Create a plan to start tracking your progress."
            }
            button {
                class: "btn btn-primary mt-4",
                onclick: move |_| {
                    spawn(async move {
                        toast::info("Plan creation not implemented yet");
                    });
                },
                "Create Study Plan"
            }
        }
    }
}