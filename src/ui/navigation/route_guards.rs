use dioxus::prelude::*;
use uuid::Uuid;

use crate::types::{Course, Route};
use crate::ui::hooks::use_course_manager;

/// Route guard result indicating whether navigation should proceed
#[derive(Debug, Clone, PartialEq)]
pub enum RouteGuardResult {
    /// Allow navigation to proceed
    Allow,
    /// Block navigation and redirect to another route
    Redirect(Route),
    /// Block navigation and show an error
    Block(String),
}

/// Route guard trait for implementing custom route validation
pub trait RouteGuard {
    fn can_navigate(&self, route: &Route) -> RouteGuardResult;
}

/// Course existence guard - validates that courses exist before navigating to course-specific routes
pub struct CourseExistenceGuard {
    courses: Vec<Course>,
}

impl CourseExistenceGuard {
    pub fn new(courses: Vec<Course>) -> Self {
        Self { courses }
    }
}

impl RouteGuard for CourseExistenceGuard {
    fn can_navigate(&self, route: &Route) -> RouteGuardResult {
        match route {
            Route::PlanView { course_id } => {
                // Parse course_id string to UUID
                let course_uuid = match Uuid::parse_str(course_id) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        return RouteGuardResult::Block("Invalid course ID format".to_string());
                    },
                };

                // Check if course exists
                if self.courses.iter().any(|c| c.id == course_uuid) {
                    RouteGuardResult::Allow
                } else {
                    RouteGuardResult::Redirect(Route::AllCourses {})
                }
            },
            _ => RouteGuardResult::Allow,
        }
    }
}

/// Hook for using route guards in components
pub fn use_route_guard() -> RouteGuardManager {
    let course_manager = use_course_manager();
    let navigator = use_navigator();

    RouteGuardManager {
        guards: vec![Box::new(CourseExistenceGuard::new(course_manager.courses.clone()))],
        navigator,
    }
}

/// Route guard manager for handling multiple guards
pub struct RouteGuardManager {
    guards: Vec<Box<dyn RouteGuard>>,
    navigator: Navigator,
}

impl RouteGuardManager {
    /// Check if navigation to a route is allowed
    pub fn can_navigate_to(&self, route: &Route) -> RouteGuardResult {
        for guard in &self.guards {
            match guard.can_navigate(route) {
                RouteGuardResult::Allow => continue,
                result => return result,
            }
        }
        RouteGuardResult::Allow
    }

    /// Navigate to a route with guard validation
    pub fn navigate_with_guards(&self, route: Route) {
        match self.can_navigate_to(&route) {
            RouteGuardResult::Allow => {
                self.navigator.push(route);
            },
            RouteGuardResult::Redirect(redirect_route) => {
                log::warn!(
                    "Route guard redirected navigation from {route:?} to {redirect_route:?}"
                );
                self.navigator.push(redirect_route);
            },
            RouteGuardResult::Block(reason) => {
                log::error!("Route guard blocked navigation to {route:?}: {reason}");
                // Could show a toast notification here
                crate::ui::toast_helpers::error(format!("Navigation blocked: {reason}"));
            },
        }
    }
}

/// Component for protecting routes with guards
#[component]
pub fn RouteGuardProvider(children: Element) -> Element {
    let route_guard = use_route_guard();
    let current_route = use_route::<Route>();

    // Check if current route is allowed
    match route_guard.can_navigate_to(&current_route) {
        RouteGuardResult::Allow => {
            rsx! { {children} }
        },
        RouteGuardResult::Redirect(redirect_route) => {
            // Redirect to allowed route
            let navigator = route_guard.navigator;
            let redirect_route = redirect_route.clone();

            spawn(async move {
                navigator.push(redirect_route);
            });

            rsx! {
                div { class: "p-8 text-center",
                    div { class: "loading loading-spinner loading-lg mb-4" }
                    p { "Redirecting..." }
                }
            }
        },
        RouteGuardResult::Block(reason) => {
            rsx! {
                div { class: "min-h-screen flex items-center justify-center bg-base-100",
                    div { class: "max-w-md mx-auto text-center p-8",
                        div { class: "text-6xl mb-4", "🚫" }
                        h1 { class: "text-3xl font-bold mb-2 text-error", "Access Denied" }
                        p { class: "text-base-content/70 mb-4", "{reason}" }
                        button {
                            class: "btn btn-primary",
                            onclick: {
                                let navigator = route_guard.navigator;
                                move |_| {
                                    navigator.push(Route::Dashboard {});
                                }
                            },
                            "Go to Dashboard"
                        }
                    }
                }
            }
        },
    }
}
