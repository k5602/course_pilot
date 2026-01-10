//! Module entity - A logical segment within a course.

use crate::domain::value_objects::{CourseId, ModuleId};

/// A module represents a logical grouping of videos within a course.
/// Modules are created by detecting topic boundaries using embeddings.
#[derive(Debug, Clone)]
pub struct Module {
    id: ModuleId,
    course_id: CourseId,
    title: String,
    sort_order: u32,
}

impl Module {
    /// Creates a new module.
    pub fn new(id: ModuleId, course_id: CourseId, title: String, sort_order: u32) -> Self {
        Self { id, course_id, title, sort_order }
    }

    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn course_id(&self) -> &CourseId {
        &self.course_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn sort_order(&self) -> u32 {
        self.sort_order
    }
}
