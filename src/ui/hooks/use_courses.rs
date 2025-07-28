use crate::storage::database::Database;
use crate::types::Course;
use crate::ui::toast_helpers;
use dioxus::prelude::*;
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;

/// Course management hook with all course-related operations
#[derive(Clone)]
pub struct CourseManager {
    db: Arc<Database>,
    pub courses: Vec<Course>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub navigate_to_course: Callback<Uuid>,
    pub refresh: Callback<()>,
    pub update_course: Callback<(Uuid, String)>,
    pub delete_course: Callback<Uuid>,
}

impl CourseManager {
    pub async fn list_courses(&self) -> Result<Vec<Course>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::storage::load_courses(&db).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn get_course(&self, id: Uuid) -> Result<Option<Course>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::storage::get_course_by_id(&db, &id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {e}")))
    }

    pub async fn create_course(&self, course: Course) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::storage::save_course(&db, &course).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn update_course(&self, course: Course) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // Verify course exists first
            let existing = crate::storage::get_course_by_id(&db, &course.id)?;
            if existing.is_none() {
                return Err(anyhow::anyhow!("Course with id {} not found", course.id));
            }

            // Update the course
            crate::storage::save_course(&db, &course).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn delete_course(&self, course_id: Uuid) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            crate::storage::delete_course(&db, &course_id).map_err(Into::into)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }
}

pub fn use_course_manager() -> CourseManager {
    let db = use_context::<Arc<Database>>();
    
    // Load courses resource
    let courses_resource = use_resource({
        let db = db.clone();
        move || {
            let db = db.clone();
            async move {
                tokio::task::spawn_blocking(move || {
                    crate::storage::load_courses(&db).map_err(Into::into)
                }).await.unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
            }
        }
    });

    let courses_state = courses_resource.read_unchecked();
    let is_loading = (*courses_state).is_none();
    let error = match &*courses_state {
        Some(Err(e)) => Some(e.to_string()),
        _ => None,
    };
    let courses = match &*courses_state {
        Some(Ok(courses)) => courses.clone(),
        _ => Vec::new(),
    };

    let navigate_to_course = use_callback({
        let navigator = use_navigator();
        move |course_id: Uuid| {
            navigator.push(crate::types::Route::PlanView {
                course_id: course_id.to_string(),
            });
        }
    });

    let refresh = use_callback(move |_| {
        // Placeholder for refresh functionality
        // In practice, this would trigger a re-fetch of courses
    });

    let update_course = use_callback({
        let db = db.clone();
        move |(course_id, new_name): (Uuid, String)| {
            let db = db.clone();
            spawn(async move {
                // Get current course and update it
                let result = tokio::task::spawn_blocking(move || {
                    if let Some(mut course) = crate::storage::get_course_by_id(&db, &course_id)? {
                        course.name = new_name;
                        crate::storage::save_course(&db, &course)?;
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!("Course not found"))
                    }
                }).await;

                match result {
                    Ok(Ok(_)) => {
                        toast_helpers::success("Course updated successfully");
                    }
                    Ok(Err(e)) => {
                        toast_helpers::error(format!("Failed to update course: {}", e));
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to update course: {}", e));
                    }
                }
            });
            // Return () to match expected callback type
        }
    });

    let delete_course = use_callback({
        let db = db.clone();
        move |course_id: Uuid| {
            let db = db.clone();
            spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    crate::storage::delete_course(&db, &course_id)
                }).await;

                match result {
                    Ok(Ok(_)) => {
                        toast_helpers::success("Course deleted successfully");
                    }
                    Ok(Err(e)) => {
                        toast_helpers::error(format!("Failed to delete course: {}", e));
                    }
                    Err(e) => {
                        toast_helpers::error(format!("Failed to delete course: {}", e));
                    }
                }
            });
            // Return () to match expected callback type
        }
    });
    
    CourseManager { 
        db,
        courses,
        is_loading,
        error,
        navigate_to_course,
        refresh,
        update_course,
        delete_course,
    }
}

/// Hook for reactive courses loading
pub fn use_courses_resource() -> Resource<Result<Vec<Course>, anyhow::Error>> {
    let course_manager = use_course_manager();

    use_resource(move || {
        let course_manager = course_manager.clone();
        async move {
            course_manager.list_courses().await
        }
    })
}

/// Hook for reactive course loading
pub fn use_course_resource(course_id: Uuid) -> Resource<Result<Option<Course>, anyhow::Error>> {
    let course_manager = use_course_manager();

    use_resource(move || {
        let course_manager = course_manager.clone();
        async move {
            course_manager.get_course(course_id).await
        }
    })
}

/// Hook for course progress using plan manager
pub fn use_course_progress(course_id: Uuid) -> (f32, String, Option<String>) {
    use super::use_plans::use_plan_manager;
    
    let plan_manager = use_plan_manager();
    
    let progress_resource = use_resource(move || {
        let plan_manager = plan_manager.clone();
        async move {
            plan_manager.get_course_progress(course_id).await
        }
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

/// Hook for course management with reactive state
pub fn use_course_management() -> (Vec<Course>, bool, Option<String>, impl Fn()) {
    let course_manager = use_course_manager();
    
    let courses_resource: Resource<Result<Vec<Course>>> = use_resource(move || {
        let course_manager = course_manager.clone();
        async move {
            course_manager.list_courses().await
        }
    });

    let courses_state = courses_resource.read_unchecked();
    let is_loading = (*courses_state).is_none();
    let error = match &*courses_state {
        Some(Err(e)) => Some(e.to_string()),
        _ => None,
    };
    let courses = match &*courses_state {
        Some(Ok(courses)) => courses.clone(),
        _ => Vec::new(),
    };

    let refresh = {
        move || {
            // Placeholder for refresh functionality
            // This would trigger a re-fetch in a real implementation
        }
    };

    (courses, is_loading, error, refresh)
}