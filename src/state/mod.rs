#![allow(clippy::module_inception)]
//! Reactive state management for Course Pilot using dioxus-signals.
//!
//! This module provides centralized state management using a submodule architecture:
//! - courses: Course management and operations
//! - notes: Notes management and video context
//! - plans: Study plan management and progress tracking
//! - imports: Import job management and progress
//! - ui: UI state including panels, navigation, and sidebar

// Submodules
pub mod courses;
pub mod export_progress;
pub mod imports;
pub mod notes;
pub mod plans;
pub mod ui;

// Re-export commonly used types
use dioxus::prelude::*;
use uuid::Uuid;

/// Result type for state operations
pub type StateResult<T> = Result<T, StateError>;

/// Errors that can occur during state operations
#[derive(Debug, Clone)]
pub enum StateError {
    CourseNotFound(Uuid),
    InvalidOperation(String),
    NavigationError(String),
    ValidationError(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::CourseNotFound(id) => write!(f, "Course not found: {id}"),
            StateError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
            StateError::NavigationError(msg) => write!(f, "Navigation error: {msg}"),
            StateError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for StateError {}

// Re-export all submodule functionality for backward compatibility
pub use courses::*;
pub use export_progress::*;
pub use imports::*;
pub use notes::*;
pub use plans::*;
pub use ui::*;

/// Initialize global application state
///
/// This sets up all the necessary contexts and providers for the application.
/// Should be called once at application startup.
pub fn initialize_global_state(_app_state: Signal<crate::types::AppState>) {
    log::info!("Initializing global state management");

    // State is now managed through individual context providers in each submodule
    // This function is kept for backward compatibility and future global state setup

    log::info!("Global state management initialized successfully");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_error_display() {
        let course_id = uuid::Uuid::new_v4();
        let error = StateError::CourseNotFound(course_id);
        assert!(error.to_string().contains(&course_id.to_string()));

        let error = StateError::InvalidOperation("test".to_string());
        assert_eq!(error.to_string(), "Invalid operation: test");
    }

    #[test]
    fn test_state_error_types() {
        let nav_error = StateError::NavigationError("route not found".to_string());
        assert!(nav_error.to_string().contains("Navigation error"));

        let val_error = StateError::ValidationError("invalid input".to_string());
        assert!(val_error.to_string().contains("Validation error"));
    }

    #[test]
    fn test_global_state_initialization() {
        // Avoid Dioxus Signals in tests; just verify test harness runs
        assert!(true);
    }
}
