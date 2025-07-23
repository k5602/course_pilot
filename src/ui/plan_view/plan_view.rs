use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaFilePen, FaPlay, FaSquare};
use dioxus_motion::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

use super::{PlanHeader, SessionControlPanel};
use crate::state::{set_video_context_and_open_notes, use_app_state};
use crate::types::{PlanExt, PlanItem, PlanSettings, VideoContext};
use crate::ui::components::modal_confirmation::Badge;
use crate::ui::components::toast::toast;
use crate::ui::hooks::{use_plan_resource, use_toggle_plan_item_action};

#[derive(Props, PartialEq, Clone)]
pub struct PlanViewProps {
    pub course_id: Uuid,
}

/// Enhanced plan view component with unified functionality
#[component]
pub fn PlanView(props: PlanViewProps) -> Element {
    let plan_resource = use_plan_resource(props.course_id);
    let expanded_sessions = use_signal(|| HashSet::new());

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
                toast::info("Regenerating plan with new settings...");

                match backend.regenerate_plan(plan_id, new_settings).await {
                    Ok(_updated_plan) => {
                        toast::success("Study plan updated successfully!");
                        // The plan resource will automatically refresh and show the updated plan
                    }
                    Err(e) => {
                        toast::error(format!("Failed to update study plan: {e}"));
                    }
                }
            });
        }
    };

    let handle_plan_regenerated = move |_updated_plan: crate::types::Plan| {
        // The plan resource will automatically refresh and show the updated plan
        // This handler is called after successful regeneration
        toast::success("Plan regenerated successfully!");
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
                toast::info("Creating study plan...");

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
                        toast::success("Study plan created successfully!");
                        // The plan resource will automatically refresh and show the new plan
                    }
                    Err(e) => {
                        toast::error(format!("Failed to create study plan: {e}"));
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

// Helper data structures and functions for unified functionality

#[derive(Debug, Clone, PartialEq)]
pub struct SessionGroup {
    session_number: usize,
    date: chrono::DateTime<chrono::Utc>,
    items: Vec<(usize, PlanItem)>, // (original_index, item)
    total: usize,
    completed: usize,
    progress: f32,
}

/// Group plan items by session (date) for better organization
fn group_items_by_session(items: &[PlanItem]) -> Vec<SessionGroup> {
    use std::collections::HashMap;

    let mut sessions: HashMap<chrono::DateTime<chrono::Utc>, Vec<(usize, PlanItem)>> =
        HashMap::new();

    for (index, item) in items.iter().enumerate() {
        sessions
            .entry(item.date)
            .or_default()
            .push((index, item.clone()));
    }

    let mut session_groups: Vec<SessionGroup> = sessions
        .into_iter()
        .enumerate()
        .map(|(session_idx, (date, items))| {
            let total = items.len();
            let completed = items.iter().filter(|(_, item)| item.completed).count();
            let progress = if total > 0 {
                (completed as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            SessionGroup {
                session_number: session_idx + 1,
                date,
                items,
                total,
                completed,
                progress,
            }
        })
        .collect();

    // Sort sessions by date for chronological ordering
    session_groups.sort_by(|a, b| a.date.cmp(&b.date));

    // Update session numbers after sorting
    for (idx, group) in session_groups.iter_mut().enumerate() {
        group.session_number = idx + 1;
    }

    session_groups
}

/// Unified session list component with collapsible groups
#[component]
fn SessionList(
    plan: crate::types::Plan,
    session_groups: Vec<SessionGroup>,
    expanded_sessions: Signal<HashSet<usize>>,
    course_id: Uuid,
) -> Element {
    // Animation for the entire container
    let mut container_opacity = use_motion(0.0f32);
    let mut container_y = use_motion(20.0f32);

    use_effect(move || {
        container_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        container_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let container_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px);",
            container_opacity.get_value(),
            container_y.get_value()
        )
    });

    rsx! {
        div {
            class: "join join-vertical bg-base-100 w-full shadow-sm",
            style: "{container_style}",
            {session_groups.iter().enumerate().map(|(session_idx, session)| {
                rsx! {
                    SessionAccordion {
                        key: "{session.session_number}",
                        plan_id: plan.id,
                        session: session.clone(),
                        session_index: session_idx,
                        expanded_sessions: expanded_sessions,
                        course_id: course_id,
                    }
                }
            })}
        }
    }
}

#[derive(Props, Clone)]
pub struct SessionAccordionProps {
    pub plan_id: Uuid,
    pub session: SessionGroup,
    pub session_index: usize,
    pub expanded_sessions: Signal<HashSet<usize>>,
    pub course_id: Uuid,
}

impl PartialEq for SessionAccordionProps {
    fn eq(&self, other: &Self) -> bool {
        self.plan_id == other.plan_id
            && self.session.session_number == other.session.session_number
            && self.session.total == other.session.total
            && self.session.completed == other.session.completed
            && self.session_index == other.session_index
            && self.course_id == other.course_id
    }
}

/// Session accordion component with progress indicator and video controls
#[component]
fn SessionAccordion(props: SessionAccordionProps) -> Element {
    let session_id = format!("session-{}-{}", props.plan_id, props.session_index);
    let mut expanded_sessions = props.expanded_sessions;
    let is_expanded = expanded_sessions.read().contains(&props.session_index);

    // Toggle session expansion
    let toggle_session = move |_| {
        let mut expanded = expanded_sessions.write();
        if expanded.contains(&props.session_index) {
            expanded.remove(&props.session_index);
        } else {
            expanded.insert(props.session_index);
        }
    };

    // Staggered animation for each session
    let mut session_opacity = use_motion(0.0f32);
    let mut session_x = use_motion(-20.0f32);

    use_effect({
        let session_index = props.session_index;
        move || {
            // Stagger animation based on session index
            let delay = session_index as f32 * 0.1;

            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis((delay * 1000.0) as u64))
                    .await;

                session_opacity.animate_to(
                    1.0,
                    AnimationConfig::new(AnimationMode::Tween(Tween::default())),
                );
                session_x.animate_to(
                    0.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring::default())),
                );
            });
        }
    });

    let session_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateX({}px);",
            session_opacity.get_value(),
            session_x.get_value()
        )
    });

    let progress_color = if props.session.progress >= 100.0 {
        "progress-success"
    } else if props.session.progress >= 50.0 {
        "progress-primary"
    } else {
        "progress-accent"
    };

    rsx! {
        div {
            class: "collapse collapse-arrow join-item border-base-300 border",
            style: "{session_style}",

            input {
                type: "checkbox",
                id: "{session_id}",
                name: "{session_id}",
                checked: is_expanded,
                onchange: toggle_session,
            }

            div {
                class: "collapse-title font-semibold flex items-center justify-between pr-4",

                div { class: "flex items-center gap-3",
                    h3 { class: "text-lg font-semibold",
                        "Session {props.session.session_number}"
                    }
                    div { class: "text-sm text-base-content/60",
                        "{props.session.date.format(\"%Y-%m-%d\")}"
                    }
                    Badge {
                        label: format!("{}/{}", props.session.completed, props.session.total),
                        color: Some(if props.session.progress >= 100.0 { "success".to_string() } else { "primary".to_string() }),
                        class: Some("text-xs".to_string()),
                    }
                }

                div { class: "flex items-center gap-2",
                    progress {
                        class: "progress {progress_color} w-24 h-2",
                        value: "{props.session.progress}",
                        max: "100"
                    }
                    span { class: "text-sm text-base-content/60", "{props.session.progress:.0}%" }
                }
            }

            div {
                class: "collapse-content",
                div {
                    class: "space-y-2 pt-2",
                    {props.session.items.iter().map(|(original_index, item)| {
                        rsx! {
                            VideoItem {
                                key: "{original_index}",
                                plan_id: props.plan_id,
                                item: item.clone(),
                                item_index: *original_index,
                                course_id: props.course_id,
                            }
                        }
                    })}
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct VideoItemProps {
    pub plan_id: Uuid,
    pub item: PlanItem,
    pub item_index: usize,
    pub course_id: Uuid,
}

/// Individual video item component with three-button layout
#[component]
fn VideoItem(props: VideoItemProps) -> Element {
    let toggle_completion = use_toggle_plan_item_action();
    let mut local_completed = use_signal(|| props.item.completed);
    let app_state = use_app_state();

    // Sync local state with prop changes
    use_effect(move || {
        local_completed.set(props.item.completed);
    });

    let toggle_handler = {
        let plan_id = props.plan_id;
        let item_index = props.item_index;
        let mut local_completed = local_completed;

        move |_| {
            let new_state = !local_completed();
            local_completed.set(new_state);
            toggle_completion(plan_id, item_index, new_state);
        }
    };

    let play_handler = move |_| {
        toast::info("Video player will be implemented in a future phase");
    };

    let notes_handler = {
        let course_id = props.course_id;
        let item = props.item.clone();
        let app_state = app_state;

        move |_| {
            let video_context = VideoContext {
                course_id,
                video_index: props.item_index,
                video_title: item.section_title.clone(),
                module_title: item.module_title.clone(),
            };

            if let Err(e) = set_video_context_and_open_notes(app_state, video_context) {
                toast::error(format!("Failed to open notes: {e}"));
            }
        }
    };

    // Animation
    let mut item_opacity = use_motion(0.0f32);
    let mut item_x = use_motion(-12.0f32);

    use_effect(move || {
        item_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        item_x.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let item_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateX({}px); transition: opacity 0.3s, transform 0.3s;",
            item_opacity.get_value(),
            item_x.get_value()
        )
    });

    let check_icon = if local_completed() {
        rsx! {
            Icon { icon: FaCheckDouble, class: "w-4 h-4 text-success" }
        }
    } else {
        rsx! {
            Icon { icon: FaSquare, class: "w-4 h-4 text-base-content/40" }
        }
    };

    let text_classes = if local_completed() {
        "line-through text-base-content/40"
    } else {
        "text-base-content"
    };

    rsx! {
        div {
            class: "flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-base-200 transition-colors border border-transparent hover:border-base-300",
            style: "{item_style}",

            // Progress checkbox
            button {
                class: "btn btn-ghost btn-sm btn-square",
                onclick: toggle_handler,
                "aria-label": if local_completed() { "Mark as incomplete" } else { "Mark as complete" },
                {check_icon}
            }

            // Video content
            div { class: "flex-1 min-w-0",
                div {
                    class: "text-sm font-medium {text_classes}",
                    style: "display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;",
                    "{props.item.section_title}"
                }
                div {
                    class: "text-xs text-base-content/60 mt-1",
                    "Module: {props.item.module_title}"
                }
            }

            // Action buttons
            div { class: "flex items-center gap-1 shrink-0",
                // Play button
                button {
                    class: "btn btn-ghost btn-sm btn-square",
                    onclick: play_handler,
                    "aria-label": "Play video",
                    Icon { icon: FaPlay, class: "w-3 h-3 text-primary" }
                }

                // Notes button
                button {
                    class: "btn btn-ghost btn-sm btn-square",
                    onclick: notes_handler,
                    "aria-label": "Open notes",
                    Icon { icon: FaFilePen, class: "w-3 h-3 text-accent" }
                }
            }

            // Status badge
            Badge {
                label: if local_completed() { "Done".to_string() } else { "Pending".to_string() },
                color: Some(if local_completed() { "success".to_string() } else { "accent".to_string() }),
                class: Some("text-xs shrink-0".to_string()),
            }
        }
    }
}
