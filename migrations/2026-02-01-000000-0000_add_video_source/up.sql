ALTER TABLE videos ADD COLUMN source_type TEXT NOT NULL DEFAULT 'youtube';
ALTER TABLE videos ADD COLUMN source_ref TEXT NOT NULL DEFAULT '';

UPDATE videos
SET source_ref = youtube_id
WHERE source_type = 'youtube' AND source_ref = '';
