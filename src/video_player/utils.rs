use std::time::Duration;

/// Utility functions for video player
pub struct VideoPlayerUtils;

impl VideoPlayerUtils {
    /// Format seconds as a human-readable time string (MM:SS or HH:MM:SS)
    pub fn format_seconds(seconds: f64) -> String {
        let total_seconds = seconds as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let secs = total_seconds % 60;

        if hours > 0 {
            format!("{hours:02}:{minutes:02}:{secs:02}")
        } else {
            format!("{minutes:02}:{secs:02}")
        }
    }

    /// Parse time string (MM:SS or HH:MM:SS) to seconds
    pub fn parse_time_string(time_str: &str) -> Result<f64, String> {
        let parts: Vec<&str> = time_str.split(':').collect();
        
        match parts.len() {
            2 => {
                // MM:SS format
                let minutes = parts[0].parse::<u64>().map_err(|_| "Invalid minutes")?;
                let seconds = parts[1].parse::<u64>().map_err(|_| "Invalid seconds")?;
                Ok((minutes * 60 + seconds) as f64)
            }
            3 => {
                // HH:MM:SS format
                let hours = parts[0].parse::<u64>().map_err(|_| "Invalid hours")?;
                let minutes = parts[1].parse::<u64>().map_err(|_| "Invalid minutes")?;
                let seconds = parts[2].parse::<u64>().map_err(|_| "Invalid seconds")?;
                Ok((hours * 3600 + minutes * 60 + seconds) as f64)
            }
            _ => Err("Invalid time format".to_string()),
        }
    }

    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Calculate video aspect ratio
    pub fn calculate_aspect_ratio(width: u32, height: u32) -> f64 {
        if height == 0 {
            16.0 / 9.0 // Default to 16:9
        } else {
            width as f64 / height as f64
        }
    }

    /// Get common aspect ratio name
    pub fn get_aspect_ratio_name(width: u32, height: u32) -> String {
        let ratio = Self::calculate_aspect_ratio(width, height);
        
        if (ratio - 16.0/9.0).abs() < 0.1 {
            "16:9".to_string()
        } else if (ratio - 4.0/3.0).abs() < 0.1 {
            "4:3".to_string()
        } else if (ratio - 21.0/9.0).abs() < 0.1 {
            "21:9".to_string()
        } else if (ratio - 1.0).abs() < 0.1 {
            "1:1".to_string()
        } else {
            format!("{:.2}:1", ratio)
        }
    }

    /// Validate YouTube video ID format
    pub fn is_valid_youtube_id(video_id: &str) -> bool {
        video_id.len() == 11 && 
        video_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') &&
        !video_id.starts_with("PLACEHOLDER_")
    }

    /// Extract YouTube video ID from URL
    pub fn extract_youtube_id(url: &str) -> Option<String> {
        // Handle various YouTube URL formats
        if let Some(captures) = regex::Regex::new(r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/)([a-zA-Z0-9_-]{11})")
            .ok()?
            .captures(url) 
        {
            captures.get(1).map(|m| m.as_str().to_string())
        } else {
            None
        }
    }

    /// Generate thumbnail URL for YouTube video
    pub fn get_youtube_thumbnail_url(video_id: &str, quality: ThumbnailQuality) -> String {
        let quality_str = match quality {
            ThumbnailQuality::Default => "default",
            ThumbnailQuality::Medium => "mqdefault",
            ThumbnailQuality::High => "hqdefault",
            ThumbnailQuality::Standard => "sddefault",
            ThumbnailQuality::MaxRes => "maxresdefault",
        };
        
        format!("https://img.youtube.com/vi/{}/{}.jpg", video_id, quality_str)
    }

