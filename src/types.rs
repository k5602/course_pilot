use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy)]
pub struct ContextualPanelState {
    pub is_open: bool,
    pub active_tab: ContextualPanelTab,
}

impl Default for ContextualPanelState {
    fn default() -> Self {
        Self {
            is_open: false, // Closed by default, user can open via button
            active_tab: ContextualPanelTab::Notes,
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
