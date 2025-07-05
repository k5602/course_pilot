//! Modern UI Module for Course Pilot
//
//! This module provides a comprehensive design system with theming support,
//! modern components, and consistent styling patterns.

pub mod components;
pub mod theme;

pub use add_course_dialog::AddCourseDialog;
pub use course_dashboard::CourseDashboard;
pub use layout::Layout;
pub use plan_view::PlanView;
pub use theme::AppTheme;

// Re-export components for easy access
pub use components::{Button, Card, Input};

pub mod add_course_dialog;
pub mod course_dashboard;
pub mod layout;
pub mod plan_view;