    /// Check if file extension is supported
    pub fn is_supported_video_format(extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(), 
            "mp4" | "webm" | "ogg" | "avi" | "mov" | "mkv" | "m4v" | "3gp" | "flv" | "wmv"
        )
    }

    /// Get MIME type for video file
    pub fn get_video_mime_type(extension: &str) -> Option<&'static str> {
        match extension.to_lowercase().as_str() {
            "mp4" | "m4v" => Some("video/mp4"),
            "webm" => Some("video/webm"),
            "ogg" => Some("video/ogg"),
            "avi" => Some("video/x-msvideo"),
            "mov" => Some("video/quicktime"),
            "mkv" => Some("video/x-matroska"),
            "3gp" => Some("video/3gpp"),
            "flv" => Some("video/x-flv"),
            "wmv" => Some("video/x-ms-wmv"),
            _ => None,
        }
    }

    /// Calculate optimal buffer size for video streaming
    pub fn calculate_buffer_size(bitrate_kbps: u32, target_seconds: f64) -> u64 {
        // Convert kbps to bytes per second, then multiply by target seconds
        let bytes_per_second = (bitrate_kbps * 1024) / 8;
        (bytes_per_second as f64 * target_seconds) as u64
    }

    /// Estimate video file size
    pub fn estimate_file_size(duration_seconds: f64, bitrate_kbps: u32) -> u64 {
        let bytes_per_second = (bitrate_kbps * 1024) / 8;
        (duration_seconds * bytes_per_second as f64) as u64
    }

    /// Generate unique player ID
    pub fn generate_player_id() -> String {
        format!("cp-video-{}", uuid::Uuid::new_v4().simple())
    }

    /// Simple debounce implementation using tokio (for desktop use)
    pub fn debounce_async<F, Fut>(func: F, delay_ms: u64) -> impl Fn()
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let delay = std::sync::Arc::new(std::sync::Mutex::new(None::<tokio::task::JoinHandle<()>>));
        
        move || {
            // Cancel existing task
            if let Ok(mut guard) = delay.lock() {
                if let Some(handle) = guard.take() {
                    handle.abort();
                }
                
                // Start new task
                let handle = tokio::spawn({
                    let func = func();
                    async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        func.await;
                    }
                });
                
                *guard = Some(handle);
            }
        }
    }

    /// Simple throttle implementation using std::time
    pub fn throttle<F>(func: F, delay_ms: u64) -> impl Fn()
    where
        F: Fn() + Send + 'static,
    {
        let last_call = std::sync::Arc::new(std::sync::Mutex::new(None::<std::time::Instant>));
        
        move || {
            let now = std::time::Instant::now();
            let should_call = {
                if let Ok(mut last) = last_call.lock() {
                    match *last {
                        Some(last_time) => {
                            if now.duration_since(last_time).as_millis() >= delay_ms as u128 {
                                *last = Some(now);
                                true
                            } else {
                                false
                            }
                        }
                        None => {
                            *last = Some(now);
                            true
                        }
                    }
                } else {
                    false
                }
            };
            
            if should_call {
                func();
            }
        }
    }
}

/// YouTube thumbnail quality options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThumbnailQuality {
    Default,    // 120x90
    Medium,     // 320x180
    High,       // 480x360
    Standard,   // 640x480
    MaxRes,     // 1280x720
}

/// Video quality presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoQuality {
    Low,        // 480p
    Medium,     // 720p
    High,       // 1080p
    Ultra,      // 4K
}

impl VideoQuality {
    pub fn get_dimensions(&self) -> (u32, u32) {
        match self {
            VideoQuality::Low => (854, 480),
            VideoQuality::Medium => (1280, 720),
            VideoQuality::High => (1920, 1080),
            VideoQuality::Ultra => (3840, 2160),
        }
    }

