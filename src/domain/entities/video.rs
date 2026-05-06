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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_video() -> Video {
        let yt_id = YouTubeVideoId::new("dQw4w9WgXcQ").unwrap();
        Video::new(
            VideoId::new(),
            ModuleId::new(),
            VideoSource::youtube(yt_id),
            "Test Video".to_string(),
            120,
            0,
        )
    }

    #[test]
    fn new_video_not_completed() {
        let video = sample_video();
        assert!(!video.is_completed());
        assert_eq!(video.title(), "Test Video");
        assert_eq!(video.duration_secs(), 120);
        assert_eq!(video.sort_order(), 0);
    }

    #[test]
    fn mark_completed_then_pending() {
        let mut video = sample_video();
        assert!(!video.is_completed());
        video.mark_completed();
        assert!(video.is_completed());
        video.mark_pending();
        assert!(!video.is_completed());
    }

    #[test]
    fn update_transcript() {
        let mut video = sample_video();
        assert!(video.transcript().is_none());
        video.update_transcript(Some("Hello world".to_string()));
        assert_eq!(video.transcript(), Some("Hello world"));
        video.update_transcript(None);
        assert!(video.transcript().is_none());
    }

    #[test]
    fn update_summary() {
        let mut video = sample_video();
        assert!(video.summary().is_none());
        video.update_summary(Some("A summary".to_string()));
        assert_eq!(video.summary(), Some("A summary"));
    }

    #[test]
    fn youtube_source_delegates() {
        let yt_id = YouTubeVideoId::new("9bZkp7q19f0").unwrap();
        let video = Video::new(
            VideoId::new(),
            ModuleId::new(),
            VideoSource::youtube(yt_id),
            "YT Video".to_string(),
            60,
            1,
        );
        assert!(video.youtube_id().is_some());
        assert!(video.local_path().is_none());
    }

    #[test]
    fn local_source_delegates() {
        let video = Video::new(
            VideoId::new(),
            ModuleId::new(),
            VideoSource::local_path("/tmp/v.mp4").unwrap(),
            "Local Video".to_string(),
            300,
            2,
        );
        assert!(video.youtube_id().is_none());
        assert_eq!(video.local_path(), Some("/tmp/v.mp4"));
    }
}
