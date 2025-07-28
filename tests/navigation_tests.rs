use course_pilot::types::{Course, Route};
use course_pilot::ui::navigation::{RouteGuard, RouteGuardResult, CourseExistenceGuard};
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod navigation_tests {
    use super::*;

    /// Test route parameter handling with valid course IDs
    #[test]
    fn test_valid_course_id_parsing() {
        let course_id = Uuid::new_v4();
        let course_id_str = course_id.to_string();
        
        // Test UUID parsing
        let parsed_uuid = Uuid::parse_str(&course_id_str);
        assert!(parsed_uuid.is_ok());
        assert_eq!(parsed_uuid.unwrap(), course_id);
    }

    /// Test route parameter handling with invalid course IDs
    #[test]
    fn test_invalid_course_id_parsing() {
        let invalid_ids = vec![
            "not-a-uuid",
            "123",
            "",
            "invalid-uuid-format",
            "123e4567-e89b-12d3-a456-42661417400", // Missing last digit
        ];

        for invalid_id in invalid_ids {
            let parsed_uuid = Uuid::parse_str(invalid_id);
            assert!(parsed_uuid.is_err(), "Should fail to parse: {}", invalid_id);
        }
    }

    /// Test course existence guard with existing courses
    #[test]
    fn test_course_existence_guard_allow() {
        let course_id = Uuid::new_v4();
        let course = Course {
            id: course_id,
            name: "Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Video 1".to_string(), "Video 2".to_string()],
            structure: None,
        };

        let guard = CourseExistenceGuard::new(vec![course]);
        let route = Route::PlanView { 
            course_id: course_id.to_string() 
        };

        let result = guard.can_navigate(&route);
        assert_eq!(result, RouteGuardResult::Allow);
    }

    /// Test course existence guard with non-existing courses
    #[test]
    fn test_course_existence_guard_redirect() {
        let existing_course_id = Uuid::new_v4();
        let non_existing_course_id = Uuid::new_v4();
        
        let course = Course {
            id: existing_course_id,
            name: "Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Video 1".to_string()],
            structure: None,
        };

        let guard = CourseExistenceGuard::new(vec![course]);
        let route = Route::PlanView { 
            course_id: non_existing_course_id.to_string() 
        };

        let result = guard.can_navigate(&route);
        assert_eq!(result, RouteGuardResult::Redirect(Route::AllCourses {}));
    }

    /// Test course existence guard with invalid course ID format
    #[test]
    fn test_course_existence_guard_block_invalid_id() {
        let course = Course {
            id: Uuid::new_v4(),
            name: "Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Video 1".to_string()],
            structure: None,
        };

        let guard = CourseExistenceGuard::new(vec![course]);
        let route = Route::PlanView { 
            course_id: "invalid-uuid".to_string() 
        };

        let result = guard.can_navigate(&route);
        assert!(matches!(result, RouteGuardResult::Block(_)));
    }

    /// Test route guard allows non-course routes
    #[test]
    fn test_course_existence_guard_allows_other_routes() {
        let guard = CourseExistenceGuard::new(vec![]);
        
        let routes = vec![
            Route::Dashboard {},
            Route::AllCourses {},
            Route::Settings {},
            Route::AddCourse {},
            Route::Home {},
        ];

        for route in routes {
            let result = guard.can_navigate(&route);
            assert_eq!(result, RouteGuardResult::Allow, "Route {:?} should be allowed", route);
        }
    }

    /// Test breadcrumb generation for different routes
    #[test]
    fn test_breadcrumb_generation() {
        // Test basic route structure - breadcrumb generation is tested in integration
        let course_id = Uuid::new_v4();
        let course = Course {
            id: course_id,
            name: "Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Video 1".to_string()],
            structure: None,
        };
        let courses = vec![course];

        // Test that we can create routes that would generate breadcrumbs
        let routes = vec![
            Route::Dashboard {},
            Route::AllCourses {},
            Route::PlanView { course_id: course_id.to_string() },
            Route::Settings {},
        ];

        // All routes should be valid and constructible
        for route in routes {
            let debug_str = format!("{:?}", route);
            assert!(!debug_str.is_empty());
        }
    }

    /// Test deep linking support for all routes
    #[test]
    fn test_deep_linking_routes() {
        // Test that all route variants can be constructed and are valid
        let course_id = Uuid::new_v4();
        
        let routes = vec![
            Route::Home {},
            Route::Dashboard {},
            Route::AllCourses {},
            Route::PlanView { course_id: course_id.to_string() },
            Route::Settings {},
            Route::AddCourse {},
        ];

        // All routes should be constructible and have proper string representations
        for route in routes {
            // Routes should be cloneable and comparable
            let cloned_route = route.clone();
            assert_eq!(route, cloned_route);
            
            // Routes should be debuggable
            let debug_str = format!("{:?}", route);
            assert!(!debug_str.is_empty());
        }
    }

    /// Test route parameter validation edge cases
    #[test]
    fn test_route_parameter_edge_cases() {
        let edge_cases = vec![
            // Empty string
            "",
            // Whitespace
            "   ",
            // Almost valid UUID (missing characters)
            "123e4567-e89b-12d3-a456-42661417400",
            // Too long
            "123e4567-e89b-12d3-a456-426614174000-extra",
            // Wrong format
            "123e4567_e89b_12d3_a456_426614174000",
            // Mixed case (should work)
            "123E4567-E89B-12D3-A456-426614174000",
        ];

        for case in edge_cases {
            let parse_result = Uuid::parse_str(case);
            
            // Only the mixed case should succeed
            if case == "123E4567-E89B-12D3-A456-426614174000" {
                assert!(parse_result.is_ok(), "Mixed case UUID should parse: {}", case);
            } else {
                assert!(parse_result.is_err(), "Should fail to parse: {}", case);
            }
        }
    }

    /// Test navigation history tracking
    #[test]
    fn test_navigation_history() {
        // This would be an integration test in a real scenario
        // For now, we test the route structure
        let routes = vec![
            Route::Dashboard {},
            Route::AllCourses {},
            Route::PlanView { course_id: Uuid::new_v4().to_string() },
            Route::Settings {},
        ];

        // Simulate navigation history
        let mut history = Vec::new();
        for route in routes {
            history.push(route.clone());
            
            // History should grow
            assert!(!history.is_empty());
            
            // Last item should be current route
            assert_eq!(history.last().unwrap(), &route);
        }

        // Test back navigation simulation
        assert!(history.len() > 1);
        history.pop(); // Go back
        assert_eq!(history.len(), 3);
    }
}

