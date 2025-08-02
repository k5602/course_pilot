use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "ffmpeg")]
use ffmpeg_next as ffmpeg;

use crate::video_player::{PlaybackState, VideoInfo, VideoPlayer, VideoSource};

/// Local video player implementation using FFmpeg
pub struct LocalVideoPlayer {
    current_info: Arc<Mutex<Option<VideoInfo>>>,
    is_fullscreen: Arc<Mutex<bool>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    current_position: Arc<Mutex<f64>>,
    volume: Arc<Mutex<f64>>,
    playback_thread: Option<thread::JoinHandle<()>>,
}

impl LocalVideoPlayer {
    /// Create a new local video player instance
    pub fn new() -> Result<Self> {
        #[cfg(feature = "ffmpeg")]
        {
            // Initialize FFmpeg
            ffmpeg::init().map_err(|e| anyhow!("Failed to initialize FFmpeg: {}", e))?;
            log::info!("FFmpeg-based video player initialized");
        }

        #[cfg(not(feature = "ffmpeg"))]
        {
            log::warn!("Video player created without FFmpeg support - limited functionality");
        }

        let current_info = Arc::new(Mutex::new(None));
        let is_fullscreen = Arc::new(Mutex::new(false));
        let playback_state = Arc::new(Mutex::new(PlaybackState::Stopped));
        let current_position = Arc::new(Mutex::new(0.0));
        let volume = Arc::new(Mutex::new(1.0));

        let local_player = Self {
            current_info,
            is_fullscreen,
            playback_state,
            current_position,
            volume,
            playback_thread: None,
        };

        Ok(local_player)
    }

    /// Get video metadata using FFmpeg
    fn get_video_metadata<P: AsRef<Path>>(&self, _path: P) -> Result<(f64, i32, i32)> {
        #[cfg(feature = "ffmpeg")]
        {
            let path = path.as_ref();

            let mut input_context = ffmpeg::format::input(&path)
                .map_err(|e| anyhow!("Failed to open video file: {}", e))?;

            let video_stream = input_context
                .streams()
                .best(ffmpeg::media::Type::Video)
                .ok_or_else(|| anyhow!("No video stream found"))?;

            let duration_seconds = if video_stream.duration() >= 0 {
                video_stream.duration() as f64 * f64::from(video_stream.time_base())
            } else if input_context.duration() >= 0 {
                input_context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE)
            } else {
                0.0
            };

            let codec_params = video_stream.parameters();
            let width = codec_params.width();
            let height = codec_params.height();

            Ok((duration_seconds, width, height))
        }

