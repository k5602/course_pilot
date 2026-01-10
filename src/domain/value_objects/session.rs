//! Session planning value objects.

use std::time::Duration;

/// User-defined cognitive limit for session planning.
/// Represents the maximum content duration per day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CognitiveLimit {
    minutes_per_day: u32,
}

impl CognitiveLimit {
    /// Creates a new cognitive limit.
    /// Defaults to 45 minutes if value is 0.
    pub fn new(minutes_per_day: u32) -> Self {
        Self { minutes_per_day: if minutes_per_day == 0 { 45 } else { minutes_per_day } }
    }

    /// Returns the limit in minutes.
    pub fn minutes(&self) -> u32 {
        self.minutes_per_day
    }

    /// Returns the limit as a Duration.
    pub fn as_duration(&self) -> Duration {
        Duration::from_secs(self.minutes_per_day as u64 * 60)
    }

    /// Returns the limit in seconds.
    pub fn seconds(&self) -> u32 {
        self.minutes_per_day * 60
    }
}

impl Default for CognitiveLimit {
    fn default() -> Self {
        Self::new(45) // Default 45 minutes per day
    }
}

/// A planned session containing videos to watch.
#[derive(Debug, Clone)]
pub struct SessionPlan {
    /// Day number (1-indexed)
    pub day: u32,
    /// Video indices in this session
    pub video_indices: Vec<usize>,
    /// Total duration of this session in seconds
    pub total_duration_secs: u32,
}

impl SessionPlan {
    pub fn new(day: u32, video_indices: Vec<usize>, total_duration_secs: u32) -> Self {
        Self { day, video_indices, total_duration_secs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cognitive_limit() {
        let limit = CognitiveLimit::default();
        assert_eq!(limit.minutes(), 45);
    }

    #[test]
    fn test_custom_cognitive_limit() {
        let limit = CognitiveLimit::new(60);
        assert_eq!(limit.minutes(), 60);
        assert_eq!(limit.seconds(), 3600);
    }
}