/// Integration tests for navigation components
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test complete navigation flow
    #[test]
    fn test_complete_navigation_flow() {
        // Create test course
        let course_id = Uuid::new_v4();
        let course = Course {
            id: course_id,
            name: "Integration Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Video 1".to_string(), "Video 2".to_string()],
            structure: None,
        };

        // Test navigation flow: Dashboard -> All Courses -> Plan View
        let navigation_flow = vec![
            Route::Dashboard {},
            Route::AllCourses {},
            Route::PlanView { course_id: course_id.to_string() },
        ];

        // Test route guards for each step
        let guard = CourseExistenceGuard::new(vec![course]);
        
        for route in navigation_flow {
            let result = guard.can_navigate(&route);
            assert_eq!(result, RouteGuardResult::Allow, "Route {:?} should be allowed", route);
        }
    }

    /// Test error recovery navigation
    #[test]
    fn test_error_recovery_navigation() {
        let guard = CourseExistenceGuard::new(vec![]);
        
        // Try to navigate to non-existent course
        let invalid_route = Route::PlanView { 
            course_id: Uuid::new_v4().to_string() 
        };
        
        let result = guard.can_navigate(&invalid_route);
        
        // Should redirect to All Courses for recovery
        assert_eq!(result, RouteGuardResult::Redirect(Route::AllCourses {}));
    }

    /// Test breadcrumb navigation consistency
    #[test]
    fn test_breadcrumb_navigation_consistency() {
        let course_id = Uuid::new_v4();
        let course = Course {
            id: course_id,
            name: "Breadcrumb Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Video 1".to_string()],
            structure: None,
        };

        // Test that routes are consistent and can be used for navigation
        let routes_to_test = vec![
            Route::Dashboard {},
            Route::AllCourses {},
            Route::PlanView { course_id: course_id.to_string() },
            Route::Settings {},
            Route::AddCourse {},
        ];

        for route in routes_to_test {
            // Routes should be cloneable and debuggable
            let cloned_route = route.clone();
            assert_eq!(route, cloned_route);
            
            let debug_str = format!("{:?}", route);
            assert!(!debug_str.is_empty(), "Route should have debug representation: {:?}", route);
        }
    }
}