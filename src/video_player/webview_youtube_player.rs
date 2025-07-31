use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::video_player::{PlaybackState, VideoInfo, VideoPlayer, VideoSource};

/// YouTube embedded player that integrates with Dioxus webview
pub struct WebViewYouTubePlayer {
    player_id: String,
    current_info: Arc<Mutex<Option<VideoInfo>>>,
    is_fullscreen: Arc<Mutex<bool>>,
    state: Arc<Mutex<PlaybackState>>,
    current_position: Arc<Mutex<f64>>,
    duration: Arc<Mutex<f64>>,
    volume: Arc<Mutex<f64>>,
    is_api_ready: Arc<Mutex<bool>>,
}

impl WebViewYouTubePlayer {
    /// Create a new webview-integrated YouTube player instance
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

    /// Execute JavaScript using the webview (placeholder - needs webview context)
    pub fn execute_script_with_webview(&self, script: &str) -> Result<()> {
        // Note: This would need to be called from within a component context
        // where use_window() is available. For now, this is a placeholder.
        log::debug!("Would execute JavaScript: {}", script);
        
        // TODO: In actual implementation, this would need webview access:
        // let window = use_window();
        // window.webview.evaluate_script(script)?;
        
        Ok(())
    }

    /// Initialize the YouTube IFrame Player API
    pub fn initialize_api(&self) -> Result<()> {
        let script = r#"
            // Load YouTube IFrame Player API if not already loaded
            if (!window.YT) {
                console.log('Loading YouTube IFrame Player API...');
                var tag = document.createElement('script');
                tag.src = 'https://www.youtube.com/iframe_api';
                var firstScriptTag = document.getElementsByTagName('script')[0];
                firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
                
                // Set up global callback
                window.onYouTubeIframeAPIReady = function() {
                    console.log('YouTube IFrame Player API ready');
                    window.ytAPIReady = true;
                };
            } else {
                console.log('YouTube IFrame Player API already loaded');
                window.ytAPIReady = true;
            }
        "#;

        self.execute_script_with_webview(script)?;
        
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
                    console.log('Creating YouTube player for video: {video_id}');
                    
                    window.ytPlayer_{player_id} = new YT.Player('{player_id}', {{
                        height: '100%',
                        width: '100%',
                        videoId: '{video_id}'{playlist_param},
                        playerVars: {{
                            'enablejsapi': 1,
                            'origin': window.location.origin,
                            'playsinline': 1,
                            'rel': 0,
                            'modestbranding': 1,
                            'controls': 0,
                            'disablekb': 1,
                            'fs': 0,
                            'iv_load_policy': 3
                        }},
                        events: {{
                            'onReady': function(event) {{
                                console.log('YouTube player ready: {player_id}');
                                window.ytPlayerReady_{player_id} = true;
                                window.ytPlayerInstance_{player_id} = event.target;
                            }},
                            'onStateChange': function(event) {{
                                console.log('YouTube player state changed: {player_id}', event.data);
                                window.ytPlayerState_{player_id} = event.data;
                                
                                // Update position and duration
                                if (event.target) {{
                                    window.ytPlayerPosition_{player_id} = event.target.getCurrentTime() || 0;
                                    window.ytPlayerDuration_{player_id} = event.target.getDuration() || 0;
                                }}
                            }},
                            'onError': function(event) {{
                                console.error('YouTube player error: {player_id}', event.data);
                                window.ytPlayerError_{player_id} = event.data;
                            }}
                        }}
                    }});
                }} else {{
                    console.log('YouTube API not ready yet, retrying...');
                    setTimeout(createYouTubePlayer, 100);
                }}
            }}

            // Wait for API to be ready, then create the player
            if (window.ytAPIReady) {{
                createYouTubePlayer();
            }} else {{
                var checkAPI = setInterval(function() {{
                    if (window.ytAPIReady) {{
                        clearInterval(checkAPI);
                        createYouTubePlayer();
                    }}
                }}, 100);
            }}
        "#, 
        player_id = self.player_id, 
        video_id = video_id,
        playlist_param = playlist_param
        );

        self.execute_script_with_webview(&script)?;
        
        log::info!("YouTube player creation initiated for video: {video_id}");
        Ok(())
    }

    /// Execute a YouTube player API method
    pub fn execute_player_method(&self, method: &str, args: &str) -> Result<()> {
        let script = format!(
            r#"
            if (window.ytPlayerInstance_{} && window.ytPlayerInstance_{}.{}) {{
                try {{
                    window.ytPlayerInstance_{}.{}({});
                    console.log('Executed YouTube player method: {}({})');
                }} catch (e) {{
                    console.error('Error executing YouTube player method {}:', e);
                }}
            }} else {{
                console.warn('YouTube player not ready for method: {}');
            }}
            "#,
            self.player_id, self.player_id, method, self.player_id, method, args, method, args, method, method
        );
        
        self.execute_script_with_webview(&script)?;
        Ok(())
    }

    /// Get a value from the YouTube player API
    pub fn get_player_value(&self, method: &str) -> Result<f64> {
        let script = format!(
            r#"
            if (window.ytPlayerInstance_{} && window.ytPlayerInstance_{}.{}) {{
                try {{
                    var value = window.ytPlayerInstance_{}.{}();
                    window.ytPlayerValue_{}_{} = value;
                    console.log('Got YouTube player value {}:', value);
                }} catch (e) {{
                    console.error('Error getting YouTube player value {}:', e);
                    window.ytPlayerValue_{}_{} = 0;
                }}
            }} else {{
                window.ytPlayerValue_{}_{} = 0;
            }}
            "#,
            self.player_id, self.player_id, method, self.player_id, method, 
            self.player_id, method, method, method, self.player_id, method,
            self.player_id, method
        );
        
        self.execute_script_with_webview(&script)?;
        
        // TODO: In a real implementation, we would need to get the value back from JavaScript
        // For now, return cached values
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

    /// Destroy the player and clean up
    pub fn destroy(&self) -> Result<()> {
        let script = format!(
            r#"
            if (window.ytPlayerInstance_{}) {{
                try {{
                    window.ytPlayerInstance_{}.destroy();
                    console.log('Destroyed YouTube player: {}');
                }} catch (e) {{
                    console.error('Error destroying YouTube player:', e);
                }}
                
                // Clean up global variables
                delete window.ytPlayerInstance_{};
                delete window.ytPlayerReady_{};
                delete window.ytPlayerState_{};
                delete window.ytPlayerError_{};
                delete window.ytPlayerPosition_{};
                delete window.ytPlayerDuration_{};
            }}
            "#,
            self.player_id, self.player_id, self.player_id,
            self.player_id, self.player_id, self.player_id, 
            self.player_id, self.player_id, self.player_id
        );

        self.execute_script_with_webview(&script)?;
        
        log::info!("Destroyed YouTube player: {}", self.player_id);
        Ok(())
    }
}

