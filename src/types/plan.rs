use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;

use super::course::DifficultyLevel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    pub id: Uuid,
    pub course_id: Uuid,
    pub settings: PlanSettings,
    pub items: Vec<PlanItem>,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanSettings {
    pub start_date: DateTime<Utc>,
    pub sessions_per_week: u8,
    pub session_length_minutes: u32,
    pub include_weekends: bool,
    pub advanced_settings: Option<AdvancedSchedulerSettings>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedSchedulerSettings {
    pub strategy: DistributionStrategy,
    pub difficulty_adaptation: bool,
    pub spaced_repetition_enabled: bool,
    pub cognitive_load_balancing: bool,
    pub user_experience_level: DifficultyLevel,
    pub custom_intervals: Option<Vec<i64>>,
    pub max_session_duration_minutes: Option<u32>,
    pub min_break_between_sessions_hours: Option<u32>,
    pub prioritize_difficult_content: bool,
    pub adaptive_pacing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum DistributionStrategy {
    ModuleBased,
    TimeBased,
    #[default]
    Hybrid,
    DifficultyBased,
    SpacedRepetition,
    Adaptive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegenerationStatus {
    Idle,
    InProgress { progress: f32, message: String },
    Completed,
    Failed { error: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanViewState {
    pub expanded_sessions: HashSet<usize>,
    pub selected_videos: HashSet<usize>,
    pub regeneration_status: RegenerationStatus,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoProgressUpdate {
    pub plan_id: Uuid,
    pub session_index: usize,
    pub video_index: usize,
    pub completed: bool,
    pub timestamp: DateTime<Utc>,
}

impl VideoProgressUpdate {
    pub fn new(plan_id: Uuid, session_index: usize, video_index: usize, completed: bool) -> Self {
        Self { plan_id, session_index, video_index, completed, timestamp: Utc::now() }
    }

    pub fn completed(plan_id: Uuid, session_index: usize, video_index: usize) -> Self {
        Self::new(plan_id, session_index, video_index, true)
    }

    pub fn uncompleted(plan_id: Uuid, session_index: usize, video_index: usize) -> Self {
        Self::new(plan_id, session_index, video_index, false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanItem {
    pub date: DateTime<Utc>,
    pub module_title: String,
    pub section_title: String,
    pub video_indices: Vec<usize>,
    pub completed: bool,
    #[serde(
        serialize_with = "crate::types::course::serialize_duration_as_secs",
        deserialize_with = "crate::types::course::deserialize_duration_from_secs"
    )]
    pub total_duration: Duration,
    #[serde(
        serialize_with = "crate::types::course::serialize_duration_as_secs",
        deserialize_with = "crate::types::course::deserialize_duration_from_secs"
    )]
    pub estimated_completion_time: Duration,
    pub overflow_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanItemIdentifier {
    pub plan_id: Uuid,
    pub item_index: usize,
}

pub trait PlanExt {
    fn get_item_identifier(&self, index: usize) -> PlanItemIdentifier;
    fn update_item_completion(&mut self, index: usize, completed: bool) -> Result<(), String>;
    fn calculate_progress(&self) -> (usize, usize, f32);
}

impl Plan {
    pub fn new(course_id: Uuid, settings: PlanSettings) -> Self {
        Self { id: Uuid::new_v4(), course_id, settings, items: Vec::new(), created_at: Utc::now() }
    }

    pub fn total_sessions(&self) -> usize {
        self.items.len()
    }

    pub fn completed_sessions(&self) -> usize {
        self.items.iter().filter(|item| item.completed).count()
    }

    pub fn progress_percentage(&self) -> f32 {
        if self.items.is_empty() {
            0.0
        } else {
            (self.completed_sessions() as f32 / self.total_sessions() as f32) * 100.0
        }
    }
}

impl PlanExt for Plan {
    fn get_item_identifier(&self, index: usize) -> PlanItemIdentifier {
        PlanItemIdentifier::new(self.id, index)
    }

    fn update_item_completion(&mut self, index: usize, completed: bool) -> Result<(), String> {
        if let Some(item) = self.items.get_mut(index) {
            item.completed = completed;
            Ok(())
        } else {
            Err(format!("Plan item index {index} out of bounds"))
        }
    }

    fn calculate_progress(&self) -> (usize, usize, f32) {
        let total_count = self.items.len();
        let completed_count = self.items.iter().filter(|item| item.completed).count();
        let percentage = if total_count > 0 {
            (completed_count as f32 / total_count as f32) * 100.0
        } else {
            0.0
        };

        (completed_count, total_count, percentage)
    }
}

impl PlanItemIdentifier {
    pub fn new(plan_id: Uuid, item_index: usize) -> Self {
        Self { plan_id, item_index }
    }
}

impl Default for AdvancedSchedulerSettings {
    fn default() -> Self {
        Self {
            strategy: DistributionStrategy::Hybrid,
            difficulty_adaptation: true,
            spaced_repetition_enabled: false,
            cognitive_load_balancing: true,
            user_experience_level: DifficultyLevel::Intermediate,
            custom_intervals: None,
            max_session_duration_minutes: None,
            min_break_between_sessions_hours: None,
            prioritize_difficult_content: false,
            adaptive_pacing: true,
        }
    }
}

impl AdvancedSchedulerSettings {
    pub fn with_strategy(strategy: DistributionStrategy) -> Self {
        Self { strategy, ..Self::default() }
    }

    pub fn for_beginner() -> Self {
        Self {
            strategy: DistributionStrategy::SpacedRepetition,
            difficulty_adaptation: true,
            spaced_repetition_enabled: true,
            cognitive_load_balancing: true,
            user_experience_level: DifficultyLevel::Beginner,
            prioritize_difficult_content: false,
            adaptive_pacing: true,
            ..Self::default()
        }
    }

    pub fn for_advanced() -> Self {
        Self {
            strategy: DistributionStrategy::Adaptive,
            difficulty_adaptation: true,
            spaced_repetition_enabled: false,
            cognitive_load_balancing: false,
            user_experience_level: DifficultyLevel::Advanced,
            prioritize_difficult_content: true,
            adaptive_pacing: true,
            ..Self::default()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.spaced_repetition_enabled && self.strategy != DistributionStrategy::SpacedRepetition
        {
            return Err(
                "Spaced repetition enabled but strategy is not SpacedRepetition".to_string()
            );
        }

        if let Some(max_duration) = self.max_session_duration_minutes {
            if !(15..=300).contains(&max_duration) {
                return Err("Session duration must be between 15 and 300 minutes".to_string());
            }
        }

        if let Some(min_break) = self.min_break_between_sessions_hours {
            if min_break > 168 {
                return Err("Minimum break between sessions cannot exceed 1 week".to_string());
            }
        }

        if let Some(ref intervals) = self.custom_intervals {
            if intervals.is_empty() {
                return Err("Custom intervals cannot be empty".to_string());
            }
            if intervals.iter().any(|&i| i <= 0) {
                return Err("All custom intervals must be positive".to_string());
            }
        }

        Ok(())
    }

    pub fn recommend_for_course(
        user_level: DifficultyLevel,
        course_complexity: DifficultyLevel,
        total_duration_hours: f32,
    ) -> Self {
        let strategy = match (user_level, course_complexity) {
            (DifficultyLevel::Beginner, _) => DistributionStrategy::SpacedRepetition,
            (
                DifficultyLevel::Intermediate,
                DifficultyLevel::Advanced | DifficultyLevel::Expert,
            ) => DistributionStrategy::Adaptive,
            (DifficultyLevel::Advanced | DifficultyLevel::Expert, _) => {
                DistributionStrategy::Hybrid
            },
            _ => DistributionStrategy::Hybrid,
        };

        let spaced_repetition = matches!(user_level, DifficultyLevel::Beginner);
        let prioritize_difficult =
            matches!(user_level, DifficultyLevel::Advanced | DifficultyLevel::Expert);

        let max_session_duration = if total_duration_hours > 20.0 {
            Some(90)
        } else if total_duration_hours < 5.0 {
            Some(45)
        } else {
            Some(60)
        };

        Self {
            strategy,
            difficulty_adaptation: true,
            spaced_repetition_enabled: spaced_repetition,
            cognitive_load_balancing: true,
            user_experience_level: user_level,
            max_session_duration_minutes: max_session_duration,
            prioritize_difficult_content: prioritize_difficult,
            adaptive_pacing: true,
            ..Self::default()
        }
    }
}

impl DistributionStrategy {
    pub fn all() -> Vec<Self> {
        vec![
            Self::ModuleBased,
            Self::TimeBased,
            Self::Hybrid,
            Self::DifficultyBased,
            Self::SpacedRepetition,
            Self::Adaptive,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ModuleBased => "Module-based",
            Self::TimeBased => "Time-based",
            Self::Hybrid => "Hybrid",
            Self::DifficultyBased => "Difficulty-based",
            Self::SpacedRepetition => "Spaced Repetition",
            Self::Adaptive => "Adaptive",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::ModuleBased => "Respects module boundaries and logical content grouping",
            Self::TimeBased => "Focuses on even time distribution across sessions",
            Self::Hybrid => "Balances both module structure and time constraints",
            Self::DifficultyBased => "Adapts pacing based on content difficulty",
            Self::SpacedRepetition => "Optimizes for memory retention with review sessions",
            Self::Adaptive => "AI-driven scheduling based on learning patterns",
        }
    }
}

impl DifficultyLevel {
    pub fn all() -> Vec<Self> {
        vec![Self::Beginner, Self::Intermediate, Self::Advanced, Self::Expert]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
            Self::Expert => "Expert",
        }
    }
}

impl Default for PlanViewState {
    fn default() -> Self {
        Self {
            expanded_sessions: HashSet::new(),
            selected_videos: HashSet::new(),
            regeneration_status: RegenerationStatus::Idle,
            last_update: Utc::now(),
        }
    }
}

impl PlanViewState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_session(&mut self, session_index: usize) {
        if self.expanded_sessions.contains(&session_index) {
            self.expanded_sessions.remove(&session_index);
        } else {
            self.expanded_sessions.insert(session_index);
        }
        self.last_update = Utc::now();
    }

    pub fn is_session_expanded(&self, session_index: usize) -> bool {
        self.expanded_sessions.contains(&session_index)
    }

    pub fn toggle_video_selection(&mut self, video_index: usize) {
        if self.selected_videos.contains(&video_index) {
            self.selected_videos.remove(&video_index);
        } else {
            self.selected_videos.insert(video_index);
        }
        self.last_update = Utc::now();
    }

    pub fn is_video_selected(&self, video_index: usize) -> bool {
        self.selected_videos.contains(&video_index)
    }

    pub fn set_regeneration_status(&mut self, status: RegenerationStatus) {
        self.regeneration_status = status;
        self.last_update = Utc::now();
    }

    pub fn clear_selections(&mut self) {
        self.expanded_sessions.clear();
        self.selected_videos.clear();
        self.last_update = Utc::now();
    }

    pub fn expanded_session_count(&self) -> usize {
        self.expanded_sessions.len()
    }

    pub fn selected_video_count(&self) -> usize {
        self.selected_videos.len()
    }
}

pub mod duration_utils {
    use std::time::Duration;

    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            if minutes > 0 { format!("{hours}h {minutes}m") } else { format!("{hours}h") }
        } else if minutes > 0 {
            format!("{minutes}m")
        } else {
            format!("{seconds}s")
        }
    }

    pub fn format_duration_verbose(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;

        if hours > 0 {
            if minutes > 0 {
                format!("{hours} hours {minutes} minutes")
            } else {
                format!("{hours} hours")
            }
        } else if minutes > 0 {
            format!("{minutes} minutes")
        } else {
            format!("{total_seconds} seconds")
        }
    }

    pub fn format_duration_decimal_hours(duration: Duration) -> String {
        let hours = duration.as_secs() as f32 / 3600.0;
        if hours >= 1.0 {
            format!("{hours:.1} hours")
        } else {
            let minutes = duration.as_secs() / 60;
            format!("{minutes} minutes")
        }
    }

    pub fn is_duration_excessive(duration: Duration, session_limit_minutes: u32) -> bool {
        let session_limit = Duration::from_secs(session_limit_minutes as u64 * 60);
        duration > session_limit
    }

    pub fn calculate_completion_time_with_buffer(
        video_duration: Duration,
        buffer_percentage: f32,
    ) -> Duration {
        let buffer_time =
            Duration::from_secs((video_duration.as_secs() as f32 * buffer_percentage) as u64);
        video_duration + buffer_time
    }

    pub fn validate_session_duration(
        sections: &[&crate::types::course::Section],
        settings: &crate::types::PlanSettings,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        let total_duration: Duration = sections.iter().map(|s| s.duration).sum();
        let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);

        if total_duration > session_limit {
            warnings.push(format!(
                "Session duration ({}) exceeds target ({})",
                format_duration(total_duration),
                format_duration(session_limit)
            ));
        }

        for section in sections {
            if section.duration.as_secs() > (settings.session_length_minutes as u64 * 60) / 2 {
                warnings.push(format!(
                    "Video '{}' is very long ({}) for session length",
                    section.title,
                    format_duration(section.duration)
                ));
            }
        }

        warnings
    }
}
