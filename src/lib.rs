//! Course Pilot - An intelligent study planning application
//!
//! This library provides the core functionality for Course Pilot, including:
//! - Data ingestion from YouTube playlists and local folders
//! - Course structure analysis
//! - Study plan generation
//! - SQLite-based persistence

#![allow(unused_mut)]
#![allow(unused_comparisons)]

// Main modules
pub mod app;
pub mod export;
pub mod gemini;
pub mod ingest;
pub mod nlp;
pub mod planner;
pub mod state;
pub mod storage;
pub mod types;
pub mod ui;
pub mod video_player;

// Re-export commonly used types
pub use types::{
    AdvancedSchedulerSettings, AppState, ContextualPanelState, ContextualPanelTab, Course,
    CourseStructure, DifficultyLevel, DistributionStrategy, ImportJob, ImportStatus, Module, Note,
    Plan, PlanExt, PlanItem, PlanSettings, RegenerationStatus, Route, Section, StructureMetadata,
    VideoContext,
};

// Re-export core functionality
pub use ingest::{import_from_local_folder, import_from_youtube};
pub use nlp::structure_course;
pub use planner::generate_plan;
pub use storage::{init_db, load_courses, load_plan, save_course, save_plan};

// Re-export import progress
pub use ingest::ImportProgress;
pub use types::ImportStage;

// Re-export UI components
pub use ui::{
    AppRoot, AppShell, BaseButton, BaseCard, BaseList, BaseModal, BasePage, Breadcrumbs,
    CourseCard, CourseGrid, Dashboard, NotesPanel, PlanView, ProgressBar, ProgressRing, Toast,
    ToastContainer,
};

// Re-export hooks
pub use ui::{
    use_backend, use_course_manager, use_courses_resource, use_export_manager, use_import_manager,
    use_modal_manager, use_navigation_manager, use_notes_manager, use_plan_manager,
    use_settings_manager,
};

// Error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CourseError {
    #[error("Import error: {0}")]
    Import(#[from] ImportError),

    #[error("NLP error: {0}")]
    Nlp(#[from] NlpError),

    #[error("Plan error: {0}")]
    Plan(#[from] PlanError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("No content found")]
    NoContent,
}

#[derive(Error, Debug)]
pub enum NlpError {
    #[error("Processing failed: {0}")]
    Processing(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Error, Debug)]
pub enum PlanError {
    #[error("Invalid settings: {0}")]
    InvalidSettings(String),

    #[error("Course not structured")]
    CourseNotStructured,

    #[error("Algorithm failed: {0}")]
    Algorithm(String),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Query failed: {query} - {message}")]
    QueryFailed { query: String, message: String },

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, CourseError>;