impl VideoPlayer for WebViewYouTubePlayer {
    fn load_and_play(&mut self, source: VideoSource) -> Result<()> {
        match source {
            VideoSource::YouTube {
                video_id,
                playlist_id,
                title,
            } => {
                // Load the video
                self.load_youtube_video(video_id.clone(), playlist_id, title)?;

                // Wait a moment for the player to initialize, then play
                // In a real implementation, we'd wait for the onReady event
                std::thread::sleep(std::time::Duration::from_millis(1000));
                
                // Start playback
                self.execute_player_method("playVideo", "")?;

                // Update state
                {
                    let mut state = self
                        .state
                        .lock()
                        .map_err(|_| anyhow!("Failed to lock state"))?;
                    *state = PlaybackState::Playing;
                }

                log::info!("Started playing YouTube video: {video_id}");
                Ok(())
            }
            VideoSource::Local { .. } => Err(anyhow!(
                "Local videos not supported by WebViewYouTubePlayer"
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
        self.get_player_value("getCurrentTime")
    }

    fn get_duration(&self) -> Result<f64> {
        self.get_player_value("getDuration")
    }

    fn is_playing(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Playing)
    }

    fn get_state(&self) -> PlaybackState {
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

        self.execute_script_with_webview(&script)?;

        {
            let mut is_fullscreen = self
                .is_fullscreen
                .lock()
                .map_err(|_| anyhow!("Failed to lock fullscreen state"))?;
            *is_fullscreen = fullscreen;
        }

        log::info!("YouTube player fullscreen mode: {fullscreen}");
        Ok(())
    }

    fn is_fullscreen(&self) -> bool {
        self.is_fullscreen.lock().map(|fs| *fs).unwrap_or(false)
    }
}

impl Default for WebViewYouTubePlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create WebViewYouTubePlayer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = WebViewYouTubePlayer::new().unwrap();
        assert!(!player.get_player_id().is_empty());
        assert_eq!(player.get_state(), PlaybackState::Stopped);
    }
}