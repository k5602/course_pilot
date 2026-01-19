//! Video entity - A single video within a module.

use crate::domain::value_objects::{ModuleId, VideoId, VideoSource, YouTubeVideoId};

/// A video represents a single learning unit within a module.
#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    id: VideoId,
    module_id: ModuleId,
    source: VideoSource,
    title: String,
    description: Option<String>,
    transcript: Option<String>,
    summary: Option<String>,
    duration_secs: u32,
    is_completed: bool,
    sort_order: u32,
}

impl Video {
    /// Creates a new video.
    pub fn new(
        id: VideoId,
        module_id: ModuleId,
        source: VideoSource,
        title: String,
        duration_secs: u32,
        sort_order: u32,
    ) -> Self {
        Self {
            id,
            module_id,
            source,
            title,
            description: None,
            transcript: None,
            summary: None,
            duration_secs,
            is_completed: false,
            sort_order,
        }
    }

    /// Creates a new video with description.
    pub fn with_description(
        id: VideoId,
        module_id: ModuleId,
        source: VideoSource,
        title: String,
        description: Option<String>,
        duration_secs: u32,
        sort_order: u32,
    ) -> Self {
        Self {
            id,
            module_id,
            source,
            title,
            description,
            transcript: None,
            summary: None,
            duration_secs,
            is_completed: false,
            sort_order,
        }
    }

    pub fn id(&self) -> &VideoId {
        &self.id
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn source(&self) -> &VideoSource {
        &self.source
    }

    pub fn youtube_id(&self) -> Option<&YouTubeVideoId> {
        self.source.youtube_id()
    }

    pub fn local_path(&self) -> Option<&str> {
        self.source.local_path_str()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn transcript(&self) -> Option<&str> {
        self.transcript.as_deref()
    }

    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
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

    /// Updates the transcript content.
    pub fn update_transcript(&mut self, transcript: Option<String>) {
        self.transcript = transcript;
    }

    /// Updates the summary content.
    pub fn update_summary(&mut self, summary: Option<String>) {
        self.summary = summary;
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
