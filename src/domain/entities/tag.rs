//! Tag entity - A label for categorizing courses.

use crate::domain::value_objects::TagId;

/// Predefined colors for tags (Tailwind palette).
pub const TAG_COLORS: &[&str] = &[
    "#6366f1", // Indigo
    "#8b5cf6", // Violet
    "#ec4899", // Pink
    "#ef4444", // Red
    "#f97316", // Orange
    "#eab308", // Yellow
    "#22c55e", // Green
    "#14b8a6", // Teal
    "#0ea5e9", // Sky
    "#6b7280", // Gray
];

/// A tag for categorizing courses.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    id: TagId,
    name: String,
    color: String,
}

impl Tag {
    /// Creates a new tag with a random color.
    pub fn new(id: TagId, name: String) -> Self {
        let color_idx = id.as_uuid().as_u128() as usize % TAG_COLORS.len();
        Self { id, name, color: TAG_COLORS[color_idx].to_string() }
    }

    /// Creates a new tag with a specific color.
    pub fn with_color(id: TagId, name: String, color: String) -> Self {
        Self { id, name, color }
    }

    pub fn id(&self) -> &TagId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn color(&self) -> &str {
        &self.color
    }
}
