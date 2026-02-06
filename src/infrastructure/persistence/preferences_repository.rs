//! User preferences repository using Diesel.

use std::sync::Arc;

use diesel::prelude::*;

use super::connection::DbPool;
use super::models::{NewUserPreferences, UserPreferencesRow};
use crate::domain::entities::UserPreferences;
use crate::domain::ports::{RepositoryError, UserPreferencesRepository};
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

        let new_prefs = NewUserPreferences {
            id: prefs.id(),
            ml_boundary_enabled: bool_to_i32(prefs.ml_boundary_enabled()),
            cognitive_limit_minutes: prefs.cognitive_limit_minutes() as i32,
            right_panel_visible: bool_to_i32(prefs.right_panel_visible()),
            right_panel_width: prefs.right_panel_width() as i32,
            onboarding_completed: bool_to_i32(prefs.onboarding_completed()),
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
    )
}

fn bool_to_i32(value: bool) -> i32 {
    if value { 1 } else { 0 }
}
