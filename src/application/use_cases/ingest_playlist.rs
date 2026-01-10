//! Ingest Playlist Use Case
//!
//! Orchestrates: Fetch → Sanitize → Embed → Detect Boundaries → Persist

use std::sync::Arc;

use crate::domain::{
    entities::{Course, Module, Video},
    ports::{CourseRepository, ModuleRepository, PlaylistFetcher, TextEmbedder, VideoRepository},
    services::{BoundaryDetector, TitleSanitizer},
    value_objects::{CourseId, ModuleId, PlaylistUrl, VideoId, YouTubeVideoId},
};

/// Error type for playlist ingestion.
#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Invalid playlist URL: {0}")]
    InvalidUrl(String),
    #[error("Failed to fetch playlist: {0}")]
    FetchFailed(String),
    #[error("Failed to generate embeddings: {0}")]
    EmbeddingFailed(String),
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
pub struct IngestPlaylistUseCase<F, E, CR, MR, VR>
where
    F: PlaylistFetcher,
    E: TextEmbedder,
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
{
    fetcher: Arc<F>,
    embedder: Arc<E>,
    course_repo: Arc<CR>,
    module_repo: Arc<MR>,
    video_repo: Arc<VR>,
    sanitizer: TitleSanitizer,
    boundary_detector: BoundaryDetector,
}

impl<F, E, CR, MR, VR> IngestPlaylistUseCase<F, E, CR, MR, VR>
where
    F: PlaylistFetcher,
    E: TextEmbedder,
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
{
    pub fn new(
        fetcher: Arc<F>,
        embedder: Arc<E>,
        course_repo: Arc<CR>,
        module_repo: Arc<MR>,
        video_repo: Arc<VR>,
    ) -> Self {
        Self {
            fetcher,
            embedder,
            course_repo,
            module_repo,
            video_repo,
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

        // 4. Generate embeddings
        let title_refs: Vec<&str> = sanitized_titles.iter().map(String::as_str).collect();
        let embeddings = self
            .embedder
            .embed_batch(&title_refs)
            .map_err(|e| IngestError::EmbeddingFailed(e.to_string()))?;

        // 5. Detect module boundaries
        let module_groups = self.boundary_detector.group_into_modules(&embeddings);

        // 6. Create course
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

        // 7. Create modules and videos
        let mut total_videos = 0;
        for (module_idx, video_indices) in module_groups.iter().enumerate() {
            let module_id = ModuleId::new();

            // Use first video title as module title (can be improved with LLM later)
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

                let video = Video::new(
                    VideoId::new(),
                    module_id.clone(),
                    youtube_id,
                    sanitized_titles[video_idx].clone(),
                    raw.duration_secs,
                    sort_order as u32,
                );
                self.video_repo
                    .save(&video)
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
