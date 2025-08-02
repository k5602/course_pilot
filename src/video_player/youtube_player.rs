use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::video_player::{PlaybackState, VideoInfo, VideoPlayer, VideoSource};

/// YouTube embedded player implementation using IFrame Player API
pub struct YouTubeEmbeddedPlayer {
    player_id: String,
    current_info: Arc<Mutex<Option<VideoInfo>>>,
    is_fullscreen: Arc<Mutex<bool>>,
    state: Arc<Mutex<PlaybackState>>,
    current_position: Arc<Mutex<f64>>,
    duration: Arc<Mutex<f64>>,
    volume: Arc<Mutex<f64>>,
    is_api_ready: Arc<Mutex<bool>>,
}

impl YouTubeEmbeddedPlayer {
    /// Create a new YouTube embedded player instance
    pub fn new() -> Result<Self> {
        let player_id = format!("youtube-player-{}", Uuid::new_v4().simple());
        
        Ok(Self {
            player_id,
            current_info: Arc::new(Mutex::new(None)),
            is_fullscreen: Arc::new(Mutex::new(false)),
            state: Arc::new(Mutex::new(PlaybackState::Stopped)),
            current_position: Arc::new(Mutex::new(0.0)),
            duration: Arc::new(Mutex::new(0.0)),
            volume: Arc::new(Mutex::new(1.0)),
            is_api_ready: Arc::new(Mutex::new(false)),
        })
    }

    /// Get the unique player ID for this instance
    pub fn get_player_id(&self) -> &str {
        &self.player_id
    }

    /// Initialize the YouTube IFrame Player API
    pub fn initialize_api(&self) -> Result<()> {
        let script = r#"
            // Load YouTube IFrame Player API if not already loaded
            if (!window.YT) {
                var tag = document.createElement('script');
                tag.src = 'https://www.youtube.com/iframe_api';
                var firstScriptTag = document.getElementsByTagName('script')[0];
                firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
            }
        "#;

        // Execute the script to load the API
        self.execute_script(script)?;
        
        log::info!("YouTube IFrame Player API initialization started");
        Ok(())
    }

    /// Create the YouTube player iframe and initialize it
    pub fn create_player(&self, video_id: &str, playlist_id: Option<&str>) -> Result<()> {
        let playlist_param = if let Some(playlist) = playlist_id {
            format!(", list: '{playlist}'")
        } else {
            String::new()
        };

        let script = format!(r#"
            // Ensure the container exists
            if (!document.getElementById('{player_id}')) {{
                console.error('YouTube player container not found: {player_id}');
                return;
            }}

            // Function to create the player
            function createYouTubePlayer() {{
                if (window.YT && window.YT.Player) {{
                    window.ytPlayer_{player_id} = new YT.Player('{player_id}', {{
                        height: '100%',
                        width: '100%',
                        videoId: '{video_id}'{playlist_param},
                        playerVars: {{
                            'enablejsapi': 1,
                            'origin': window.location.origin,
                            'playsinline': 1,
                            'rel': 0,
                            'modestbranding': 1
                        }},
                        events: {{
                            'onReady': function(event) {{
                                console.log('YouTube player ready: {player_id}');
                                window.ytPlayerReady_{player_id} = true;
                            }},
                            'onStateChange': function(event) {{
                                console.log('YouTube player state changed: {player_id}', event.data);
                                window.ytPlayerState_{player_id} = event.data;
                            }},
                            'onError': function(event) {{
                                console.error('YouTube player error: {player_id}', event.data);
                                window.ytPlayerError_{player_id} = event.data;
                            }}
                        }}
                    }});
                }} else {{
                    // API not ready yet, try again in 100ms
                    setTimeout(createYouTubePlayer, 100);
                }}
            }}

            // Create the player
            createYouTubePlayer();
        "#, 
        player_id = self.player_id, 
        video_id = video_id,
        playlist_param = playlist_param
        );

        self.execute_script(&script)?;
        
        log::info!("YouTube player creation initiated for video: {video_id}");
        Ok(())
    }

    /// Execute JavaScript in the webview
    fn execute_script(&self, script: &str) -> Result<()> {
        // Note: In a real implementation, we would need access to the webview
        // This would typically be passed in during initialization or accessed via context
        // For now, we'll log the script that would be executed
        log::debug!("Executing JavaScript: {}", script);
        
        // TODO: In actual implementation with webview access:
        // window.webview.evaluate_script(script)?;
        
        Ok(())
    }

