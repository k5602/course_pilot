use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub raw_titles: Vec<String>,
    pub structure: Option<CourseStructure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseStructure {
    pub modules: Vec<Module>,
    pub metadata: StructureMetadata,
}

impl CourseStructure {
    pub fn aggregate_total_duration(&self) -> Duration {
        self.modules.iter().map(|m| m.total_duration).sum()
    }
    pub fn with_aggregated_metadata(mut self) -> Self {
        let total_videos = self.modules.iter().map(|m| m.sections.len()).sum();
        let total_duration = self.aggregate_total_duration();
        self.metadata.total_videos = total_videos;
        self.metadata.total_duration = total_duration;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructureMetadata {
    pub total_videos: usize,
    pub total_duration: std::time::Duration,
    pub estimated_duration_hours: Option<f32>,
    pub difficulty_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Module {
    pub title: String,
    pub sections: Vec<Section>,
    pub total_duration: Duration,
}

impl Module {
    pub fn aggregate_total_duration(&self) -> Duration {
        self.sections.iter().map(|s| s.duration).sum()
    }
}

use serde::{Deserializer, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub title: String,
    pub video_index: usize,
    #[serde(
        serialize_with = "serialize_duration_as_secs",
        deserialize_with = "deserialize_duration_from_secs"
    )]
    pub duration: Duration,
}

// Custom serde for Duration as seconds
fn serialize_duration_as_secs<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

fn deserialize_duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

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

/// Advanced scheduler settings for sophisticated planning algorithms
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

/// Distribution strategies for course content scheduling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DistributionStrategy {
    ModuleBased,
    TimeBased,
    Hybrid,
    DifficultyBased,
    SpacedRepetition,
    Adaptive,
}

/// Content difficulty levels for adaptive scheduling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Plan regeneration status for progress tracking
#[derive(Debug, Clone, PartialEq)]
pub enum RegenerationStatus {
    Idle,
    InProgress { progress: f32, message: String },
    Completed,
    Failed { error: String },
}

/// State management for the PlanView component
#[derive(Debug, Clone, PartialEq)]
pub struct PlanViewState {
    pub expanded_sessions: HashSet<usize>,
    pub selected_videos: HashSet<usize>,
    pub regeneration_status: RegenerationStatus,
    pub last_update: DateTime<Utc>,
}

/// Video progress update for tracking completion status
#[derive(Debug, Clone, PartialEq)]
pub struct VideoProgressUpdate {
    pub plan_id: Uuid,
    pub session_index: usize,
    pub video_index: usize,
    pub completed: bool,
    pub timestamp: DateTime<Utc>,
}

impl VideoProgressUpdate {
    /// Create a new video progress update
    pub fn new(plan_id: Uuid, session_index: usize, video_index: usize, completed: bool) -> Self {
        Self {
            plan_id,
            session_index,
            video_index,
            completed,
            timestamp: Utc::now(),
        }
    }

    /// Create a completion update
    pub fn completed(plan_id: Uuid, session_index: usize, video_index: usize) -> Self {
        Self::new(plan_id, session_index, video_index, true)
    }