        #[cfg(not(feature = "ffmpeg"))]
        {
            // Fallback when FFmpeg is not available
            log::warn!("FFmpeg not available, using placeholder metadata");
            Ok((60.0, 1920, 1080)) // Default values
        }
    }

    /// Start playback thread for FFmpeg-based video processing
    fn start_playback_thread(&mut self, path: std::path::PathBuf) -> Result<()> {
        let playback_state = Arc::clone(&self.playback_state);
        let current_position = Arc::clone(&self.current_position);
        let current_info = Arc::clone(&self.current_info);

        // Set state to playing
        {
            let mut state = playback_state
                .lock()
                .map_err(|_| anyhow!("Failed to lock playback state"))?;
            *state = PlaybackState::Playing;
        }

        let handle = thread::spawn(move || {
            if let Err(e) =
                Self::playback_loop(path, playback_state, current_position, current_info)
            {
                log::error!("Playback thread error: {e}");
            }
        });

        self.playback_thread = Some(handle);
        Ok(())
    }

    /// Main playback loop (simplified for demonstration)
    fn playback_loop(
        _path: std::path::PathBuf,
        playback_state: Arc<Mutex<PlaybackState>>,
        current_position: Arc<Mutex<f64>>,
        _current_info: Arc<Mutex<Option<VideoInfo>>>,
    ) -> Result<()> {
        let start_time = Instant::now();

        loop {
            // Check if we should continue playing
            {
                let state = playback_state
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock playback state"))?;
                match *state {
                    PlaybackState::Stopped => break,
                    PlaybackState::Paused => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    PlaybackState::Playing => {}
                    _ => {}
                }
            }

            // Update position (simplified - in real implementation, this would be based on actual frame timing)
            let elapsed = start_time.elapsed().as_secs_f64();
            {
                let mut position = current_position
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock current position"))?;
                *position = elapsed;
            }

            // Simulate frame processing delay
            thread::sleep(Duration::from_millis(33)); // ~30 FPS
        }

        Ok(())
    }

    /// Load a local video file
    pub fn load_video_file<P: AsRef<Path>>(&mut self, path: P, title: String) -> Result<()> {
        let path = path.as_ref();

        // Validate file exists
        if !path.exists() {
            return Err(anyhow!("Video file does not exist: {}", path.display()));
        }

        // Validate file extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("mp4") | Some("avi") | Some("mov") | Some("mkv") | Some("webm") => {}
            Some(ext) => log::warn!("Unsupported video format: {ext}"),
            None => return Err(anyhow!("File has no extension: {}", path.display())),
        }

        // Get video metadata using FFmpeg
        let (duration, width, height) = self.get_video_metadata(path)?;

        // Create video info
        let source = VideoSource::Local {
            path: path.to_path_buf(),
            title: title.clone(),
        };
        let mut video_info = VideoInfo::new(source);
        video_info.duration_seconds = Some(duration);

        // Update current info
        {
            let mut info = self
                .current_info
                .lock()
                .map_err(|_| anyhow!("Failed to lock current info"))?;
            *info = Some(video_info);
        }

        // Open the video file with the system's default video player
        self.open_with_system_player(path)?;

        // Start playback thread for state tracking
        self.start_playback_thread(path.to_path_buf())?;

        log::info!("Loaded video file: {title} ({width}x{height}, {duration:.2}s)");
        Ok(())
    }

    /// Open video file with the system's default video player
    fn open_with_system_player<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &path.to_string_lossy()])
                .spawn()
                .map_err(|e| anyhow!("Failed to open video with system player: {}", e))?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(path)
                .spawn()
                .map_err(|e| anyhow!("Failed to open video with system player: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(path)
                .spawn()
                .map_err(|e| anyhow!("Failed to open video with system player: {}", e))?;
        }

        log::info!("Opened video file with system player: {}", path.display());
        Ok(())
    }

    /// Open YouTube video URL with the system's default browser
    fn open_youtube_url(&self, url: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", url])
                .spawn()
                .map_err(|e| anyhow!("Failed to open YouTube URL with system browser: {}", e))?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .spawn()
                .map_err(|e| anyhow!("Failed to open YouTube URL with system browser: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(url)
                .spawn()
                .map_err(|e| anyhow!("Failed to open YouTube URL with system browser: {}", e))?;
        }

        log::info!("Opened YouTube URL with system browser: {}", url);
        Ok(())
    }

    /// Get supported video formats
    pub fn get_supported_formats() -> Vec<&'static str> {
        vec!["mp4", "avi", "mov", "mkv", "webm", "flv", "wmv", "m4v"]
    }

    /// Check if a file format is supported
    pub fn is_format_supported(extension: &str) -> bool {
        let ext = extension.to_lowercase();
        Self::get_supported_formats().contains(&ext.as_str())
    }
}

impl VideoPlayer for LocalVideoPlayer {
    fn load_and_play(&mut self, source: VideoSource) -> Result<()> {
        match source {
            VideoSource::Local { path, title } => {
                self.load_video_file(&path, title)?;
                Ok(())
            }
            VideoSource::YouTube { .. } => {
                Err(anyhow!("YouTube videos not supported by LocalVideoPlayer"))
            }
        }
    }

