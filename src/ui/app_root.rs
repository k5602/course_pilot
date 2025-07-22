use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_signals::Signal;
use log::info;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::storage::database::Database;
use crate::types::{AppState, Route};
use crate::ui::backend_adapter::Backend;
use crate::ui::components::{toast, ToastContainer};
use crate::ui::layout::AppShell;
use crate::ui::theme_unified::{AppTheme, ThemeContext};

/// Clean app initialization and context providers only
#[component]
pub fn AppRoot() -> Element {
    // Initialize core services
    let services = use_app_services();
    
    // Provide all contexts
    use_context_provider(|| Signal::new(ThemeContext::new()));
    toast::provide_toast_manager();
    provide_context(services.backend);
    provide_context(services.app_state);
    
    // Handle theme synchronization
    use_theme_sync();
    
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.out.css") }
        ToastContainer {}
        AppShell {}
    }
}

/// Initialize core application services
struct AppServices {
    backend: Arc<Backend>,
    app_state: Signal<AppState>,
}

fn use_app_services() -> AppServices {
    let db_path = PathBuf::from("course_pilot.db");
    let db = Arc::new(Database::new(&db_path).expect("Failed to initialize database"));
    let backend = Arc::new(Backend::new(db.clone()));
    
    // Load initial data
    let initial_state = load_initial_state(&db);
    let app_state = use_signal(|| initial_state);
    
    AppServices { backend, app_state }
}

/// Load initial application state from database
fn load_initial_state(db: &Arc<Database>) -> AppState {
    let courses = crate::storage::load_courses(db).unwrap_or_default();
    let mut plans = Vec::new();
    let mut notes = Vec::new();

    // Load related data for each course
    for course in &courses {
        if let Ok(Some(plan)) = crate::storage::get_plan_by_course_id(db, &course.id) {
            plans.push(plan);
        }
        if let Ok(conn) = db.get_conn() {
            if let Ok(mut course_notes) = crate::storage::get_notes_by_course(&conn, course.id) {
                notes.append(&mut course_notes);
            }
        }
    }

    AppState {
        courses,
        plans,
        notes,
        current_route: Route::Dashboard,
        active_import: None,
        contextual_panel: Default::default(),
        sidebar_open_mobile: false,
    }
}

/// Handle theme synchronization with WebView
fn use_theme_sync() {
    let window = use_window();
    let theme_signal = crate::ui::theme_unified::use_theme_context();
    
    use_effect(move || {
        let theme_name = theme_signal.read().theme.to_string();
        info!("🎨 Applying theme to WebView: {}", theme_name);

        let _ = window.webview.evaluate_script(&format!(
            "document.documentElement.setAttribute('data-theme', '{}');",
            theme_name
        ));

        toast::toast::info(format!("Theme set to: {}", theme_name));
    });
}

impl fmt::Display for AppTheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
