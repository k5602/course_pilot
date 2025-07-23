use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaFilePen, FaPlay, FaSquare};
use dioxus_motion::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

use crate::state::{set_video_context_and_open_notes, use_app_state};
use crate::types::{Plan, PlanItem, VideoContext};
use crate::ui::components::modal_confirmation::Badge;
use crate::ui::components::toast::toast;
use crate::ui::hooks::use_toggle_plan_item_action;

/// Session group data structure for organizing plan items by date
#[derive(Debug, Clone, PartialEq)]
pub struct SessionGroup {
    pub session_number: usize,
    pub date: chrono::DateTime<chrono::Utc>,
    pub items: Vec<(usize, PlanItem)>, // (original_index, item)
    pub total: usize,
    pub completed: usize,
    pub progress: f32,
}

/// Group plan items by session (date) for better organization
pub fn group_items_by_session(items: &[PlanItem]) -> Vec<SessionGroup> {
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

#[derive(Props, PartialEq, Clone)]
pub struct SessionListProps {
    pub plan: Plan,
    pub session_groups: Vec<SessionGroup>,
    pub expanded_sessions: Signal<HashSet<usize>>,
    pub course_id: Uuid,
}

/// Unified session list component with collapsible groups and smooth animations
#[component]
pub fn SessionList(props: SessionListProps) -> Element {
    // Animation for the entire container with staggered entrance
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
            "opacity: {}; transform: translateY({}px); transition: all 0.3s ease-out;",
            container_opacity.get_value(),
            container_y.get_value()
        )
    });

    // Efficient rendering optimization for large session lists
    let visible_sessions = use_memo(move || {
        // For now, show all sessions. In the future, we could implement virtual scrolling
        // for plans with 100+ sessions if performance becomes an issue
        props.session_groups.clone()
    });

    rsx! {
        div {
            class: "join join-vertical bg-base-100 w-full shadow-sm rounded-lg overflow-hidden",
            style: "{container_style}",

            // Render session groups with staggered animations
            {visible_sessions.iter().enumerate().map(|(session_idx, session)| {
                rsx! {
                    SessionAccordion {
                        key: "{session.session_number}-{session.date.timestamp()}",
                        plan_id: props.plan.id,
                        session: session.clone(),
                        session_index: session_idx,
                        expanded_sessions: props.expanded_sessions,
                        course_id: props.course_id,
                    }
                }
            })}

            // Empty state for plans with no sessions
            if visible_sessions.is_empty() {
                div {
                    class: "p-8 text-center text-base-content/60",
                    div { class: "text-lg font-medium mb-2", "No sessions scheduled" }
                    div { class: "text-sm", "Your study plan will appear here once generated." }
                }
            }
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

/// Session accordion component with DaisyUI collapse styling and smooth animations
#[component]
fn SessionAccordion(props: SessionAccordionProps) -> Element {
    let session_id = format!("session-{}-{}", props.plan_id, props.session_index);
    let mut expanded_sessions = props.expanded_sessions;
    let is_expanded = expanded_sessions.read().contains(&props.session_index);

    // Toggle session expansion with smooth animation
    let toggle_session = move |_| {
        let mut expanded = expanded_sessions.write();
        if expanded.contains(&props.session_index) {
            expanded.remove(&props.session_index);
        } else {
            expanded.insert(props.session_index);
        }
    };

    // Staggered animation for each session accordion
    let mut session_opacity = use_motion(0.0f32);
    let mut session_x = use_motion(-20.0f32);

    use_effect({
        let session_index = props.session_index;
        move || {
            // Stagger animation based on session index for smooth entrance
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
            "opacity: {}; transform: translateX({}px); transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);",
            session_opacity.get_value(),
            session_x.get_value()
        )
    });

    // Dynamic progress color based on completion percentage
    let progress_color = if props.session.progress >= 100.0 {
        "progress-success"
    } else if props.session.progress >= 75.0 {
        "progress-primary"
    } else if props.session.progress >= 50.0 {
        "progress-accent"
    } else {
        "progress-warning"
    };

    // Badge color based on completion status
    let badge_color = if props.session.progress >= 100.0 {
        "success"
    } else if props.session.progress > 0.0 {
        "primary"
    } else {
        "ghost"
    };

    rsx! {
        div {
            class: "collapse collapse-arrow join-item border-base-300 border-b last:border-b-0 hover:bg-base-50 transition-colors duration-200",
            style: "{session_style}",

            input {
                type: "checkbox",
                id: "{session_id}",
                name: "{session_id}",
                checked: is_expanded,
                onchange: toggle_session,
                class: "peer",
            }

            // Session header with progress information
            div {
                class: "collapse-title font-semibold flex items-center justify-between pr-4 py-4 cursor-pointer select-none",

                // Left side: Session info and badge
                div {
                    class: "flex items-center gap-3 min-w-0 flex-1",

                    h3 {
                        class: "text-lg font-semibold text-base-content",
                        "Session {props.session.session_number}"
                    }

                    div {
                        class: "text-sm text-base-content/60 hidden sm:block",
                        "{props.session.date.format(\"%a, %b %d\")}"
                    }

                    Badge {
                        label: format!("{}/{}", props.session.completed, props.session.total),
                        color: Some(badge_color.to_string()),
                        class: Some("text-xs font-medium".to_string()),
                    }
                }

                // Right side: Progress indicator
                div {
                    class: "flex items-center gap-3 shrink-0",

                    div {
                        class: "flex flex-col items-end gap-1",

                        progress {
                            class: "progress {progress_color} w-20 h-2",
                            value: "{props.session.progress}",
                            max: "100",
                            "aria-label": "Session progress"
                        }

                        span {
                            class: "text-xs text-base-content/60 font-medium",
                            "{props.session.progress:.0}%"
                        }
                    }
                }
            }

            // Collapsible content with video items
            div {
                class: "collapse-content bg-base-50/50",

                div {
                    class: "space-y-1 pt-2 pb-4",

                    // Render video items with smooth animations
                    {props.session.items.iter().enumerate().map(|(video_idx, (original_index, item))| {
                        rsx! {
                            VideoItem {
                                key: "{original_index}-{item.section_title}",
                                plan_id: props.plan_id,
                                item: item.clone(),
                                item_index: *original_index,
                                video_index: video_idx,
                                course_id: props.course_id,
                                is_session_expanded: is_expanded,
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
    pub video_index: usize,
    pub course_id: Uuid,
    pub is_session_expanded: bool,
}

/// Individual video item component with three-button layout and smooth interactions
#[component]
fn VideoItem(props: VideoItemProps) -> Element {
    let toggle_completion = use_toggle_plan_item_action();
    let mut local_completed = use_signal(|| props.item.completed);
    let is_updating = use_signal(|| false);
    let app_state = use_app_state();

    // Sync local state with prop changes
    use_effect(move || {
        local_completed.set(props.item.completed);
    });

    // Toggle completion handler with optimistic updates
    let toggle_handler = {
        let plan_id = props.plan_id;
        let item_index = props.item_index;
        let mut local_completed = local_completed;
        let mut is_updating = is_updating;
        let toggle_completion = toggle_completion.clone();

        move |_| {
            let new_state = !local_completed();
            local_completed.set(new_state);
            is_updating.set(true);

            // Clone values for the async block
            let toggle_completion = toggle_completion.clone();
            let mut is_updating = is_updating;

            // Optimistic update with backend sync
            spawn(async move {
                toggle_completion(plan_id, item_index, new_state);

                // Small delay to show loading state
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                is_updating.set(false);
            });
        }
    };

    // Play button handler (placeholder for future implementation)
    let play_handler = move |_| {
        toast::info("Video player will be implemented in a future phase");
    };

    // Notes button handler with video context
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
            } else {
                toast::success("Notes panel opened for this video");
            }
        }
    };

    // Smooth entrance animation when session expands
    let mut item_opacity = use_motion(0.0f32);
    let mut item_x = use_motion(-12.0f32);

    use_effect({
        let is_expanded = props.is_session_expanded;
        let video_index = props.video_index;

        move || {
            if is_expanded {
                // Stagger video item animations when session expands
                let delay = video_index as f32 * 0.05;

                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis((delay * 1000.0) as u64))
                        .await;

                    item_opacity.animate_to(
                        1.0,
                        AnimationConfig::new(AnimationMode::Tween(Tween::default())),
                    );
                    item_x.animate_to(
                        0.0,
                        AnimationConfig::new(AnimationMode::Spring(Spring::default())),
                    );
                });
            } else {
                // Reset animation when session collapses
                item_opacity.animate_to(
                    0.0,
                    AnimationConfig::new(AnimationMode::Tween(Tween::default())),
                );
                item_x.animate_to(
                    -12.0,
                    AnimationConfig::new(AnimationMode::Tween(Tween::default())),
                );
            }
        }
    });

    let item_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateX({}px); transition: all 0.2s ease-out;",
            if props.is_session_expanded {
                item_opacity.get_value()
            } else {
                1.0
            },
            if props.is_session_expanded {
                item_x.get_value()
            } else {
                0.0
            }
        )
    });

    // Dynamic icons and styling based on completion state
    let check_icon = if local_completed() {
        rsx! {
            Icon {
                icon: FaCheckDouble,
                class: "w-4 h-4 text-success transition-colors duration-200"
            }
        }
    } else {
        rsx! {
            Icon {
                icon: FaSquare,
                class: "w-4 h-4 text-base-content/40 hover:text-base-content/60 transition-colors duration-200"
            }
        }
    };

    let text_classes = if local_completed() {
        "line-through text-base-content/50 transition-all duration-300"
    } else {
        "text-base-content transition-all duration-300"
    };

    let item_bg_class = if local_completed() {
        "bg-success/5 border-success/20"
    } else {
        "bg-base-100 border-base-300 hover:border-base-400"
    };

    rsx! {
        div {
            class: "flex items-center gap-3 px-4 py-3 rounded-lg {item_bg_class} border transition-all duration-200 hover:shadow-sm group",
            style: "{item_style}",

            // Progress checkbox with loading state
            button {
                class: "btn btn-ghost btn-sm btn-square hover:btn-primary transition-all duration-200",
                disabled: is_updating(),
                onclick: toggle_handler,
                "aria-label": if local_completed() { "Mark as incomplete" } else { "Mark as complete" },

                if is_updating() {
                    span { class: "loading loading-spinner loading-xs" }
                } else {
                    {check_icon}
                }
            }

            // Video content with truncated text
            div {
                class: "flex-1 min-w-0",

                div {
                    class: "text-sm font-medium {text_classes} leading-tight",
                    style: "display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;",
                    title: "{props.item.section_title}",
                    "{props.item.section_title}"
                }

                div {
                    class: "text-xs text-base-content/60 mt-1 truncate",
                    title: "Module: {props.item.module_title}",
                    "Module: {props.item.module_title}"
                }
            }

            // Action buttons with hover effects
            div {
                class: "flex items-center gap-1 shrink-0 opacity-60 group-hover:opacity-100 transition-opacity duration-200",

                // Play button
                button {
                    class: "btn btn-ghost btn-sm btn-square hover:btn-primary hover:text-primary-content transition-all duration-200",
                    onclick: play_handler,
                    "aria-label": "Play video",
                    title: "Play video",
                    Icon {
                        icon: FaPlay,
                        class: "w-3 h-3"
                    }
                }

                // Notes button
                button {
                    class: "btn btn-ghost btn-sm btn-square hover:btn-accent hover:text-accent-content transition-all duration-200",
                    onclick: notes_handler,
                    "aria-label": "Open notes for this video",
                    title: "Open notes",
                    Icon {
                        icon: FaFilePen,
                        class: "w-3 h-3"
                    }
                }
            }

            // Status badge with dynamic styling
            Badge {
                label: if local_completed() { "Done".to_string() } else { "Pending".to_string() },
                color: Some(if local_completed() { "success".to_string() } else { "ghost".to_string() }),
                class: Some("text-xs shrink-0 transition-all duration-200".to_string()),
            }
        }
    }
}
