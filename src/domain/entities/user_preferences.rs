//! User preferences entity - persisted app settings.

use crate::domain::value_objects::{UserId, VideoQuality};

/// Configuration for constructing UserPreferences.
pub struct UserPreferencesConfig {
    pub ml_boundary_enabled: bool,
    pub cognitive_limit_minutes: u32,
    pub right_panel_visible: bool,
    pub right_panel_width: u32,
    pub onboarding_completed: bool,
    pub preferred_quality: VideoQuality,
    pub boundary_batch_size: u32,
}

/// User preferences stored in the database.
#[derive(Debug, Clone, PartialEq)]
pub struct UserPreferences {
    id: UserId,
    ml_boundary_enabled: bool,
    cognitive_limit_minutes: u32,
    right_panel_visible: bool,
    right_panel_width: u32,
    onboarding_completed: bool,
    preferred_quality: VideoQuality,
    boundary_batch_size: u32,
}

impl UserPreferences {
    /// Creates a new preferences object.
    pub fn new(id: impl Into<UserId>, config: UserPreferencesConfig) -> Self {
        Self {
            id: id.into(),
            ml_boundary_enabled: config.ml_boundary_enabled,
            cognitive_limit_minutes: config.cognitive_limit_minutes,
            right_panel_visible: config.right_panel_visible,
            right_panel_width: config.right_panel_width,
            onboarding_completed: config.onboarding_completed,
            preferred_quality: config.preferred_quality,
            boundary_batch_size: config.boundary_batch_size,
        }
    }

    /// Creates default preferences for the given user id.
    pub fn defaults(id: impl Into<UserId>) -> Self {
        Self {
            id: id.into(),
            ml_boundary_enabled: false,
            cognitive_limit_minutes: 45,
            right_panel_visible: false,
            right_panel_width: 320,
            onboarding_completed: false,
            preferred_quality: VideoQuality::P720,
            boundary_batch_size: 5,
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the typed user identifier.
    pub fn user_id(&self) -> &UserId {
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

    pub fn right_panel_width(&self) -> u32 {
        self.right_panel_width
    }

    pub fn onboarding_completed(&self) -> bool {
        self.onboarding_completed
    }

    pub fn preferred_quality(&self) -> VideoQuality {
        self.preferred_quality
    }

    pub fn boundary_batch_size(&self) -> u32 {
        self.boundary_batch_size
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

    pub fn set_right_panel_width(&mut self, width: u32) {
        self.right_panel_width = width;
    }

    pub fn set_onboarding_completed(&mut self, completed: bool) {
        self.onboarding_completed = completed;
    }

    pub fn set_preferred_quality(&mut self, quality: VideoQuality) {
        self.preferred_quality = quality;
    }

    pub fn set_boundary_batch_size(&mut self, size: u32) {
        self.boundary_batch_size = size;
    }
}
