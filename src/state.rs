//! State Management System
//!
//! This module provides safe state operations that prevent corruption
//! by eliminating direct mutations and providing atomic updates.

use crate::types::{AppState, Course, CourseStructure, ImportJob, ImportStatus, Route};
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
            StateError::CourseNotFound(id) => write!(f, "Course not found: {}", id),
            StateError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            StateError::NavigationError(msg) => write!(f, "Navigation error: {}", msg),
            StateError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for StateError {}

// Direct state action functions

/// Add a course to the state
pub fn add_course(mut app_state: Signal<AppState>, course: Course) -> StateResult<()> {
    // Validation
    if course.name.trim().is_empty() {
        return Err(StateError::ValidationError(
            "Course name cannot be empty".to_string(),
        ));
    }

    // Check for duplicate names
    {
        let state = app_state.read();
        if state.courses.iter().any(|c| c.name == course.name) {
            return Err(StateError::ValidationError(
                "Course with this name already exists".to_string(),
            ));
        }
    }

    // Atomic update
    app_state.write().courses.push(course);
    log::info!("Course added successfully");
    Ok(())
}

/// Update a course in the state
pub fn update_course(
    mut app_state: Signal<AppState>,
    id: Uuid,
    updated_course: Course,
) -> StateResult<()> {
    let mut state = app_state.write();
    let course_index = state
        .courses
        .iter()
        .position(|c| c.id == id)
        .ok_or(StateError::CourseNotFound(id))?;

    // Validation
    if updated_course.name.trim().is_empty() {
        return Err(StateError::ValidationError(
            "Course name cannot be empty".to_string(),
        ));
    }

    // Check for duplicate names (excluding current course)
    if state
        .courses
        .iter()
        .any(|c| c.id != id && c.name == updated_course.name)
    {
        return Err(StateError::ValidationError(
            "Course with this name already exists".to_string(),
        ));
    }

    // Atomic update
    state.courses[course_index] = updated_course;
    log::info!("Course {} updated successfully", id);
    Ok(())
}

/// Delete a course from the state
pub fn delete_course(mut app_state: Signal<AppState>, id: Uuid) -> StateResult<()> {
    let mut state = app_state.write();
    let initial_len = state.courses.len();

    // Atomic update
    state.courses.retain(|c| c.id != id);

    if state.courses.len() == initial_len {
        return Err(StateError::CourseNotFound(id));
    }

    // Navigate away from plan view if we're deleting the current course
    if let Route::PlanView(current_id) = &state.current_route {
        if *current_id == id {
            state.current_route = Route::Dashboard;
            log::info!("Navigated to dashboard after deleting current course");
        }
    }

    log::info!("Course {} deleted successfully", id);
    Ok(())
}

/// Duplicate a course
pub fn duplicate_course(app_state: Signal<AppState>, id: Uuid) -> StateResult<()> {
    let original_course = {
        let state = app_state.read();
        state
            .courses
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or(StateError::CourseNotFound(id))?
    };

    // Create duplicate with new ID and unique name
    let mut duplicate = original_course;
    duplicate.id = Uuid::new_v4();
    duplicate.name = generate_unique_name(app_state, &duplicate.name)?;
    duplicate.created_at = chrono::Utc::now();

    // Add duplicate
    add_course(app_state, duplicate)?;
    log::info!("Course {} duplicated successfully", id);
    Ok(())
}

/// Structure a course with the given structure
pub fn structure_course(
    mut app_state: Signal<AppState>,
    id: Uuid,
    structure: CourseStructure,
) -> StateResult<()> {
    let mut state = app_state.write();
    let course_index = state
        .courses
        .iter()
        .position(|c| c.id == id)
        .ok_or(StateError::CourseNotFound(id))?;

    // Validation
    if structure.modules.is_empty() {
        return Err(StateError::ValidationError(
            "Course structure cannot be empty".to_string(),
        ));
    }

    // Atomic update
    state.courses[course_index].structure = Some(structure);
    log::info!("Course {} structured successfully", id);
    Ok(())
}

/// Generate a unique course name
fn generate_unique_name(app_state: Signal<AppState>, base_name: &str) -> StateResult<String> {
    let state = app_state.read();
    let mut counter = 1;
    let mut new_name = format!("{} (Copy)", base_name);

    // Keep incrementing until we find a unique name
    while state.courses.iter().any(|c| c.name == new_name) {
        counter += 1;
        new_name = format!("{} (Copy {})", base_name, counter);

        // Prevent infinite loops
        if counter > 1000 {
            return Err(StateError::ValidationError(
                "Could not generate unique course name".to_string(),
            ));
        }
    }

    Ok(new_name)
}

