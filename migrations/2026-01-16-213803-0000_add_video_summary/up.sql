-- Add transcript and summary columns to videos table
ALTER TABLE videos ADD COLUMN transcript TEXT;
ALTER TABLE videos ADD COLUMN summary TEXT;
