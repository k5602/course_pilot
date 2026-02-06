ALTER TABLE user_preferences
ADD right_panel_width INTEGER NOT NULL DEFAULT 320;

UPDATE user_preferences
SET right_panel_width = 320
WHERE right_panel_width IS NULL;
