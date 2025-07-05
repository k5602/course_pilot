//! Modern UI Module for Course Pilot
//
//! This module provides a comprehensive design system with theming support,
//! modern components, and consistent styling patterns.

pub mod components;
pub mod layout;
pub mod theme;

pub use components::add_course_dialog::AddCourseDialog;
pub use components::course_dashboard::CourseDashboard;
pub use components::plan_view::PlanView;
pub use layout::Layout;
pub use theme::AppTheme;

// Re-export components for easy access
pub use components::*;
