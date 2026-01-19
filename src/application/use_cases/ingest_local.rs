//! Ingest Local Library Use Case
//!
//! Orchestrates: Scan → Sanitize → Group → Persist

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use crate::domain::{
    entities::{Course, Module, Video},
    ports::{
        CourseRepository, LocalMediaScanner, ModuleRepository, RawLocalMediaMetadata,
        SearchRepository, VideoRepository,
    },
    services::TitleSanitizer,
    value_objects::{CourseId, ModuleId, PlaylistUrl, VideoId, VideoSource},
};

/// Error type for local ingestion.
#[derive(Debug, thiserror::Error)]
pub enum IngestLocalError {
    #[error("Invalid root path: {0}")]
    InvalidRoot(String),
    #[error("Failed to scan local media: {0}")]
    ScanFailed(String),
    #[error("Failed to persist: {0}")]
    PersistFailed(String),
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
pub struct IngestLocalUseCase<S, CR, MR, VR, SR>
where
    S: LocalMediaScanner,
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
    SR: SearchRepository,
{
    scanner: Arc<S>,
    course_repo: Arc<CR>,
    module_repo: Arc<MR>,
    video_repo: Arc<VR>,
    search_repo: Arc<SR>,
    sanitizer: TitleSanitizer,
}

impl<S, CR, MR, VR, SR> IngestLocalUseCase<S, CR, MR, VR, SR>
where
    S: LocalMediaScanner,
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
    SR: SearchRepository,
{
    pub fn new(
        scanner: Arc<S>,
        course_repo: Arc<CR>,
        module_repo: Arc<MR>,
        video_repo: Arc<VR>,
        search_repo: Arc<SR>,
    ) -> Self {
        Self {
            scanner,
            course_repo,
            module_repo,
            video_repo,
            search_repo,
            sanitizer: TitleSanitizer::new(),
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

        // 1. Scan local media
        let mut raw_media = self
            .scanner
            .scan(root)
            .await
            .map_err(|e| IngestLocalError::ScanFailed(e.to_string()))?;

        if raw_media.is_empty() {
            return Err(IngestLocalError::ScanFailed("No media files found".to_string()));
        }

        // 2. Sort for deterministic grouping
        raw_media.sort_by(|a, b| a.path.cmp(&b.path));

        // 3. Group by directory
        let grouped = group_by_folder(root, &raw_media);

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
            course_id.clone(),
            course_name,
            playlist_url.clone(),
            playlist_url.playlist_id().to_string(),
            None,
        );

        self.course_repo
            .save(&course)
            .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

        self.search_repo
            .index_course(course.id(), course.name(), course.description())
            .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

        // 5. Create modules and videos
        let mut total_videos = 0;
        for (module_idx, (folder_path, items)) in grouped.into_iter().enumerate() {
            let module_id = ModuleId::new();
            let module_title = module_title_for(root, &folder_path, &self.sanitizer);

            let module =
                Module::new(module_id.clone(), course_id.clone(), module_title, module_idx as u32);

            self.module_repo
                .save(&module)
                .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

            for (sort_order, item) in items.into_iter().enumerate() {
                let source = VideoSource::local_path(&item.path)
                    .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

                let title = self.sanitizer.sanitize(&item.title);

                let video = Video::with_description(
                    VideoId::new(),
                    module_id.clone(),
                    source,
                    title,
                    None,
                    item.duration_secs,
                    sort_order as u32,
                );

                self.video_repo
                    .save(&video)
                    .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

                self.search_repo
                    .index_video(&video.id().as_uuid().to_string(), video.title(), None, &course_id)
                    .map_err(|e| IngestLocalError::PersistFailed(e.to_string()))?;

                total_videos += 1;
            }
        }

        Ok(IngestLocalOutput {
            course_id,
            modules_count: grouped_len(root, &raw_media),
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

    grouped
}

fn module_title_for(root: &str, folder: &str, sanitizer: &TitleSanitizer) -> String {
    let root_path = Path::new(root);
    let folder_path = Path::new(folder);

    if folder_path == root_path {
        return "Root".to_string();
    }

    folder_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| sanitizer.sanitize(s))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Module".to_string())
}

fn grouped_len(root: &str, items: &[RawLocalMediaMetadata]) -> usize {
    group_by_folder(root, items).len()
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
            },
            RawLocalMediaMetadata {
                path: "/root/folder1/video2.mp4".to_string(),
                title: "video2".to_string(),
                duration_secs: 20,
            },
            RawLocalMediaMetadata {
                path: "/root/folder1/video3.mp4".to_string(),
                title: "video3".to_string(),
                duration_secs: 30,
            },
            RawLocalMediaMetadata {
                path: "/root/folder2/sub/video4.mp4".to_string(),
                title: "video4".to_string(),
                duration_secs: 40,
            },
        ];

        let grouped = group_by_folder(root, &items);
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped.get("/root").unwrap().len(), 1);
        assert_eq!(grouped.get("/root/folder1").unwrap().len(), 2);
        assert_eq!(grouped.get("/root/folder2/sub").unwrap().len(), 1);
    }

    #[test]
    fn test_module_title_for() {
        let sanitizer = TitleSanitizer::new();
        let root = "/courses/rust";

        assert_eq!(module_title_for(root, "/courses/rust", &sanitizer), "Root");
        assert_eq!(
            module_title_for(root, "/courses/rust/Introduction", &sanitizer),
            "Introduction"
        );
        assert_eq!(module_title_for(root, "/courses/rust/01_Basics", &sanitizer), "01_Basics");
    }
}
