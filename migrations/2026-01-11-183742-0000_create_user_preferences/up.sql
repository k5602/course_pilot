CREATE TABLE user_preferences (
    id TEXT PRIMARY KEY NOT NULL DEFAULT 'default',
    ml_boundary_enabled INTEGER NOT NULL DEFAULT 0,
    cognitive_limit_minutes INTEGER NOT NULL DEFAULT 45
);

-- Insert default row
INSERT INTO user_preferences (id) VALUES ('default');