    /// Create an uncomplete update
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportJob {
    pub id: Uuid,
    pub status: ImportStatus,
    pub progress_percentage: f32,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportStatus {
    Starting,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub courses: Vec<Course>,
    pub plans: Vec<Plan>,
    pub notes: Vec<Note>,
    pub current_route: Route,
    pub active_import: Option<ImportJob>,
    pub contextual_panel: ContextualPanelState,
    pub sidebar_open_mobile: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Route {
    #[default]
    Dashboard,
    AddCourse,
    PlanView(Uuid),
    Settings,
    #[cfg(debug_assertions)]
    ToastTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CourseStatus {
    Structured,
    Unstructured,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextualPanelTab {
    Notes,
    Player,
}

/// Video context for notes integration
#[derive(Debug, Clone, PartialEq)]
pub struct VideoContext {
    pub course_id: Uuid,
    pub video_index: usize,
    pub video_title: String,
    pub module_title: String,
}

#[derive(Debug, Clone)]
pub struct ContextualPanelState {
    pub is_open: bool,
    pub active_tab: ContextualPanelTab,
    pub video_context: Option<VideoContext>,
}

impl Default for ContextualPanelState {
    fn default() -> Self {
        Self {
            is_open: false, // Closed by default, user can open via button
            active_tab: ContextualPanelTab::Notes,
            video_context: None,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            courses: Vec::new(),
            plans: Vec::new(),
            notes: Vec::new(),
            current_route: Route::Dashboard,
            active_import: None,
            contextual_panel: ContextualPanelState::default(),
            sidebar_open_mobile: false,
        }
    }
}

/// Represents a user note tied to a specific video (section) in a course.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: Uuid,
    pub course_id: Uuid, // Always present: which course this note belongs to
    pub video_id: Option<Uuid>, // None for course-level notes, Some for video-level notes
    pub video_index: Option<usize>, // Video index within the course for plan view integration
    pub content: String, // Markdown or rich text
    pub timestamp: Option<u32>, // Seconds into the video
    pub tags: Vec<String>, // Tagging support
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Course {
    pub fn new(name: String, raw_titles: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
            raw_titles,
            structure: None,
        }
    }

    pub fn video_count(&self) -> usize {
        self.raw_titles.len()
    }

    pub fn is_structured(&self) -> bool {
        self.structure.is_some()
    }
}

impl Plan {
    pub fn new(course_id: Uuid, settings: PlanSettings) -> Self {
        Self {
            id: Uuid::new_v4(),
            course_id,
            settings,
            items: Vec::new(),
            created_at: Utc::now(),
        }
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

/// Identifier for a plan item using composite key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanItemIdentifier {
    pub plan_id: Uuid,
    pub item_index: usize,
}

impl PlanItemIdentifier {
    pub fn new(plan_id: Uuid, item_index: usize) -> Self {
        Self {
            plan_id,
            item_index,
        }
    }
}

/// Extension trait for Plan operations
pub trait PlanExt {
    fn get_item_identifier(&self, index: usize) -> PlanItemIdentifier;
    fn update_item_completion(&mut self, index: usize, completed: bool) -> Result<(), String>;
    fn calculate_progress(&self) -> (usize, usize, f32);
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
    /// Create new settings with a specific strategy
    pub fn with_strategy(strategy: DistributionStrategy) -> Self {
        Self {
            strategy,
            ..Self::default()
        }
    }

    /// Create settings optimized for beginners
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

    /// Create settings optimized for advanced users
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

    /// Validate the settings for consistency
    pub fn validate(&self) -> Result<(), String> {
        if self.spaced_repetition_enabled && self.strategy != DistributionStrategy::SpacedRepetition
        {
            return Err(
                "Spaced repetition enabled but strategy is not SpacedRepetition".to_string(),
            );
        }

        if let Some(max_duration) = self.max_session_duration_minutes {
            if max_duration < 15 || max_duration > 300 {
                return Err("Session duration must be between 15 and 300 minutes".to_string());
            }
        }

        if let Some(min_break) = self.min_break_between_sessions_hours {
            if min_break > 168 {
                // 1 week
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

    /// Get recommended settings based on course complexity and user level
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
            }
            _ => DistributionStrategy::Hybrid,
        };

        let spaced_repetition = matches!(user_level, DifficultyLevel::Beginner);
        let prioritize_difficult = matches!(
            user_level,
            DifficultyLevel::Advanced | DifficultyLevel::Expert
        );

        // Adjust session duration based on course length
        let max_session_duration = if total_duration_hours > 20.0 {
            Some(90) // Longer sessions for extensive courses
        } else if total_duration_hours < 5.0 {
            Some(45) // Shorter sessions for brief courses
        } else {
            Some(60) // Standard session length
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

impl Default for DistributionStrategy {
    fn default() -> Self {
        DistributionStrategy::Hybrid
    }
}

impl Default for DifficultyLevel {
    fn default() -> Self {
        DifficultyLevel::Intermediate
    }
}

impl DistributionStrategy {
    /// Get all available distribution strategies
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

    /// Get human-readable name for the strategy
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

    /// Get description for the strategy
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
    /// Get all available difficulty levels
    pub fn all() -> Vec<Self> {
        vec![
            Self::Beginner,
            Self::Intermediate,
            Self::Advanced,
            Self::Expert,
        ]
    }

    /// Get human-readable name for the difficulty level
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
    /// Create a new PlanViewState instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle session expansion state
    pub fn toggle_session(&mut self, session_index: usize) {
        if self.expanded_sessions.contains(&session_index) {
            self.expanded_sessions.remove(&session_index);
        } else {
            self.expanded_sessions.insert(session_index);
        }
        self.last_update = Utc::now();
    }

    /// Check if a session is expanded
    pub fn is_session_expanded(&self, session_index: usize) -> bool {
        self.expanded_sessions.contains(&session_index)
    }

    /// Toggle video selection state
    pub fn toggle_video_selection(&mut self, video_index: usize) {
        if self.selected_videos.contains(&video_index) {
            self.selected_videos.remove(&video_index);
        } else {
            self.selected_videos.insert(video_index);
        }
        self.last_update = Utc::now();
    }

    /// Check if a video is selected
    pub fn is_video_selected(&self, video_index: usize) -> bool {
        self.selected_videos.contains(&video_index)
    }

    /// Update regeneration status
    pub fn set_regeneration_status(&mut self, status: RegenerationStatus) {
        self.regeneration_status = status;
        self.last_update = Utc::now();
    }

    /// Clear all selections and expanded states
    pub fn clear_selections(&mut self) {
        self.expanded_sessions.clear();
        self.selected_videos.clear();
        self.last_update = Utc::now();
    }

    /// Get count of expanded sessions
    pub fn expanded_session_count(&self) -> usize {
        self.expanded_sessions.len()
    }

    /// Get count of selected videos
    pub fn selected_video_count(&self) -> usize {
        self.selected_videos.len()
    }
}

impl ImportJob {
    pub fn new(message: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            status: ImportStatus::Starting,
            progress_percentage: 0.0,
            message,
            created_at: Utc::now(),
        }
    }

    pub fn update_progress(&mut self, percentage: f32, message: String) {
        self.progress_percentage = percentage.clamp(0.0, 100.0);
        self.message = message;
        if percentage >= 100.0 {
            self.status = ImportStatus::Completed;
        } else {
            self.status = ImportStatus::InProgress;
        }
    }

    pub fn mark_failed(&mut self, error_message: String) {
        self.status = ImportStatus::Failed;
        self.message = error_message;
    }
}
