ALTER TABLE user_preferences
ADD COLUMN boundary_batch_size INTEGER NOT NULL DEFAULT 5;
