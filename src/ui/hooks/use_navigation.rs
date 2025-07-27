use crate::state::use_courses_reactive;
use crate::types::{Course, Route};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub route: Option<Route>,
    pub active: bool,
}

/// Navigation manager hook using modern reactive patterns
#[derive(Clone)]
pub struct NavigationManager {
    pub current_route: Route,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub navigate_to: Callback<Route>,
    pub go_back: Callback<()>,
}

pub fn use_navigation_manager() -> NavigationManager {
    let courses = use_courses_reactive();
    let current_route = use_route::<Route>();
    let navigator = use_navigator();

    let breadcrumbs = generate_breadcrumbs(current_route.clone(), &courses());

    let navigate_to = use_callback(move |route: Route| {
        navigator.push(route);
    });

    let go_back = use_callback(move |_| {
        navigator.go_back();
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
        Route::Home {} => vec![BreadcrumbItem {
            label: "Home".to_string(),
            route: None,
            active: true,
        }],
        Route::Dashboard {} => vec![BreadcrumbItem {
            label: "Dashboard".to_string(),
            route: None,
            active: true,
        }],
        Route::PlanView { course_id } => {
            // Parse course_id string to UUID
            let course_uuid = match uuid::Uuid::parse_str(&course_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    return vec![BreadcrumbItem {
                        label: "Invalid Course".to_string(),
                        route: None,
                        active: true,
                    }];
                }
            };

            let course_name = courses
                .iter()
                .find(|c| c.id == course_uuid)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown Course".to_string());

            vec![
                BreadcrumbItem {
                    label: "Dashboard".to_string(),
                    route: Some(Route::Dashboard {}),
                    active: false,
                },
                BreadcrumbItem {
                    label: course_name,
                    route: None,
                    active: true,
                },
            ]
        }
        Route::Settings {} => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard {}),
                active: false,
            },
            BreadcrumbItem {
                label: "Settings".to_string(),
                route: None,
                active: true,
            },
        ],
        Route::AddCourse {} => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard {}),
                active: false,
            },
            BreadcrumbItem {
                label: "Add Course".to_string(),
                route: None,
                active: true,
            },
        ],
        Route::AllCourses {} => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard {}),
                active: false,
            },
            BreadcrumbItem {
                label: "All Courses".to_string(),
                route: None,
                active: true,
            },
        ],
        #[cfg(debug_assertions)]
        Route::ToastTest {} => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard {}),
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
