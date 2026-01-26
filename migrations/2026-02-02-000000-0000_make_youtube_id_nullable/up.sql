-- Make videos.youtube_id nullable by rebuilding the table
PRAGMA foreign_keys=off;

CREATE TABLE videos_new (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    youtube_id TEXT,
    title TEXT NOT NULL,
    duration_secs INTEGER NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL,
    description TEXT,
    transcript TEXT,
    summary TEXT,
    source_type TEXT NOT NULL DEFAULT 'youtube',
    source_ref TEXT NOT NULL DEFAULT ''
);

INSERT INTO videos_new (
    id,
    module_id,
    youtube_id,
    title,
    duration_secs,
    is_completed,
    sort_order,
    description,
    transcript,
    summary,
    source_type,
    source_ref
)
SELECT
    id,
    module_id,
    youtube_id,
    title,
    duration_secs,
    is_completed,
    sort_order,
    description,
    transcript,
    summary,
    source_type,
    source_ref
FROM videos;

DROP TABLE videos;
ALTER TABLE videos_new RENAME TO videos;

CREATE INDEX idx_videos_module_id ON videos(module_id);
CREATE INDEX idx_videos_youtube_id ON videos(youtube_id);

PRAGMA foreign_keys=on;
