use dioxus::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

// Local imports
use crate::ui::components::toast;
use crate::ui::components::ToastContainer;
use crate::ui::layout::AppShell;
use crate::ui::theme_unified::{self};
use course_pilot::storage::{
    get_notes_by_course, get_plan_by_course_id, init_db, init_notes_table, load_courses,
};
use course_pilot::types::{AppState, Route};
use dioxus_desktop::use_window;
use dioxus_signals::Signal;
use dioxus_toast::ToastManager;
use log::info;

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

    // Provide theme context at the root using Dioxus signals/context API
    use_context_provider(|| Signal::new(theme_unified::ThemeContext::new()));
    // Provide toast manager context at the root
    use_context_provider(|| Signal::new(ToastManager::default()));

    // Provide other contexts
    provide_context(conn.clone());
    provide_context(app_state.clone());

    // Apply theme on mount and whenever theme changes
    let theme_signal = crate::ui::theme_unified::use_theme_context();
    use_effect(move || {
        let theme = theme_signal.read();
        let theme_name = theme.theme.as_str().to_string();
        info!("🎨 Applying theme: {}", theme_name);

        let window = use_window();
        // TODO: Use the correct Dioxus Desktop API for JS eval if/when available.
        // let _ = window.eval(&format!(
        //     "document.documentElement.setAttribute('data-theme', '{}');",
        //     theme_name
        // ));

        // Show a welcome message with the current theme
        toast::toast::info(format!("Welcome to Course Pilot! (Theme: {})", theme_name));

        ()
    });

    // --- RENDER ---
    rsx! {
        // Required for DaisyUI components
        // Note: These are now loaded from the local build in index.html
        // to support custom themes and better performance

        // Toast container for notifications
        ToastContainer {}

        // App shell
        AppShell {}
    }
}
