//! Storage module for Course Pilot
//!
//! This module provides database operations and persistence functionality
//! using SQLite for reliable data storage.

pub mod database;

// Re-export main database functions
pub use database::{
    delete_course, delete_plan, get_course_by_id, get_plan_by_course_id, init_db, load_courses,
    load_plan, save_course, save_plan,
};

// Re-export error types
pub use crate::DatabaseError;

// Re-export Connection type for convenience
pub use rusqlite::Connection;