/// Navigate to a route with validation
pub fn navigate_to(mut app_state: Signal<AppState>, route: Route) -> StateResult<()> {
    // Validation
    match &route {
        Route::PlanView(course_id) => {
            if *course_id == Uuid::nil() {
                return Err(StateError::NavigationError(
                    "Invalid course ID for plan view".to_string(),
                ));
            }

            // Verify course exists
            let state = app_state.read();
            if !state.courses.iter().any(|c| c.id == *course_id) {
                return Err(StateError::NavigationError(
                    "Course not found for plan view".to_string(),
                ));
            }
        }
        _ => {}
    }

    // Atomic update
    app_state.write().current_route = route;
    log::info!("Navigation completed successfully");
    Ok(())
}

/// Start an import job
pub fn start_import(mut app_state: Signal<AppState>, job: ImportJob) -> StateResult<()> {
    app_state.write().active_import = Some(job);
    log::info!("Import started");
    Ok(())
}

/// Update import progress
pub fn update_import(
    mut app_state: Signal<AppState>,
    id: Uuid,
    progress: f32,
    message: String,
) -> StateResult<()> {
    let mut state = app_state.write();
    if let Some(ref mut import) = state.active_import {
        if import.id == id {
            import.update_progress(progress, message);
            return Ok(());
        }
    }
    Err(StateError::InvalidOperation(
        "No active import found".to_string(),
    ))
}

/// Complete an import job
pub fn complete_import(mut app_state: Signal<AppState>, id: Uuid) -> StateResult<()> {
    let mut state = app_state.write();
    if let Some(ref mut import) = state.active_import {
        if import.id == id {
            import.status = ImportStatus::Completed;
            return Ok(());
        }
    }
    Err(StateError::InvalidOperation(
        "No active import found".to_string(),
    ))
}

/// Fail an import job
pub fn fail_import(mut app_state: Signal<AppState>, id: Uuid, error: String) -> StateResult<()> {
    let mut state = app_state.write();
    if let Some(ref mut import) = state.active_import {
        if import.id == id {
            import.mark_failed(error);
            return Ok(());
        }
    }
    Err(StateError::InvalidOperation(
        "No active import found".to_string(),
    ))
}

/// Clear the active import
pub fn clear_import(mut app_state: Signal<AppState>) -> StateResult<()> {
    app_state.write().active_import = None;
    log::info!("Import cleared");
    Ok(())
}

// Read-only helper functions

/// Check if a course exists
pub fn course_exists(app_state: Signal<AppState>, id: Uuid) -> bool {
    app_state.read().courses.iter().any(|c| c.id == id)
}

/// Get a course by ID
pub fn get_course(app_state: Signal<AppState>, id: Uuid) -> Option<Course> {
    app_state
        .read()
        .courses
        .iter()
        .find(|c| c.id == id)
        .cloned()
}

/// Get all courses
pub fn get_courses(app_state: Signal<AppState>) -> Vec<Course> {
    app_state.read().courses.clone()
}

/// Get current route
pub fn get_current_route(app_state: Signal<AppState>) -> Route {
    app_state.read().current_route.clone()
}

/// Get active import
pub fn get_active_import(app_state: Signal<AppState>) -> Option<ImportJob> {
    app_state.read().active_import.clone()
}

/// Get course statistics
pub fn get_course_stats(app_state: Signal<AppState>) -> (usize, usize, usize) {
    let state = app_state.read();
    let total = state.courses.len();
    let structured = state.courses.iter().filter(|c| c.is_structured()).count();
    let videos = state.courses.iter().map(|c| c.video_count()).sum();
    (total, structured, videos)
}

// Reactive hooks for components

/// Hook for reactive access to courses
pub fn use_courses() -> Memo<Vec<Course>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || app_state.read().courses.clone())
}

/// Hook for reactive access to a specific course
pub fn use_course(id: Uuid) -> Memo<Option<Course>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || {
        app_state
            .read()
            .courses
            .iter()
            .find(|c| c.id == id)
            .cloned()
    })
}

/// Hook for reactive access to current route
pub fn use_current_route() -> Memo<Route> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || app_state.read().current_route.clone())
}

/// Hook for reactive access to active import
pub fn use_active_import() -> Memo<Option<ImportJob>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || app_state.read().active_import.clone())
}

