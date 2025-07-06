//! Course Pilot - An intelligent study planning application
//!
//! This library provides the core functionality for Course Pilot, including:
//! - Data ingestion from YouTube playlists and local folders
//! - NLP-powered course structure analysis
//! - Intelligent study plan generation
//! - SQLite-based persistence
//! - Dioxus-based desktop UI

// Main modules
pub mod app;
pub mod ingest;
pub mod nlp;
pub mod planner;
pub mod state;
pub mod storage;
pub mod types;
pub mod ui;

// Re-export commonly used types for convenience
pub use types::{
    AppState, Course, CourseStructure, ImportJob, ImportStatus, Module, Plan, PlanItem,
    PlanSettings, Route, Section, StructureMetadata,
};

// Re-export main UI components
pub use ui::components::{
    add_course_dialog::AddCourseDialog, course_dashboard::course_dashboard, plan_view::PlanView,
};

// Re-export core functionality
pub use ingest::{import_from_local_folder, import_from_youtube};
pub use nlp::structure_course;
pub use planner::generate_plan;
pub use storage::{init_db, load_courses, load_plan, save_course, save_plan};

// Custom error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CourseError {
    #[error("Import error: {0}")]
    Import(#[from] ImportError),

    #[error("NLP processing error: {0}")]
    Nlp(#[from] NlpError),

    #[error("Planning error: {0}")]
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

    #[error("No valid content found")]
    NoContent,
}

#[derive(Error, Debug)]
pub enum NlpError {
    #[error("Model loading failed: {0}")]
    ModelLoad(String),

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

    #[error("Planning algorithm failed: {0}")]
    Algorithm(String),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Data not found: {0}")]
    NotFound(String),
}

// Global result type for convenience
pub type Result<T> = std::result::Result<T, CourseError>;
