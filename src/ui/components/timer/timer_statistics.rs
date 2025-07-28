use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerSession {
    pub id: uuid::Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: u32,
    pub timer_type: TimerType,
    pub completed: bool,
    pub course_id: Option<uuid::Uuid>,
    pub video_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimerType {
    Work,
    Break,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TimerStats {
    pub total_sessions: u32,
    pub completed_work_sessions: u32,
    pub completed_break_sessions: u32,
    pub total_focus_time_minutes: u32,
    pub total_break_time_minutes: u32,
    pub sessions_today: u32,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub sessions_by_date: HashMap<String, u32>, // Date string -> session count
    pub average_session_length: f32,
    pub productivity_score: f32,
}

impl TimerStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_session(&mut self, session: &TimerSession) {
        self.total_sessions += 1;

        if session.completed {
            match session.timer_type {
                TimerType::Work => {
                    self.completed_work_sessions += 1;
                    self.total_focus_time_minutes += session.duration_minutes;
                }
                TimerType::Break => {
                    self.completed_break_sessions += 1;
                    self.total_break_time_minutes += session.duration_minutes;
                }
            }

            // Update daily stats
            let date_key = session.start_time.format("%Y-%m-%d").to_string();
            *self.sessions_by_date.entry(date_key.clone()).or_insert(0) += 1;

            // Update today's sessions
            let today = Local::now().format("%Y-%m-%d").to_string();
            if date_key == today {
                self.sessions_today += 1;
            }

            // Update streak
            self.update_streak();

            // Recalculate averages
            self.recalculate_metrics();
        }
    }

    fn update_streak(&mut self) {
        // Simple streak calculation - consecutive days with sessions
        let today = Local::now().format("%Y-%m-%d").to_string();
        let yesterday = (Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        if self.sessions_by_date.get(&today).unwrap_or(&0) > &0 {
            if self.sessions_by_date.get(&yesterday).unwrap_or(&0) > &0 {
                self.current_streak += 1;
            } else {
                self.current_streak = 1;
            }
        } else {
            self.current_streak = 0;
        }

        if self.current_streak > self.longest_streak {
            self.longest_streak = self.current_streak;
        }
    }

    fn recalculate_metrics(&mut self) {
        if self.completed_work_sessions > 0 {
            self.average_session_length =
                self.total_focus_time_minutes as f32 / self.completed_work_sessions as f32;
        }

        // Simple productivity score based on completed vs total sessions
        if self.total_sessions > 0 {
            self.productivity_score =
                (self.completed_work_sessions as f32 / self.total_sessions as f32) * 100.0;
        }
    }

    pub fn get_weekly_stats(&self) -> Vec<(String, u32)> {
        let mut weekly_data = Vec::new();
        let today = Local::now();

        for i in 0..7 {
            let date = today - chrono::Duration::days(i);
            let date_key = date.format("%Y-%m-%d").to_string();
            let day_name = date.format("%a").to_string();
            let sessions = self.sessions_by_date.get(&date_key).unwrap_or(&0);
            weekly_data.push((day_name, *sessions));
        }

        weekly_data.reverse();
        weekly_data
    }
}

#[component]
pub fn TimerStatistics(stats: TimerStats) -> Element {
    let weekly_data = stats.get_weekly_stats();

    rsx! {
        div { class: "space-y-4",
            // Overview Cards
            div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                div { class: "stat bg-base-200 rounded-lg p-4",
                    div { class: "stat-title text-xs", "Total Sessions" }
                    div { class: "stat-value text-lg", "{stats.total_sessions}" }
                }

                div { class: "stat bg-base-200 rounded-lg p-4",
                    div { class: "stat-title text-xs", "Focus Time" }
                    div { class: "stat-value text-lg", "{stats.total_focus_time_minutes / 60}h {stats.total_focus_time_minutes % 60}m" }
                }

                div { class: "stat bg-base-200 rounded-lg p-4",
                    div { class: "stat-title text-xs", "Current Streak" }
                    div { class: "stat-value text-lg", "{stats.current_streak}" }
                    div { class: "stat-desc text-xs", "days" }
                }

                div { class: "stat bg-base-200 rounded-lg p-4",
                    div { class: "stat-title text-xs", "Productivity" }
                    div { class: "stat-value text-lg", "{stats.productivity_score:.1}%" }
                }
            }

            // Weekly Chart
            div { class: "card bg-base-100 border border-base-300",
                div { class: "card-body",
                    h3 { class: "card-title text-sm mb-4", "Weekly Activity" }

                    div { class: "flex justify-between items-end h-32 gap-2",
                        {weekly_data.iter().map(|(day, sessions)| {
                            let height = if *sessions > 0 {
                                (((*sessions as f32) / 10.0) * 100.0).min(100.0)
                            } else {
                                5.0
                            };

                            rsx! {
                                div { key: "{day}", class: "flex flex-col items-center gap-1",
                                    div {
                                        class: "bg-primary rounded-t w-8 transition-all duration-300",
                                        style: "height: {height}%",
                                    }
                                    div { class: "text-xs text-base-content/70", "{day}" }
                                    div { class: "text-xs font-semibold", "{sessions}" }
                                }
                            }
                        })}
                    }
                }
            }

            // Detailed Stats
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                div { class: "card bg-base-100 border border-base-300",
                    div { class: "card-body",
                        h3 { class: "card-title text-sm mb-3", "Session Breakdown" }

                        div { class: "space-y-2",
                            div { class: "flex justify-between",
                                span { class: "text-sm", "Work Sessions:" }
                                span { class: "font-semibold", "{stats.completed_work_sessions}" }
                            }
                            div { class: "flex justify-between",
                                span { class: "text-sm", "Break Sessions:" }
                                span { class: "font-semibold", "{stats.completed_break_sessions}" }
                            }
                            div { class: "flex justify-between",
                                span { class: "text-sm", "Avg Session:" }
                                span { class: "font-semibold", "{stats.average_session_length:.1}min" }
                            }
                            div { class: "flex justify-between",
                                span { class: "text-sm", "Today's Sessions:" }
                                span { class: "font-semibold", "{stats.sessions_today}" }
                            }
                        }
                    }
                }

                div { class: "card bg-base-100 border border-base-300",
                    div { class: "card-body",
                        h3 { class: "card-title text-sm mb-3", "Achievements" }

                        div { class: "space-y-2",
                            div { class: "flex items-center gap-2",
                                span { class: "text-lg", "🔥" }
                                span { class: "text-sm", "Longest Streak: {stats.longest_streak} days" }
                            }

                            if stats.completed_work_sessions >= 10 {
                                div { class: "flex items-center gap-2",
                                    span { class: "text-lg", "🏆" }
                                    span { class: "text-sm", "Focus Master (10+ sessions)" }
                                }
                            }

                            if stats.productivity_score >= 80.0 {
                                div { class: "flex items-center gap-2",
                                    span { class: "text-lg", "⭐" }
                                    span { class: "text-sm", "High Productivity (80%+)" }
                                }
                            }

                            if stats.current_streak >= 7 {
                                div { class: "flex items-center gap-2",
                                    span { class: "text-lg", "💪" }
                                    span { class: "text-sm", "Weekly Warrior" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
