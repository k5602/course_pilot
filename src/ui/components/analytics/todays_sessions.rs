use crate::storage::Database;
use crate::types::{Plan, PlanItem};
use crate::ui::components::ProgressRing;
use crate::ui::hooks::use_toggle_plan_item_action;
use chrono::Local;
use dioxus::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

#[component]
pub fn TodaysSessions() -> Element {
    let db = use_context::<Arc<Database>>();
    let today = Local::now().date_naive();

    // Load all plans and filter today's sessions
    let todays_sessions_resource = use_resource(move || {
        let db = db.clone();
        async move {
            tokio::task::spawn_blocking(move || {
                // Load all courses to get their plans
                let courses = crate::storage::load_courses(&db)?;
                let mut sessions = Vec::new();

                for course in courses {
                    if let Ok(Some(plan)) = crate::storage::get_plan_by_course_id(&db, &course.id) {
                        for (index, item) in plan.items.iter().enumerate() {
                            let item_date = item.date.with_timezone(&Local).date_naive();
                            if item_date == today {
                                sessions.push((plan.clone(), index, item.clone()));
                            }
                        }
                    }
                }

                // Sort by time
                sessions.sort_by_key(|(_, _, item)| item.date);
                Ok::<Vec<(Plan, usize, PlanItem)>, anyhow::Error>(sessions)
            })
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("Failed to load sessions")))
        }
    });

    match &*todays_sessions_resource.read_unchecked() {
        Some(Ok(todays_sessions)) => {
            if todays_sessions.is_empty() {
                return rsx! {
                    div { class: "text-center py-8 text-base-content/60",
                        div { class: "text-4xl mb-2", "📅" }
                        p { "No sessions scheduled for today" }
                        p { class: "text-sm mt-2", "Take a break or plan your next study session!" }
                    }
                };
            }

            rsx! {
                div { class: "space-y-3",
                    h3 { class: "font-semibold text-lg mb-4", "Today's Sessions" }

                    {todays_sessions.iter().map(|(plan, index, item)| {
                        let progress_percentage = (plan.completed_sessions() as f32 / plan.total_sessions().max(1) as f32) * 100.0;

                        rsx! {
                            SessionCard {
                                key: "{plan.id}-{index}",
                                plan: plan.clone(),
                                item: item.clone(),
                                session_index: *index,
                                progress_percentage
                            }
                        }
                    })}
                }
            }
        }
        Some(Err(e)) => rsx! {
            div { class: "alert alert-error",
                "Failed to load today's sessions: {e}"
            }
        },
        None => rsx! {
            div { class: "space-y-3",
                div { class: "skeleton h-16 w-full" }
                div { class: "skeleton h-16 w-full" }
                div { class: "skeleton h-16 w-full" }
            }
        },
    }
}

#[derive(Props, PartialEq, Clone)]
struct SessionCardProps {
    plan: Plan,
    item: PlanItem,
    session_index: usize,
    progress_percentage: f32,
}

#[component]
fn SessionCard(props: SessionCardProps) -> Element {
    let toggle_completion = use_toggle_plan_item_action();
    let time_str = props
        .item
        .date
        .with_timezone(&Local)
        .format("%H:%M")
        .to_string();
    let duration_str = crate::types::duration_utils::format_duration(props.item.total_duration);

    let handle_toggle_completion = {
        let plan_id = props.plan.id;
        let session_index = props.session_index;

        move |_| {
            toggle_completion.call((plan_id, session_index));
        }
    };

    rsx! {
        div {
            class: format!(
                "card bg-base-100 border-l-4 {}",
                if props.item.completed { "border-l-success bg-success/5" } else { "border-l-primary" }
            ),
            div { class: "card-body p-4",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center gap-3",
                        div { class: "flex-shrink-0",
                            ProgressRing {
                                value: props.progress_percentage as u32,
                                max: 100,
                                size: 32,
                                thickness: 3
                            }
                        }
                        div {
                            h4 { class: "font-medium", "{props.item.module_title}" }
                            p { class: "text-sm text-base-content/70", "{props.item.section_title}" }
                            div { class: "flex items-center gap-2 text-xs text-base-content/50 mt-1",
                                span { "⏰ {time_str}" }
                                span { "•" }
                                span { "⏱️ {duration_str}" }
                                span { "•" }
                                span { "{props.item.video_indices.len()} videos" }
                            }
                        }
                    }

                    div { class: "flex items-center gap-2",
                        if props.item.completed {
                            div { class: "badge badge-success badge-sm", "Completed" }
                        } else {
                            SessionQuickStart {
                                plan_id: props.plan.id,
                                session_index: props.session_index,
                                item: props.item.clone()
                            }
                        }

                        input {
                            type: "checkbox",
                            class: "checkbox checkbox-sm",
                            checked: props.item.completed,
                            onclick: handle_toggle_completion
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SessionQuickStartProps {
    plan_id: Uuid,
    session_index: usize,
    item: PlanItem,
}

#[component]
fn SessionQuickStart(props: SessionQuickStartProps) -> Element {
    let toggle_completion = use_toggle_plan_item_action();

    let handle_start_session = {
        let plan_id = props.plan_id;
        let session_index = props.session_index;

        move |_| {
            // Mark session as started/completed
            toggle_completion.call((plan_id, session_index));
        }
    };

    rsx! {
        button {
            class: "btn btn-primary btn-sm",
            onclick: handle_start_session,
            "▶️ Start Session"
        }
    }
}
