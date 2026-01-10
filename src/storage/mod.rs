//! Storage module for Course Pilot
//!
//! Submodules:
//! - core: DB pool, schema init, optimization
//! - courses: course CRUD
//! - plans: plan CRUD
//! - progress: video progress tracking
//! - analytics: clustering analytics
//! - notes: notes persistence
//! - settings: app settings

pub mod analytics;
pub mod core;
pub mod courses;
pub mod notes;
pub mod plans;
pub mod progress;
pub mod settings;
pub mod utils;

// Re-export main storage API
pub use analytics::{
    ClusteringAnalytics, ClusteringPerformancePoint, ProcessingTimeStats, QualityDistribution,
    get_clustering_analytics, get_clustering_performance_history,
    get_courses_by_clustering_quality, get_similar_courses_by_clustering,
    update_clustering_metadata,
};

pub use core::{
    ConnectionPoolHealth, Database, DatabasePerformanceMetrics, get_database_performance_metrics,
    init_db, optimize_database,
};

pub use courses::{delete_course, get_course_by_id, load_courses, save_course};
pub use plans::{delete_plan, get_plan_by_course_id, load_plan, save_plan};
pub use progress::{get_session_progress, get_video_completion_status, save_video_progress};

// Re-export error types
pub use crate::DatabaseError;

// Re-export notes functions
pub use notes::{
    create_note_pooled, delete_note_pooled, export_notes_markdown_by_course_pooled,
    export_notes_markdown_by_video_pooled, get_all_notes_pooled, get_course_level_notes_pooled,
    get_note_by_id_pooled, get_notes_by_course, get_notes_by_course_pooled,
    get_notes_by_video_index_pooled, get_notes_by_video_pooled, init_notes_table,
    search_notes_advanced_pooled, search_notes_pooled, update_note_pooled,
};

// Re-export settings
pub use settings::{
    AppSettings, CourseNamingPattern, ImportPreferences, VideoQualityPreference, save_app_settings,
    use_app_settings,
};
