//! Ingest Playlist Use Case
//!
//! Orchestrates: Fetch → Sanitize → Group → Persist

use std::sync::Arc;

use diesel::ExpressionMethods;
use diesel::RunQueryDsl;

use crate::domain::{
    entities::{Course, Module, Video},
    ports::{
        CourseRepository, ModuleRepository, ModuleTitleGenerator, PlaylistFetcher,
        SearchRepository, VideoRepository,
    },
    services::{BoundaryDetector, TitleSanitizer},
    value_objects::{CourseId, ModuleId, PlaylistUrl, VideoId, VideoSource, YouTubeVideoId},
};
use crate::infrastructure::{media_hash, persistence::DbPool};

/// Error type for playlist ingestion.
#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Invalid playlist URL: {0}")]
    InvalidUrl(String),
    #[error("Failed to fetch playlist: {0}")]
    FetchFailed(String),
    #[error("Failed to persist: {0}")]
    PersistFailed(String),
    #[error("Course already exists: {0}")]
    AlreadyExists(String),
}

impl From<diesel::result::Error> for IngestError {
    fn from(e: diesel::result::Error) -> Self {
        IngestError::PersistFailed(e.to_string())
    }
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
#[allow(dead_code)]
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
    title_generator: Option<Arc<dyn ModuleTitleGenerator>>,
    pool: Option<Arc<DbPool>>,
}

