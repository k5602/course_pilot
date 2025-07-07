use dioxus::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

// Local imports
use crate::ui::layout::AppShell;
use crate::ui::theme_unified;
use course_pilot::storage::{get_notes_by_course, get_plan_by_course_id, init_db, init_notes_table, load_courses};
use course_pilot::types::{AppState, Route};
use dioxus_toast::ToastManager;

/// AppRoot: The root entry point for the Dioxus app.
/// - Initializes SQLite DB and loads all data into AppState
/// - Provides AppState and DB connection in context
/// - Provides theme context for DaisyUI theming
/// - Links required stylesheets (DaisyUI/Tailwind)
/// - Mounts the AppShell layout
#[component]
pub fn AppRoot() -> Element {
    // --- DATABASE AND STATE INITIALIZATION ---
    let db_path = PathBuf::from("course_pilot.db");
    let conn = Rc::new(init_db(&db_path).expect("Failed to open DB"));
    init_notes_table(&conn).expect("Failed to init notes table");

    let courses = load_courses(&conn).unwrap_or_default();
    let mut plans = Vec::new();
    let mut notes = Vec::new();
    for course in &courses {
        if let Ok(Some(plan)) = get_plan_by_course_id(&conn, &course.id) {
            plans.push(plan);
        }
        if let Ok(mut course_notes) = get_notes_by_course(&conn, course.id) {
            notes.append(&mut course_notes);
        }
    }

    // --- CONTEXT PROVIDERS ---
    let app_state = use_signal(|| AppState {
        courses,
        plans,
        notes,
        current_route: Route::Dashboard,
        active_import: None,
        contextual_panel: Default::default(),
        sidebar_open_mobile: false,
    });

    use_context_provider(|| Signal::new(ToastManager::default()));
    provide_context(conn.clone());
    provide_context(app_state.clone());
    theme_unified::provide_theme_context();

    // --- RENDER ---
    rsx! {

        // Mount the main application shell, which will consume the contexts provided above.
        AppShell {}
    }
}
