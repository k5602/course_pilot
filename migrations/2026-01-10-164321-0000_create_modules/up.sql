-- Create modules table
CREATE TABLE modules (
    id TEXT PRIMARY KEY NOT NULL,
    course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE INDEX idx_modules_course_id ON modules(course_id);
