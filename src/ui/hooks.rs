//! Custom hooks for backend actions/state in Course Pilot UI.
//! Provides ergonomic, reactive access to AppState, notes, courses, and planner APIs.
//! Uses Dioxus signals, and error handling for robust integration.

pub mod use_courses;
pub mod use_navigation;
pub mod use_modals;

// Re-export commonly used hooks
pub use use_courses::{use_course_manager, use_course_progress};
pub use use_navigation::{use_navigation_manager, BreadcrumbItem};
pub use use_modals::{use_modal_manager, use_form_manager};

use anyhow::Result;
use crate::types::{AppState, Course, Note, Plan};
use dioxus::prelude::*;
use futures::Future;

use std::sync::Arc;
use uuid::Uuid;

// Re-export for convenience
pub use crate::ui::components::toast::ToastVariant;

// --- AppState Hook ---

/// Provides global access to the AppState signal.
/// Call at the root of your app and use in all components.
pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}

/// Provides access to the global Database instance.
pub fn use_db() -> Arc<crate::storage::database::Database> {
    use_context::<Arc<crate::storage::database::Database>>()
}

/// Provides access to the async backend adapter.
pub fn use_backend_adapter() -> Arc<crate::ui::backend_adapter::Backend> {
    use_context::<Arc<crate::ui::backend_adapter::Backend>>()
}

// --- Courses Hooks ---

/// Returns a memoized list of all courses.
pub fn use_courses() -> Memo<Vec<Course>> {
    let app_state = use_app_state();
    use_memo(move || app_state.read().courses.clone())
}

/// Returns a memoized course by ID.
#[allow(dead_code)]
pub fn use_course(id: uuid::Uuid) -> Memo<Option<Course>> {
    let app_state = use_app_state();
    use_memo(move || {
        app_state
            .read()
            .courses
            .iter()
            .find(|c| c.id == id)
            .cloned()
    })
}

/// Load courses using proper async patterns with use_resource
pub fn use_courses_resource() -> Resource<Result<Vec<Course>>> {
    let backend = use_backend_adapter();
    use_resource(move || {
        let backend = backend.clone();
        async move {
            backend.list_courses().await
        }
    })
}

/// Get a function to add courses asynchronously (for use in event handlers)
pub fn use_add_course_action() -> impl Fn(Course) + Clone {
    let backend = use_backend_adapter();
    
    move |course: Course| {
        let backend = backend.clone();
        spawn(async move {
            match backend.create_course(course).await {
                Ok(_) => {
                    crate::ui::components::toast::toast::success("Course added successfully");
                }
                Err(e) => {
                    crate::ui::components::toast::toast::error(&format!("Failed to add course: {}", e));
                }
            }
        });
    }
}

// --- Notes Hooks ---

/// Returns a memoized list of notes for a given course or video.
/// If video_id is Some, returns video-level notes; if None, returns course-level notes.
/// Always queries the DB for latest notes.
pub fn use_notes(course_id: Uuid, video_id: Option<Uuid>) -> impl Future<Output = Result<Vec<Note>>> {
    let backend_adapter = use_backend_adapter();
    async move {
        if let Some(video_id) = video_id {
            backend_adapter.list_notes_by_video(video_id).await
        } else {
            backend_adapter.list_notes_by_course(course_id).await
        }
    }
}

/// Load notes using proper async patterns with use_resource
pub fn use_notes_resource(course_id: Uuid, video_id: Option<Uuid>) -> Resource<Result<Vec<Note>>> {
    let backend = use_backend_adapter();
    use_resource(move || {
        let backend = backend.clone();
        async move {
            if let Some(video_id) = video_id {
                backend.list_notes_by_video(video_id).await
            } else {
                backend.list_notes_by_course(course_id).await
            }
        }
    })
}

/// Get a function to save notes asynchronously (for use in event handlers)
pub fn use_save_note_action() -> impl Fn(Note) + Clone {
    let backend = use_backend_adapter();
    
    move |note: Note| {
        let backend = backend.clone();
        spawn(async move {
            match backend.save_note(note).await {
                Ok(_) => {
                    crate::ui::components::toast::toast::success("Note saved successfully");
                }
                Err(e) => {
                    crate::ui::components::toast::toast::error(&format!("Failed to save note: {}", e));
                }
            }
        });
    }
}

/// Get a function to delete notes asynchronously (for use in event handlers)
pub fn use_delete_note_action() -> impl Fn(Uuid) + Clone {
    let backend = use_backend_adapter();
    
    move |note_id: Uuid| {
        let backend = backend.clone();
        spawn(async move {
            match backend.delete_note(note_id).await {
                Ok(_) => {
                    crate::ui::components::toast::toast::success("Note deleted successfully");
                }
                Err(e) => {
                    crate::ui::components::toast::toast::error(&format!("Failed to delete note: {}", e));
                }
            }
        });
    }
}

// --- Planner Hooks ---

/// Load plan using proper async patterns with use_resource
pub fn use_plan_resource(course_id: Uuid) -> Resource<Result<Option<Plan>>> {
    let backend = use_backend_adapter();
    use_resource(move || {
        let backend = backend.clone();
        async move {
            backend.get_plan_by_course(course_id).await
        }
    })
}

/// Load plan progress using use_resource for reactive progress loading
pub fn use_plan_progress_resource(plan_id: Uuid) -> Resource<Result<crate::ui::backend_adapter::ProgressInfo>> {
    let backend = use_backend_adapter();
    use_resource(move || {
        let backend = backend.clone();
        async move {
            backend.get_plan_progress(plan_id).await
        }
    })
}

/// Get a function to save plans asynchronously (for use in event handlers)
pub fn use_save_plan_action() -> impl Fn(Plan) + Clone {
    let backend = use_backend_adapter();
    
    move |plan: Plan| {
        let backend = backend.clone();
        spawn(async move {
            match backend.save_plan(plan).await {
                Ok(_) => {
                    crate::ui::components::toast::toast::success("Plan saved successfully");
                }
                Err(e) => {
                    crate::ui::components::toast::toast::error(&format!("Failed to save plan: {}", e));
                }
            }
        });
    }
}

/// Get a function to toggle plan item completion asynchronously with state refresh
pub fn use_toggle_plan_item_action() -> impl Fn(Uuid, usize, bool) + Clone {
    let backend = use_backend_adapter();
    
    move |plan_id: Uuid, item_index: usize, completed: bool| {
        let backend = backend.clone();
        spawn(async move {
            match backend.update_plan_item_completion(plan_id, item_index, completed).await {
                Ok(_) => {
                    crate::ui::components::toast::toast::success("Progress updated");
                    // For desktop apps, the UI will automatically refresh through reactive state
                }
                Err(e) => {
                    crate::ui::components::toast::toast::error(&format!("Failed to update progress: {}", e));
                }
            }
        });
    }
}

// --- Utility: Toast Feedback ---

/// Shows a toast notification using dioxus-toast.
pub fn use_show_toast() -> Arc<dyn Fn(&str, ToastVariant) + Send + Sync> {
    Arc::new(move |message: &str, variant: ToastVariant| {
        crate::ui::components::toast::show_toast(message, variant);
    })
}

// --- End of hooks.rs ---
