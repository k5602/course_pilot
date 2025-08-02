use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaFilePen, FaPlay, FaSquare};
use dioxus_motion::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

use crate::state::set_video_context_and_open_notes_reactive;
use crate::types::{Plan, PlanItem, VideoContext};
use crate::ui::{Badge, toast_helpers, use_app_state};
use crate::video_player::{VideoPlayerManager, VideoSource};



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
            // Calculate total videos and completed videos across all items in the session
            let total_videos: usize = items.iter().map(|(_, item)| item.video_indices.len()).sum();
            
            // For now, we'll use plan item completion as a proxy for individual video completion
            // In a full implementation, this would check individual video completion status
            let completed_videos: usize = items.iter()
                .map(|(_, item)| {
                    if item.completed {
                        item.video_indices.len() // All videos in completed items are considered completed
                    } else {
                        0 // No videos in incomplete items are considered completed
                    }
                })
                .sum();
            
            let progress = if total_videos > 0 {
                (completed_videos as f32 / total_videos as f32) * 100.0
            } else {
                0.0
            };

            SessionGroup {
                session_number: session_idx + 1,
                date,
                items,
                total: total_videos,
                completed: completed_videos,
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
                        label: format!("{}/{} videos", props.session.completed, props.session.total),
                        color: Some(badge_color.to_string()),
                        class: Some("text-xs font-medium".to_string()),
                    }

                    // Duration display
                    if let Some(first_item) = props.session.items.first() {
                        div {
                            class: "text-xs text-base-content/60 bg-base-200 px-2 py-1 rounded-full",
                            title: "Estimated session duration with buffer time",
                            "{crate::types::duration_utils::format_duration(first_item.1.total_duration)} / {crate::types::duration_utils::format_duration(first_item.1.estimated_completion_time)}"
                        }
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

                // Display overflow warnings if any
                if let Some(first_item) = props.session.items.first() {
                    if !first_item.1.overflow_warnings.is_empty() {
                        div {
                            class: "bg-warning/10 border border-warning/20 rounded-lg p-3 mb-3 mt-2",
                            div {
                                class: "flex items-start gap-2",
                                div {
                                    class: "text-warning text-sm",
                                    "⚠️"
                                }
                                div {
                                    class: "flex-1",
                                    div {
                                        class: "text-sm font-medium text-warning mb-1",
                                        "Session Duration Warning"
                                    }
                                    ul {
                                        class: "text-xs text-base-content/70 space-y-1",
                                        {first_item.1.overflow_warnings.iter().map(|warning| rsx! {
                                            li { "• {warning}" }
                                        })}
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "space-y-2 pt-2 pb-4",

                    // Render individual videos within each session item
                    for (session_item_idx, (original_index, item)) in props.session.items.iter().enumerate() {
                        // If the item has multiple videos, render each video individually
                        if item.video_indices.len() > 1 {
                            for (video_idx_in_item, &video_index) in item.video_indices.iter().enumerate() {
                                VideoContentItem {
                                    key: "{original_index}-{video_index}-{video_idx_in_item}",
                                    plan_id: props.plan_id,
                                    plan_item: item.clone(),
                                    plan_item_index: *original_index,
                                    video_index: video_index,
                                    video_index_in_item: video_idx_in_item,
                                    session_item_index: session_item_idx,
                                    course_id: props.course_id,
                                    is_session_expanded: is_expanded,
                                }
                            }
                        } else {
                            // Single video item - render as before but with new component
                            VideoContentItem {
                                key: "{original_index}-{item.video_indices.first().unwrap_or(&0)}",
                                plan_id: props.plan_id,
                                plan_item: item.clone(),
                                plan_item_index: *original_index,
                                video_index: *item.video_indices.first().unwrap_or(&0),
                                video_index_in_item: 0,
                                session_item_index: session_item_idx,
                                course_id: props.course_id,
                                is_session_expanded: is_expanded,
                            }
                        }
                    }
                    
                    // Session progress summary
                    SessionProgressBar {
                        completed_videos: props.session.completed,
                        total_videos: props.session.total,
                    }
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct VideoContentItemProps {
    pub plan_id: Uuid,
    pub plan_item: PlanItem,
    pub plan_item_index: usize,
    pub video_index: usize,
    pub video_index_in_item: usize,
    pub session_item_index: usize,
    pub course_id: Uuid,
    pub is_session_expanded: bool,
}

/// Individual video content item component with DaisyUI styling and individual video completion tracking
#[component]
fn VideoContentItem(props: VideoContentItemProps) -> Element {
    let app_state = use_app_state();
    
    // Individual video completion tracking
    let video_completed = use_signal(|| {
        // For now, we'll use the plan item completion status as a fallback
        // In a full implementation, this would check individual video completion
        props.plan_item.completed
    });
    let is_updating = use_signal(|| false);

    // Get video title from course data
    let video_title = use_memo(move || {
        let courses = app_state.read().courses.clone();
        if let Some(course) = courses.iter().find(|c| c.id == props.course_id) {
            course.get_video_title(props.video_index)
                .unwrap_or(&format!("Video {}", props.video_index + 1))
                .to_string()
        } else {
            format!("Video {}", props.video_index + 1)
        }
    });

    // Toggle individual video completion handler
    let toggle_video_completion = {
        let mut video_completed = video_completed;
        let mut is_updating = is_updating;
        let _plan_id = props.plan_id;
        let _video_index = props.video_index;
        let _session_item_index = props.session_item_index;

        move |_| {
            let new_state = !video_completed();
            video_completed.set(new_state);
            is_updating.set(true);

            // Clone values for the async block
            let mut is_updating = is_updating;

            // For now, we'll just update the local state
            // In a full implementation, this would call a backend method to update individual video completion
            spawn(async move {
                // TODO: Implement backend call for individual video completion tracking
                // let video_progress = VideoProgressUpdate::new(plan_id, session_item_index, video_index, new_state);
                // backend.update_video_progress(video_progress).await;
                
                // Simulate API call delay
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                is_updating.set(false);
                
                if new_state {
                    toast_helpers::success("Video marked as completed");
                } else {
                    toast_helpers::info("Video marked as incomplete");
                }
            });
        }
    };

    // Play button handler for individual video
    let play_handler = {
        let course_id = props.course_id;
        let video_index = props.video_index;
        let video_title = video_title.clone();
        let db = use_context::<std::sync::Arc<crate::storage::Database>>();

        move |_| {
            let course_id = course_id;
            let video_index = video_index;
            let video_title = video_title();
            let db = db.clone();
            
            spawn(async move {
                // Get the course data directly from database to ensure consistency
                match tokio::task::spawn_blocking({
                    let db = db.clone();
                    move || crate::storage::get_course_by_id(&db, &course_id)
                }).await {
                    Ok(Ok(Some(course))) => {
                    // Try to get video metadata first, fallback to raw_titles
                    let video_source = if let Some(video_metadata) = course.get_video_metadata(video_index) {
                        // Debug logging to see what's in the metadata
                        log::info!("Video metadata for index {}: title='{}', video_id={:?}, source_url={:?}, is_local={}", 
                                   video_index, video_metadata.title, video_metadata.video_id, video_metadata.source_url, video_metadata.is_local);
                        
                        // Use structured video metadata
                        if let Some(source) = video_metadata.get_video_source() {
                            source
                        } else {
                            log::error!("Could not create video source from metadata for video index {}: video_id={:?}, source_url={:?}, is_local={}", 
                                       video_index, video_metadata.video_id, video_metadata.source_url, video_metadata.is_local);
                            toast_helpers::error("Invalid video metadata");
                            return;
                        }
                    } else if let Some(title) = course.get_video_title(video_index) {
                        // Fallback to raw title analysis
                        if is_youtube_video(title) {
                            if let Some(video_id) = extract_youtube_video_id(title) {
                                VideoSource::YouTube {
                                    video_id,
                                    playlist_id: None,
                                    title: clean_youtube_title(title),
                                }
                            } else {
                                log::warn!("Could not extract YouTube video ID from title: {}", title);
                                toast_helpers::error("Could not extract YouTube video ID");
                                return;
                            }
                        } else {
                            // Assume it's a local video
                            VideoSource::Local {
                                path: std::path::PathBuf::from(title),
                                title: title.to_string(),
                            }
                        }
                    } else {
                        log::error!("Video index {} not found in course", video_index);
                        toast_helpers::error("Video not found in course data");
                        return;
                    };

                    // Create video player manager and play the video
                    match VideoPlayerManager::new() {
                        Ok(mut player_manager) => {
                            match player_manager.play_video(video_source) {
                                Ok(()) => {
                                    toast_helpers::success(format!("Playing: {}", video_title));
                                }
                                Err(e) => {
                                    log::error!("Failed to play video: {e}");
                                    toast_helpers::error(&format!("Failed to play video: {e}"));
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create video player: {e}");
                            toast_helpers::error("Failed to initialize video player");
                        }
                    }
                    }
                    Ok(Ok(None)) => {
                        log::error!("Course not found in database: {}", course_id);
                        toast_helpers::error("Course not found");
                    }
                    Ok(Err(e)) => {
                        log::error!("Database error loading course {}: {}", course_id, e);
                        toast_helpers::error("Failed to load course data");
                    }
                    Err(e) => {
                        log::error!("Task error loading course {}: {}", course_id, e);
                        toast_helpers::error("Failed to load course data");
                    }
                }
            });
        }
    };

    let notes_handler = {
        let course_id = props.course_id;
        let video_index = props.video_index;
        let video_title = video_title.clone();
        let plan_item = props.plan_item.clone();

        move |_| {
            let video_context = VideoContext {
                course_id,
                video_index,
                video_title: video_title(),
                module_title: plan_item.module_title.clone(),
            };

            if let Err(e) = set_video_context_and_open_notes_reactive(video_context) {
                toast_helpers::error(format!("Failed to open notes: {e}"));
            } else {
                toast_helpers::success("Notes panel opened for this video");
            }
        }
    };

    // Smooth entrance animation when session expands
    let mut item_opacity = use_motion(0.0f32);
    let mut item_x = use_motion(-12.0f32);

    use_effect({
        let is_expanded = props.is_session_expanded;
        let video_index_in_item = props.video_index_in_item;
        let session_item_index = props.session_item_index;

        move || {
            if is_expanded {
                // Stagger video item animations when session expands
                let delay = (session_item_index * 2 + video_index_in_item) as f32 * 0.05;

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
    let _check_icon = if video_completed() {
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

    let text_classes = if video_completed() {
        "line-through text-base-content/50 transition-all duration-300"
    } else {
        "text-base-content transition-all duration-300"
    };

    let card_classes = if video_completed() {
        "card bg-success/10 border-success/30 hover:bg-success/20 transition-colors duration-200"
    } else {
        "card bg-base-100 border-base-300 hover:bg-base-200/50 transition-colors duration-200"
    };

    rsx! {
        div { 
            class: "{card_classes} border shadow-sm",
            style: "{item_style}",
            
            div { class: "card-body p-3 flex-row items-center gap-3",
                // Completion checkbox with DaisyUI styling
                div { class: "form-control",
                    label { class: "cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-sm checkbox-primary",
                            checked: video_completed(),
                            disabled: is_updating(),
                            onchange: toggle_video_completion,
                        }
                        if is_updating() {
                            span { class: "loading loading-spinner loading-xs ml-2" }
                        }
                    }
                }
                
                // Video info section
                div { class: "flex-1 min-w-0",
                    h4 { 
                        class: "font-medium text-sm {text_classes} truncate", 
                        title: "{video_title()}",
                        "{video_title()}" 
                    }
                    div { class: "text-xs text-base-content/60 mt-1 truncate",
                        title: "Module: {props.plan_item.module_title}",
                        "Module: {props.plan_item.module_title}"
                    }
                    
                    // Show video index within multi-video items
                    if props.plan_item.video_indices.len() > 1 {
                        div { class: "flex items-center gap-1 mt-1",
                            span { class: "text-xs text-base-content/50 bg-base-200 px-2 py-0.5 rounded-full",
                                "Video {props.video_index_in_item + 1} of {props.plan_item.video_indices.len()}"
                            }
                        }
                    }
                }
                
                // Action buttons with DaisyUI styling
                div { class: "flex items-center gap-2 shrink-0",
                    // Play button
                    button {
                        class: "btn btn-sm btn-primary btn-outline hover:btn-primary",
                        onclick: play_handler,
                        title: "Play video",
                        span { class: "flex items-center gap-1",
                            Icon {
                                icon: FaPlay,
                                class: "w-3 h-3"
                            }
                            span { class: "hidden sm:inline", "Play" }
                        }
                    }
                    
                    // Notes button
                    button {
                        class: "btn btn-sm btn-ghost btn-square hover:btn-accent",
                        onclick: notes_handler,
                        title: "Open notes",
                        Icon {
                            icon: FaFilePen,
                            class: "w-3 h-3"
                        }
                    }
                }
                
                // Status badge
                div { class: "badge badge-sm",
                    class: if video_completed() { "badge-success" } else { "badge-ghost" },
                    if video_completed() { "✓ Done" } else { "Pending" }
                }
            }
        }
    }
}
/// Helper function to determine if a video title/URL is from YouTube
fn is_youtube_video(title: &str) -> bool {
    title.contains("youtube.com") || title.contains("youtu.be") || 
    // For now, assume most imported videos are YouTube unless they look like file paths
    (!title.contains('/') && !title.ends_with(".mp4") && !title.ends_with(".avi") && !title.ends_with(".mov"))
}

/// Extract YouTube video ID from various URL formats or return None
fn extract_youtube_video_id(url_or_title: &str) -> Option<String> {
    // Try to extract from full YouTube URLs
    if let Some(start) = url_or_title.find("v=") {
        let id_start = start + 2;
        let id_end = url_or_title[id_start..]
            .find('&')
            .map(|pos| id_start + pos)
            .unwrap_or(url_or_title.len());
        let video_id = &url_or_title[id_start..id_end];
        if video_id.len() == 11 {
            return Some(video_id.to_string());
        }
    }
    
    // Try to extract from youtu.be URLs
    if let Some(start) = url_or_title.find("youtu.be/") {
        let id_start = start + 9;
        let id_end = url_or_title[id_start..]
            .find('?')
            .map(|pos| id_start + pos)
            .unwrap_or(url_or_title.len());
        let video_id = &url_or_title[id_start..id_end];
        if video_id.len() == 11 {
            return Some(video_id.to_string());
        }
    }
    
    // If no URL pattern found, return None
    // In a real implementation, you might want to search for the video by title
    None
}

/// Clean YouTube title by removing URL parts if present
fn clean_youtube_title(title: &str) -> String {
    // If it's a URL, try to extract just the title part
    if title.contains("youtube.com") || title.contains("youtu.be") {
        // For now, just return the original title
        // In a real implementation, you might want to fetch the actual video title from YouTube API
        title.to_string()
    } else {
        title.to_string()
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct SessionProgressBarProps {
    pub completed_videos: usize,
    pub total_videos: usize,
}

/// Session progress bar component with DaisyUI styling
#[component]
fn SessionProgressBar(props: SessionProgressBarProps) -> Element {
    let progress_percentage = if props.total_videos > 0 {
        (props.completed_videos as f32 / props.total_videos as f32) * 100.0
    } else {
        0.0
    };
    
    let progress_color = if progress_percentage >= 100.0 {
        "progress-success"
    } else if progress_percentage >= 75.0 {
        "progress-primary"
    } else if progress_percentage >= 50.0 {
        "progress-accent"
    } else {
        "progress-warning"
    };

    rsx! {
        div { class: "mt-4 pt-3 border-t border-base-300",
            div { class: "flex items-center justify-between mb-2",
                span { class: "text-sm font-medium text-base-content",
                    "Session Progress"
                }
                span { class: "text-xs text-base-content/60",
                    "{props.completed_videos}/{props.total_videos} videos completed"
                }
            }
            
            progress {
                class: "progress {progress_color} w-full h-2",
                value: "{progress_percentage}",
                max: "100",
                "aria-label": "Session completion progress"
            }
        }
    }
}