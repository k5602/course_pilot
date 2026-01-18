ALTER TABLE user_preferences
ADD COLUMN right_panel_visible INTEGER NOT NULL DEFAULT 1;

-- Ensure existing row has a value (defensive)
UPDATE user_preferences
SET right_panel_visible = 1
WHERE right_panel_visible IS NULL;
