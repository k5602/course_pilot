use crate::state::use_courses_reactive;
use crate::types::{Course, Route};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub route: Option<Route>,
    pub active: bool,
}

/// Enhanced navigation manager with deep linking and browser navigation support
#[derive(Clone)]
pub struct NavigationManager {
    pub current_route: Route,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub navigate_to: Callback<Route>,
    pub go_back: Callback<()>,
    pub go_forward: Callback<()>,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub replace_route: Callback<Route>,
}

pub fn use_navigation_manager() -> NavigationManager {
    let courses = use_courses_reactive();
    let current_route = use_route::<Route>();
    let navigator = use_navigator();

    // Track navigation history for better back/forward support
    let navigation_history = use_signal(Vec::<Route>::new);
    let current_history_index = use_signal(|| 0usize);

    let breadcrumbs = generate_breadcrumbs(current_route.clone(), &courses());

    // Enhanced navigation with history tracking
    let navigate_to = use_callback({
        let mut navigation_history = navigation_history;
        let mut current_history_index = current_history_index;
        let navigator = navigator;

        move |route: Route| {
            // Add to history
            let mut history = navigation_history();
            let current_index = current_history_index();

            // Remove any forward history if we're navigating from middle of history
            if current_index < history.len() - 1 {
                history.truncate(current_index + 1);
            }

            history.push(route.clone());
            navigation_history.set(history);
            current_history_index.set(current_history_index() + 1);

            navigator.push(route);
        }
    });

    let go_back = use_callback({
        let mut current_history_index = current_history_index;
        let navigator = navigator;

        move |_| {
            let current_index = current_history_index();
            if current_index > 0 {
                current_history_index.set(current_index - 1);
            }
            navigator.go_back();
        }
    });

    let go_forward = use_callback({
        let mut current_history_index = current_history_index;
        let navigator = navigator;

        move |_| {
            let history = navigation_history();
            let current_index = current_history_index();
            if current_index < history.len() - 1 {
                current_history_index.set(current_index + 1);
            }
            navigator.go_forward();
        }
    });

    let replace_route = use_callback({
        let navigator = navigator;

        move |route: Route| {
            navigator.replace(route);
        }
    });

    // Calculate navigation capabilities
    let history = navigation_history();
    let current_index = current_history_index();
    let can_go_back = current_index > 0;
    let can_go_forward = !history.is_empty() && current_index < history.len() - 1;

    NavigationManager {
        current_route,
        breadcrumbs,
        navigate_to,
        go_back,
        go_forward,
        can_go_back,
        can_go_forward,
        replace_route,
    }
}

/// Enhanced breadcrumb generation with better deep linking support
fn generate_breadcrumbs(current_route: Route, courses: &[Course]) -> Vec<BreadcrumbItem> {
    match current_route {
        Route::Home {} => vec![BreadcrumbItem {
            label: "Dashboard".to_string(),
            route: Some(Route::Dashboard {}),
            active: true,
        }],
        Route::Dashboard {} => vec![BreadcrumbItem {
            label: "Dashboard".to_string(),
            route: None,
            active: true,
        }],
        Route::PlanView { course_id } => {
            // Parse course_id string to UUID with better error handling
            let course_uuid = match uuid::Uuid::parse_str(&course_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    return vec![
                        BreadcrumbItem {
                            label: "Dashboard".to_string(),
                            route: Some(Route::Dashboard {}),
                            active: false,
                        },
                        BreadcrumbItem {
                            label: "Invalid Course ID".to_string(),
                            route: None,
                            active: true,
                        },
                    ];
                }
            };

            // Find course with better fallback handling
            let course = courses.iter().find(|c| c.id == course_uuid);
            let course_name = course
                .map(|c| c.name.clone())
                .unwrap_or_else(|| format!("Course {}", &course_id[..8])); // Show first 8 chars of ID

            let mut breadcrumbs = vec![
                BreadcrumbItem {
                    label: "Dashboard".to_string(),
                    route: Some(Route::Dashboard {}),
                    active: false,
                },
                BreadcrumbItem {
                    label: "All Courses".to_string(),
                    route: Some(Route::AllCourses {}),
                    active: false,
                },
            ];

            // Add course-specific breadcrumb
            if course.is_some() {
                breadcrumbs.push(BreadcrumbItem {
                    label: format!("{course_name} - Study Plan"),
                    route: None,
                    active: true,
                });
            } else {
                breadcrumbs.push(BreadcrumbItem {
                    label: "Course Not Found".to_string(),
                    route: None,
                    active: true,
                });
            }

            breadcrumbs
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
                label: "Import Course".to_string(),
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
