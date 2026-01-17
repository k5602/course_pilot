//! User preferences entity - persisted app settings.

/// User preferences stored in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct UserPreferences {
    id: String,
    ml_boundary_enabled: bool,
    cognitive_limit_minutes: u32,
}

impl UserPreferences {
    /// Creates a new preferences object.
    pub fn new(id: String, ml_boundary_enabled: bool, cognitive_limit_minutes: u32) -> Self {
        Self { id, ml_boundary_enabled, cognitive_limit_minutes }
    }

    /// Creates default preferences for the given user id.
    pub fn defaults(id: String) -> Self {
        Self { id, ml_boundary_enabled: false, cognitive_limit_minutes: 45 }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn ml_boundary_enabled(&self) -> bool {
        self.ml_boundary_enabled
    }

    pub fn cognitive_limit_minutes(&self) -> u32 {
        self.cognitive_limit_minutes
    }

    pub fn set_ml_boundary_enabled(&mut self, enabled: bool) {
        self.ml_boundary_enabled = enabled;
    }

    pub fn set_cognitive_limit_minutes(&mut self, minutes: u32) {
        self.cognitive_limit_minutes = minutes;
    }
}
