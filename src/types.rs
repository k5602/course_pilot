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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructureMetadata {
    pub total_videos: usize,
    pub estimated_duration_hours: Option<f32>,
    pub difficulty_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Module {
    pub title: String,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub title: String,
    pub video_index: usize,
    pub estimated_duration: Option<Duration>,
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
    pub current_route: Route,
    pub active_import: Option<ImportJob>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Dashboard,
    AddCourse,
    PlanView(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CourseStatus {
    Structured,
    Unstructured,
    Pending,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            courses: Vec::new(),
            current_route: Route::Dashboard,
            active_import: None,
        }
    }
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
