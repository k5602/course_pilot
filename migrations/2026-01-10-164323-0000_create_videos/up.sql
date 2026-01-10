-- Create videos table
CREATE TABLE videos (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    youtube_id TEXT NOT NULL,
    title TEXT NOT NULL,
    duration_secs INTEGER NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL
);

CREATE INDEX idx_videos_module_id ON videos(module_id);
CREATE INDEX idx_videos_youtube_id ON videos(youtube_id);
