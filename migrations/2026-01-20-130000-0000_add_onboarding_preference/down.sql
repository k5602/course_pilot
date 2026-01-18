BEGIN TRANSACTION;

CREATE TABLE user_preferences_backup (
    id TEXT PRIMARY KEY NOT NULL DEFAULT 'default',
    ml_boundary_enabled INTEGER NOT NULL DEFAULT 0,
    cognitive_limit_minutes INTEGER NOT NULL DEFAULT 45,
    right_panel_visible INTEGER NOT NULL DEFAULT 1
);

INSERT INTO user_preferences_backup (id, ml_boundary_enabled, cognitive_limit_minutes, right_panel_visible)
SELECT id, ml_boundary_enabled, cognitive_limit_minutes, right_panel_visible
FROM user_preferences;

DROP TABLE user_preferences;

ALTER TABLE user_preferences_backup RENAME TO user_preferences;

COMMIT;
