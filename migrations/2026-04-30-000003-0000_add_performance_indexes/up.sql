CREATE INDEX IF NOT EXISTS idx_videos_module_id ON videos(module_id);
CREATE INDEX IF NOT EXISTS idx_modules_course_id ON modules(course_id);
CREATE INDEX IF NOT EXISTS idx_videos_source_type ON videos(source_type);
