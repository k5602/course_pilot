//! Ingest Local Library Use Case
//!
//! Orchestrates: Scan -> Sanitize -> Group -> Persist

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::domain::{
    entities::{Course, Module, Video},
    ports::{
        CourseRepository, LocalMediaError, LocalMediaScanner, ModuleRepository,
        ModuleTitleGenerator, RawLocalMediaMetadata, SearchEntry, SearchRepository,
        VideoRepository,
    },
    services::{BoundaryDetector, SubtitleCleaner, TitleSanitizer, title_number_sequence},
    value_objects::{CourseId, ModuleId, PlaylistUrl, VideoId, VideoSource},
};
use crate::infrastructure::media_hash;

/// Error type for local ingestion.
#[derive(Debug, thiserror::Error)]
pub enum IngestLocalError {
    #[error("Invalid root path: {0}")]
    InvalidRoot(String),
    #[error(transparent)]
    ScanFailed(#[from] LocalMediaError),
    #[error("Failed to persist: {0}")]
    PersistFailed(String),
    #[error("Course already exists: {0}")]
    AlreadyExists(String),
}

/// Input for the ingest local library use case.
pub struct IngestLocalInput {
    pub root_path: String,
    pub course_name: Option<String>,
}

/// Output of the ingest local library use case.
#[derive(Debug)]
pub struct IngestLocalOutput {
    pub course_id: CourseId,
    pub modules_count: usize,
    pub videos_count: usize,
}

/// Use case for ingesting a local media library into a structured course.
pub struct IngestLocalUseCase {
    scanner: Arc<dyn LocalMediaScanner>,
    course_repo: Arc<dyn CourseRepository>,
    module_repo: Arc<dyn ModuleRepository>,
    video_repo: Arc<dyn VideoRepository>,
    search_repo: Arc<dyn SearchRepository>,
    sanitizer: TitleSanitizer,
    boundary_batch_size: usize,
    title_generator: Option<Arc<dyn ModuleTitleGenerator>>,
}

impl IngestLocalUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scanner: Arc<dyn LocalMediaScanner>,
        course_repo: Arc<dyn CourseRepository>,
        module_repo: Arc<dyn ModuleRepository>,
        video_repo: Arc<dyn VideoRepository>,
        search_repo: Arc<dyn SearchRepository>,
        title_generator: Option<Arc<dyn ModuleTitleGenerator>>,
        boundary_batch_size: usize,
    ) -> Self {
        Self {
            scanner,
            course_repo,
            module_repo,
            video_repo,
            search_repo,
            sanitizer: TitleSanitizer::new(),
            boundary_batch_size,
            title_generator,
        }
    }

    /// Executes the local ingestion pipeline.
    pub async fn execute(
        &self,
        input: IngestLocalInput,
    ) -> Result<IngestLocalOutput, IngestLocalError> {
        let root = input.root_path.trim();
        if root.is_empty() {
            return Err(IngestLocalError::InvalidRoot("root path is empty".to_string()));
        }

        // 1. Check for duplicate
        let source_hash = media_hash::compute_source_hash(root);
        if let Ok(Some(existing)) = self.course_repo.find_by_source_hash(&source_hash) {
            return Err(IngestLocalError::AlreadyExists(existing.name().to_string()));
        }

        // 2. Scan local media
        let mut raw_media = self.scanner.scan(root).await?;

        if raw_media.is_empty() {
            return Err(IngestLocalError::ScanFailed(LocalMediaError::Io(
                "No media files found".to_string(),
            )));
        }

        // 2. Sort for deterministic grouping
        raw_media.sort_by(|a, b| a.path.cmp(&b.path));

        // 3. Group by directory
        let grouped = group_by_folder(root, &raw_media);
        let grouped =
            split_root_group_if_needed(root, &grouped, &self.sanitizer, self.boundary_batch_size);

        // 4. Create course (use synthetic playlist URL for persistence)
        let course_id = CourseId::new();
        let list_id = format!("local-{}", course_id.as_uuid());
        let playlist_url =
            PlaylistUrl::new(&format!("https://www.youtube.com/playlist?list={list_id}"))
                .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

        let course_name = input.course_name.unwrap_or_else(|| {
            let base =
                Path::new(root).file_name().and_then(|s| s.to_str()).unwrap_or("Local Course");
            self.sanitizer.sanitize(base)
        });

        let course = Course::new(
            course_id,
            course_name.clone(),
            playlist_url.clone(),
            playlist_url.playlist_id().to_string(),
            None,
            Some(source_hash),
        );

        // 5. Pre-compute module/video data (async title generation + subtitle reads, outside transaction)
        struct PendingVideo {
            path: String,
            title: String,
            duration_secs: u32,
            transcript: Option<String>,
        }

        struct PendingModule {
            module_id: ModuleId,
            title: String,
            videos: Vec<PendingVideo>,
        }

        let cleaner = SubtitleCleaner::new();
        let mut pending_modules = Vec::new();
        let mut total_videos = 0;

        for (module_idx, (_folder_path, items)) in grouped.into_iter().enumerate() {
            let module_id = ModuleId::new();
            let module_video_titles: Vec<String> =
                items.iter().map(|item| self.sanitizer.sanitize(&item.title)).collect();
            let module_title = crate::application::generate_module_title(
                self.title_generator.as_ref(),
                &module_video_titles,
                &course_name,
                module_idx,
            )
            .await;

            let mut videos = Vec::with_capacity(items.len());
            for item in items {
                let title = self.sanitizer.sanitize(&item.title);
                let transcript = item
                    .subtitles
                    .first()
                    .and_then(|sub| {
                        fs::read_to_string(&sub.path).ok().map(|raw| cleaner.clean(&raw))
                    })
                    .filter(|s| !s.trim().is_empty());

                videos.push(PendingVideo {
                    path: item.path,
                    title,
                    duration_secs: item.duration_secs,
                    transcript,
                });
            }
            total_videos += videos.len();
            pending_modules.push(PendingModule { module_id, title: module_title, videos });
        }

        // 6. Persist using repository batch methods
        self.course_repo
            .save(&course)
            .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;
        self.search_repo
            .index_course(course.id(), course.name(), course.description())
            .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

        let mut all_modules = Vec::with_capacity(pending_modules.len());
        let mut all_videos = Vec::with_capacity(total_videos);
        let mut video_search_entries = Vec::with_capacity(total_videos);

        for (module_idx, pm) in pending_modules.iter().enumerate() {
            let module = Module::new(pm.module_id, course_id, pm.title.clone(), module_idx as u32);
            all_modules.push(module);

            for (sort_order, video_data) in pm.videos.iter().enumerate() {
                let source = VideoSource::local_path(&video_data.path)
                    .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;
                let mut video = Video::with_description(
                    VideoId::new(),
                    pm.module_id,
                    source,
                    video_data.title.clone(),
                    None,
                    video_data.duration_secs,
                    sort_order as u32,
                );
                if video_data.transcript.is_some() {
                    video.update_transcript(video_data.transcript.clone());
                }
                video_search_entries.push(SearchEntry {
                    entity_type: "video".to_string(),
                    entity_id: video.id().as_uuid().to_string(),
                    title: video_data.title.clone(),
                    content: String::new(),
                    course_id: course_id.as_uuid().to_string(),
                });
                all_videos.push(video);
            }
        }

        self.module_repo
            .save_batch(&all_modules)
            .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;
        self.video_repo
            .save_batch(&all_videos)
            .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;
        self.search_repo
            .index_batch(&video_search_entries)
            .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

        Ok(IngestLocalOutput {
            course_id,
            modules_count: grouped_len(root, &raw_media, self.boundary_batch_size),
            videos_count: total_videos,
        })
    }
}