/// Hook for course statistics
pub fn use_course_stats() -> Memo<(usize, usize, usize)> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || {
        let courses = &app_state.read().courses;
        let total = courses.len();
        let structured = courses.iter().filter(|c| c.is_structured()).count();
        let videos = courses.iter().map(|c| c.video_count()).sum();
        (total, structured, videos)
    })
}

/// Hook for getting app state signal
pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}

/// Hook for getting tag statistics from notes
pub fn use_tag_statistics() -> Memo<std::collections::HashMap<String, usize>> {
    let app_state = use_context::<Signal<AppState>>();
    use_memo(move || {
        let mut stats = std::collections::HashMap::new();
        for note in &app_state.read().notes {
            for tag in &note.tags {
                *stats.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        stats
    })
}

/// Async-safe course structuring
pub async fn async_structure_course(
    app_state: Signal<AppState>,
    course_id: Uuid,
    raw_titles: Vec<String>,
) -> StateResult<()> {
    // Structure the course (this should be an async operation)
    match crate::nlp::structure_course(raw_titles) {
        Ok(structure) => {
            structure_course(app_state, course_id, structure)?;
            Ok(())
        }
        Err(e) => Err(StateError::InvalidOperation(format!(
            "Failed to structure course: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Course;
    use chrono::Utc;
    // Helper to set up a test environment with AppState
    fn setup_test_env() -> Signal<AppState> {
        // For unit tests, we'll create the signal directly
        Signal::new(AppState::default())
    }

    fn create_test_course(name: &str) -> Course {
        Course {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Lesson 1".to_string()],
            structure: None,
        }
    }

    #[test]
    fn test_add_course() {
        let app_state = setup_test_env();
        let course = create_test_course("Test Course");

        assert!(add_course(app_state, course.clone()).is_ok());
        assert_eq!(app_state.read().courses.len(), 1);
        assert_eq!(app_state.read().courses[0].id, course.id);
    }

    #[test]
    fn test_add_course_validation() {
        let app_state = setup_test_env();
        let invalid_course = create_test_course("");

        let result = add_course(app_state, invalid_course);
        assert!(matches!(result, Err(StateError::ValidationError(_))));
        assert_eq!(app_state.read().courses.len(), 0);
    }

    #[test]
    fn test_add_duplicate_course_name() {
        let app_state = setup_test_env();
        let course1 = create_test_course("Test Course");
        let course2 = create_test_course("Test Course");

        assert!(add_course(app_state, course1).is_ok());
        let result = add_course(app_state, course2);
        assert!(matches!(result, Err(StateError::ValidationError(_))));
        assert_eq!(app_state.read().courses.len(), 1);
    }

    #[test]
    fn test_delete_course_not_found() {
        let app_state = setup_test_env();
        let non_existent_id = Uuid::new_v4();

        let result = delete_course(app_state, non_existent_id);
        assert!(matches!(result, Err(StateError::CourseNotFound(_))));
    }

    #[test]
    fn test_duplicate_course() {
        let app_state = setup_test_env();
        let course = create_test_course("Original Course");
        let course_id = course.id;

        assert!(add_course(app_state, course.clone()).is_ok());
        assert!(duplicate_course(app_state, course_id).is_ok());
        assert_eq!(app_state.read().courses.len(), 2);

        // Check that duplicate has different ID but similar name
        let courses = app_state.read().courses.clone();
        let original = courses.iter().find(|c| c.id == course_id).unwrap();
        let duplicate = courses.iter().find(|c| c.id != course_id).unwrap();

        assert_ne!(original.id, duplicate.id);
        assert!(duplicate.name.starts_with(&original.name));
        assert!(duplicate.name.contains("Copy"));
    }

    #[test]
    fn test_update_course() {
        let app_state = setup_test_env();
        let mut course = create_test_course("Original Name");
        add_course(app_state, course.clone()).unwrap();

        course.name = "Updated Name".to_string();
        assert!(update_course(app_state, course.id, course.clone()).is_ok());

        let state = app_state.read();
        assert_eq!(state.courses[0].name, "Updated Name");
    }

    #[test]
    fn test_delete_course() {
        let app_state = setup_test_env();
        let course = create_test_course("To Be Deleted");
        add_course(app_state, course.clone()).unwrap();

        assert_eq!(app_state.read().courses.len(), 1);
        assert!(delete_course(app_state, course.id).is_ok());
        assert_eq!(app_state.read().courses.len(), 0);
    }
}
