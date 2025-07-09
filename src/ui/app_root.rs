use dioxus::prelude::*;
use dioxus_desktop::use_window; // Import the hook to interact with the desktop window
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;

// Local imports
use crate::ui::components::toast;
use crate::ui::components::ToastContainer;
use crate::ui::layout::AppShell;
use crate::ui::theme_unified::{self, AppTheme, ThemeContext};
use course_pilot::storage::{
    get_notes_by_course, get_plan_by_course_id, init_db, init_notes_table, load_courses,
};
use course_pilot::types::{AppState, Route};
use dioxus_signals::Signal;
use dioxus_toast::ToastManager;
use log::info;

/// AppRoot: The root entry point for the Dioxus app.
#[component]
pub fn AppRoot() -> Element {
    // Get a handle to the desktop window to run JavaScript
    let window = use_window();

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

    // Provide theme context. ThemeContext::new() loads the theme from the config file.
    use_context_provider(|| Signal::new(ThemeContext::new()));

    // Provide toast manager context at the root
    use_context_provider(|| Signal::new(ToastManager::default()));

    // Provide other contexts
    provide_context(conn.clone());
    provide_context(app_state.clone());

    // --- THEME SYNCHRONIZATION ---
    // This effect hook is the key to the solution.
    // It runs whenever the theme_signal changes, ensuring the WebView UI is always in sync.
    let theme_signal = crate::ui::theme_unified::use_theme_context();
    use_effect(move || {
        let theme_name = theme_signal.read().theme.to_string();
        info!("🎨 Applying theme to WebView: {}", theme_name);

        // This command sends JavaScript to the WebView to set the theme attribute.
        // We use evaluate_script, which is the correct method on the wry::WebView object.
        let _ = window.webview.evaluate_script(&format!(
            "document.documentElement.setAttribute('data-theme', '{}');",
            theme_name
        ));

        // Show a toast to confirm the theme has been applied
        toast::toast::info(format!("Theme set to: {}", theme_name));
    });

    // --- RENDER ---
    rsx! {
        // The ToastContainer is the single source for rendering all notifications.
        ToastContainer {}

        // App shell which contains the rest of the UI
        AppShell {}
    }
}

// Implement Display for AppTheme
impl fmt::Display for AppTheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
