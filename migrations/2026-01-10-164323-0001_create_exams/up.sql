-- Create exams table
CREATE TABLE exams (
    id TEXT PRIMARY KEY NOT NULL,
    video_id TEXT NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
    question_json TEXT NOT NULL,
    score REAL,
    passed BOOLEAN
);

CREATE INDEX idx_exams_video_id ON exams(video_id);