fn group_by_folder(
    root: &str,
    items: &[RawLocalMediaMetadata],
) -> BTreeMap<String, Vec<RawLocalMediaMetadata>> {
    let mut grouped: BTreeMap<String, Vec<RawLocalMediaMetadata>> = BTreeMap::new();
    for item in items {
        let path = Path::new(&item.path);
        let folder = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| root.to_string());

        grouped.entry(folder).or_default().push(item.clone());
    }

    for entries in grouped.values_mut() {
        entries.sort_by(|a, b| {
            let a_key = title_number_sequence(&a.title);
            let b_key = title_number_sequence(&b.title);

            match (a_key, b_key) {
                (Some(a_seq), Some(b_seq)) => a_seq.cmp(&b_seq).then_with(|| a.title.cmp(&b.title)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.title.cmp(&b.title),
            }
        });
    }

    grouped
}

fn split_root_group_if_needed(
    root: &str,
    grouped: &BTreeMap<String, Vec<RawLocalMediaMetadata>>,
    sanitizer: &TitleSanitizer,
    boundary_batch_size: usize,
) -> BTreeMap<String, Vec<RawLocalMediaMetadata>> {
    if grouped.len() != 1 {
        return grouped.clone();
    }

    let (folder, items) = match grouped.iter().next() {
        Some(entry) => entry,
        None => return BTreeMap::new(),
    };

    let root_path = Path::new(root);
    let folder_path = Path::new(folder);
    let canonical_root = fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let canonical_folder =
        fs::canonicalize(folder_path).unwrap_or_else(|_| folder_path.to_path_buf());

    if canonical_folder != canonical_root {
        return grouped.clone();
    }

    let detector = BoundaryDetector::with_batch_size(boundary_batch_size);
    let raw_titles: Vec<&str> = items.iter().map(|item| item.title.as_str()).collect();
    let groups = detector.group_by_titles(&raw_titles);
    if groups.len() <= 1 {
        return grouped.clone();
    }

    let mut split = BTreeMap::new();
    for (idx, indices) in groups.iter().enumerate() {
        let title = indices
            .first()
            .and_then(|&i| items.get(i))
            .map(|item| sanitizer.sanitize(&item.title))
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("Module {}", idx + 1));

        let module_label = if title.starts_with("Module ") {
            title
        } else {
            format!("Module {} - {}", idx + 1, title)
        };

        let key = format!("{root}/{module_label}");
        let mut bucket = Vec::new();
        for &i in indices {
            if let Some(item) = items.get(i) {
                bucket.push(item.clone());
            }
        }
        split.insert(key, bucket);
    }

    split
}

