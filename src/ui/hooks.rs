//! Custom hooks for backend actions/state in Course Pilot UI.
//! Provides ergonomic, reactive access to AppState, notes, courses, and planner APIs.
//! Uses Dioxus signals, rusqlite DB connection from context, and error handling for robust integration.

use anyhow::Result;
use course_pilot::storage::database;
use course_pilot::storage::notes;
use course_pilot::types::{AppState, Course, Note, Plan};
use dioxus::prelude::{UseFuture, use_future, spawn, *};
use rusqlite::Connection;
use std::rc::Rc;
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
pub fn use_db() -> Rc<course_pilot::storage::database::Database> {
    use_context::<Rc<course_pilot::storage::database::Database>>()
}

/// Provides access to the async backend adapter.
pub fn use_backend_adapter() -> std::sync::Arc<crate::ui::backend_adapter::Backend> {
    use_context::<std::sync::Arc<crate::ui::backend_adapter::Backend>>()
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

/// Add a new course and persist to DB.
#[allow(dead_code)]
pub fn use_add_course() -> Rc<dyn FnMut(Course) -> Result<()>> {
   let mut app_state = use_app_state();
   let backend_adapter = use_backend_adapter();
   let show_toast = use_show_toast();
   Rc::new(move |course: Course| {
       // NOTE: This should be refactored to async for true async integration.
       // For now, call blocking for compatibility.
       let mut state = app_state.write();
       match futures::executor::block_on(backend_adapter.as_ref().create_course(course.clone())) {
           Ok(_) => {
               state.courses.push(course);
               show_toast("Course added", ToastVariant::Success);
               Ok(())
           }
           Err(e) => {
               show_toast("Failed to add course", ToastVariant::Error);
               Err(e.into())
           }
       }
   })
}

// --- Notes Hooks ---

/// Returns a memoized list of notes for a given course or video.
/// If video_id is Some, returns video-level notes; if None, returns course-level notes.
/// Always queries the DB for latest notes.
pub fn use_notes(course_id: Uuid, video_id: Option<Uuid>) -> UseFuture {
    let backend_adapter = use_backend_adapter();
    use_future(move || async move {
        if let Some(video_id) = video_id {
            backend_adapter.as_ref().list_notes_by_video(video_id).await.unwrap_or_default()
        } else {
            backend_adapter.as_ref().list_notes_by_course(course_id).await.unwrap_or_default()
        }
    })
}

/// Add or update a note and persist to DB.
#[allow(dead_code)]
pub fn use_save_note() -> Rc<dyn FnMut(Note) -> Result<()>> {
    let mut app_state = use_app_state();
    let backend_adapter = use_backend_adapter();
    let show_toast = use_show_toast();
    Rc::new(move |note: Note| {
        // NOTE: This should be refactored to async for true async integration.
        // For now, call blocking for compatibility.
        match futures::executor::block_on(backend_adapter.as_ref().save_note(note.clone())) {
            Ok(_) => {
                let mut state = app_state.write();
                if let Some(existing) = state.notes.iter_mut().find(|n| n.id == note.id) {
                    *existing = note;
                    show_toast("Note updated", ToastVariant::Success);
                } else {
                    state.notes.push(note);
                    show_toast("Note added", ToastVariant::Success);
                }
                Ok(())
            }
            Err(e) => {
                show_toast("Failed to save note", ToastVariant::Error);
                Err(e.into())
            }
        }
    })
}

/// Delete a note by ID and from DB, with toast feedback.
#[allow(dead_code)]
pub fn use_delete_note() -> Rc<dyn FnMut(Uuid) -> Result<()>> {
    let mut app_state = use_app_state();
    let backend_adapter = use_backend_adapter();
    let show_toast = use_show_toast();
    Rc::new(move |note_id: Uuid| {
        // NOTE: This should be refactored to async for true async integration.
        // For now, call blocking for compatibility.
        match futures::executor::block_on(backend_adapter.as_ref().delete_note(note_id)) {
            Ok(_) => {
                let mut state = app_state.write();
                let before = state.notes.len();
                state.notes.retain(|n| n.id != note_id);
                if state.notes.len() < before {
                    show_toast("Note deleted", ToastVariant::Success);
                }
                Ok(())
            }
            Err(e) => {
                show_toast("Failed to delete note", ToastVariant::Error);
                Err(e.into())
            }
        }
    })
}

// --- Planner Hooks ---

/// Returns a memoized plan for a given course (always queries DB).
pub fn use_plan(course_id: Uuid) -> UseFuture {
    let backend_adapter = use_backend_adapter();
    use_future(move || async move {
        backend_adapter.as_ref().get_plan_by_course(course_id).await.unwrap_or(None)
    })
}

/// Save or update a plan and persist to DB.
#[allow(dead_code)]
pub fn use_save_plan() -> Rc<dyn FnMut(Plan) -> Result<()>> {
    let mut app_state = use_app_state();
    let backend_adapter = use_backend_adapter();
    let show_toast = use_show_toast();
    Rc::new(move |plan: Plan| {
        // NOTE: This should be refactored to async for true async integration.
        // For now, call blocking for compatibility.
        match futures::executor::block_on(backend_adapter.as_ref().save_plan(plan.clone())) {
            Ok(_) => {
                let mut state = app_state.write();
                if let Some(existing) = state.plans.iter_mut().find(|p| p.id == plan.id) {
                    *existing = plan;
                } else {
                    state.plans.push(plan);
                }
                show_toast("Plan saved", ToastVariant::Success);
                Ok(())
            }
            Err(e) => {
                show_toast("Failed to save plan", ToastVariant::Error);
                Err(e.into())
            }
        }
    })
}

// --- Utility: Toast Feedback ---

/// Shows a toast notification using dioxus-toast.
pub fn use_show_toast() -> Rc<dyn Fn(&str, ToastVariant)> {
    Rc::new(move |message: &str, variant: ToastVariant| {
        crate::ui::components::toast::show_toast(message, variant);
    })
}

// --- End of hooks.rs ---
