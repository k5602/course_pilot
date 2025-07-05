//! Intelligent Study Plan View
//!
//! This component provides an advanced, motivating learning experience with:
//! - Timeline-based study plan visualization
//! - Smart progress tracking and analytics
//! - Achievement system and streak tracking
//! - Adaptive scheduling recommendations
//! - Interactive study sessions with focus mode

use crate::nlp::structure_course;
use crate::planner::generate_plan;
use crate::state::{async_structure_course, navigate_to, use_app_state, use_course};
use crate::types::{AppState, Course, Plan, PlanSettings, Route};
use crate::ui::navigation::navigate_to_dashboard;
use chrono::{DateTime, Duration, Local, Utc};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{
    MdBookmark, MdCheckCircle, MdSchedule, MdTrendingUp,
};
use dioxus_free_icons::icons::md_av_icons::{MdLibraryBooks, MdPlayArrow, MdPlayCircleFilled};
use dioxus_free_icons::icons::md_content_icons::MdCreate;
use dioxus_free_icons::icons::md_device_icons::MdAccessTime;
use dioxus_motion::prelude::*;
use dioxus_toast::{ToastInfo, ToastManager};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Pending,
    InProgress,
    Completed,
    Overdue,
    TodayFocus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudySession {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub estimated_duration: Duration,
    pub difficulty: DifficultyLevel,
    pub status: SessionStatus,
    pub scheduled_date: DateTime<Local>,
    pub content_items: Vec<String>,
    pub completed_items: Vec<bool>,
    pub module_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub unlocked: bool,
    pub progress: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LearningAnalytics {
    pub total_study_time: Duration,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub completion_rate: f32,
    pub sessions_completed: i32,
    pub sessions_total: i32,
    pub average_session_time: Duration,
    pub momentum_score: f32,
}

impl Default for LearningAnalytics {
    fn default() -> Self {
        Self {
            total_study_time: Duration::zero(),
            current_streak: 0,
            longest_streak: 0,
            completion_rate: 0.0,
            sessions_completed: 0,
            sessions_total: 0,
            average_session_time: Duration::zero(),
            momentum_score: 0.0,
        }
    }
}

#[component]
pub fn PlanView(course_id: Uuid) -> Element {
    let app_state = use_app_state();
    let course_signal = use_course(course_id);
    let mut sidebar_open = use_signal(|| false);
    let mut selected_session = use_signal(|| Option::<Uuid>::None);
    let mut toast: Signal<ToastManager> = use_context();
    let _plan = use_signal(|| Option::<Plan>::None);
    let mut is_loading = use_signal(|| true);
    let is_structuring = use_signal(|| false);
    let _is_planning = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    // Initialize course data using reactive course signal
    use_effect(move || {
        let course_exists = crate::state::course_exists(app_state, course_id);
        if course_exists {
            is_loading.set(false);
            error_message.set(None);
        } else {
            error_message.set(Some("Course not found".to_string()));
            is_loading.set(false);
        }
    });

    // Get current course from reactive signal
    let current_course = match course_signal.read().as_ref() {
        Some(c) => c.clone(),
        None => {
            if *is_loading.read() {
                return rsx! {
                    div { class: "plan-view-container",
                        "Loading course..."
                    }
                };
            } else {
                return rsx! {
                    div { class: "plan-view-container",
                        div { class: "error-state",
                            h2 { "Course not found" }
                            button {
                                onclick: move |_| {
                                    if let Err(e) = navigate_to(app_state, Route::Dashboard) {
                                        log::error!("Failed to navigate to dashboard: {:?}", e);
                                    }
                                },
                                "← Back to Dashboard"
                            }
                        }
                    }
                };
            }
        }
    };

    // Structure course action using async state function
    let structure_course_action = move |_| {
        if let Some(current_course) = course_signal.read().clone() {
            let app_state_clone = app_state;
            let mut toast_clone = toast;
            let mut error_message_clone = error_message;
            let mut is_structuring_clone = is_structuring;

            spawn(async move {
                is_structuring_clone.set(true);
                error_message_clone.set(None);

                match async_structure_course(
                    app_state_clone,
                    current_course.id,
                    current_course.raw_titles.clone(),
                )
                .await
                {
                    Ok(()) => {
                        toast_clone
                            .write()
                            .popup(ToastInfo::simple("Course structured successfully!"));
                    }
                    Err(e) => {
                        error_message_clone.set(Some(format!("Failed to structure course: {}", e)));
                    }
                }

                is_structuring_clone.set(false);
            });
        }
    };

    // Generate study sessions and analytics only if course is structured
    let (study_sessions, analytics, achievements, today_focus) = if current_course.is_structured() {
        let sessions = generate_study_sessions(&current_course);
        let analytics = calculate_analytics(&sessions);
        let achievements = generate_achievements(&analytics);
        let today_focus = {
            let today = Local::now().date_naive();
            sessions
                .iter()
                .find(|s| {
                    s.scheduled_date.date_naive() == today && s.status == SessionStatus::Pending
                })
                .cloned()
        };
        (sessions, analytics, achievements, today_focus)
    } else {
        (Vec::new(), LearningAnalytics::default(), Vec::new(), None)
    };

    // Progress animations
    let mut overall_progress = use_motion(0.0f32);
    let mut streak_animation = use_motion(analytics.current_streak as f32);

    use_effect(move || {
        let progress = analytics.completion_rate * 100.0;
        overall_progress.animate_to(
            progress,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );

        streak_animation.animate_to(
            analytics.current_streak as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/plan_view/style.css")
        }

        div { class: "plan-view-container",
            // Error message display
            if let Some(error) = error_message.read().as_ref() {
                div { class: "error-message",
                    style: "background: var(--plan-danger-light); color: var(--plan-danger); padding: var(--space-4); border-radius: var(--radius-md); margin-bottom: var(--space-6); border: 1px solid var(--plan-danger);",
                    "⚠️ {error}"
                }
            }

            // Loading overlay for structuring
            if *is_structuring.read() {
                div { class: "loading-overlay",
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 9999;",
                    div { class: "loading-content",
                        style: "background: var(--plan-card); padding: var(--space-8); border-radius: var(--radius-lg); text-align: center; box-shadow: var(--shadow-xl);",
                        div { style: "font-size: 3rem; margin-bottom: var(--space-4);", "🧠" }
                        h3 { style: "margin: 0 0 var(--space-2) 0; color: var(--plan-text-primary);", "Analyzing Course Structure" }
                        p { style: "margin: 0; color: var(--plan-text-secondary);", "Please wait while we analyze your course content..." }
                    }
                }
            }
            // Course Overview Section
            section { class: "course-overview",
                div { class: "course-overview-header",
                    div { class: "course-overview-content",
                            h1 { class: "course-title", "{current_course.name}" }
                            p { class: "course-subtitle",
                                if current_course.is_structured() {
                                    "Your personalized learning journey • {analytics.sessions_total} sessions planned"
                                } else {
                                    "Ready to structure your learning journey • {current_course.video_count()} videos to analyze"
                                }
                            }

                        // Smart recommendations
                        if let Some(focus_session) = &today_focus {
                            div { class: "today-focus-badge",
                                Icon { width: 16, height: 16, fill: "currentColor", icon: MdTrendingUp }
                                span { "Today's Focus: {focus_session.title}" }
                            }
                        }
                    }

                    div { class: "course-actions",
                        button {
                            class: "course-action-btn",
                            onclick: move |_| sidebar_open.set(!sidebar_open()),
                            Icon { width: 16, height: 16, fill: "currentColor", icon: MdTrendingUp }
                            span { "Analytics" }
                        }
                        button {
                            class: "course-action-btn",
                            onclick: move |_| {
                                toast.write().popup(ToastInfo::simple("Settings coming soon!"));
                            },
                            Icon { width: 16, height: 16, fill: "currentColor", icon: MdCreate }
                            span { "Customize" }
                        }
                        if !current_course.is_structured() {
                            button {
                                class: "course-action-btn",
                                style: "background: var(--plan-primary); color: var(--plan-text-inverse); font-weight: 600;",
                                disabled: *is_structuring.read(),
                                onclick: structure_course_action,
                                if *is_structuring.read() {
                                    "🔄 Analyzing Course..."
                                } else {
                                    "🧠 Structure Course"
                                }
                            }
                        }
                        button {
                            class: "course-action-btn",
                            onclick: move |_| {
                                if let Err(e) = navigate_to(app_state, Route::Dashboard) {
                                    log::error!("Failed to navigate to dashboard: {:?}", e);
                                }
                            },
                            "← Dashboard"
                        }
                    }
                }

                // Course Metrics - only show if course is structured
                if current_course.is_structured() {
                    div { class: "course-metrics",
                        div { class: "metric-card",
                            div { class: "metric-icon",
                                Icon { width: 24, height: 24, fill: "currentColor", icon: MdCheckCircle }
                            }
                            div { class: "metric-progress",
                                ProgressRing {
                                    progress: overall_progress.get_value() / 100.0,
                                    size: 60,
                                    stroke_width: 4
                                }
                            }
                            h3 { class: "metric-value", "{overall_progress.get_value().round() as i32}%" }
                            p { class: "metric-label", "Overall Progress" }
                        }

                        div { class: "metric-card",
                            div { class: "metric-icon",
                                Icon { width: 24, height: 24, fill: "currentColor", icon: MdAccessTime }
                            }
                            h3 { class: "metric-value", "{format_duration(analytics.total_study_time)}" }
                            p { class: "metric-label", "Time Invested" }
                            div { class: "metric-trend",
                                if analytics.momentum_score > 0.7 {
                                    "🔥 Great momentum!"
                                } else if analytics.momentum_score > 0.4 {
                                    "📈 Building momentum"
                                } else {
                                    "💪 Let's get started!"
                                }
                            }
                        }

                        div { class: "metric-card",
                            div { class: "metric-icon",
                                Icon { width: 24, height: 24, fill: "currentColor", icon: MdTrendingUp }
                            }
                            h3 { class: "metric-value", "{streak_animation.get_value().round() as i32}" }
                            p { class: "metric-label", "Day Streak" }
                            if analytics.current_streak >= analytics.longest_streak {
                                div { class: "metric-trend", "🎉 Personal best!" }
                            }
                        }

                        div { class: "metric-card",
                            div { class: "metric-icon",
                                Icon { width: 24, height: 24, fill: "currentColor", icon: MdLibraryBooks }
                            }
                            h3 { class: "metric-value", "{analytics.sessions_completed}/{analytics.sessions_total}" }
                            p { class: "metric-label", "Sessions" }
                            div { class: "metric-trend",
                                if analytics.completion_rate > 0.8 {
                                    "🌟 Excellent progress!"
                                } else if analytics.completion_rate > 0.5 {
                                    "⚡ You're ahead of schedule!"
                                } else if analytics.completion_rate > 0.2 {
                                    "📚 Keep going!"
                                } else {
                                    "🚀 Ready to start?"
                                }
                            }
                        }
                    }
                }
            }

            // Study Plan Timeline
            section { class: "study-plan-section",
                div { class: "study-plan-header",
                    h2 { class: "study-plan-title", "Your Learning Timeline" }
                }

                if current_course.is_structured() && !study_sessions.is_empty() {
                    div { class: "study-timeline",
                        div { class: "timeline-line" }

                        for (_index, (module_name, sessions)) in group_sessions_by_module(&study_sessions).iter().enumerate() {
                            div { class: "timeline-section",
                                div { class: "section-icon",
                                    Icon { width: 24, height: 24, fill: "currentColor", icon: MdLibraryBooks }
                                }

                                div { class: "section-header",
                                    h3 { class: "section-title", "{module_name}" }
                                    div { class: "section-meta",
                                        span { "{sessions.len()} sessions" }
                                        span { "•" }
                                        span { "{format_duration(sessions.iter().map(|s| s.estimated_duration).sum())}" }
                                        span { "•" }
                                        span {
                                            match calculate_module_difficulty(sessions) {
                                                DifficultyLevel::Easy => "Easy",
                                                DifficultyLevel::Medium => "Medium",
                                                DifficultyLevel::Hard => "Advanced",
                                            }
                                        }
                                    }
                                }

                                div { class: "study-sessions",
                                    for session in sessions {
                                        StudySessionCard {
                                            session: session.clone(),
                                            on_start: move |session_id| {
                                                selected_session.set(Some(session_id));
                                                toast.write().popup(ToastInfo::simple("Starting focus session..."));
                                            },
                                            on_complete: move |_session_id| {
                                                toast.write().popup(ToastInfo::simple("Session completed! 🎉"));
                                            },
                                            on_bookmark: move |_session_id| {
                                                toast.write().popup(ToastInfo::simple("Bookmarked for review"));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if !current_course.is_structured() {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "🧠" }
                        h3 { class: "empty-state-title", "Course Structure Needed" }
                        p { class: "empty-state-description",
                            "Analyze the course content to create a structured learning path with modules and sessions."
                        }
                        button {
                            class: "course-primary-action",
                            disabled: *is_structuring.read(),
                            onclick: structure_course_action,
                            if *is_structuring.read() {
                                "🔄 Analyzing Course Structure..."
                            } else {
                                "🧠 Structure Course"
                            }
                        }
                    }
                } else {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "📋" }
                        h3 { class: "empty-state-title", "No Study Sessions Yet" }
                        p { class: "empty-state-description",
                            "Generate your personalized study plan to start learning."
                        }
                    }
                }
            }

            // Achievement Section - only show if course is structured
            if current_course.is_structured() && !achievements.is_empty() {
                section { class: "achievement-section",
                    div { class: "achievement-header",
                        Icon { width: 24, height: 24, fill: "currentColor", icon: MdTrendingUp }
                        h3 { class: "achievement-title", "Achievements" }
                    }

                    div { class: "achievement-grid",
                        for achievement in &achievements {
                            div {
                                class: format!("achievement-badge {}", if achievement.unlocked { "unlocked" } else { "" }),
                                title: achievement.description.clone(),
                                div { class: "badge-icon", "{achievement.icon}" }
                                div { class: "badge-label", "{achievement.title}" }
                                if !achievement.unlocked && achievement.progress > 0.0 {
                                    div { class: "badge-progress", "{(achievement.progress * 100.0).round() as i32}%" }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Analytics Sidebar
        if sidebar_open() {
            div { class: "sidebar open",
                div { class: "sidebar-header",
                    h3 { class: "sidebar-title", "Learning Analytics" }
                    button {
                        class: "sidebar-close",
                        onclick: move |_| sidebar_open.set(false),
                        "✕"
                    }
                }

                div { class: "analytics-section",
                    h4 { "Performance Insights" }
                    div { class: "analytics-item",
                        span { class: "analytics-label", "Completion Rate" }
                        span { class: "analytics-value", "{(analytics.completion_rate * 100.0).round() as i32}%" }
                    }
                    div { class: "analytics-item",
                        span { class: "analytics-label", "Avg Session Time" }
                        span { class: "analytics-value", "{format_duration(analytics.average_session_time)}" }
                    }
                    div { class: "analytics-item",
                        span { class: "analytics-label", "Study Momentum" }
                        span { class: "analytics-value",
                            if analytics.momentum_score > 0.8 { "🔥 High" }
                            else if analytics.momentum_score > 0.5 { "📈 Medium" }
                            else { "💪 Building" }
                        }
                    }
                }

                div { class: "streak-section",
                    h4 { "Study Streak" }
                    div { class: "streak-counter",
                        h2 { class: "streak-number", "{analytics.current_streak}" }
                        p { class: "streak-label", "days in a row" }
                    }

                    div { class: "streak-calendar",
                        for day in 0..14 {
                            div {
                                class: format!("streak-day {}",
                                    if day == 13 { "today" }
                                    else if day < analytics.current_streak { "completed" }
                                    else { "" }
                                ),
                                "{day + 1}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StudySessionCard(
    session: StudySession,
    on_start: EventHandler<Uuid>,
    on_complete: EventHandler<Uuid>,
    on_bookmark: EventHandler<Uuid>,
) -> Element {
    let status_class = match session.status {
        SessionStatus::Completed => "completed",
        SessionStatus::InProgress => "in-progress",
        SessionStatus::TodayFocus => "today-focus",
        SessionStatus::Overdue => "overdue",
        SessionStatus::Pending => "",
    };

    let difficulty_class = match session.difficulty {
        DifficultyLevel::Easy => "easy",
        DifficultyLevel::Medium => "medium",
        DifficultyLevel::Hard => "hard",
    };

    let completion_percentage = session
        .completed_items
        .iter()
        .filter(|&&completed| completed)
        .count() as f32
        / session.completed_items.len().max(1) as f32;

    rsx! {
        div { class: format!("study-session-card {}", status_class),
            div { class: "session-header",
                h4 { class: "session-title", "{session.title}" }
            }

            div { class: format!("session-status {}", status_class),
                div { class: format!("status-indicator {}", status_class) }
                span {
                    match session.status {
                        SessionStatus::Completed => "READY",
                        SessionStatus::InProgress => "IN PROGRESS",
                        SessionStatus::TodayFocus => "TODAY'S FOCUS",
                        SessionStatus::Overdue => "OVERDUE",
                        SessionStatus::Pending => "PENDING",
                    }
                }
            }

            div { class: "session-meta",
                div { class: "meta-item",
                    Icon { width: 12, height: 12, fill: "currentColor", icon: MdAccessTime }
                    span { "{format_duration(session.estimated_duration)}" }
                }
                div { class: "meta-item",
                    Icon { width: 12, height: 12, fill: "currentColor", icon: MdSchedule }
                    span { "{session.scheduled_date.format(\"%b %d\")}" }
                }
                div { class: format!("difficulty-badge {}", difficulty_class),
                    match session.difficulty {
                        DifficultyLevel::Easy => "Easy",
                        DifficultyLevel::Medium => "Medium",
                        DifficultyLevel::Hard => "Hard",
                    }
                }
            }

            if !session.description.is_empty() {
                p { class: "session-description", "{session.description}" }
            }

            div { class: "session-content",
                ul { class: "content-list",
                    for (_index, (item, &completed)) in session.content_items.iter().zip(session.completed_items.iter()).enumerate() {
                        li { class: "content-item",
                            div {
                                class: format!("content-checkbox {}", if completed { "checked" } else { "" }),
                                onclick: move |_| {
                                    // TODO: Implement checkbox toggle functionality
                                }
                            }
                            span { "{item}" }
                        }
                    }
                }

                if completion_percentage > 0.0 && completion_percentage < 1.0 {
                    div { class: "session-progress",
                        div {
                            class: "progress-bar",
                            style: format!("width: {}%", completion_percentage * 100.0),
                        }
                        span { class: "progress-text", "{(completion_percentage * 100.0).round() as i32}% complete" }
                    }
                }
            }

            div { class: "session-actions",
                match session.status {
                    SessionStatus::Completed => rsx! {
                        button {
                            class: "session-primary-btn completed",
                            disabled: true,
                            Icon { width: 16, height: 16, fill: "currentColor", icon: MdCheckCircle }
                            span { "Completed" }
                        }
                    },
                    SessionStatus::InProgress => rsx! {
                        button {
                            class: "session-primary-btn",
                            onclick: move |_| {
                                on_start.call(session.id);
                            },
                            Icon { width: 16, height: 16, fill: "currentColor", icon: MdPlayCircleFilled }
                            span { "Continue" }
                        }
                    },
                    _ => rsx! {
                        button {
                            class: "session-primary-btn",
                            onclick: move |_| {
                                on_start.call(session.id);
                            },
                            Icon { width: 16, height: 16, fill: "currentColor", icon: MdPlayArrow }
                            span {
                                if matches!(session.status, SessionStatus::TodayFocus) {
                                    "Start Today's Focus"
                                } else {
                                    "Start Session"
                                }
                            }
                        }
                    }
                }

                button {
                    class: "session-secondary-btn",
                    onclick: move |_| {
                        on_bookmark.call(session.id);
                    },
                    title: "Bookmark for review",
                    Icon { width: 16, height: 16, fill: "currentColor", icon: MdBookmark }
                }
            }
        }
    }
}

#[component]
fn ProgressRing(progress: f32, size: i32, stroke_width: i32) -> Element {
    let radius = (size - stroke_width) / 2;
    let circumference = 2.0 * std::f32::consts::PI * radius as f32;
    let stroke_dasharray = circumference;
    let stroke_dashoffset = circumference * (1.0 - progress.clamp(0.0, 1.0));

    rsx! {
        div { class: "progress-ring",
            svg {
                width: "{size}",
                height: "{size}",
                circle {
                    class: "progress-ring-bg",
                    cx: "{size / 2}",
                    cy: "{size / 2}",
                    r: "{radius}",
                }
                circle {
                    class: "progress-ring-fill",
                    cx: "{size / 2}",
                    cy: "{size / 2}",
                    r: "{radius}",
                    "stroke-dasharray": "{stroke_dasharray}",
                    "stroke-dashoffset": "{stroke_dashoffset}",
                }
            }
            div { class: "progress-ring-text", "{(progress * 100.0).round() as i32}%" }
        }
    }
}

// Helper functions
fn generate_study_sessions(course: &Course) -> Vec<StudySession> {
    let mut sessions = Vec::new();
    let mut session_date = Local::now();

    if let Some(structure) = &course.structure {
        for (module_index, module) in structure.modules.iter().enumerate() {
            // Create sessions for each module
            let sessions_per_module = (module.sections.len() / 2).max(1);

            for session_index in 0..sessions_per_module {
                let start_section = session_index * 2;
                let end_section = (start_section + 2).min(module.sections.len());

                let content_items: Vec<String> = module.sections[start_section..end_section]
                    .iter()
                    .map(|section| section.title.clone())
                    .collect();

                let session = StudySession {
                    id: Uuid::new_v4(),
                    title: format!("{} - Part {}", module.title, session_index + 1),
                    description: format!(
                        "Study sections {}-{} of {}",
                        start_section + 1,
                        end_section,
                        module.title
                    ),
                    estimated_duration: Duration::minutes((content_items.len() * 15) as i64),
                    difficulty: match module.title.as_str() {
                        title if title.contains("Advanced") || title.contains("Expert") => {
                            DifficultyLevel::Hard
                        }
                        title if title.contains("Basic") || title.contains("Intro") => {
                            DifficultyLevel::Easy
                        }
                        _ => DifficultyLevel::Medium,
                    },
                    status: if module_index == 0 && session_index == 0 {
                        SessionStatus::TodayFocus
                    } else if session_index < module_index {
                        SessionStatus::Completed
                    } else {
                        SessionStatus::Pending
                    },
                    scheduled_date: session_date,
                    content_items: content_items.clone(),
                    completed_items: vec![false; content_items.len()],
                    module_name: module.title.clone(),
                };

                sessions.push(session);
                session_date = session_date + Duration::days(2); // Space sessions 2 days apart
            }
        }
    }

    sessions
}

fn calculate_analytics(sessions: &[StudySession]) -> LearningAnalytics {
    let completed_sessions = sessions
        .iter()
        .filter(|s| s.status == SessionStatus::Completed)
        .count();
    let total_sessions = sessions.len();
    let completion_rate = if total_sessions > 0 {
        completed_sessions as f32 / total_sessions as f32
    } else {
        0.0
    };

    LearningAnalytics {
        total_study_time: Duration::hours(completed_sessions as i64 * 2), // Assume 2 hours per session
        current_streak: 5,                                                // Mock data
        longest_streak: 7,                                                // Mock data
        completion_rate,
        sessions_completed: completed_sessions as i32,
        sessions_total: total_sessions as i32,
        average_session_time: Duration::minutes(90),
        momentum_score: completion_rate * 0.8 + 0.2, // Simple momentum calculation
    }
}

fn generate_achievements(analytics: &LearningAnalytics) -> Vec<Achievement> {
    vec![
        Achievement {
            id: "first_session".to_string(),
            title: "First Steps".to_string(),
            description: "Complete your first study session".to_string(),
            icon: "🎯".to_string(),
            unlocked: analytics.sessions_completed > 0,
            progress: if analytics.sessions_completed > 0 {
                1.0
            } else {
                0.0
            },
        },
        Achievement {
            id: "week_streak".to_string(),
            title: "Week Warrior".to_string(),
            description: "Study for 7 days in a row".to_string(),
            icon: "🔥".to_string(),
            unlocked: analytics.current_streak >= 7,
            progress: (analytics.current_streak as f32 / 7.0).clamp(0.0, 1.0),
        },
        Achievement {
            id: "half_complete".to_string(),
            title: "Halfway Hero".to_string(),
            description: "Complete 50% of the course".to_string(),
            icon: "⭐".to_string(),
            unlocked: analytics.completion_rate >= 0.5,
            progress: analytics.completion_rate * 2.0,
        },
        Achievement {
            id: "speed_learner".to_string(),
            title: "Speed Learner".to_string(),
            description: "Complete sessions faster than average".to_string(),
            icon: "⚡".to_string(),
            unlocked: analytics.momentum_score > 0.8,
            progress: analytics.momentum_score,
        },
        Achievement {
            id: "course_master".to_string(),
            title: "Course Master".to_string(),
            description: "Complete the entire course".to_string(),
            icon: "🏆".to_string(),
            unlocked: analytics.completion_rate >= 1.0,
            progress: analytics.completion_rate,
        },
        Achievement {
            id: "consistency_king".to_string(),
            title: "Consistency King".to_string(),
            description: "Study regularly for 30 days".to_string(),
            icon: "👑".to_string(),
            unlocked: analytics.longest_streak >= 30,
            progress: (analytics.longest_streak as f32 / 30.0).clamp(0.0, 1.0),
        },
    ]
}

fn group_sessions_by_module(sessions: &[StudySession]) -> Vec<(String, Vec<StudySession>)> {
    let mut modules: HashMap<String, Vec<StudySession>> = HashMap::new();

    for session in sessions {
        modules
            .entry(session.module_name.clone())
            .or_default()
            .push(session.clone());
    }

    modules.into_iter().collect()
}

fn calculate_module_difficulty(sessions: &[StudySession]) -> DifficultyLevel {
    let hard_count = sessions
        .iter()
        .filter(|s| matches!(s.difficulty, DifficultyLevel::Hard))
        .count();
    let total = sessions.len();

    if hard_count as f32 / total as f32 > 0.5 {
        DifficultyLevel::Hard
    } else if hard_count > 0 {
        DifficultyLevel::Medium
    } else {
        DifficultyLevel::Easy
    }
}

fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
