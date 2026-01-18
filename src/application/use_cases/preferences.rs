//! User preferences use case.

use std::sync::Arc;

use crate::domain::entities::UserPreferences;
use crate::domain::ports::{RepositoryError, UserPreferencesRepository};

/// Input for updating user preferences.
#[derive(Debug, Clone)]
pub struct UpdatePreferencesInput {
    pub ml_boundary_enabled: bool,
    pub cognitive_limit_minutes: u32,
    pub right_panel_visible: bool,
    pub onboarding_completed: bool,
}

/// Use case for loading and updating user preferences.
pub struct PreferencesUseCase<PR>
where
    PR: UserPreferencesRepository,
{
    prefs_repo: Arc<PR>,
    default_id: String,
}

impl<PR> PreferencesUseCase<PR>
where
    PR: UserPreferencesRepository,
{
    /// Creates a new preferences use case.
    pub fn new(prefs_repo: Arc<PR>) -> Self {
        Self { prefs_repo, default_id: "default".to_string() }
    }

    /// Loads preferences or returns defaults if not found.
    pub fn load(&self) -> Result<UserPreferences, RepositoryError> {
        match self.prefs_repo.load(&self.default_id)? {
            Some(prefs) => Ok(prefs),
            None => Ok(UserPreferences::defaults(self.default_id.clone())),
        }
    }

    /// Updates and persists preferences.
    pub fn update(
        &self,
        input: UpdatePreferencesInput,
    ) -> Result<UserPreferences, RepositoryError> {
        let mut prefs = self.load()?;
        prefs.set_ml_boundary_enabled(input.ml_boundary_enabled);
        prefs.set_cognitive_limit_minutes(input.cognitive_limit_minutes);
        prefs.set_right_panel_visible(input.right_panel_visible);
        prefs.set_onboarding_completed(input.onboarding_completed);
        self.prefs_repo.save(&prefs)?;
        Ok(prefs)
    }
}
