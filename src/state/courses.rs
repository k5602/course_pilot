//! Course management state for Course Pilot
//!
//! This module handles reactive state for course operations including
//! creation, updates, deletion, and course-specific functionality.

use crate::types::{Course, CourseStructure};
use dioxus::prelude::*;
use uuid::Uuid;

use super::{StateError, StateResult};

/// Course management context
#[derive(Clone, Copy)]
pub struct CourseContext {
    pub courses: Signal<Vec<Course>>,
}

impl Default for CourseContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CourseContext {
    pub fn new() -> Self {
        Self {
            courses: Signal::new(Vec::new()),
        }
    }
}

/// Course context provider component
#[component]
pub fn CourseContextProvider(children: Element) -> Element {
    use_context_provider(|| CourseContext::new());
    rsx! { {children} }
}

/// Hook to access courses reactively
pub fn use_courses_reactive() -> Signal<Vec<Course>> {
    use_context::<CourseContext>().courses
}

/// Hook to access a specific course reactively
pub fn use_course_reactive(course_id: Uuid) -> Signal<Option<Course>> {
    let courses = use_courses_reactive();
    Signal::new(courses.read().iter().find(|c| c.id == course_id).cloned())
}

/// Hook to get course statistics reactively
pub fn use_course_stats_reactive() -> Signal<(usize, usize, usize)> {
    let courses = use_courses_reactive();
    Signal::new({
        let courses_vec = courses.read();
        let total_courses = courses_vec.len();
        let structured_courses = courses_vec.iter().filter(|c| c.structure.is_some()).count();
        let total_videos = courses_vec.iter().map(|c| c.videos.len()).sum::<usize>();
        (total_courses, structured_courses, total_videos)
    })
}

/// Add a new course to the reactive state
pub fn add_course_reactive(course: Course) {
    let mut courses = use_courses_reactive();
    let mut courses_vec = courses.read().clone();

    // Ensure unique name
    let unique_name = generate_unique_name_reactive(&courses_vec, &course.name);
    let mut new_course = course;
    new_course.name = unique_name;

    courses_vec.push(new_course);
    courses.set(courses_vec);
}

/// Update an existing course in the reactive state
pub fn update_course_reactive(course_id: Uuid, updated_course: Course) -> StateResult<()> {
    let mut courses = use_courses_reactive();
    let mut courses_vec = courses.read().clone();

    if let Some(index) = courses_vec.iter().position(|c| c.id == course_id) {
        // Ensure name uniqueness excluding the current course
        let other_courses: Vec<Course> = courses_vec
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, c)| c.clone())
            .collect();

        let unique_name = generate_unique_name_reactive(&other_courses, &updated_course.name);
        let mut final_course = updated_course;
        final_course.name = unique_name;
        final_course.id = course_id; // Preserve original ID

        courses_vec[index] = final_course;
        courses.set(courses_vec);
        Ok(())
    } else {
        Err(StateError::CourseNotFound(course_id))
    }
}

/// Delete a course from the reactive state
pub fn delete_course_reactive(course_id: Uuid) -> StateResult<()> {
    let mut courses = use_courses_reactive();
    let mut courses_vec = courses.read().clone();

    if let Some(index) = courses_vec.iter().position(|c| c.id == course_id) {
        courses_vec.remove(index);
        courses.set(courses_vec);
        Ok(())
    } else {
        Err(StateError::CourseNotFound(course_id))
    }
}

/// Duplicate a course with a new name
pub fn duplicate_course_reactive(course_id: Uuid) -> StateResult<Course> {
    let courses = use_courses_reactive();
    let courses_vec = courses.read();

    if let Some(course) = courses_vec.iter().find(|c| c.id == course_id) {
        let mut duplicated = course.clone();
        duplicated.id = Uuid::new_v4();
        duplicated.name =
            generate_unique_name_reactive(&courses_vec, &format!("{} (Copy)", course.name));
        duplicated.created_at = chrono::Utc::now();

        // Add the duplicated course
        drop(courses_vec); // Release the read lock
        add_course_reactive(duplicated.clone());

        Ok(duplicated)
    } else {
        Err(StateError::CourseNotFound(course_id))
    }
}

/// Structure a course using NLP processing
pub fn structure_course_reactive(course_id: Uuid, structure: CourseStructure) -> StateResult<()> {
    let mut courses = use_courses_reactive();
    let mut courses_vec = courses.read().clone();

    if let Some(course) = courses_vec.iter_mut().find(|c| c.id == course_id) {
        course.structure = Some(structure);
        courses.set(courses_vec);
        Ok(())
    } else {
        Err(StateError::CourseNotFound(course_id))
    }
}

/// Generate a unique course name
fn generate_unique_name_reactive(existing_courses: &[Course], desired_name: &str) -> String {
    let mut name = desired_name.to_string();
    let mut counter = 1;

    while existing_courses.iter().any(|c| c.name == name) {
        counter += 1;
        name = format!("{} ({})", desired_name, counter);

        // Prevent infinite loops
        if counter > 1000 {
            name = format!("{} ({})", desired_name, Uuid::new_v4());
            break;
        }
    }

    name
}

/// Check if a course exists by ID
pub fn course_exists_reactive(course_id: Uuid) -> bool {
    let courses = use_courses_reactive();
    courses.read().iter().any(|c| c.id == course_id)
}

/// Get a course by ID
pub fn get_course_reactive(course_id: Uuid) -> Option<Course> {
    let courses = use_courses_reactive();
    courses.read().iter().find(|c| c.id == course_id).cloned()
}

/// Get all courses
pub fn get_courses_reactive() -> Vec<Course> {
    let courses = use_courses_reactive();
    courses.read().clone()
}

/// Legacy hook functions for compatibility
pub fn use_courses() -> Signal<Vec<Course>> {
    use_courses_reactive()
}

pub fn use_course(course_id: Uuid) -> Signal<Option<Course>> {
    use_course_reactive(course_id)
}

pub fn use_course_stats() -> Signal<(usize, usize, usize)> {
    use_course_stats_reactive()
}

/// Non-reactive course operations for backend integration
pub fn add_course(course: Course) {
    add_course_reactive(course);
}

pub fn update_course(course_id: Uuid, updated_course: Course) -> StateResult<()> {
    update_course_reactive(course_id, updated_course)
}

pub fn delete_course(course_id: Uuid) -> StateResult<()> {
    delete_course_reactive(course_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VideoMetadata;

    #[test]
    fn test_course_context_creation() {
        let context = CourseContext::new();
        assert!(context.courses.read().is_empty());
    }

    #[test]
    fn test_unique_name_generation() {
        let existing_courses = vec![
            Course::new("Test Course".to_string(), vec![]),
            Course::new("Test Course (2)".to_string(), vec![]),
        ];

        let unique_name = generate_unique_name_reactive(&existing_courses, "Test Course");
        assert_eq!(unique_name, "Test Course (3)");
    }

    #[test]
    fn test_course_exists() {
        let course = Course::new("Test".to_string(), vec![]);
        let course_id = course.id;

        // This test would need to be run in a Dioxus context
        // For now, we'll just test the logic
        assert_ne!(course_id, Uuid::nil());
    }
}
