use chrono::Utc;
use dioxus::prelude::*;
use std::time::Duration;
use uuid::Uuid;

use crate::ui::components::timer::timer_settings::PomodoroSettings;
use crate::ui::components::timer::timer_statistics::{TimerSession, TimerStats, TimerType};
use crate::ui::components::timer::{TimerSettings, TimerStatistics};
use crate::ui::components::{ToastVariant, show_toast};

#[derive(Clone, Copy, PartialEq)]
enum TimerState {
    Stopped,
    Running,
    Paused,
}

#[derive(Clone, Copy, PartialEq)]
enum TimerMode {
    Work,
    ShortBreak,
    LongBreak,
}

impl TimerMode {
    fn duration(&self, settings: &PomodoroSettings) -> Duration {
        match self {
            TimerMode::Work => Duration::from_secs(settings.work_duration_minutes as u64 * 60),
            TimerMode::ShortBreak => {
                Duration::from_secs(settings.short_break_duration_minutes as u64 * 60)
            },
            TimerMode::LongBreak => {
                Duration::from_secs(settings.long_break_duration_minutes as u64 * 60)
            },
        }
    }

    fn label(&self) -> &'static str {
        match self {
            TimerMode::Work => "Focus Time",
            TimerMode::ShortBreak => "Short Break",
            TimerMode::LongBreak => "Long Break",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            TimerMode::Work => "🍅",
            TimerMode::ShortBreak => "☕",
            TimerMode::LongBreak => "🌟",
        }
    }

    fn to_timer_type(&self) -> TimerType {
        match self {
            TimerMode::Work => TimerType::Work,
            TimerMode::ShortBreak | TimerMode::LongBreak => TimerType::Break,
        }
    }

    fn next(&self, completed_work_sessions: u32, settings: &PomodoroSettings) -> Self {
        match self {
            TimerMode::Work => {
                if completed_work_sessions > 0
                    && completed_work_sessions % settings.sessions_until_long_break == 0
                {
                    TimerMode::LongBreak
                } else {
                    TimerMode::ShortBreak
                }
            },
            TimerMode::ShortBreak | TimerMode::LongBreak => TimerMode::Work,
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct PomodoroTimerProps {
    #[props(optional)]
    pub course_id: Option<Uuid>,
    #[props(optional)]
    pub video_title: Option<String>,
    #[props(optional)]
    pub on_session_complete: Option<EventHandler<TimerSession>>,
}

#[component]
pub fn PomodoroTimer(props: PomodoroTimerProps) -> Element {
    // Clone props values to avoid ownership issues
    let course_id = props.course_id;
    let video_title = props.video_title.clone();
    let on_session_complete = props.on_session_complete;

    // Create additional clones for different closures
    let video_title_for_effect = video_title.clone();
    let video_title_for_handler = video_title.clone();
    let video_title_for_ui = video_title.clone();

    // Settings and statistics (using regular signals for now)
    let mut settings = use_signal(PomodoroSettings::default);
    let mut stats = use_signal(TimerStats::new);

    // Timer state
    let mut timer_state = use_signal(|| TimerState::Stopped);
    let mut timer_mode = use_signal(|| TimerMode::Work);
    let mut remaining_time = use_signal(|| TimerMode::Work.duration(&settings()));
    let mut completed_work_sessions = use_signal(|| 0u32);
    let mut current_session = use_signal(|| None::<TimerSession>);

    // UI state
    let mut show_settings = use_signal(|| false);
    let mut show_statistics = use_signal(|| false);

    // Initialize persistent data
    use_effect(move || {
        // In a real implementation, load from persistent storage
        // For now, we'll use the default values
        remaining_time.set(timer_mode().duration(&settings()));
    });

    // Timer effect using modern Dioxus patterns
    use_effect(move || {
        if timer_state() == TimerState::Running {
            let course_id_clone = course_id;
            let video_title_clone = video_title_for_effect.clone();
            let on_session_complete_clone = on_session_complete;

            spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    if timer_state() != TimerState::Running {
                        break;
                    }

                    let current_remaining = remaining_time();
                    if current_remaining <= Duration::from_secs(1) {
                        // Timer finished
                        timer_state.set(TimerState::Stopped);

                        // Complete current session
                        if let Some(mut session) = current_session() {
                            session.end_time = Some(Utc::now());
                            session.completed = true;

                            // Update statistics
                            stats.with_mut(|s| s.add_session(&session));

                            // Call completion handler
                            if let Some(handler) = &on_session_complete_clone {
                                handler.call(session.clone());
                            }

                            // Update work session counter
                            if timer_mode() == TimerMode::Work {
                                completed_work_sessions.set(completed_work_sessions() + 1);
                            }
                        }

                        // Show desktop notification
                        if settings().notifications_enabled {
                            show_desktop_notification(&timer_mode(), &settings());
                        }

                        // Determine next mode
                        let next_mode = timer_mode().next(completed_work_sessions(), &settings());

                        // Auto-start next session or wait for user
                        let should_auto_start = match next_mode {
                            TimerMode::Work => settings().auto_start_work,
                            TimerMode::ShortBreak | TimerMode::LongBreak => {
                                settings().auto_start_breaks
                            },
                        };

                        timer_mode.set(next_mode);
                        remaining_time.set(next_mode.duration(&settings()));

                        if should_auto_start {
                            // Start new session automatically
                            start_new_session(
                                &mut current_session,
                                next_mode,
                                course_id_clone,
                                video_title_clone.clone(),
                                next_mode.duration(&settings()),
                            );
                            timer_state.set(TimerState::Running);
                        } else {
                            // Wait for user to start
                            current_session.set(None);
                        }

                        // Show toast notification
                        show_toast(
                            format!(
                                "{} completed! {} is ready.",
                                timer_mode().label(),
                                next_mode.label()
                            ),
                            ToastVariant::Success,
                        );

                        break;
                    } else {
                        remaining_time.set(current_remaining - Duration::from_secs(1));
                    }
                }
            });
        }
    });

    let handle_start_pause = move |_| {
        match timer_state() {
            TimerState::Stopped => {
                // Start new session
                start_new_session(
                    &mut current_session,
                    timer_mode(),
                    course_id,
                    video_title_for_handler.clone(),
                    remaining_time(),
                );
                timer_state.set(TimerState::Running);
            },
            TimerState::Paused => {
                timer_state.set(TimerState::Running);
            },
            TimerState::Running => {
                timer_state.set(TimerState::Paused);
            },
        }
    };

    let handle_reset = move |_| {
        timer_state.set(TimerState::Stopped);
        remaining_time.set(timer_mode().duration(&settings()));
        current_session.set(None);
    };

    let handle_skip = move |_| {
        // Complete current session as skipped
        if let Some(mut session) = current_session() {
            session.end_time = Some(Utc::now());
            session.completed = false; // Mark as skipped
            stats.with_mut(|s| s.add_session(&session));
        }

        // Move to next mode
        let next_mode = timer_mode().next(completed_work_sessions(), &settings());
        timer_mode.set(next_mode);
        remaining_time.set(next_mode.duration(&settings()));
        timer_state.set(TimerState::Stopped);
        current_session.set(None);
    };

    let handle_settings_change = move |new_settings: PomodoroSettings| {
        settings.set(new_settings);
        // Update remaining time if timer is stopped
        if timer_state() == TimerState::Stopped {
            remaining_time.set(timer_mode().duration(&settings()));
        }
        show_settings.set(false);
    };

    let minutes = remaining_time().as_secs() / 60;
    let seconds = remaining_time().as_secs() % 60;
    let total_duration = timer_mode().duration(&settings());
    let progress = 1.0 - (remaining_time().as_secs() as f32 / total_duration.as_secs() as f32);

    // Use the UI clone for display
    let video_title_display = video_title_for_ui;

    rsx! {
        div { class: "space-y-4",
            // Main Timer Card
            div { class: "card bg-base-100 border border-base-300",
                div { class: "card-body p-6 text-center",
                    // Header with mode and session info
                    div { class: "flex items-center justify-between mb-4",
                        div { class: "flex items-center gap-2",
                            span { class: "text-2xl", "{timer_mode().icon()}" }
                            h3 { class: "font-semibold text-lg", "{timer_mode().label()}" }
                        }

                        div { class: "flex gap-1",
                            button {
                                class: "btn btn-ghost btn-sm",
                                onclick: move |_| show_statistics.set(!show_statistics()),
                                "📊"
                            }
                            button {
                                class: "btn btn-ghost btn-sm",
                                onclick: move |_| show_settings.set(!show_settings()),
                                "⚙️"
                            }
                        }
                    }

                    // Session context (if connected to video)
                    if let Some(video_title) = &video_title_display {
                        div { class: "text-sm text-base-content/70 mb-4",
                            "📹 {video_title}"
                        }
                    }

                    // Timer display
                    div { class: "relative mb-6",
                        div {
                            class: format!(
                                "radial-progress {}",
                                match timer_mode() {
                                    TimerMode::Work => "text-primary",
                                    TimerMode::ShortBreak => "text-secondary",
                                    TimerMode::LongBreak => "text-accent",
                                }
                            ),
                            style: "--value:{(progress * 100.0) as i32}; --size:10rem; --thickness:6px;",
                            div { class: "text-3xl font-mono font-bold",
                                "{minutes:02}:{seconds:02}"
                            }
                        }
                    }

                    // Progress indicator
                    div { class: "text-sm text-base-content/70 mb-4",
                        match timer_state() {
                            TimerState::Running => "Timer is running...",
                            TimerState::Paused => "Timer is paused",
                            TimerState::Stopped => "Ready to start",
                        }
                    }

                    // Controls
                    div { class: "flex justify-center gap-2 mb-4",
                        button {
                            class: format!(
                                "btn {}",
                                match timer_state() {
                                    TimerState::Running => "btn-warning",
                                    TimerState::Paused => "btn-primary",
                                    TimerState::Stopped => "btn-primary",
                                }
                            ),
                            onclick: handle_start_pause,
                            match timer_state() {
                                TimerState::Running => "Pause",
                                TimerState::Paused => "Resume",
                                TimerState::Stopped => "Start",
                            }
                        }

                        button {
                            class: "btn btn-ghost",
                            onclick: handle_reset,
                            disabled: timer_state() == TimerState::Stopped,
                            "Reset"
                        }

                        button {
                            class: "btn btn-outline",
                            onclick: handle_skip,
                            disabled: timer_state() == TimerState::Stopped,
                            "Skip"
                        }
                    }

                    // Session stats
                    div { class: "grid grid-cols-2 gap-4 text-sm",
                        div { class: "stat bg-base-200 rounded p-2",
                            div { class: "stat-title text-xs", "Work Sessions" }
                            div { class: "stat-value text-lg", "{completed_work_sessions()}" }
                        }

                        div { class: "stat bg-base-200 rounded p-2",
                            div { class: "stat-title text-xs", "Total Focus" }
                            div { class: "stat-value text-lg", "{stats().total_focus_time_minutes / 60}h" }
                        }
                    }
                }
            }

            // Settings Modal
            if show_settings() {
                div { class: "modal modal-open",
                    div { class: "modal-backdrop",
                        onclick: move |_| show_settings.set(false)
                    }
                    div { class: "modal-box max-w-4xl",
                        div { class: "flex justify-between items-center mb-4",
                            h3 { class: "font-bold text-lg", "Timer Settings" }
                            button {
                                class: "btn btn-sm btn-circle btn-ghost",
                                onclick: move |_| show_settings.set(false),
                                "✕"
                            }
                        }

                        TimerSettings {
                            settings: settings(),
                            on_settings_change: handle_settings_change
                        }
                    }
                }
            }

            // Statistics Modal
            if show_statistics() {
                div { class: "modal modal-open",
                    div { class: "modal-backdrop",
                        onclick: move |_| show_statistics.set(false)
                    }
                    div { class: "modal-box max-w-6xl",
                        div { class: "flex justify-between items-center mb-4",
                            h3 { class: "font-bold text-lg", "Timer Statistics" }
                            button {
                                class: "btn btn-sm btn-circle btn-ghost",
                                onclick: move |_| show_statistics.set(false),
                                "✕"
                            }
                        }

                        TimerStatistics { stats: stats() }
                    }
                }
            }
        }
    }
}

fn start_new_session(
    current_session: &mut Signal<Option<TimerSession>>,
    mode: TimerMode,
    course_id: Option<Uuid>,
    video_title: Option<String>,
    duration: Duration,
) {
    let session = TimerSession {
        id: Uuid::new_v4(),
        start_time: Utc::now(),
        end_time: None,
        duration_minutes: (duration.as_secs() / 60) as u32,
        timer_type: mode.to_timer_type(),
        completed: false,
        course_id,
        video_title,
    };

    current_session.set(Some(session));
}

fn show_desktop_notification(mode: &TimerMode, settings: &PomodoroSettings) {
    if !settings.notifications_enabled {
        return;
    }

    let message = match mode {
        TimerMode::Work => &settings.work_notification_message,
        TimerMode::ShortBreak | TimerMode::LongBreak => &settings.break_notification_message,
    };

    // Log notification for now (desktop notifications can be added later)
    log::info!("Timer notification: {} Complete! - {}", mode.label(), message);

    // Always show toast notification for immediate feedback
    show_toast(format!("{} complete! {}", mode.label(), message), ToastVariant::Info);
}
