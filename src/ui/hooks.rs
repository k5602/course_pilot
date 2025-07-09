//! Custom hooks for backend actions/state in Course Pilot UI.
//! Provides ergonomic, reactive access to AppState, notes, courses, and planner APIs.
//! Uses Dioxus signals, rusqlite DB connection from context, and error handling for robust integration.

use anyhow::Result;
use course_pilot::storage::database;
use course_pilot::storage::notes;
use course_pilot::types::{AppState, Course, Note, Plan};
use dioxus::prelude::*;
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

/// Provides access to the global rusqlite DB connection.
pub fn use_db_conn() -> Rc<Connection> {
    use_context::<Rc<Connection>>()
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
    let conn = use_db_conn();
    let show_toast = use_show_toast();
    Rc::new(move |course: Course| {
        let mut state = app_state.write();
        match database::save_course(&conn, &course) {
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
pub fn use_notes(course_id: Uuid, video_id: Option<Uuid>) -> Memo<Vec<Note>> {
    let conn = use_db_conn();
    use_memo(move || {
        if let Some(video_id) = video_id {
            notes::get_notes_by_video(&conn, video_id).unwrap_or_default()
        } else {
            notes::get_notes_by_course(&conn, course_id).unwrap_or_default()
        }
    })
}

/// Add or update a note and persist to DB.
#[allow(dead_code)]
pub fn use_save_note() -> Rc<dyn FnMut(Note) -> Result<()>> {
    let mut app_state = use_app_state();
    let conn = use_db_conn();
    let show_toast = use_show_toast();
    Rc::new(move |note: Note| {
        // If note exists, update; else, create
        let exists = notes::get_note_by_id(&conn, note.id)?.is_some();
        if exists {
            match notes::update_note(&conn, &note) {
                Ok(_) => {
                    let mut state = app_state.write();
                    if let Some(existing) = state.notes.iter_mut().find(|n| n.id == note.id) {
                        *existing = note;
                    }
                    show_toast("Note updated", ToastVariant::Success);
                    Ok(())
                }
                Err(e) => {
                    show_toast("Failed to update note", ToastVariant::Error);
                    Err(e.into())
                }
            }
        } else {
            match notes::create_note(&conn, &note) {
                Ok(_) => {
                    let mut state = app_state.write();
                    state.notes.push(note);
                    show_toast("Note added", ToastVariant::Success);
                    Ok(())
                }
                Err(e) => {
                    show_toast("Failed to add note", ToastVariant::Error);
                    Err(e.into())
                }
            }
        }
    })
}

/// Delete a note by ID and from DB, with toast feedback.
#[allow(dead_code)]
pub fn use_delete_note() -> Rc<dyn FnMut(Uuid) -> Result<()>> {
    let mut app_state = use_app_state();
    let conn = use_db_conn();
    let show_toast = use_show_toast();
    Rc::new(move |note_id: Uuid| {
        notes::delete_note(&conn, note_id)?;
        let mut state = app_state.write();
        let before = state.notes.len();
        state.notes.retain(|n| n.id != note_id);
        if state.notes.len() < before {
            show_toast("Note deleted", ToastVariant::Success);
        }
        Ok(())
    })
}

// --- Planner Hooks ---

/// Returns a memoized plan for a given course (always queries DB).
pub fn use_plan(course_id: Uuid) -> Memo<Option<Plan>> {
    let _app_state = use_app_state();
    let conn = use_db_conn();
    use_memo(move || database::get_plan_by_course_id(&conn, &course_id).unwrap_or(None))
}

/// Save or update a plan and persist to DB.
#[allow(dead_code)]
pub fn use_save_plan() -> Rc<dyn FnMut(Plan) -> Result<()>> {
    let mut app_state = use_app_state();
    let conn = use_db_conn();
    let show_toast = use_show_toast();
    Rc::new(move |plan: Plan| match database::save_plan(&conn, &plan) {
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
