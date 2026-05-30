CREATE TABLE chat_messages (
    id TEXT PRIMARY KEY NOT NULL,
    video_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(video_id) REFERENCES videos(id) ON DELETE CASCADE
);
CREATE INDEX idx_chat_messages_video_id ON chat_messages(video_id);