    fn pause(&mut self) -> Result<()> {
        let mut state = self
            .playback_state
            .lock()
            .map_err(|_| anyhow!("Failed to lock playback state"))?;

        if *state == PlaybackState::Playing {
            *state = PlaybackState::Paused;
            log::info!("Video paused");
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        let mut state = self
            .playback_state
            .lock()
            .map_err(|_| anyhow!("Failed to lock playback state"))?;

        if *state == PlaybackState::Paused {
            *state = PlaybackState::Playing;
            log::info!("Video resumed");
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        {
            let mut state = self
                .playback_state
                .lock()
                .map_err(|_| anyhow!("Failed to lock playback state"))?;
            *state = PlaybackState::Stopped;
        }

        {
            let mut position = self
                .current_position
                .lock()
                .map_err(|_| anyhow!("Failed to lock current position"))?;
            *position = 0.0;
        }

        log::info!("Video stopped");
        Ok(())
    }

    fn seek(&mut self, position_seconds: f64) -> Result<()> {
        let mut position = self
            .current_position
            .lock()
            .map_err(|_| anyhow!("Failed to lock current position"))?;
        *position = position_seconds.max(0.0);

        log::info!("Seeked to {position_seconds} seconds");
        Ok(())
    }

    fn set_volume(&mut self, volume: f64) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);

        {
            let mut vol = self
                .volume
                .lock()
                .map_err(|_| anyhow!("Failed to lock volume"))?;
            *vol = clamped_volume;
        }

        // Update video info
        {
            let mut info = self
                .current_info
                .lock()
                .map_err(|_| anyhow!("Failed to lock current info"))?;
            if let Some(ref mut video_info) = *info {
                video_info.volume = clamped_volume;
            }
        }

        log::info!("Volume set to {clamped_volume}");
        Ok(())
    }

    fn get_position(&self) -> Result<f64> {
        let position = self
            .current_position
            .lock()
            .map_err(|_| anyhow!("Failed to lock current position"))?;
        Ok(*position)
    }

    fn get_duration(&self) -> Result<f64> {
        let info = self
            .current_info
            .lock()
            .map_err(|_| anyhow!("Failed to lock current info"))?;

        if let Some(ref video_info) = *info {
            Ok(video_info.duration_seconds.unwrap_or(0.0))
        } else {
            Ok(0.0)
        }
    }

    fn is_playing(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Playing)
    }

    fn get_state(&self) -> PlaybackState {
        self.playback_state
            .lock()
            .map(|state| *state)
            .unwrap_or(PlaybackState::Stopped)
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        // Store fullscreen state
        {
            let mut is_fullscreen = self
                .is_fullscreen
                .lock()
                .map_err(|_| anyhow!("Failed to lock fullscreen state"))?;
            *is_fullscreen = fullscreen;
        }

        // Update video info
        {
            let mut info = self
                .current_info
                .lock()
                .map_err(|_| anyhow!("Failed to lock current info"))?;
            if let Some(ref mut video_info) = *info {
                video_info.is_fullscreen = fullscreen;
            }
        }

        log::info!("Fullscreen mode: {fullscreen}");
        Ok(())
    }

    fn is_fullscreen(&self) -> bool {
        self.is_fullscreen.lock().map(|fs| *fs).unwrap_or(false)
    }
}

impl Default for LocalVideoPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create LocalVideoPlayer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_support() {
        assert!(LocalVideoPlayer::is_format_supported("mp4"));
        assert!(LocalVideoPlayer::is_format_supported("MP4"));
        assert!(LocalVideoPlayer::is_format_supported("avi"));
        assert!(LocalVideoPlayer::is_format_supported("mov"));
        assert!(!LocalVideoPlayer::is_format_supported("txt"));
    }

    #[test]
    fn test_supported_formats() {
        let formats = LocalVideoPlayer::get_supported_formats();
        assert!(formats.contains(&"mp4"));
        assert!(formats.contains(&"avi"));
        assert!(formats.contains(&"mov"));
    }
}
