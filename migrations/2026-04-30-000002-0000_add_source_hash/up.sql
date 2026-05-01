ALTER TABLE courses ADD COLUMN source_hash TEXT;
CREATE INDEX IF NOT EXISTS idx_courses_source_hash ON courses(source_hash);
