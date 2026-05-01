//! User preferences repository using Diesel.

use std::sync::Arc;

use diesel::prelude::*;

use super::connection::DbPool;
use super::models::{NewUserPreferences, UserPreferencesRow};
use crate::domain::entities::UserPreferences;
use crate::domain::ports::{RepositoryError, UserPreferencesRepository};
use crate::domain::value_objects::VideoQuality;
use crate::schema::user_preferences;

/// SQLite-backed user preferences repository.
pub struct SqliteUserPreferencesRepository {
    pool: Arc<DbPool>,
}

impl SqliteUserPreferencesRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl UserPreferencesRepository for SqliteUserPreferencesRepository {
    fn load(&self, id: &str) -> Result<Option<UserPreferences>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let row = user_preferences::table
            .find(id)
            .first::<UserPreferencesRow>(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(row.map(row_to_preferences))
    }

    fn save(&self, prefs: &UserPreferences) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let quality_str = quality_to_str(prefs.preferred_quality());
        let new_prefs = NewUserPreferences {
            id: prefs.id(),
            ml_boundary_enabled: bool_to_i32(prefs.ml_boundary_enabled()),
            cognitive_limit_minutes: prefs.cognitive_limit_minutes() as i32,
            right_panel_visible: bool_to_i32(prefs.right_panel_visible()),
            right_panel_width: prefs.right_panel_width() as i32,
            onboarding_completed: bool_to_i32(prefs.onboarding_completed()),
            preferred_quality: &quality_str,
        };

        diesel::replace_into(user_preferences::table)
            .values(&new_prefs)
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

fn row_to_preferences(row: UserPreferencesRow) -> UserPreferences {
    UserPreferences::new(
        row.id,
        row.ml_boundary_enabled != 0,
        row.cognitive_limit_minutes as u32,
        row.right_panel_visible != 0,
        row.right_panel_width as u32,
        row.onboarding_completed != 0,
        str_to_quality(&row.preferred_quality),
    )
}

fn bool_to_i32(value: bool) -> i32 {
    if value { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_to_quality_round_trips_all_variants() {
        for v in VideoQuality::variants() {
            let s = quality_to_str(*v);
            let back = str_to_quality(&s);
            assert_eq!(back, *v, "round-trip failed for {v}");
        }
    }

    #[test]
    fn str_to_quality_accepts_p_suffix() {
        assert_eq!(str_to_quality("240p"), VideoQuality::P240);
        assert_eq!(str_to_quality("720p"), VideoQuality::P720);
        assert_eq!(str_to_quality("1080p"), VideoQuality::P1080);
    }

    #[test]
    fn str_to_quality_is_case_insensitive() {
        assert_eq!(str_to_quality("P720"), VideoQuality::P720);
        assert_eq!(str_to_quality("Best"), VideoQuality::Best);
        assert_eq!(str_to_quality("BEST"), VideoQuality::Best);
    }

    #[test]
    fn str_to_quality_falls_back_to_p720() {
        assert_eq!(str_to_quality("unknown"), VideoQuality::P720);
        assert_eq!(str_to_quality(""), VideoQuality::P720);
    }

    #[test]
    fn new_user_preferences_defaults_have_p720() {
        let prefs = UserPreferences::defaults("test".to_string());
        assert_eq!(prefs.preferred_quality(), VideoQuality::P720);
    }

    #[test]
    fn user_preferences_set_and_get_quality() {
        let mut prefs = UserPreferences::defaults("test".to_string());
        assert_eq!(prefs.preferred_quality(), VideoQuality::P720);
        prefs.set_preferred_quality(VideoQuality::P1080);
        assert_eq!(prefs.preferred_quality(), VideoQuality::P1080);
    }

    #[test]
    fn row_to_preferences_uses_preferred_quality() {
        let row = UserPreferencesRow {
            id: "test".to_string(),
            ml_boundary_enabled: 0,
            cognitive_limit_minutes: 45,
            right_panel_visible: 1,
            onboarding_completed: 0,
            right_panel_width: 320,
            preferred_quality: "p1080".to_string(),
        };
        let prefs = row_to_preferences(row);
        assert_eq!(prefs.preferred_quality(), VideoQuality::P1080);
    }
}

fn str_to_quality(s: &str) -> VideoQuality {
    match s.to_lowercase().trim() {
        "p240" | "240p" => VideoQuality::P240,
        "p360" | "360p" => VideoQuality::P360,
        "p480" | "480p" => VideoQuality::P480,
        "p720" | "720p" => VideoQuality::P720,
        "p1080" | "1080p" => VideoQuality::P1080,
        "best" => VideoQuality::Best,
        _ => VideoQuality::P720,
    }
}

fn quality_to_str(q: VideoQuality) -> String {
    match q {
        VideoQuality::P240 => "p240".to_string(),
        VideoQuality::P360 => "p360".to_string(),
        VideoQuality::P480 => "p480".to_string(),
        VideoQuality::P720 => "p720".to_string(),
        VideoQuality::P1080 => "p1080".to_string(),
        VideoQuality::Best => "best".to_string(),
    }
}
