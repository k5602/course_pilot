use crate::state::{use_course_reactive, use_courses_reactive};
use crate::types::{Course, Route};
use crate::ui::components::toast::toast;
use dioxus::prelude::*;
use uuid::Uuid;

/// Course management hook with all course-related operations
#[derive(Clone)]
pub struct CourseManager {
    pub courses: Vec<Course>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub create_course: Callback<String>,
    pub update_course: Callback<(Uuid, String)>,
    pub delete_course: Callback<Uuid>,
    pub navigate_to_course: Callback<Uuid>,
    pub refresh: Callback<()>,
}

pub fn use_course_manager() -> CourseManager {
    let backend = crate::ui::hooks::use_backend_adapter();

    // Use reactive courses from modern state management
    let courses = use_courses_reactive();

    // Load courses
    let backend_clone = backend.clone();
    let courses_resource = use_resource(move || {
        let backend = backend_clone.clone();
        async move { backend.list_courses().await }
    });

    let courses_state = courses_resource.read_unchecked();
    let is_loading = (*courses_state).is_none();
    let error = match &*courses_state {
        Some(Err(e)) => Some(e.to_string()),
        _ => None,
    };

    // Event handlers with state refresh
    let create_course = use_callback({
        let backend = backend.clone();
        let courses_resource = courses_resource;
        move |name: String| {
            if name.trim().is_empty() {
                toast::error("Course name cannot be empty");
                return;
            }

            let backend = backend.clone();
            let mut courses_resource = courses_resource;
            let new_course = Course::new(name, vec![]);
            spawn(async move {
                match backend.create_course(new_course).await {
                    Ok(_) => {
                        toast::success("Course created successfully");
                        // Refresh the courses list
                        courses_resource.restart();
                    }
                    Err(e) => toast::error(format!("Failed to create course: {e}")),
                }
            });
        }
    });

    let update_course = use_callback({
        let backend = backend.clone();
        let courses_resource = courses_resource;
        move |(course_id, new_name): (Uuid, String)| {
            if new_name.trim().is_empty() {
                toast::error("Course name cannot be empty");
                return;
            }

            let backend = backend.clone();
            let mut courses_resource = courses_resource;
            spawn(async move {
                // Get current course and update it
                if let Ok(Some(mut course)) = backend.get_course(course_id).await {
                    course.name = new_name;
                    match backend.update_course(course).await {
                        Ok(_) => {
                            toast::success("Course updated successfully");
                            // Refresh the courses list
                            courses_resource.restart();
                        }
                        Err(e) => toast::error(format!("Failed to update course: {e}")),
                    }
                }
            });
        }
    });

    let delete_course = use_callback({
        let backend = backend.clone();
        let courses_resource = courses_resource;
        move |course_id: Uuid| {
            let backend = backend.clone();
            let mut courses_resource = courses_resource;
            spawn(async move {
                match backend.delete_course(course_id).await {
                    Ok(_) => {
                        toast::success("Course deleted successfully");
                        // Refresh the courses list
                        courses_resource.restart();
                    }
                    Err(e) => toast::error(format!("Failed to delete course: {e}")),
                }
            });
        }
    });

    let navigate_to_course = use_callback({
        let navigator = use_navigator();
        move |course_id: Uuid| {
            navigator.push(Route::PlanView {
                course_id: course_id.to_string(),
            });
        }
    });

    let refresh = use_callback({
        let mut courses_resource = courses_resource;
        move |_| {
            courses_resource.restart();
        }
    });

    CourseManager {
        courses: courses(),
        is_loading,
        error,
        create_course,
        update_course,
        delete_course,
        navigate_to_course,
        refresh,
    }
}

/// Course progress hook using reactive patterns
pub fn use_course_progress(course_id: Uuid) -> (f32, String, Option<String>) {
    let backend = crate::ui::hooks::use_backend_adapter();

    let progress_resource = use_resource(move || {
        let backend = backend.clone();
        async move { backend.get_course_progress(course_id).await }
    });

    match &*progress_resource.read_unchecked() {
        Some(Ok(Some(progress_info))) => {
            let progress = progress_info.percentage / 100.0;
            let status = if progress >= 1.0 {
                "Completed".to_string()
            } else if progress > 0.0 {
                "In Progress".to_string()
            } else {
                "Not Started".to_string()
            };
            let badge_color = if progress >= 1.0 {
                Some("success".to_string())
            } else if progress > 0.0 {
                Some("accent".to_string())
            } else {
                Some("neutral".to_string())
            };
            (progress, status, badge_color)
        }
        Some(Ok(None)) => (0.0, "Not Started".to_string(), Some("neutral".to_string())),
        Some(Err(_)) => (0.0, "Error".to_string(), Some("error".to_string())),
        None => (0.0, "Loading...".to_string(), Some("neutral".to_string())),
    }
}

/// Hook for reactive access to a specific course
pub fn use_course_reactive_hook(course_id: Uuid) -> Memo<Option<Course>> {
    use_course_reactive(course_id)
}

/// Hook for reactive access to all courses
pub fn use_courses_reactive_hook() -> Memo<Vec<Course>> {
    use_courses_reactive()
}
