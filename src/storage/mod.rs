//! Storage module for Course Pilot
//!
//! This module provides database operations and persistence functionality
//! using SQLite for reliable data storage.

pub mod connection_manager;
pub mod database;
pub mod maintenance;
pub mod migrations;
pub mod notes;
pub mod optimized_queries;
pub mod preference_storage;
pub mod settings;

// Re-export main database functions
pub use database::{
    ClusteringAnalytics,
    ClusteringPerformancePoint,
    ConnectionPoolHealth,
    Database,
    DatabasePerformanceMetrics,
    ProcessingTimeStats,
    QualityDistribution,
    delete_course,
    delete_plan,
    get_clustering_analytics,
    get_clustering_performance_history,
    get_course_by_id,
    // Enhanced clustering functions
    get_courses_by_clustering_quality,
    get_database_performance_metrics,
    get_plan_by_course_id,
    get_similar_courses_by_clustering,
    init_db,
    load_courses,
    load_plan,
    optimize_database,
    save_course,
    save_plan,
    save_video_progress,
    get_video_completion_status,
    get_session_progress,
    update_clustering_metadata,
};

// Re-export error types
pub use crate::error_handling::DatabaseError;

// Re-export notes functions for convenience
pub use notes::{
    get_notes_by_course, init_notes_table,
    // Pooled versions (preferred for new code)
    create_note_pooled, update_note_pooled, delete_note_pooled,
    get_all_notes_pooled, get_notes_by_course_pooled, get_notes_by_video_pooled,
    get_course_level_notes_pooled, get_notes_by_video_index_pooled,
    get_note_by_id_pooled, search_notes_pooled, search_notes_advanced_pooled,
    export_notes_markdown_by_course_pooled, export_notes_markdown_by_video_pooled,
};

// Re-export settings functions for convenience
pub use settings::{
    AppSettings, CourseNamingPattern, ImportPreferences, VideoQualityPreference, save_app_settings,
    use_app_settings,
};

// Re-export preference storage functions for convenience
pub use preference_storage::PreferenceStorage;

// Re-export optimized queries for convenience
pub use optimized_queries::{
    ActivityItem, ActivityType, CourseStatistics, DatabasePerformanceStats, IndexInfo,
    OptimizedQueries, QueryAnalysis, QueryPlanStep, SearchResult, SearchResultType, TableCounts,
};

// Re-export connection manager for convenience
pub use connection_manager::{ConnectionManager, DatabaseStats, IndexUsage};

// Re-export maintenance utilities for convenience
pub use maintenance::{
    DatabaseMaintenance, HealthReport, HealthStatus, MaintenanceReport, MaintenanceSchedule,
};

// Re-export migration utilities for convenience
pub use migrations::{
    MigrationManager, MigrationRecord, ValidationReport, CURRENT_SCHEMA_VERSION,
};

// Seed data functionality removed

// Re-export Connection type for convenience
pub use rusqlite::Connection;
