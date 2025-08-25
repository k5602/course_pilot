#![allow(dead_code)]
//! Thin compatibility facade for the storage database API.
//!
//! This module preserves the previous `crate::storage::database::*` API surface
//! while delegating all implementations to the new, smaller submodules:
//! - core:     DB pool, schema initialization (no migrations), optimization, metrics
//! - courses:  Course CRUD and metadata handling
//! - plans:    Plan CRUD and queries
//! - progress: Video progress tracking CRUD/queries
//! - analytics:Clustering analytics and similarity utilities
//!
//! Notably, the legacy migration and maintenance systems have been removed. The
//! storage now eagerly creates required tables at startup (idempotent) and
//! exposes optimization/metrics utilities directly from `core`.
//!
//! Existing code that imports from `crate::storage::database::*` should continue
//! to compile and work without changes.

/// Core database lifecycle: pool, schema init (no migrations), optimization, metrics.
pub use crate::storage::core::{
    ConnectionPoolHealth, Database, DatabasePerformanceMetrics, get_database_performance_metrics,
    init_db, optimize_database,
};

/// Courses persistence API (CRUD and load helpers).
pub use crate::storage::courses::{delete_course, get_course_by_id, load_courses, save_course};

/// Plans persistence API (CRUD and retrieval by course or id).
pub use crate::storage::plans::{delete_plan, get_plan_by_course_id, load_plan, save_plan};

/// Video progress tracking API (save and query completion/session progress).
pub use crate::storage::progress::{
    get_session_progress, get_video_completion_status, save_video_progress,
};

/// Clustering analytics and similarity utilities.
pub use crate::storage::analytics::{
    ClusteringAnalytics, ClusteringPerformancePoint, ProcessingTimeStats, QualityDistribution,
    get_clustering_analytics, get_clustering_performance_history,
    get_courses_by_clustering_quality, get_similar_courses_by_clustering,
    update_clustering_metadata,
};
