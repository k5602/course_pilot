use dioxus::prelude::*;
use dioxus_motion::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

use super::{PlanHeader, SessionControlPanel, SessionList, group_items_by_session};
use crate::types::{PlanExt, PlanSettings};
use crate::ui::toast_helpers;
use crate::ui::use_plan_resource;

#[derive(Props, PartialEq, Clone)]
pub struct PlanViewProps {
    pub course_id: Uuid,
}

/// Enhanced plan view component with unified functionality
#[component]
pub fn PlanView(props: PlanViewProps) -> Element {
    let plan_resource = use_plan_resource(props.course_id);
    let expanded_sessions = use_signal(HashSet::new);

    // Show loading toast only once when plan is None
    use_effect(use_reactive!(|plan_resource| {
        if plan_resource.read().is_none() {
            spawn(async move {
                toast_helpers::info("Loading study plan...");
            });
        }
    }));

    match &*plan_resource.read_unchecked() {
        None => render_loading_state(),
        Some(Err(err)) => render_error_state(err),
        Some(Ok(Some(plan))) => {
            let (completed_sections, total_sections, progress_percentage) =
                plan.calculate_progress();
            let progress = progress_percentage.round() as u8;

            rsx! {
                render_enhanced_plan_content {
                    plan: plan.clone(),
                    progress: progress,
                    completed_sections: completed_sections,
                    total_sections: total_sections,
                    expanded_sessions: expanded_sessions,
                    course_id: props.course_id,
                }
            }
        }
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
        toast_helpers::error(error_msg);
    });

    rsx! {
        section {
            class: "w-full max-w-3xl mx-auto px-4 py-8 flex flex-col items-center justify-center",
            div { class: "text-error", "Failed to load study plan." }
            button {
                class: "btn btn-outline btn-sm mt-4",
                onclick: move |_| {
                    toast_helpers::info("Please refresh the page to retry loading the plan");
                },
                "Retry"
            }
        }
    }
}

/// Render enhanced plan content with unified functionality
#[component]
fn render_enhanced_plan_content(
    plan: crate::types::Plan,
    progress: u8,
    completed_sections: usize,
    total_sections: usize,
    expanded_sessions: Signal<HashSet<usize>>,
    course_id: Uuid,
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

    let handle_settings_change = {
        let backend = crate::ui::hooks::use_backend_adapter();
        let plan_id = plan.id;

        move |new_settings: PlanSettings| {
            let backend = backend.clone();

            spawn(async move {
                toast_helpers::info("Regenerating plan with new settings...");

                match backend.regenerate_plan(plan_id, new_settings).await {
                    Ok(_updated_plan) => {
                        toast_helpers::success("Study plan updated successfully!");
                        // The plan resource will automatically refresh and show the updated plan
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to update study plan: {e}"));
                    }
                }
            });
        }
    };

    let handle_plan_regenerated = move |_updated_plan: crate::types::Plan| {
        // The plan resource will automatically refresh and show the updated plan
        // This handler is called after successful regeneration
        toast_helpers::success("Plan regenerated successfully!");
    };

    // Group plan items by session for better organization
    let session_groups = group_items_by_session(&plan.items);

    rsx! {
        section {
            class: "w-full max-w-4xl mx-auto px-4 py-8",
            h1 { class: "text-2xl font-bold mb-6", "Study Plan" }

            PlanHeader {
                plan_id: plan.id,
                progress: progress,
                completed_sections: completed_sections,
                total_sections: total_sections,
            }

            // Duration summary and validation feedback
            {render_duration_summary(&plan)}

            SessionControlPanel {
                plan: plan.clone(),
                on_settings_change: handle_settings_change,
                on_plan_regenerated: handle_plan_regenerated,
            }

            div {
                style: "{list_style}",
                class: "mt-6",
                SessionList {
                    plan: plan.clone(),
                    session_groups: session_groups,
                    expanded_sessions: expanded_sessions,
                    course_id: course_id,
                }
            }
        }
    }
}

