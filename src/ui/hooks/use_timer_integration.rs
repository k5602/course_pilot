use crate::ui::components::timer::timer_statistics::{TimerSession, TimerStats};
use dioxus::prelude::*;
use uuid::Uuid;

/// Hook for integrating Pomodoro timer with video sessions
pub fn use_timer_integration() -> TimerIntegration {
    let active_timer_session = use_signal(|| None::<TimerSession>);
    let timer_stats = use_signal(TimerStats::new);

    TimerIntegration {
        active_timer_session,
        timer_stats,
    }
}

pub struct TimerIntegration {
    pub active_timer_session: Signal<Option<TimerSession>>,
    pub timer_stats: Signal<TimerStats>,
}

impl TimerIntegration {
    /// Start a timer session for a specific video
    pub fn start_video_session(&mut self, course_id: Uuid, video_title: String) {
        // This would typically create a new timer session
        // For now, we'll just store the context
        log::info!("Starting timer session for video: {video_title} in course: {course_id}");
    }

    /// Complete a timer session and update statistics
    pub fn complete_session(&mut self, session: TimerSession) {
        self.timer_stats.with_mut(|stats| {
            stats.add_session(&session);
        });

        // Clear active session
        self.active_timer_session.set(None);

        log::info!("Timer session completed: {session:?}");
    }

    /// Get current timer statistics
    pub fn get_stats(&self) -> TimerStats {
        self.timer_stats.read().clone()
    }

    /// Check if there's an active timer session
    pub fn has_active_session(&self) -> bool {
        self.active_timer_session.read().is_some()
    }

    /// Get the active session if any
    pub fn get_active_session(&self) -> Option<TimerSession> {
        self.active_timer_session.read().clone()
    }
}
