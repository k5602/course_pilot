//! Courses State Management
//!
//! Focused state management for courses using modern Dioxus signals

use crate::types::Course;
use dioxus::prelude::*;
use uuid::Uuid;

/// Courses state context
#[derive(Clone, Copy)]
pub struct CoursesState {
    pub courses: Signal<Vec<Course>>,
}

/// Provide courses context
pub fn provide_courses_context(initial_courses: Vec<Course>) {
    use_context_provider(|| CoursesState {
        courses: Signal::new(initial_courses),
    });
}

/// Hook to get courses state
pub fn use_courses_state() -> CoursesState {
    use_context::<CoursesState>()
}

/// Hook for reactive access to all courses
pub fn use_courses() -> ReadOnlySignal<Vec<Course>> {
    let state = use_courses_state();
    state.courses.into()
}

/// Hook for reactive access to a specific course
pub fn use_course(id: Uuid) -> Memo<Option<Course>> {
    let courses = use_courses();
    use_memo(move || courses.read().iter().find(|c| c.id == id).cloned())
}

/// Hook for course statistics
pub fn use_course_stats() -> Memo<(usize, usize, usize)> {
    let courses = use_courses();
    use_memo(move || {
        let courses_vec = courses.read();
        let total = courses_vec.len();
        let structured = courses_vec.iter().filter(|c| c.is_structured()).count();
        let videos = courses_vec.iter().map(|c| c.video_count()).sum();
        (total, structured, videos)
    })
}

/// Actions for courses
pub mod actions {
    use super::*;
    use crate::state::StateError;

    /// Add a course
    pub fn add_course(course: Course) -> Result<(), StateError> {
        let mut state = use_courses_state();

        // Validation
        if course.name.trim().is_empty() {
            return Err(StateError::ValidationError(
                "Course name cannot be empty".to_string(),
            ));
        }

        // Check for duplicate names
        {
            let courses = state.courses.read();
            if courses.iter().any(|c| c.name == course.name) {
                return Err(StateError::ValidationError(
                    "Course with this name already exists".to_string(),
                ));
            }
        }

        // Add course
        state.courses.write().push(course);
        log::info!("Course added successfully");
        Ok(())
    }

    /// Update a course
    pub fn update_course(id: Uuid, updated_course: Course) -> Result<(), StateError> {
        let mut state = use_courses_state();
        let mut courses = state.courses.write();

        let course_index = courses
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
        if courses
            .iter()
            .any(|c| c.id != id && c.name == updated_course.name)
        {
            return Err(StateError::ValidationError(
                "Course with this name already exists".to_string(),
            ));
        }

        courses[course_index] = updated_course;
        log::info!("Course {id} updated successfully");
        Ok(())
    }

    /// Delete a course
    pub fn delete_course(id: Uuid) -> Result<(), StateError> {
        let mut state = use_courses_state();
        let mut courses = state.courses.write();
        let initial_len = courses.len();

        courses.retain(|c| c.id != id);

        if courses.len() == initial_len {
            return Err(StateError::CourseNotFound(id));
        }

        log::info!("Course {id} deleted successfully");
        Ok(())
    }
}
