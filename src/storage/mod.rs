//! Storage module for Course Pilot
//!
//! This module provides database operations and persistence functionality
//! using SQLite for reliable data storage.

pub mod database;
pub mod notes;
pub mod settings;

// Re-export main database functions
pub use database::{
    delete_course, delete_plan, get_course_by_id, get_plan_by_course_id, init_db, load_courses,
    load_plan, save_course, save_plan,
};

// Re-export error types
pub use crate::DatabaseError;

// Re-export notes functions for convenience
pub use notes::{get_notes_by_course, init_notes_table};

// Re-export settings functions for convenience
pub use settings::{AppSettings, save_app_settings, use_app_settings};

// Re-export Connection type for convenience
pub use rusqlite::Connection;
