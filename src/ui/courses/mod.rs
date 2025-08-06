//! Courses Module for Course Pilot
//!
//! This module provides course management functionality including
//! course listing, filtering, searching, and all course actions.

pub mod all_courses_view;
pub mod course_actions;
pub mod course_card;
pub mod course_grid;

// Re-export all course components
pub use all_courses_view::AllCoursesView;
pub use course_actions::{CourseActions, CourseActionsProps, ContentReorganizationModals, ContentReorganizationModalsProps};
pub use course_card::{CourseCard, CourseCardProps};
pub use course_grid::{CourseGrid, CourseGridProps};
