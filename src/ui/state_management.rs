use dioxus::prelude::*;
use crate::types::{AppState, Course};

/// Modern state management utilities using Dioxus signals
/// This module provides reactive state patterns for Course Pilot

/// Global signal for courses - provides reactive access across the app
static COURSES: GlobalSignal<Vec<Course>> = Signal::global(|| Vec::new());

/// Global signal for active import status
static ACTIVE_IMPORT: GlobalSignal<Option<crate::types::ImportJob>> = Signal::global(|| None);

/// Initialize global state from the main app state
pub fn initialize_global_state(app_state: Signal<AppState>) {
    // Sync courses to global signal
    use_effect(move || {
        let courses = app_state.read().courses.clone();
        *COURSES.write() = courses;
    });
    
    // Sync active import to global signal
    use_effect(move || {
        let active_import = app_state.read().active_import.clone();
        *ACTIVE_IMPORT.write() = active_import;
    });
}

/// Hook to get reactive courses
pub fn use_reactive_courses() -> Memo<Vec<Course>> {
    use_memo(move || COURSES.read().clone())
}

/// Hook to get reactive active import
pub fn use_reactive_active_import() -> Memo<Option<crate::types::ImportJob>> {
    use_memo(move || ACTIVE_IMPORT.read().clone())
}

/// Hook to get a specific course reactively
pub fn use_reactive_course(course_id: uuid::Uuid) -> Memo<Option<Course>> {
    use_memo(move || {
        COURSES.read().iter().find(|c| c.id == course_id).cloned()
    })
}

/// Hook to get course count reactively
pub fn use_reactive_course_count() -> Memo<usize> {
    use_memo(move || COURSES.read().len())
}

/// Hook to get structured course count reactively
pub fn use_reactive_structured_course_count() -> Memo<usize> {
    use_memo(move || {
        COURSES.read().iter().filter(|c| c.is_structured()).count()
    })
}

/// Hook to get total video count across all courses reactively
pub fn use_reactive_total_video_count() -> Memo<usize> {
    use_memo(move || {
        COURSES.read().iter().map(|c| c.video_count()).sum()
    })
}

/// Actions for updating global state
pub mod actions {
    use super::*;
    
    /// Add a course to global state
    pub fn add_course_to_global_state(course: Course) {
        COURSES.write().push(course);
    }
    
    /// Update a course in global state
    pub fn update_course_in_global_state(course_id: uuid::Uuid, updated_course: Course) {
        let mut courses = COURSES.write();
        if let Some(index) = courses.iter().position(|c| c.id == course_id) {
            courses[index] = updated_course;
        }
    }
    
    /// Remove a course from global state
    pub fn remove_course_from_global_state(course_id: uuid::Uuid) {
        COURSES.write().retain(|c| c.id != course_id);
    }
    
    /// Set active import in global state
    pub fn set_active_import(import_job: Option<crate::types::ImportJob>) {
        *ACTIVE_IMPORT.write() = import_job;
    }
}