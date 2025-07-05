//! Modern UI Module for Course Pilot
//
//! This module provides a comprehensive design system with theming support,
//! modern components, and consistent styling patterns.

pub mod components;
pub mod layout;
pub mod navigation;
pub mod theme;

pub use components::add_course_dialog::AddCourseDialog;
pub use components::course_dashboard::course_dashboard;
pub use components::plan_view::PlanView;
pub use layout::Layout;
pub use navigation::{
    NavigationActions, async_navigate_to, handle_navigation_with_fallback, navigate_to_add_course,
    navigate_to_dashboard, navigate_to_plan_view, safe_navigate_to, use_navigation,
};
pub use theme::AppTheme;

// Re-export components for easy access
pub use components::*;
