ALTER TABLE user_preferences
ADD COLUMN onboarding_completed INTEGER NOT NULL DEFAULT 0;

-- Ensure existing row has a value (defensive)
UPDATE user_preferences
SET onboarding_completed = 0
WHERE onboarding_completed IS NULL;
