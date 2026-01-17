-- Create FTS5 virtual table for full-text search
-- Supports searching across courses, videos, and notes
CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
    entity_type,      -- 'course', 'video', 'note'
    entity_id,        -- UUID reference to the actual entity
    title,            -- Primary searchable field (course name, video title)
    content,          -- Secondary searchable field (description, note content)
    course_id,        -- For linking results back to course
    tokenize = 'porter unicode61'
);

-- Populate search index with existing data

-- Index courses
INSERT INTO search_index (entity_type, entity_id, title, content, course_id)
SELECT 'course', id, name, COALESCE(description, ''), id
FROM courses;

-- Index videos (join with modules to get course_id)
INSERT INTO search_index (entity_type, entity_id, title, content, course_id)
SELECT 'video', v.id, v.title, COALESCE(v.description, ''), m.course_id
FROM videos v
JOIN modules m ON v.module_id = m.id;

-- Index notes (join through videos and modules to get course_id)
INSERT INTO search_index (entity_type, entity_id, title, content, course_id)
SELECT 'note', n.id, v.title, n.content, m.course_id
FROM notes n
JOIN videos v ON n.video_id = v.id
JOIN modules m ON v.module_id = m.id;