    /// Check if the YouTube player is ready
    pub fn is_player_ready(&self) -> Result<bool> {
        let _script = format!("window.ytPlayerReady_{} === true", self.player_id);
        
        // TODO: In actual implementation, this would evaluate the script and return the result
        // For now, we'll assume it's ready after a short delay
        Ok(true)
    }

    /// Get the current player state from JavaScript
    fn get_js_player_state(&self) -> Result<i32> {
        let _script = format!("window.ytPlayerState_{} || -1", self.player_id);
        
        // TODO: In actual implementation, this would evaluate the script and return the result
        // For now, we'll return the current state
        let state = self.state.lock().map_err(|_| anyhow!("Failed to lock state"))?;
        Ok(match *state {
            PlaybackState::Stopped => -1,
            PlaybackState::Playing => 1,
            PlaybackState::Paused => 2,
            PlaybackState::Buffering => 3,
            PlaybackState::Error => -1,
        })
    }

    /// Execute a YouTube player API method
    fn execute_player_method(&self, method: &str, args: &str) -> Result<()> {
        let script = format!(
            "if (window.ytPlayer_{} && window.ytPlayer_{}.{}) {{ window.ytPlayer_{}.{}({}); }}",
            self.player_id, self.player_id, method, self.player_id, method, args
        );
        
        self.execute_script(&script)?;
        Ok(())
    }

    /// Get a value from the YouTube player API
    fn get_player_value(&self, method: &str) -> Result<f64> {
        let _script = format!(
            "window.ytPlayer_{} && window.ytPlayer_{}.{} ? window.ytPlayer_{}.{}() : 0",
            self.player_id, self.player_id, method, self.player_id, method
        );
        
        // TODO: In actual implementation, this would evaluate the script and return the result
        // For now, we'll return cached values
        match method {
            "getCurrentTime" => {
                let position = self.current_position.lock().map_err(|_| anyhow!("Failed to lock position"))?;
                Ok(*position)
            }
            "getDuration" => {
                let duration = self.duration.lock().map_err(|_| anyhow!("Failed to lock duration"))?;
                Ok(*duration)
            }
            "getVolume" => {
                let volume = self.volume.lock().map_err(|_| anyhow!("Failed to lock volume"))?;
                Ok(*volume * 100.0) // YouTube API returns volume as 0-100
            }
            _ => Ok(0.0)
        }
    }

