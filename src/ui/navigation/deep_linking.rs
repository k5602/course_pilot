use crate::types::Route;
use crate::ui::hooks::use_course_manager;
use dioxus::prelude::*;

/// Deep linking verification component that ensures routes work when accessed directly
#[component]
pub fn DeepLinkingHandler() -> Element {
    let current_route = use_route::<Route>();
    let course_manager = use_course_manager();
    let navigator = use_navigator();

    // Verify deep linking support for current route
    use_effect(move || {
        verify_deep_link_support(&current_route, &course_manager.courses, navigator);
    });

    rsx! {
        // This component doesn't render anything, it just handles deep linking verification
        div { style: "display: none;" }
    }
}

/// Verify that the current route supports deep linking and handle any issues
fn verify_deep_link_support(
    route: &Route,
    courses: &[crate::types::Course],
    _navigator: Navigator,
) {
    match route {
        Route::PlanView { course_id } => {
            // Verify course exists for deep linking to plan view
            if let Ok(course_uuid) = uuid::Uuid::parse_str(course_id) {
                if !courses.iter().any(|c| c.id == course_uuid) {
                    log::warn!("Deep link to non-existent course: {course_id}");
                    // Don't redirect immediately - let the route component handle it
                    // This is just for logging and monitoring
                }
            } else {
                log::error!("Deep link with invalid course ID format: {course_id}");
            }
        }
        Route::Home {} => {
            // Home should always redirect to dashboard
            log::info!("Deep link to home, redirecting to dashboard");
        }
        Route::Dashboard {} | Route::AllCourses {} | Route::Settings {} | Route::AddCourse {} => {
            // These routes should always work for deep linking
            log::debug!("Deep link to {route:?} - supported");
        }
        #[cfg(debug_assertions)]
        Route::ToastTest {} => {
            log::debug!("Deep link to toast test - debug only");
        }
    }
}

/// Component for testing deep linking functionality
#[cfg(debug_assertions)]
#[component]
pub fn DeepLinkingTester() -> Element {
    let navigator = use_navigator();
    let course_manager = use_course_manager();

    let test_deep_links = move |_| {
        let navigator = navigator;
        let courses = course_manager.courses.clone();

        spawn(async move {
            // Test all route types
            let test_routes = vec![
                Route::Dashboard {},
                Route::AllCourses {},
                Route::Settings {},
                Route::AddCourse {},
            ];

            // Add course-specific routes if courses exist
            let mut all_test_routes = test_routes;
            if let Some(course) = courses.first() {
                all_test_routes.push(Route::PlanView {
                    course_id: course.id.to_string(),
                });
            }

            // Test navigation to each route
            for route in all_test_routes {
                log::info!("Testing deep link to: {route:?}");
                navigator.push(route);

                // Small delay between tests using tokio
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Return to dashboard
            navigator.push(Route::Dashboard {});
        });
    };

    rsx! {
        div { class: "p-4 bg-warning/10 border border-warning rounded-lg",
            h3 { class: "font-bold mb-2", "Deep Linking Tester (Debug Only)" }
            p { class: "text-sm mb-4", "Test deep linking functionality for all routes." }
            button {
                class: "btn btn-warning btn-sm",
                onclick: test_deep_links,
                "Test All Deep Links"
            }
        }
    }
}

/// Hook for handling deep linking in components
pub fn use_deep_linking() -> DeepLinkingManager {
    let navigator = use_navigator();
    let current_route = use_route::<Route>();

    DeepLinkingManager {
        current_route,
        navigator,
    }
}

/// Manager for deep linking functionality
pub struct DeepLinkingManager {
    pub current_route: Route,
    pub navigator: Navigator,
}

impl DeepLinkingManager {
    /// Check if the current route supports deep linking
    pub fn supports_deep_linking(&self) -> bool {
        match &self.current_route {
            Route::Home {} => false, // Always redirects
            Route::Dashboard {}
            | Route::AllCourses {}
            | Route::Settings {}
            | Route::AddCourse {} => true,
            Route::PlanView { course_id } => {
                // Only supports deep linking if course ID is valid format
                uuid::Uuid::parse_str(course_id).is_ok()
            }
            #[cfg(debug_assertions)]
            Route::ToastTest {} => true,
        }
    }

    /// Get a shareable URL for the current route
    pub fn get_shareable_url(&self) -> String {
        match &self.current_route {
            Route::Home {} => "/".to_string(),
            Route::Dashboard {} => "/dashboard".to_string(),
            Route::AllCourses {} => "/courses".to_string(),
            Route::PlanView { course_id } => format!("/plan/{course_id}"),
            Route::Settings {} => "/settings".to_string(),
            Route::AddCourse {} => "/import".to_string(),
            #[cfg(debug_assertions)]
            Route::ToastTest {} => "/toast-test".to_string(),
        }
    }

    /// Navigate to a route with deep linking support verification
    pub fn navigate_with_verification(&self, route: Route) {
        // Log the navigation for debugging
        log::info!("Navigating to {route:?} with deep linking verification");

        // Navigate normally - route guards will handle validation
        self.navigator.push(route);
    }
}