/// Render no plan state
fn render_no_plan_state(course_id: Uuid) -> Element {
    let backend = crate::ui::hooks::use_backend_adapter();
    let is_creating = use_signal(|| false);

    let handle_create_plan = {
        let backend = backend.clone();
        let is_creating = is_creating;

        move |_| {
            let backend = backend.clone();
            let mut is_creating = is_creating;

            spawn(async move {
                is_creating.set(true);
                toast_helpers::info("Creating study plan...");

                // Create default plan settings
                let settings = PlanSettings {
                    start_date: chrono::Utc::now() + chrono::Duration::days(1),
                    sessions_per_week: 3,
                    session_length_minutes: 60,
                    include_weekends: false,
                    advanced_settings: None,
                };

                match backend.generate_plan(course_id, settings).await {
                    Ok(_plan) => {
                        toast_helpers::success("Study plan created successfully!");
                        // The plan resource will automatically refresh and show the new plan
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to create study plan: {e}"));
                    }
                }

                is_creating.set(false);
            });
        }
    };

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
                disabled: is_creating(),
                onclick: handle_create_plan,
                if is_creating() {
                    span { class: "loading loading-spinner loading-sm mr-2" }
                    "Creating Plan..."
                } else {
                    "Create Study Plan"
                }
            }
        }
    }
}
/// Render duration summary and validation feedback
fn render_duration_summary(plan: &crate::types::Plan) -> Element {
    // Calculate total duration and warnings
    let total_video_duration: std::time::Duration =
        plan.items.iter().map(|item| item.total_duration).sum();

    let total_estimated_time: std::time::Duration = plan
        .items
        .iter()
        .map(|item| item.estimated_completion_time)
        .sum();

    let total_warnings: Vec<&String> = plan
        .items
        .iter()
        .flat_map(|item| &item.overflow_warnings)
        .collect();

    let sessions_with_warnings = plan
        .items
        .iter()
        .filter(|item| !item.overflow_warnings.is_empty())
        .count();

    rsx! {
        div {
            class: "bg-base-100 border border-base-300 rounded-lg p-4 mb-6",

            div {
                class: "flex items-center justify-between mb-3",
                h3 {
                    class: "text-lg font-semibold text-base-content",
                    "Course Duration Overview"
                }
                div {
                    class: "text-sm text-base-content/60",
                    "{plan.items.len()} sessions planned"
                }
            }

            // Duration statistics
            div {
                class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-4",

                div {
                    class: "bg-primary/5 border border-primary/20 rounded-lg p-3",
                    div {
                        class: "text-xs text-primary font-medium mb-1",
                        "Total Video Content"
                    }
                    div {
                        class: "text-lg font-bold text-primary",
                        "{crate::types::duration_utils::format_duration_verbose(total_video_duration)}"
                    }
                }

                div {
                    class: "bg-accent/5 border border-accent/20 rounded-lg p-3",
                    div {
                        class: "text-xs text-accent font-medium mb-1",
                        "Estimated Study Time"
                    }
                    div {
                        class: "text-lg font-bold text-accent",
                        "{crate::types::duration_utils::format_duration_verbose(total_estimated_time)}"
                    }
                    div {
                        class: "text-xs text-accent/70 mt-1",
                        "Includes buffer time"
                    }
                }

                div {
                    class: if sessions_with_warnings > 0 { "bg-warning/5 border border-warning/20 rounded-lg p-3" } else { "bg-success/5 border border-success/20 rounded-lg p-3" },
                    div {
                        class: if sessions_with_warnings > 0 { "text-xs text-warning font-medium mb-1" } else { "text-xs text-success font-medium mb-1" },
                        "Session Validation"
                    }
                    div {
                        class: if sessions_with_warnings > 0 { "text-lg font-bold text-warning" } else { "text-lg font-bold text-success" },
                        if sessions_with_warnings > 0 {
                            "{sessions_with_warnings} warnings"
                        } else {
                            "All sessions OK"
                        }
                    }
                    if sessions_with_warnings > 0 {
                        div {
                            class: "text-xs text-warning/70 mt-1",
                            "Check sessions below"
                        }
                    }
                }
            }

            // Global warnings summary
            if !total_warnings.is_empty() {
                div {
                    class: "bg-warning/10 border border-warning/20 rounded-lg p-3",
                    div {
                        class: "flex items-start gap-2 mb-2",
                        div {
                            class: "text-warning text-sm",
                            "⚠️"
                        }
                        div {
                            class: "flex-1",
                            div {
                                class: "text-sm font-medium text-warning mb-1",
                                "Plan Duration Warnings ({total_warnings.len()})"
                            }
                            div {
                                class: "text-xs text-base-content/70",
                                "Some sessions may exceed your preferred session length. Consider adjusting session duration or splitting content."
                            }
                        }
                    }
                }
            }
        }
    }
}
