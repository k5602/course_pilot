//! User preferences use case.

use std::sync::Arc;

use crate::domain::entities::UserPreferences;
use crate::domain::ports::{RepositoryError, UserPreferencesRepository};
use crate::domain::value_objects::VideoQuality;

/// Input for updating user preferences.
#[derive(Debug, Clone)]
pub struct UpdatePreferencesInput {
    pub ml_boundary_enabled: bool,
    pub cognitive_limit_minutes: u32,
    pub right_panel_visible: bool,
    pub right_panel_width: u32,
    pub onboarding_completed: bool,
    pub preferred_quality: VideoQuality,
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
        prefs.set_right_panel_width(input.right_panel_width);
        prefs.set_onboarding_completed(input.onboarding_completed);
        prefs.set_preferred_quality(input.preferred_quality);
        self.prefs_repo.save(&prefs)?;
        Ok(prefs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    struct MockPrefsRepo {
        stored: Mutex<Option<UserPreferences>>,
    }

    impl MockPrefsRepo {
        fn new() -> Self {
            Self { stored: Mutex::new(None) }
        }
    }

    impl UserPreferencesRepository for MockPrefsRepo {
        fn load(&self, _id: &str) -> Result<Option<UserPreferences>, RepositoryError> {
            Ok(self.stored.lock().unwrap().clone())
        }
        fn save(&self, prefs: &UserPreferences) -> Result<(), RepositoryError> {
            *self.stored.lock().unwrap() = Some(prefs.clone());
            Ok(())
        }
    }

    #[test]
    fn update_preserves_preferred_quality() {
        let repo = Arc::new(MockPrefsRepo::new());
        let uc = PreferencesUseCase::new(repo.clone());

        let input = UpdatePreferencesInput {
            ml_boundary_enabled: true,
            cognitive_limit_minutes: 30,
            right_panel_visible: false,
            right_panel_width: 400,
            onboarding_completed: true,
            preferred_quality: VideoQuality::P1080,
        };

        let result = uc.update(input).unwrap();
        assert_eq!(result.preferred_quality(), VideoQuality::P1080);

        let loaded = uc.load().unwrap();
        assert_eq!(loaded.preferred_quality(), VideoQuality::P1080);
    }

    #[test]
    fn load_returns_defaults_when_not_found() {
        let repo = Arc::new(MockPrefsRepo::new());
        let uc = PreferencesUseCase::new(repo);
        let prefs = uc.load().unwrap();
        assert_eq!(prefs.preferred_quality(), VideoQuality::P720);
    }
}
