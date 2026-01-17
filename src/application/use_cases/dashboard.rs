//! Dashboard analytics use case.

use std::sync::Arc;

use crate::domain::entities::AppAnalytics;
use crate::domain::ports::{CourseRepository, ModuleRepository, RepositoryError, VideoRepository};

/// Use case for loading dashboard analytics.
///
/// Aggregates counts and durations across all courses.
pub struct LoadDashboardUseCase<CR, MR, VR>
where
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
{
    course_repo: Arc<CR>,
    module_repo: Arc<MR>,
    video_repo: Arc<VR>,
}

impl<CR, MR, VR> LoadDashboardUseCase<CR, MR, VR>
where
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
{
    /// Creates a new dashboard analytics use case.
    pub fn new(course_repo: Arc<CR>, module_repo: Arc<MR>, video_repo: Arc<VR>) -> Self {
        Self { course_repo, module_repo, video_repo }
    }

    /// Loads aggregated analytics for the dashboard.
    pub fn execute(&self) -> Result<AppAnalytics, RepositoryError> {
        let courses = self.course_repo.find_all()?;

        let mut total_modules: u32 = 0;
        let mut total_videos: u32 = 0;
        let mut completed_videos: u32 = 0;
        let mut total_duration_secs: u64 = 0;
        let mut completed_duration_secs: u64 = 0;
        let mut videos_with_summary: u32 = 0;

        for course in &courses {
            let modules = self.module_repo.find_by_course(course.id())?;
            total_modules += modules.len() as u32;

            let videos = self.video_repo.find_by_course(course.id())?;
            total_videos += videos.len() as u32;

            for video in videos {
                let duration = video.duration_secs() as u64;
                total_duration_secs += duration;

                if video.is_completed() {
                    completed_videos += 1;
                    completed_duration_secs += duration;
                }

                if video.summary().is_some() {
                    videos_with_summary += 1;
                }
            }
        }

        Ok(AppAnalytics::new(
            courses.len() as u32,
            total_modules,
            total_videos,
            completed_videos,
            total_duration_secs,
            completed_duration_secs,
            videos_with_summary,
        ))
    }
}
