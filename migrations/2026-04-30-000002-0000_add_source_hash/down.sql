DROP INDEX IF EXISTS idx_courses_source_hash;
ALTER TABLE courses DROP COLUMN source_hash;
