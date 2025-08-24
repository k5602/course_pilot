use crate::types::{Plan, PlanItem};
use crate::ui::hooks::use_backend;
use chrono::{DateTime, Duration as ChronoDuration, Local, Utc};
use dioxus::prelude::*;

#[component]
pub fn UpcomingDeadlines() -> Element {
    let backend = use_backend();

    let upcoming_deadlines_resource = use_resource(move || {
        let backend = backend.clone();
        async move {
            // Load all courses to get their plans via backend
            let courses = backend
                .list_courses()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load courses: {}", e))?;

            let mut upcoming_sessions: Vec<(Plan, usize, PlanItem, String)> = Vec::new();
            let now = Utc::now();
            let next_week = now + ChronoDuration::days(7);

            for course in courses {
                if let Ok(Some(plan)) = backend.get_plan_by_course(course.id).await {
                    for (index, item) in plan.items.iter().enumerate() {
                        // Only include future sessions within the next week
                        if item.date > now && item.date <= next_week && !item.completed {
                            upcoming_sessions.push((
                                plan.clone(),
                                index,
                                item.clone(),
                                course.name.clone(),
                            ));
                        }
                    }
                }
            }

            // Sort by date
            upcoming_sessions.sort_by_key(|(_, _, item, _)| item.date);
            Ok::<Vec<(Plan, usize, PlanItem, String)>, anyhow::Error>(upcoming_sessions)
        }
    });

    match &*upcoming_deadlines_resource.read_unchecked() {
        Some(Ok(upcoming_sessions)) => {
            if upcoming_sessions.is_empty() {
                return rsx! {
                    div { class: "card bg-base-100 shadow-sm border border-base-300",
                        div { class: "card-body p-4 text-center",
                            div { class: "text-4xl mb-2", "📅" }
                            h3 { class: "card-title text-lg justify-center", "No Upcoming Deadlines" }
                            p { class: "text-base-content/60", "You're all caught up for the next week!" }
                            p { class: "text-sm text-base-content/50 mt-1", "Create more study plans to see upcoming sessions" }
                        }
                    }
                };
            }

            rsx! {
                div { class: "card bg-base-100 shadow-sm border border-base-300",
                    div { class: "card-body p-4",
                        h3 { class: "card-title text-lg flex items-center gap-2",
                            span { "⏰" }
                            "Upcoming Deadlines"
                            span { class: "badge badge-primary badge-sm", "{upcoming_sessions.len()}" }
                        }

                        div { class: "space-y-2 mt-4",
                            {upcoming_sessions.iter().take(5).map(|(plan, index, item, course_name)| {
                                let urgency = get_urgency_level(item.date);

                                rsx! {
                                    UpcomingDeadlineCard {
                                        key: "{plan.id}-{index}",
                                        plan_id: plan.id,
                                        session_index: *index,
                                        item: item.clone(),
                                        course_name: course_name.clone(),
                                        urgency
                                    }
                                }
                            })}
                        }

                        if upcoming_sessions.len() > 5 {
                            div { class: "text-center mt-3",
                                p { class: "text-sm text-base-content/60",
                                    "And {upcoming_sessions.len() - 5} more sessions this week..."
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! {
            div { class: "alert alert-error",
                "Failed to load upcoming deadlines: {e}"
            }
        },
        None => rsx! {
            div { class: "skeleton h-32 w-full" }
        },
    }
}

#[derive(Props, PartialEq, Clone)]
struct UpcomingDeadlineCardProps {
    plan_id: uuid::Uuid,
    session_index: usize,
    item: PlanItem,
    course_name: String,
    urgency: UrgencyLevel,
}

#[derive(Clone, PartialEq)]
enum UrgencyLevel {
    Critical, // Within 24 hours
    High,     // Within 2 days
    Medium,   // Within 3-4 days
    Low,      // 5+ days
}

#[component]
fn UpcomingDeadlineCard(props: UpcomingDeadlineCardProps) -> Element {
    let local_time = props.item.date.with_timezone(&Local);
    let time_str = local_time.format("%a %m/%d at %H:%M").to_string();
    let duration_str = crate::types::duration_utils::format_duration(props.item.total_duration);

    let (urgency_color, urgency_icon, urgency_text) = match props.urgency {
        UrgencyLevel::Critical => ("border-l-error text-error", "🚨", "Due soon"),
        UrgencyLevel::High => ("border-l-warning text-warning", "⚠️", "Due tomorrow"),
        UrgencyLevel::Medium => ("border-l-info text-info", "📅", "This week"),
        UrgencyLevel::Low => ("border-l-success text-success", "⏳", "Next week"),
    };

    rsx! {
        div { class: "flex items-center gap-3 p-3 bg-base-200 rounded-lg border-l-4 {urgency_color}",
            div { class: "text-lg", "{urgency_icon}" }
            div { class: "flex-1",
                div { class: "flex items-center justify-between",
                    h4 { class: "font-semibold text-sm", "{props.course_name}" }
                    span { class: "badge badge-outline badge-xs {urgency_color}", "{urgency_text}" }
                }
                p { class: "text-xs text-base-content/70 mt-1", "{props.item.module_title}" }
                div { class: "flex items-center gap-2 text-xs text-base-content/50 mt-1",
                    span { "📅 {time_str}" }
                    span { "•" }
                    span { "⏱️ {duration_str}" }
                    span { "•" }
                    span { "{props.item.video_indices.len()} videos" }
                }
            }

            div { class: "flex flex-col gap-1",
                button {
                    class: "btn btn-primary btn-xs",
                    onclick: move |_| {
                        // Navigate to the plan view for this session
                        let navigator = dioxus_router::prelude::use_navigator();
                        navigator.push(format!("/plan/{}", props.plan_id));
                    },
                    "View Plan"
                }
                button {
                    class: "btn btn-ghost btn-xs",
                    onclick: move |_| {
                        // Could implement a "snooze" or reschedule feature
                    },
                    "Reschedule"
                }
            }
        }
    }
}

fn get_urgency_level(deadline: DateTime<Utc>) -> UrgencyLevel {
    let now = Utc::now();
    let time_until = deadline.signed_duration_since(now);

    if time_until <= ChronoDuration::days(1) {
        UrgencyLevel::Critical
    } else if time_until <= ChronoDuration::days(2) {
        UrgencyLevel::High
    } else if time_until <= ChronoDuration::days(4) {
        UrgencyLevel::Medium
    } else {
        UrgencyLevel::Low
    }
}
