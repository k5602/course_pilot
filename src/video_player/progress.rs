//! Video progress tracking
//!
//! Simple progress persistence for tracking video completion.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Progress record for a single video
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoProgress {
    /// Course ID this video belongs to
    pub course_id: Uuid,
    /// Video index within the course
    pub video_index: usize,
    /// Last known playback position in seconds
    pub position_seconds: f64,
    /// Total duration of the video in seconds
    pub duration_seconds: f64,
    /// Whether the video has been marked as completed
    pub completed: bool,
    /// Timestamp of last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl VideoProgress {
    /// Create a new progress record
    pub fn new(course_id: Uuid, video_index: usize, duration_seconds: f64) -> Self {
        Self {
            course_id,
            video_index,
            position_seconds: 0.0,
            duration_seconds,
            completed: false,
            updated_at: chrono::Utc::now(),
        }
    }

    /// Update the playback position
    pub fn update_position(&mut self, position: f64) {
        self.position_seconds = position;
        self.updated_at = chrono::Utc::now();

        // Auto-complete if watched 90% or more
        if self.duration_seconds > 0.0 {
            let watched_ratio = position / self.duration_seconds;
            if watched_ratio >= 0.9 {
                self.completed = true;
            }
        }
    }

    /// Mark as completed manually
    pub fn mark_completed(&mut self) {
        self.completed = true;
        self.updated_at = chrono::Utc::now();
    }

    /// Reset progress
    pub fn reset(&mut self) {
        self.position_seconds = 0.0;
        self.completed = false;
        self.updated_at = chrono::Utc::now();
    }

    /// Get progress as a percentage (0.0 to 100.0)
    pub fn progress_percentage(&self) -> f64 {
        if self.duration_seconds > 0.0 {
            (self.position_seconds / self.duration_seconds * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> f64 {
        (self.duration_seconds - self.position_seconds).max(0.0)
    }
}

/// Aggregate progress for an entire course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseProgress {
    pub course_id: Uuid,
    pub total_videos: usize,
    pub completed_videos: usize,
    pub total_watch_time_seconds: f64,
    pub total_duration_seconds: f64,
}

impl CourseProgress {
    /// Create from a list of video progress records
    pub fn from_videos(course_id: Uuid, videos: &[VideoProgress]) -> Self {
        let total_videos = videos.len();
        let completed_videos = videos.iter().filter(|v| v.completed).count();
        let total_watch_time_seconds = videos.iter().map(|v| v.position_seconds).sum();
        let total_duration_seconds = videos.iter().map(|v| v.duration_seconds).sum();

        Self {
            course_id,
            total_videos,
            completed_videos,
            total_watch_time_seconds,
            total_duration_seconds,
        }
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_videos > 0 {
            (self.completed_videos as f64 / self.total_videos as f64 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    /// Check if course is completed
    pub fn is_completed(&self) -> bool {
        self.total_videos > 0 && self.completed_videos == self.total_videos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_progress() {
        let mut progress = VideoProgress::new(Uuid::new_v4(), 0, 600.0);
        assert_eq!(progress.progress_percentage(), 0.0);
        assert!(!progress.completed);

        progress.update_position(300.0);
        assert_eq!(progress.progress_percentage(), 50.0);
        assert!(!progress.completed);

        progress.update_position(550.0);
        assert!(progress.completed); // 90%+ = auto complete
    }

    #[test]
    fn test_course_progress() {
        let course_id = Uuid::new_v4();
        let videos = vec![
            VideoProgress {
                course_id,
                video_index: 0,
                position_seconds: 600.0,
                duration_seconds: 600.0,
                completed: true,
                updated_at: chrono::Utc::now(),
            },
            VideoProgress {
                course_id,
                video_index: 1,
                position_seconds: 300.0,
                duration_seconds: 600.0,
                completed: false,
                updated_at: chrono::Utc::now(),
            },
        ];

        let progress = CourseProgress::from_videos(course_id, &videos);
        assert_eq!(progress.total_videos, 2);
        assert_eq!(progress.completed_videos, 1);
        assert_eq!(progress.completion_percentage(), 50.0);
        assert!(!progress.is_completed());
    }
}