    pub fn get_typical_bitrate_kbps(&self) -> u32 {
        match self {
            VideoQuality::Low => 1000,
            VideoQuality::Medium => 2500,
            VideoQuality::High => 5000,
            VideoQuality::Ultra => 15000,
        }
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    start_time: std::time::Instant,
    checkpoints: Vec<(String, std::time::Instant)>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            checkpoints: Vec::new(),
        }
    }

    pub fn checkpoint(&mut self, name: &str) {
        self.checkpoints.push((name.to_string(), std::time::Instant::now()));
    }

    pub fn get_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn get_checkpoint_durations(&self) -> Vec<(String, Duration)> {
        let mut results = Vec::new();
        let mut last_time = self.start_time;

        for (name, time) in &self.checkpoints {
            results.push((name.clone(), time.duration_since(last_time)));
            last_time = *time;
        }

        results
    }

    pub fn report(&self) -> String {
        let mut report = format!("Total elapsed: {:?}\n", self.get_elapsed());
        
        for (name, duration) in self.get_checkpoint_durations() {
            report.push_str(&format!("  {}: {:?}\n", name, duration));
        }
        
        report
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_seconds() {
        assert_eq!(VideoPlayerUtils::format_seconds(30.0), "00:30");
        assert_eq!(VideoPlayerUtils::format_seconds(90.0), "01:30");
        assert_eq!(VideoPlayerUtils::format_seconds(3661.0), "01:01:01");
        assert_eq!(VideoPlayerUtils::format_seconds(0.0), "00:00");
    }

    #[test]
    fn test_parse_time_string() {
        assert_eq!(VideoPlayerUtils::parse_time_string("01:30").unwrap(), 90.0);
        assert_eq!(VideoPlayerUtils::parse_time_string("01:01:01").unwrap(), 3661.0);
        assert_eq!(VideoPlayerUtils::parse_time_string("00:00").unwrap(), 0.0);
        assert!(VideoPlayerUtils::parse_time_string("invalid").is_err());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(VideoPlayerUtils::format_bytes(512), "512 B");
        assert_eq!(VideoPlayerUtils::format_bytes(1536), "1.5 KB");
        assert_eq!(VideoPlayerUtils::format_bytes(1048576), "1.0 MB");
        assert_eq!(VideoPlayerUtils::format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_aspect_ratio() {
        assert!((VideoPlayerUtils::calculate_aspect_ratio(1920, 1080) - 16.0/9.0).abs() < 0.01);
        assert_eq!(VideoPlayerUtils::get_aspect_ratio_name(1920, 1080), "16:9");
        assert_eq!(VideoPlayerUtils::get_aspect_ratio_name(1280, 960), "4:3");
        assert_eq!(VideoPlayerUtils::get_aspect_ratio_name(1920, 1920), "1:1");
    }

    #[test]
    fn test_youtube_id_validation() {
        assert!(VideoPlayerUtils::is_valid_youtube_id("dQw4w9WgXcQ"));
        assert!(!VideoPlayerUtils::is_valid_youtube_id("PLACEHOLDER_123"));
        assert!(!VideoPlayerUtils::is_valid_youtube_id(""));
        assert!(!VideoPlayerUtils::is_valid_youtube_id("too_short"));
        assert!(!VideoPlayerUtils::is_valid_youtube_id("way_too_long_id"));
    }

    #[test]
    fn test_video_format_support() {
        assert!(VideoPlayerUtils::is_supported_video_format("mp4"));
        assert!(VideoPlayerUtils::is_supported_video_format("MP4"));
        assert!(VideoPlayerUtils::is_supported_video_format("webm"));
        assert!(!VideoPlayerUtils::is_supported_video_format("txt"));
        assert!(!VideoPlayerUtils::is_supported_video_format("exe"));
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(VideoPlayerUtils::get_video_mime_type("mp4"), Some("video/mp4"));
        assert_eq!(VideoPlayerUtils::get_video_mime_type("webm"), Some("video/webm"));
        assert_eq!(VideoPlayerUtils::get_video_mime_type("unknown"), None);
    }

    #[test]
    fn test_file_size_estimation() {
        // 1 minute video at 1000 kbps should be about 7.5 MB
        let estimated = VideoPlayerUtils::estimate_file_size(60.0, 1000);
        assert!(estimated > 7_000_000 && estimated < 8_000_000);
    }

    #[test]
    fn test_video_quality() {
        let hd = VideoQuality::High;
        assert_eq!(hd.get_dimensions(), (1920, 1080));
        assert_eq!(hd.get_typical_bitrate_kbps(), 5000);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.checkpoint("test");
        
        let durations = monitor.get_checkpoint_durations();
        assert_eq!(durations.len(), 1);
        assert_eq!(durations[0].0, "test");
        assert!(durations[0].1.as_millis() >= 10);
    }

    #[test]
    fn test_player_id_generation() {
        let id1 = VideoPlayerUtils::generate_player_id();
        let id2 = VideoPlayerUtils::generate_player_id();
        
        assert_ne!(id1, id2);
        assert!(id1.starts_with("cp-video-"));
        assert!(id2.starts_with("cp-video-"));
    }
}