//! Course Pilot - An intelligent study planning application
//!
//! This library provides the core functionality for Course Pilot, including:
//! - Data ingestion from YouTube playlists and local folders
//! - NLP-powered course structure analysis
//! - Intelligent study plan generation
//! - SQLite-based persistence
//!

// Main modules
pub mod app;
pub mod error_handling;
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

// Re-export commonly used types for convenience
pub use types::{
    AdvancedSchedulerSettings, AppState, ClusteringAlgorithm, ClusteringMetadata,
    ClusteringStrategy, ContextualPanelState, ContextualPanelTab, Course, CourseStructure,
    DifficultyLevel, DistributionStrategy, ImportJob, ImportStatus, Module, Note, Plan, PlanExt,
    PlanItem, PlanSettings, RegenerationStatus, Route, Section, StructureMetadata, VideoContext,
};

// Re-export core functionality
pub use ingest::{import_from_local_folder, import_from_youtube};
pub use nlp::structure_course;
pub use planner::generate_plan;
pub use storage::{init_db, load_courses, load_plan, save_course, save_plan};

// Re-export enhanced integrated functions
pub use ingest::{ImportProgress, import_and_structure_local_folder, import_and_structure_youtube};
pub use storage::{
    ClusteringAnalytics, ClusteringPerformancePoint, ProcessingTimeStats, QualityDistribution,
    get_clustering_analytics, get_clustering_performance_history,
    get_courses_by_clustering_quality, get_similar_courses_by_clustering,
    update_clustering_metadata,
};
pub use types::ImportStage;

// Re-export UI components for external use
pub use ui::{
    AppRoot, AppShell, BaseButton, BaseCard, BaseList, BaseModal, BasePage, Breadcrumbs,
    CourseCard, CourseGrid, Dashboard, NotesPanel, PlanView, ProgressBar, ProgressRing, Toast,
    ToastContainer,
};

// Re-export commonly used hooks
pub use ui::{
    use_app_state, use_backend, use_course_manager, use_courses_resource, use_export_manager,
    use_import_manager, use_modal_manager, use_navigation_manager, use_notes_manager,
    use_plan_manager, use_settings_manager,
};

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

    #[error("Database error: {0}")]
    Database(String),

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

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Data not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum Phase3Error {
    #[error("Backend operation failed: {0}")]
    Backend(#[from] anyhow::Error),

    #[error("Plan item not found: plan_id={plan_id}, item_index={item_index}")]
    PlanItemNotFound {
        plan_id: uuid::Uuid,
        item_index: usize,
    },

    #[error("Ingest operation failed: {0}")]
    Ingest(String),

    #[error("UI state synchronization failed: {0}")]
    StateSyncError(String),
}

/// Helper function to handle async errors consistently
pub fn handle_async_error(error: anyhow::Error, operation: &str) {
    log::error!("Async operation '{operation}' failed: {error}");

    let user_message = match error.downcast_ref::<Phase3Error>() {
        Some(Phase3Error::PlanItemNotFound { .. }) => {
            "The item you're trying to update no longer exists. Please refresh the page."
        }
        Some(Phase3Error::Backend(_)) => "A server error occurred. Please try again in a moment.",
        Some(Phase3Error::Ingest(msg)) => &format!("Import failed: {msg}"),
        Some(Phase3Error::StateSyncError(_)) => {
            "UI state synchronization failed. Please refresh the page."
        }
        _ => "An unexpected error occurred. Please try again.",
    };

    log::info!("User-friendly error message: {user_message}");
}

// Global result type for convenience
pub type Result<T> = std::result::Result<T, CourseError>;
