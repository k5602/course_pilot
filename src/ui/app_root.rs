use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_signals::Signal;
use log::{error, info};
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::state::{
    ContextualPanelContextProvider, CourseContextProvider, ImportContextProvider,
    MobileSidebarContextProvider, PlanContextProvider, VideoContextProvider,
    initialize_global_state,
};
use crate::storage::core::Database;
use crate::types::{AppState, Route};

use crate::ui::{ToastContainer, provide_toast_manager, toast_helpers};
// Backend hooks are accessed through individual components
use crate::ui::{AppTheme, ThemeContext};

#[component]
pub fn AppRoot() -> Element {
    // Initialize core services
    let services = use_app_services();

    // Provide all contexts
    use_context_provider(|| Signal::new(ThemeContext::new()));
    provide_toast_manager();
    provide_context(services.database);
    provide_context(services.app_state);

    // Handle theme synchronization
    use_theme_sync();

    rsx! {
        document::Style { {include_str!("../../assets/tailwind.out.css")} }
        ToastContainer {}

        // Wrap the app with modern context providers
        CourseContextProvider {
            PlanContextProvider {
                ImportContextProvider {
                    ContextualPanelContextProvider {
                        MobileSidebarContextProvider {
                            VideoContextProvider {
                                crate::state::ExportProgressProvider {
                                    AppWithContexts { app_state: services.app_state }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AppWithContexts(app_state: Signal<AppState>) -> Element {
    // Initialize modern state management after contexts are provided
    initialize_global_state(app_state);

    rsx! {
        Router::<Route> {}
    }
}

/// Initialize core application services
struct AppServices {
    database: Arc<Database>,
    app_state: Signal<AppState>,
}

fn use_app_services() -> AppServices {
    let db_path = PathBuf::from("course_pilot.db");

    let db = match Database::new(&db_path) {
        Ok(database) => Arc::new(database),
        Err(e) => {
            error!("Failed to initialize database at {db_path:?}: {e}");
            toast_helpers::error(
                "Failed to initialize database. Please check file permissions and try again."
                    .to_string(),
            );
            // Create a fallback in-memory database to prevent app crash
            match Database::new(std::path::Path::new(":memory:")) {
                Ok(fallback_db) => {
                    toast_helpers::warning(
                        "Using temporary in-memory database. Data will not be saved.".to_string(),
                    );
                    Arc::new(fallback_db)
                },
                Err(fallback_err) => {
                    error!("Failed to create fallback database: {fallback_err}");
                    panic!("Cannot initialize any database. Application cannot continue.");
                },
            }
        },
    };

    // Load initial data
    let initial_state = load_initial_state(&db);
    let app_state = use_signal(|| initial_state);

    AppServices { database: db, app_state }
}

/// Load initial application state from database
fn load_initial_state(db: &Arc<Database>) -> AppState {
    let courses = match crate::storage::load_courses(db) {
        Ok(courses) => courses,
        Err(e) => {
            error!("Failed to load courses from database: {e}");
            toast_helpers::error(
                "Failed to load courses. Starting with empty course list.".to_string(),
            );
            Vec::new()
        },
    };

    let mut plans = Vec::new();
    let mut notes = Vec::new();

    // Load related data for each course
    for course in &courses {
        // Load plans with error handling
        match crate::storage::get_plan_by_course_id(db, &course.id) {
            Ok(Some(plan)) => plans.push(plan),
            Ok(None) => {}, // No plan exists for this course, which is fine
            Err(e) => {
                error!("Failed to load plan for course {}: {}", course.name, e);
                // Continue loading other data instead of failing completely
            },
        }

        // Load notes with error handling
        match db.get_conn() {
            Ok(conn) => {
                match crate::storage::get_notes_by_course(&conn, course.id) {
                    Ok(mut course_notes) => notes.append(&mut course_notes),
                    Err(e) => {
                        error!("Failed to load notes for course {}: {}", course.name, e);
                        // Continue loading other data instead of failing completely
                    },
                }
            },
            Err(e) => {
                error!("Failed to get database connection for loading notes: {e}");
                // Continue without loading notes for this course
            },
        }
    }

    AppState {
        courses,
        plans,
        notes,
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
        info!("🎨 Applying theme to WebView: {theme_name}");

        let _ = window.webview.evaluate_script(&format!(
            "document.documentElement.setAttribute('data-theme', '{theme_name}');"
        ));

        toast_helpers::info(format!("Theme set to: {theme_name}"));
    });
}

impl fmt::Display for AppTheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