    /// Load a YouTube video by ID
    pub fn load_youtube_video(
        &mut self,
        video_id: String,
        playlist_id: Option<String>,
        title: String,
    ) -> Result<()> {
        // Initialize the API if not already done
        self.initialize_api()?;

        // Create video info
        let source = VideoSource::YouTube {
            video_id: video_id.clone(),
            playlist_id: playlist_id.clone(),
            title: title.clone(),
        };
        let video_info = VideoInfo::new(source);

        // Update current info
        {
            let mut info = self
                .current_info
                .lock()
                .map_err(|_| anyhow!("Failed to lock current info"))?;
            *info = Some(video_info);
        }

        // Create the player
        self.create_player(&video_id, playlist_id.as_deref())?;

        // Set initial state
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow!("Failed to lock state"))?;
            *state = PlaybackState::Stopped;
        }

        log::info!("Loaded YouTube video: {title} (ID: {video_id})");
        Ok(())
    }

    /// Load a new video in the existing player
    pub fn load_video_by_id(&self, video_id: &str, start_seconds: Option<f64>) -> Result<()> {
        let args = if let Some(start) = start_seconds {
            format!("'{}', {}", video_id, start)
        } else {
            format!("'{}'", video_id)
        };

        self.execute_player_method("loadVideoById", &args)?;
        
        // Update state
        {
            let mut state = self.state.lock().map_err(|_| anyhow!("Failed to lock state"))?;
            *state = PlaybackState::Buffering;
        }

        log::info!("Loading new video: {video_id}");
        Ok(())
    }

    /// Cue a video without playing it
    pub fn cue_video_by_id(&self, video_id: &str, start_seconds: Option<f64>) -> Result<()> {
        let args = if let Some(start) = start_seconds {
            format!("'{}', {}", video_id, start)
        } else {
            format!("'{}'", video_id)
        };

        self.execute_player_method("cueVideoById", &args)?;
        
        log::info!("Cued video: {video_id}");
        Ok(())
    }

    /// Navigate to next video in playlist
    pub fn next_video(&self) -> Result<()> {
        self.execute_player_method("nextVideo", "")?;
        log::info!("Navigating to next video in playlist");
        Ok(())
    }

    /// Navigate to previous video in playlist
    pub fn previous_video(&self) -> Result<()> {
        self.execute_player_method("previousVideo", "")?;
        log::info!("Navigating to previous video in playlist");
        Ok(())
    }

    /// Play video at specific index in playlist
    pub fn play_video_at(&self, index: usize) -> Result<()> {
        self.execute_player_method("playVideoAt", &index.to_string())?;
        log::info!("Playing video at index: {index}");
        Ok(())
    }

    /// Set playback rate
    pub fn set_playback_rate(&self, rate: f64) -> Result<()> {
        self.execute_player_method("setPlaybackRate", &rate.to_string())?;
        log::info!("Set playback rate to: {rate}");
        Ok(())
    }

    /// Get available playback rates
    pub fn get_available_playback_rates(&self) -> Result<Vec<f64>> {
        // TODO: In actual implementation, this would query the player
        // For now, return common YouTube playback rates
        Ok(vec![0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0])
    }

    /// Mute the player
    pub fn mute(&self) -> Result<()> {
        self.execute_player_method("mute", "")?;
        log::info!("Muted YouTube player");
        Ok(())
    }

    /// Unmute the player
    pub fn unmute(&self) -> Result<()> {
        self.execute_player_method("unMute", "")?;
        log::info!("Unmuted YouTube player");
        Ok(())
    }

    /// Check if player is muted
    pub fn is_muted(&self) -> Result<bool> {
        // TODO: In actual implementation, this would query the player
        let volume = self.volume.lock().map_err(|_| anyhow!("Failed to lock volume"))?;
        Ok(*volume == 0.0)
    }

    /// Destroy the player and clean up
    pub fn destroy(&self) -> Result<()> {
        let script = format!(
            r#"
            if (window.ytPlayer_{}) {{
                window.ytPlayer_{}.destroy();
                delete window.ytPlayer_{};
                delete window.ytPlayerReady_{};
                delete window.ytPlayerState_{};
                delete window.ytPlayerError_{};
            }}
            "#,
            self.player_id, self.player_id, self.player_id, 
            self.player_id, self.player_id, self.player_id
        );

        self.execute_script(&script)?;
        
        log::info!("Destroyed YouTube player: {}", self.player_id);
        Ok(())
    }

    /// Generate YouTube embed URL
    pub fn generate_embed_url(video_id: &str, playlist_id: Option<&str>) -> String {
        if let Some(playlist) = playlist_id {
            format!("https://www.youtube.com/embed/{video_id}?list={playlist}&enablejsapi=1")
        } else {
            format!("https://www.youtube.com/embed/{video_id}?enablejsapi=1")
        }
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
}

impl VideoPlayer for YouTubeEmbeddedPlayer {
    fn load_and_play(&mut self, source: VideoSource) -> Result<()> {
        match source {
            VideoSource::YouTube {
                video_id,
                playlist_id,
                title,
            } => {
                // Create YouTube URL
                let url = if let Some(playlist) = &playlist_id {
                    format!("https://www.youtube.com/watch?v={}&list={}", video_id, playlist)
                } else {
                    format!("https://www.youtube.com/watch?v={}", video_id)
                };

                // Open YouTube video in system browser
                self.open_youtube_url(&url)?;

                // Update state
                {
                    let mut state = self
                        .state
                        .lock()
                        .map_err(|_| anyhow!("Failed to lock state"))?;
                    *state = PlaybackState::Playing;
                }

                // Create video info
                let video_source = VideoSource::YouTube {
                    video_id: video_id.clone(),
                    playlist_id,
                    title: title.clone(),
                };
                let video_info = VideoInfo::new(video_source);

                // Update current info
                {
                    let mut info = self
                        .current_info
                        .lock()
                        .map_err(|_| anyhow!("Failed to lock current info"))?;
                    *info = Some(video_info);
                }

                log::info!("Opened YouTube video in browser: {} ({})", title, video_id);
                Ok(())
            }
            VideoSource::Local { .. } => Err(anyhow!(
                "Local videos not supported by YouTubeEmbeddedPlayer"
            )),
        }
    }

