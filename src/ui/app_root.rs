use dioxus::prelude::*;
use dioxus_desktop::use_window; 
use std::fmt;
use std::path::PathBuf;
// Local imports
use crate::ui::components::toast;
use crate::ui::components::ToastContainer;
use crate::ui::layout::AppShell;
use crate::ui::theme_unified::{AppTheme, ThemeContext};
use crate::storage::database::Database;
use crate::types::{AppState, Route};
use dioxus_signals::Signal;
use dioxus_toast::ToastManager;
use log::info;
use std::sync::Arc;
use crate::ui::backend_adapter::Backend;

// CommandPalette and CommandAction imports removed (feature deferred)

/// AppRoot: The root entry point for the Dioxus app.
#[component]
pub fn AppRoot() -> Element {
    let window = use_window();

    // --- DATABASE AND STATE INITIALIZATION ---
    let db_path = PathBuf::from("course_pilot.db");
    let db = Arc::new(Database::new(&db_path).expect("Failed to initialize database"));
    let backend_adapter: Arc<Backend> = Arc::new(Backend::new(db.clone()));

    // Load initial data (sync, for initial AppState)
    let courses = crate::storage::load_courses(&db).unwrap_or_default();
    let mut plans = Vec::new();
    let mut notes = Vec::new();

    for course in &courses {
        if let Ok(Some(plan)) = crate::storage::get_plan_by_course_id(&db, &course.id) {
            plans.push(plan);
        }
        if let Ok(conn) = db.get_conn() {
            if let Ok(mut course_notes) = crate::storage::get_notes_by_course(&conn, course.id) {
                notes.append(&mut course_notes);
            }
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
    provide_context(backend_adapter);
    provide_context(app_state.clone());

    // --- THEME SYNCHRONIZATION ---
    // It runs whenever the theme_signal changes, ensuring the WebView UI is always in sync.
    let theme_signal = crate::ui::theme_unified::use_theme_context();
    use_effect(move || {
        let theme_name = theme_signal.read().theme.to_string();
        info!("🎨 Applying theme to WebView: {}", theme_name);

        // This command sends JavaScript to the WebView to set the theme attribute.
        let _ = window.webview.evaluate_script(&format!(
            "document.documentElement.setAttribute('data-theme', '{}');",
            theme_name
        ));

        // Show a toast to confirm the theme has been applied
        toast::toast::info(format!("Theme set to: {}", theme_name));
    });

    // --- RENDER ---
    rsx! {
        // Load CSS stylesheet
        document::Stylesheet {
            href: asset!("/assets/tailwind.out.css")
        }

        // The ToastContainer is the single source for rendering all notifications.
        ToastContainer {}

        // Command Palette modal is deferred (feature commented out for now)

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