fn grouped_len(root: &str, items: &[RawLocalMediaMetadata], boundary_batch_size: usize) -> usize {
    let grouped = group_by_folder(root, items);
    let sanitizer = TitleSanitizer::new();
    split_root_group_if_needed(root, &grouped, &sanitizer, boundary_batch_size).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_folder() {
        let root = "/root";
        let items = vec![
            RawLocalMediaMetadata {
                path: "/root/video1.mp4".to_string(),
                title: "video1".to_string(),
                duration_secs: 10,
                subtitles: Vec::new(),
            },
            RawLocalMediaMetadata {
                path: "/root/folder1/video2.mp4".to_string(),
                title: "video2".to_string(),
                duration_secs: 20,
                subtitles: Vec::new(),
            },
            RawLocalMediaMetadata {
                path: "/root/folder1/video3.mp4".to_string(),
                title: "video3".to_string(),
                duration_secs: 30,
                subtitles: Vec::new(),
            },
            RawLocalMediaMetadata {
                path: "/root/folder2/sub/video4.mp4".to_string(),
                title: "video4".to_string(),
                duration_secs: 40,
                subtitles: Vec::new(),
            },
        ];

        let grouped = group_by_folder(root, &items);
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped.get("/root").unwrap().len(), 1);
        assert_eq!(grouped.get("/root/folder1").unwrap().len(), 2);
        assert_eq!(grouped.get("/root/folder2/sub").unwrap().len(), 1);
    }
}