    fn pause(&mut self) -> Result<()> {
        self.execute_player_method("pauseVideo", "")?;

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow!("Failed to lock state"))?;
            *state = PlaybackState::Paused;
        }

        log::info!("YouTube player paused");
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.execute_player_method("playVideo", "")?;

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow!("Failed to lock state"))?;
            *state = PlaybackState::Playing;
        }

        log::info!("YouTube player resumed");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.execute_player_method("stopVideo", "")?;

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow!("Failed to lock state"))?;
            *state = PlaybackState::Stopped;
        }

        {
            let mut position = self
                .current_position
                .lock()
                .map_err(|_| anyhow!("Failed to lock position"))?;
            *position = 0.0;
        }

        log::info!("YouTube player stopped");
        Ok(())
    }

    fn seek(&mut self, position_seconds: f64) -> Result<()> {
        let clamped_position = position_seconds.max(0.0);
        self.execute_player_method("seekTo", &format!("{}, true", clamped_position))?;

        {
            let mut position = self
                .current_position
                .lock()
                .map_err(|_| anyhow!("Failed to lock position"))?;
            *position = clamped_position;
        }

        log::info!("YouTube player seek to {clamped_position} seconds");
        Ok(())
    }

    fn set_volume(&mut self, volume: f64) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        let youtube_volume = (clamped_volume * 100.0) as i32; // YouTube API expects 0-100
        
        self.execute_player_method("setVolume", &youtube_volume.to_string())?;

        {
            let mut vol = self
                .volume
                .lock()
                .map_err(|_| anyhow!("Failed to lock volume"))?;
            *vol = clamped_volume;
        }

        log::info!("YouTube player volume set to {clamped_volume}");
        Ok(())
    }

    fn get_position(&self) -> Result<f64> {
        // In a real implementation, this would query the player
        self.get_player_value("getCurrentTime")
    }

    fn get_duration(&self) -> Result<f64> {
        // In a real implementation, this would query the player
        self.get_player_value("getDuration")
    }

    fn is_playing(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Playing)
    }

    fn get_state(&self) -> PlaybackState {
        // In a real implementation, we'd sync this with the JavaScript player state
        self.state
            .lock()
            .map(|state| *state)
            .unwrap_or(PlaybackState::Stopped)
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        // YouTube IFrame API doesn't directly control fullscreen
        // This would typically be handled by the browser/webview
        let script = if fullscreen {
            format!(
                r#"
                var iframe = document.getElementById('{}');
                if (iframe && iframe.requestFullscreen) {{
                    iframe.requestFullscreen();
                }} else if (iframe && iframe.webkitRequestFullscreen) {{
                    iframe.webkitRequestFullscreen();
                }} else if (iframe && iframe.mozRequestFullScreen) {{
                    iframe.mozRequestFullScreen();
                }}
                "#,
                self.player_id
            )
        } else {
            r#"
            if (document.exitFullscreen) {
                document.exitFullscreen();
            } else if (document.webkitExitFullscreen) {
                document.webkitExitFullscreen();
            } else if (document.mozCancelFullScreen) {
                document.mozCancelFullScreen();
            }
            "#.to_string()
        };

        self.execute_script(&script)?;

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

        log::info!("YouTube player fullscreen mode: {fullscreen}");
        Ok(())
    }

    fn is_fullscreen(&self) -> bool {
        self.is_fullscreen.lock().map(|fs| *fs).unwrap_or(false)
    }
}

impl Default for YouTubeEmbeddedPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create YouTubeEmbeddedPlayer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_url_generation() {
        let url = YouTubeEmbeddedPlayer::generate_embed_url("dQw4w9WgXcQ", None);
        assert_eq!(
            url,
            "https://www.youtube.com/embed/dQw4w9WgXcQ?enablejsapi=1"
        );

        let url_with_playlist = YouTubeEmbeddedPlayer::generate_embed_url(
            "dQw4w9WgXcQ",
            Some("PLrAXtmRdnEQy6nuLMfO2GiN6h6b4kAacl"),
        );
        assert_eq!(
            url_with_playlist,
            "https://www.youtube.com/embed/dQw4w9WgXcQ?list=PLrAXtmRdnEQy6nuLMfO2GiN6h6b4kAacl&enablejsapi=1"
        );
    }
}
