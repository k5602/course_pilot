//! Ingest Playlist Use Case
//!
//! Orchestrates: Fetch → Sanitize → Group → Persist

use std::sync::Arc;

use crate::domain::{
    entities::{Course, Module, Video},
    ports::{
        CourseRepository, ModuleRepository, PlaylistFetcher, SearchRepository, VideoRepository,
    },
    services::{BoundaryDetector, TitleSanitizer},
    value_objects::{CourseId, ModuleId, PlaylistUrl, VideoId, VideoSource, YouTubeVideoId},
};

/// Error type for playlist ingestion.
#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Invalid playlist URL: {0}")]
    InvalidUrl(String),
    #[error("Failed to fetch playlist: {0}")]
    FetchFailed(String),
    #[error("Failed to persist: {0}")]
    PersistFailed(String),
}

/// Input for the ingest playlist use case.
pub struct IngestPlaylistInput {
    pub playlist_url: String,
    pub course_name: Option<String>,
}

/// Output of the ingest playlist use case.
#[derive(Debug)]
pub struct IngestPlaylistOutput {
    pub course_id: CourseId,
    pub modules_count: usize,
    pub videos_count: usize,
}

/// Use case for ingesting a YouTube playlist into a structured course.
pub struct IngestPlaylistUseCase<F, CR, MR, VR, SR>
where
    F: PlaylistFetcher,
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
    SR: SearchRepository,
{
    fetcher: Arc<F>,
    course_repo: Arc<CR>,
    module_repo: Arc<MR>,
    video_repo: Arc<VR>,
    search_repo: Arc<SR>,
    sanitizer: TitleSanitizer,
    boundary_detector: BoundaryDetector,
}

impl<F, CR, MR, VR, SR> IngestPlaylistUseCase<F, CR, MR, VR, SR>
where
    F: PlaylistFetcher,
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
    SR: SearchRepository,
{
    pub fn new(
        fetcher: Arc<F>,
        course_repo: Arc<CR>,
        module_repo: Arc<MR>,
        video_repo: Arc<VR>,
        search_repo: Arc<SR>,
    ) -> Self {
        Self {
            fetcher,
            course_repo,
            module_repo,
            video_repo,
            search_repo,
            sanitizer: TitleSanitizer::new(),
            boundary_detector: BoundaryDetector::new(),
        }
    }

    /// Executes the playlist ingestion pipeline.
    pub async fn execute(
        &self,
        input: IngestPlaylistInput,
    ) -> Result<IngestPlaylistOutput, IngestError> {
        // 1. Parse and validate URL
        let playlist_url = PlaylistUrl::new(&input.playlist_url)
            .map_err(|e| IngestError::InvalidUrl(e.to_string()))?;

        // 2. Fetch playlist metadata
        let raw_videos = self
            .fetcher
            .fetch_playlist(&playlist_url)
            .await
            .map_err(|e| IngestError::FetchFailed(e.to_string()))?;

        if raw_videos.is_empty() {
            return Err(IngestError::FetchFailed("Playlist is empty".to_string()));
        }

        // 3. Sanitize titles
        let sanitized_titles: Vec<String> =
            raw_videos.iter().map(|v| self.sanitizer.sanitize(&v.title)).collect();

        // 4. Group videos into modules (simple batch grouping)
        let module_groups = self.boundary_detector.group_into_modules(raw_videos.len());

        // 5. Create course
        let course_name = input
            .course_name
            .unwrap_or_else(|| sanitized_titles.first().cloned().unwrap_or_default());
        let course_id = CourseId::new();
        let course = Course::new(
            course_id.clone(),
            course_name,
            playlist_url.clone(),
            playlist_url.playlist_id().to_string(),
            None,
        );
        self.course_repo.save(&course).map_err(|e| IngestError::PersistFailed(e.to_string()))?;

        self.search_repo
            .index_course(course.id(), course.name(), course.description())
            .map_err(|e| IngestError::PersistFailed(e.to_string()))?;

        // 6. Create modules and videos
        let mut total_videos = 0;
        for (module_idx, video_indices) in module_groups.iter().enumerate() {
            let module_id = ModuleId::new();

            // Use first video title as module title
            let module_title = video_indices
                .first()
                .map(|&i| sanitized_titles[i].clone())
                .unwrap_or_else(|| format!("Module {}", module_idx + 1));

            let module =
                Module::new(module_id.clone(), course_id.clone(), module_title, module_idx as u32);
            self.module_repo
                .save(&module)
                .map_err(|e| IngestError::PersistFailed(e.to_string()))?;

            // Create videos in this module
            for (sort_order, &video_idx) in video_indices.iter().enumerate() {
                let raw = &raw_videos[video_idx];
                let youtube_id = YouTubeVideoId::new(&raw.youtube_id)
                    .map_err(|e| IngestError::PersistFailed(e.to_string()))?;
                let source = VideoSource::youtube(youtube_id);

                let video = Video::with_description(
                    VideoId::new(),
                    module_id.clone(),
                    source,
                    sanitized_titles[video_idx].clone(),
                    raw.description.clone(),
                    raw.duration_secs,
                    sort_order as u32,
                );
                self.video_repo
                    .save(&video)
                    .map_err(|e| IngestError::PersistFailed(e.to_string()))?;

                self.search_repo
                    .index_video(
                        &video.id().as_uuid().to_string(),
                        video.title(),
                        raw.description.as_deref(),
                        &course_id,
                    )
                    .map_err(|e| IngestError::PersistFailed(e.to_string()))?;

                total_videos += 1;
            }
        }

        Ok(IngestPlaylistOutput {
            course_id,
            modules_count: module_groups.len(),
            videos_count: total_videos,
        })
    }
}
