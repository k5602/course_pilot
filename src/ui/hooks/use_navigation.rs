use crate::types::{Course, Route};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub route: Option<Route>,
    pub active: bool,
}

/// Navigation manager hook
#[derive(Clone)]
pub struct NavigationManager {
    pub current_route: Route,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub navigate_to: EventHandler<Route>,
    pub go_back: EventHandler<()>,
}

pub fn use_navigation_manager() -> NavigationManager {
    let app_state = crate::ui::hooks::use_app_state();
    let current_route = app_state.read().current_route;
    let courses = app_state.read().courses.clone();

    let breadcrumbs = generate_breadcrumbs(current_route, &courses);

    let navigate_to = EventHandler::new({
        let mut app_state = app_state;
        move |route: Route| {
            app_state.write().current_route = route;
        }
    });

    let go_back = EventHandler::new({
        let mut app_state = app_state;
        move |_| {
            // Simple back navigation - go to dashboard
            app_state.write().current_route = Route::Dashboard;
        }
    });

    NavigationManager {
        current_route,
        breadcrumbs,
        navigate_to,
        go_back,
    }
}

/// Generate breadcrumbs based on current route
fn generate_breadcrumbs(current_route: Route, courses: &[Course]) -> Vec<BreadcrumbItem> {
    match current_route {
        Route::Dashboard => vec![BreadcrumbItem {
            label: "Dashboard".to_string(),
            route: None,
            active: true,
        }],
        Route::PlanView(course_id) => {
            let course_name = courses
                .iter()
                .find(|c| c.id == course_id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown Course".to_string());

            vec![
                BreadcrumbItem {
                    label: "Dashboard".to_string(),
                    route: Some(Route::Dashboard),
                    active: false,
                },
                BreadcrumbItem {
                    label: course_name,
                    route: None,
                    active: true,
                },
            ]
        }
        Route::Settings => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard),
                active: false,
            },
            BreadcrumbItem {
                label: "Settings".to_string(),
                route: None,
                active: true,
            },
        ],
        Route::AddCourse => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard),
                active: false,
            },
            BreadcrumbItem {
                label: "Add Course".to_string(),
                route: None,
                active: true,
            },
        ],
        Route::AllCourses => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard),
                active: false,
            },
            BreadcrumbItem {
                label: "All Courses".to_string(),
                route: None,
                active: true,
            },
        ],
        #[cfg(debug_assertions)]
        Route::ToastTest => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard),
                active: false,
            },
            BreadcrumbItem {
                label: "Toast Test".to_string(),
                route: None,
                active: true,
            },
        ],
    }
}
