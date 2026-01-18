//! User preferences entity - persisted app settings.

/// User preferences stored in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct UserPreferences {
    id: String,
    ml_boundary_enabled: bool,
    cognitive_limit_minutes: u32,
    right_panel_visible: bool,
}

impl UserPreferences {
    /// Creates a new preferences object.
    pub fn new(
        id: String,
        ml_boundary_enabled: bool,
        cognitive_limit_minutes: u32,
        right_panel_visible: bool,
    ) -> Self {
        Self { id, ml_boundary_enabled, cognitive_limit_minutes, right_panel_visible }
    }

    /// Creates default preferences for the given user id.
    pub fn defaults(id: String) -> Self {
        Self {
            id,
            ml_boundary_enabled: false,
            cognitive_limit_minutes: 45,
            right_panel_visible: true,
        }
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

    pub fn right_panel_visible(&self) -> bool {
        self.right_panel_visible
    }

    pub fn set_ml_boundary_enabled(&mut self, enabled: bool) {
        self.ml_boundary_enabled = enabled;
    }

    pub fn set_cognitive_limit_minutes(&mut self, minutes: u32) {
        self.cognitive_limit_minutes = minutes;
    }

    pub fn set_right_panel_visible(&mut self, visible: bool) {
        self.right_panel_visible = visible;
    }
}
