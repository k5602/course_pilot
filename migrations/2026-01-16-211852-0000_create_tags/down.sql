-- Drop indexes first
DROP INDEX IF EXISTS idx_course_tags_tag_id;
DROP INDEX IF EXISTS idx_course_tags_course_id;

-- Drop tables
DROP TABLE IF EXISTS course_tags;
DROP TABLE IF EXISTS tags;
