-- SQLite does not support DROP COLUMN in older versions.
-- Recreate the table without the column.
CREATE TABLE user_preferences_new (
    id TEXT PRIMARY KEY NOT NULL DEFAULT 'default',
    ml_boundary_enabled INTEGER NOT NULL DEFAULT 0,
    cognitive_limit_minutes INTEGER NOT NULL DEFAULT 45,
    right_panel_visible INTEGER NOT NULL DEFAULT 1,
    onboarding_completed INTEGER NOT NULL DEFAULT 0,
    right_panel_width INTEGER NOT NULL DEFAULT 320,
    preferred_quality TEXT NOT NULL DEFAULT 'p720'
);

INSERT INTO user_preferences_new (id, ml_boundary_enabled, cognitive_limit_minutes, right_panel_visible, onboarding_completed, right_panel_width, preferred_quality)
SELECT id, ml_boundary_enabled, cognitive_limit_minutes, right_panel_visible, onboarding_completed, right_panel_width, preferred_quality FROM user_preferences;

DROP TABLE user_preferences;
ALTER TABLE user_preferences_new RENAME TO user_preferences;
