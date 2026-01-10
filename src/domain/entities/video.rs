//! Video entity - A single video within a module.

use crate::domain::value_objects::{ModuleId, VideoId, YouTubeVideoId};

/// A video represents a single learning unit within a module.
#[derive(Debug, Clone)]
pub struct Video {
    id: VideoId,
    module_id: ModuleId,
    youtube_id: YouTubeVideoId,
    title: String,
    duration_secs: u32,
    is_completed: bool,
    sort_order: u32,
}

impl Video {
    /// Creates a new video.
    pub fn new(
        id: VideoId,
        module_id: ModuleId,
        youtube_id: YouTubeVideoId,
        title: String,
        duration_secs: u32,
        sort_order: u32,
    ) -> Self {
        Self { id, module_id, youtube_id, title, duration_secs, is_completed: false, sort_order }
    }

    pub fn id(&self) -> &VideoId {
        &self.id
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn youtube_id(&self) -> &YouTubeVideoId {
        &self.youtube_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn duration_secs(&self) -> u32 {
        self.duration_secs
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    pub fn sort_order(&self) -> u32 {
        self.sort_order
    }

    /// Marks the video as completed.
    pub fn mark_completed(&mut self) {
        self.is_completed = true;
    }

    /// Marks the video as pending (not completed).
    pub fn mark_pending(&mut self) {
        self.is_completed = false;
    }
}
