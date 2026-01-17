//! Analytics entity - Aggregated data for the dashboard.

/// Aggregated analytics for the dashboard overview.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppAnalytics {
    total_courses: u32,
    total_modules: u32,
    total_videos: u32,
    completed_videos: u32,
    total_duration_secs: u64,
    completed_duration_secs: u64,
    videos_with_summary: u32,
}

impl AppAnalytics {
    /// Creates a new analytics snapshot.
    pub fn new(
        total_courses: u32,
        total_modules: u32,
        total_videos: u32,
        completed_videos: u32,
        total_duration_secs: u64,
        completed_duration_secs: u64,
        videos_with_summary: u32,
    ) -> Self {
        Self {
            total_courses,
            total_modules,
            total_videos,
            completed_videos,
            total_duration_secs,
            completed_duration_secs,
            videos_with_summary,
        }
    }

    pub fn total_courses(&self) -> u32 {
        self.total_courses
    }

    pub fn total_modules(&self) -> u32 {
        self.total_modules
    }

    pub fn total_videos(&self) -> u32 {
        self.total_videos
    }

    pub fn completed_videos(&self) -> u32 {
        self.completed_videos
    }

    pub fn total_duration_secs(&self) -> u64 {
        self.total_duration_secs
    }

    pub fn completed_duration_secs(&self) -> u64 {
        self.completed_duration_secs
    }

    pub fn videos_with_summary(&self) -> u32 {
        self.videos_with_summary
    }

    /// Returns completion percentage (0.0 - 100.0).
    pub fn completion_percent(&self) -> f32 {
        if self.total_videos == 0 {
            return 0.0;
        }
        (self.completed_videos as f32 / self.total_videos as f32) * 100.0
    }

    /// Returns summary coverage percentage (0.0 - 100.0).
    pub fn summary_coverage_percent(&self) -> f32 {
        if self.total_videos == 0 {
            return 0.0;
        }
        (self.videos_with_summary as f32 / self.total_videos as f32) * 100.0
    }

    /// Returns total duration in minutes (rounded down).
    pub fn total_duration_minutes(&self) -> u64 {
        self.total_duration_secs / 60
    }

    /// Returns completed duration in minutes (rounded down).
    pub fn completed_duration_minutes(&self) -> u64 {
        self.completed_duration_secs / 60
    }
}
