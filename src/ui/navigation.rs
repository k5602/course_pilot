//! Navigation System for Course Pilot
//!
//! This module provides a centralized, safe navigation system that prevents
//! state corruption and handles proper state transitions between views.

use crate::types::{AppState, Route};
use dioxus::prelude::*;
use uuid::Uuid;

/// Navigation error types
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationError {
    InvalidRoute,
    NavigationBlocked,
    StateCorruption,
}

/// Navigation result type
pub type NavigationResult = Result<(), NavigationError>;

/// Validate if navigation to a route is allowed
pub fn is_valid_route(route: &Route) -> bool {
    match route {
        Route::Dashboard => true,
        Route::AddCourse => true,
        Route::PlanView(course_id) => {
            // Validate that course_id is not nil UUID
            *course_id != Uuid::nil()
        }
    }
}

/// Safe navigation function that prevents state corruption
pub fn safe_navigate_to(mut app_state: Signal<AppState>, route: Route) -> NavigationResult {
    // Validate navigation
    if !is_valid_route(&route) {
        log::error!("Invalid navigation route: {:?}", route);
        return Err(NavigationError::InvalidRoute);
    }

    // Additional route-specific validation
    match &route {
        Route::PlanView(course_id) => {
            if *course_id == Uuid::nil() {
                log::error!("Attempted to navigate to PlanView with nil course_id");
                return Err(NavigationError::InvalidRoute);
            }
        }
        _ => {}
    }

    // Get current route for logging
    let current_route = app_state.read().current_route.clone();

    // Update app state
    app_state.write().current_route = route.clone();

    log::info!("Navigation successful: {:?} -> {:?}", current_route, route);
    Ok(())
}

/// Navigate to dashboard (safe fallback)
pub fn navigate_to_dashboard(app_state: Signal<AppState>) -> NavigationResult {
    safe_navigate_to(app_state, Route::Dashboard)
}

/// Navigate to add course dialog
pub fn navigate_to_add_course(app_state: Signal<AppState>) -> NavigationResult {
    safe_navigate_to(app_state, Route::AddCourse)
}

/// Navigate to plan view with course validation
pub fn navigate_to_plan_view(app_state: Signal<AppState>, course_id: Uuid) -> NavigationResult {
    if course_id == Uuid::nil() {
        log::error!("Attempted to navigate to PlanView with nil course_id");
        return Err(NavigationError::InvalidRoute);
    }
    safe_navigate_to(app_state, Route::PlanView(course_id))
}

/// Async navigation function for use in spawn contexts
pub fn async_navigate_to(mut app_state: Signal<AppState>, route: Route) -> NavigationResult {
    // Validate navigation
    if !is_valid_route(&route) {
        log::error!("Invalid async navigation route: {:?}", route);
        return Err(NavigationError::InvalidRoute);
    }

    // Get current route for logging
    let current_route = app_state.read().current_route.clone();

    // Update app state
    app_state.write().current_route = route.clone();

    log::info!(
        "Async navigation successful: {:?} -> {:?}",
        current_route,
        route
    );
    Ok(())
}

/// Navigation hook for components
pub fn use_navigation() -> NavigationActions {
    let app_state = use_context::<Signal<AppState>>();
    NavigationActions { app_state }
}

/// Navigation actions for easier use in components
#[derive(Clone, Copy)]
pub struct NavigationActions {
    app_state: Signal<AppState>,
}

impl NavigationActions {
    /// Navigate to a route with proper error handling
    pub fn navigate_to(&self, route: Route) -> NavigationResult {
        safe_navigate_to(self.app_state, route)
    }

    /// Navigate to dashboard (safe fallback)
    pub fn navigate_to_dashboard(&self) -> NavigationResult {
        navigate_to_dashboard(self.app_state)
    }

    /// Navigate to add course dialog
    pub fn navigate_to_add_course(&self) -> NavigationResult {
        navigate_to_add_course(self.app_state)
    }

    /// Navigate to plan view with course validation
    pub fn navigate_to_plan_view(&self, course_id: Uuid) -> NavigationResult {
        navigate_to_plan_view(self.app_state, course_id)
    }

    /// Get current route
    pub fn current_route(&self) -> Route {
        self.app_state.read().current_route.clone()
    }
}

/// Handle navigation with fallback to dashboard on error
pub fn handle_navigation_with_fallback(app_state: Signal<AppState>, route: Route) {
    match safe_navigate_to(app_state, route) {
        Ok(()) => {
            log::debug!("Navigation successful");
        }
        Err(e) => {
            log::error!("Navigation failed: {:?}", e);
            // Fallback to dashboard on navigation failure
            if let Err(fallback_err) = navigate_to_dashboard(app_state) {
                log::error!(
                    "Fallback navigation to dashboard failed: {:?}",
                    fallback_err
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_validation() {
        // Valid routes
        assert!(is_valid_route(&Route::Dashboard));
        assert!(is_valid_route(&Route::AddCourse));
        assert!(is_valid_route(&Route::PlanView(Uuid::new_v4())));

        // Invalid route
        assert!(!is_valid_route(&Route::PlanView(Uuid::nil())));
    }

    #[test]
    fn test_plan_view_course_id_validation() {
        // Valid course ID
        let valid_id = Uuid::new_v4();
        assert!(is_valid_route(&Route::PlanView(valid_id)));

        // Invalid course ID (nil UUID)
        assert!(!is_valid_route(&Route::PlanView(Uuid::nil())));
    }
}