impl<F, CR, MR, VR, SR> IngestPlaylistUseCase<F, CR, MR, VR, SR>
where
    F: PlaylistFetcher,
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
    SR: SearchRepository,
{
    async fn generate_module_title(
        &self,
        titles: &[String],
        course_name: &str,
        module_idx: usize,
    ) -> String {
        if let Some(ref generator) = self.title_generator
            && let Ok(title) =
                generator.generate_module_title(titles, course_name, module_idx).await
            && !title.is_empty()
        {
            return title;
        }
        titles.first().cloned().unwrap_or_else(|| format!("Module {}", module_idx + 1))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fetcher: Arc<F>,
        course_repo: Arc<CR>,
        module_repo: Arc<MR>,
        video_repo: Arc<VR>,
        search_repo: Arc<SR>,
        title_generator: Option<Arc<dyn ModuleTitleGenerator>>,
        pool: Option<Arc<DbPool>>,
    ) -> Self {
        Self {
            fetcher,
            course_repo,
            module_repo,
            video_repo,
            search_repo,
            sanitizer: TitleSanitizer::new(),
            boundary_detector: BoundaryDetector::new(),
            title_generator,
            pool,
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

        // 2. Check for duplicate
        let source_hash = media_hash::compute_source_hash(playlist_url.playlist_id());
        if let Ok(Some(existing)) = self.course_repo.find_by_source_hash(&source_hash) {
            return Err(IngestError::AlreadyExists(existing.name().to_string()));
        }

        // 3. Fetch playlist metadata
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

        // 4. Group videos into modules (title-aware with batch fallback)
        let sanitized_titles_refs: Vec<&str> =
            sanitized_titles.iter().map(|s| s.as_str()).collect();
        let module_groups = self.boundary_detector.group_by_titles(&sanitized_titles_refs);

        // 5. Create course
        let course_name = input
            .course_name
            .unwrap_or_else(|| sanitized_titles.first().cloned().unwrap_or_default());
        let course_id = CourseId::new();
        let course = Course::new(
            course_id.clone(),
            course_name.clone(),
            playlist_url.clone(),
            playlist_url.playlist_id().to_string(),
            None,
            Some(source_hash),
        );
        // 6. Pre-compute module/video data (async title generation, outside transaction)
        struct PendingVideo {
            youtube_id: String,
            title: String,
            description: Option<String>,
            duration_secs: u32,
        }

        struct PendingModule {
            module_id: ModuleId,
            title: String,
            videos: Vec<PendingVideo>,
        }

        let mut pending_modules = Vec::with_capacity(module_groups.len());
        let mut total_videos = 0;

        for (module_idx, video_indices) in module_groups.iter().enumerate() {
            let module_id = ModuleId::new();

            let module_video_titles: Vec<String> =
                video_indices.iter().map(|&i| sanitized_titles[i].clone()).collect();
            let module_title =
                self.generate_module_title(&module_video_titles, &course_name, module_idx).await;

            let videos: Vec<PendingVideo> = video_indices
                .iter()
                .map(|&i| {
                    let raw = &raw_videos[i];
                    PendingVideo {
                        youtube_id: raw.youtube_id.clone(),
                        title: sanitized_titles[i].clone(),
                        description: raw.description.clone(),
                        duration_secs: raw.duration_secs,
                    }
                })
                .collect();

            total_videos += videos.len();
            pending_modules.push(PendingModule { module_id, title: module_title, videos });
        }

        // 7. Persist everything atomically in a single transaction
        if let Some(ref pool) = self.pool {
            use crate::infrastructure::persistence::models::{NewCourse, NewModule, NewVideo};
            use crate::schema::{courses, modules, videos};
            use diesel::connection::Connection;
            use diesel::sql_types::Text;

            let course_id_str = course_id.as_uuid().to_string();

            let mut conn = pool.get().map_err(|e| IngestError::PersistFailed(e.to_string()))?;

            conn.transaction::<_, IngestError, _>(|tx| {
                let new_course = NewCourse {
                    id: &course_id_str,
                    name: course.name(),
                    source_url: course.source_url().raw(),
                    playlist_id: course.playlist_id(),
                    description: course.description(),
                    source_hash: course.source_hash(),
                };
                diesel::insert_into(courses::table)
                    .values(&new_course)
                    .on_conflict(courses::id)
                    .do_update()
                    .set((
                        courses::name.eq(course.name()),
                        courses::description.eq(course.description()),
                    ))
                    .execute(tx)?;

                diesel::sql_query(
                    "INSERT INTO search_index (entity_type, entity_id, title, content, course_id) VALUES ('course', ?, ?, ?, ?)",
                )
                .bind::<Text, _>(&course_id_str)
                .bind::<Text, _>(course.name())
                .bind::<Text, _>("")
                .bind::<Text, _>(&course_id_str)
                .execute(tx)?;

                for (module_idx, pm) in pending_modules.iter().enumerate() {
                    let module_id_str = pm.module_id.as_uuid().to_string();

                    let new_module = NewModule {
                        id: &module_id_str,
                        course_id: &course_id_str,
                        title: &pm.title,
                        sort_order: module_idx as i32,
                    };
                    diesel::insert_into(modules::table)
                        .values(&new_module)
                        .on_conflict(modules::id)
                        .do_update()
                        .set((
                            modules::title.eq(&pm.title),
                            modules::sort_order.eq(module_idx as i32),
                        ))
                        .execute(tx)?;

                    for (sort_order, video) in pm.videos.iter().enumerate() {
                        let video_id = VideoId::new();
                        let video_id_str = video_id.as_uuid().to_string();

                        let new_video = NewVideo {
                            id: &video_id_str,
                            module_id: &module_id_str,
                            youtube_id: Some(&video.youtube_id),
                            title: &video.title,
                            duration_secs: video.duration_secs as i32,
                            is_completed: false,
                            sort_order: sort_order as i32,
                            description: video.description.as_deref(),
                            transcript: None,
                            summary: None,
                            source_type: "youtube",
                            source_ref: &video.youtube_id,
                            key_points: None,
                            key_terms: None,
                        };
                        diesel::insert_into(videos::table)
                            .values(&new_video)
                            .on_conflict(videos::id)
                            .do_update()
                            .set((
                                videos::title.eq(&video.title),
                                videos::duration_secs.eq(video.duration_secs as i32),
                                videos::is_completed.eq(false),
                                videos::sort_order.eq(sort_order as i32),
                                videos::description.eq(video.description.as_deref()),
                                videos::transcript.eq::<Option<&str>>(None),
                                videos::summary.eq::<Option<&str>>(None),
                                videos::module_id.eq(&module_id_str),
                            ))
                            .execute(tx)?;

                        diesel::sql_query(
                            "INSERT INTO search_index (entity_type, entity_id, title, content, course_id) VALUES ('video', ?, ?, ?, ?)",
                        )
                        .bind::<Text, _>(&video_id_str)
                        .bind::<Text, _>(&video.title)
                        .bind::<Text, _>(video.description.as_deref().unwrap_or(""))
                        .bind::<Text, _>(&course_id_str)
                        .execute(tx)?;
                    }
                }

                Ok(())
            })?;
        } else {
            self.course_repo
                .save(&course)
                .map_err(|e| IngestError::PersistFailed(e.to_string()))?;
            self.search_repo
                .index_course(course.id(), course.name(), course.description())
                .map_err(|e| IngestError::PersistFailed(e.to_string()))?;
            for (module_idx, pm) in pending_modules.iter().enumerate() {
                let module = Module::new(
                    pm.module_id.clone(),
                    course_id.clone(),
                    pm.title.clone(),
                    module_idx as u32,
                );
                self.module_repo
                    .save(&module)
                    .map_err(|e| IngestError::PersistFailed(e.to_string()))?;
                for (sort_order, video_data) in pm.videos.iter().enumerate() {
                    let youtube_id = YouTubeVideoId::new(&video_data.youtube_id)
                        .map_err(|e| IngestError::PersistFailed(e.to_string()))?;
                    let source = VideoSource::youtube(youtube_id);
                    let video = Video::with_description(
                        VideoId::new(),
                        pm.module_id.clone(),
                        source,
                        video_data.title.clone(),
                        video_data.description.clone(),
                        video_data.duration_secs,
                        sort_order as u32,
                    );
                    self.video_repo
                        .save(&video)
                        .map_err(|e| IngestError::PersistFailed(e.to_string()))?;
                    self.search_repo
                        .index_video(
                            &video.id().as_uuid().to_string(),
                            video.title(),
                            video_data.description.as_deref(),
                            &course_id,
                        )
                        .map_err(|e| IngestError::PersistFailed(e.to_string()))?;
                }
            }
        }

        Ok(IngestPlaylistOutput {
            course_id,
            modules_count: module_groups.len(),
            videos_count: total_videos,
        })
    }
}
