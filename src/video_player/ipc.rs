/*!
IPC helpers for the video player (Phase 3)

This module centralizes JavaScript snippets and small utilities to control and
synchronize both Local HTML5 <video> players and YouTube IFrame API players
inside the Dioxus Desktop webview.

Goals:
- Single source of truth for JS strings used by the desktop webview bridge.
- Avoid copy/paste of ad-hoc JS in components and hooks.
- Keep the Rust side strongly-typed and consistent with existing state contracts.

Usage:
- Build a JS snippet (e.g., `local::play_pause("cp-video-123")`) and evaluate it
  with the desktop webview.
- Optionally, use the `JsExecutor` trait to run scripts through a DesktopContext.

Conventions used by the YouTube player side:
- Player instance/global state variables mirror the current YouTube component:
  window.ytPlayer_<id>              -> raw YT.Player(...)
  window.ytPlayerInstance_<id>      -> event.target (YT player instance)
  window.ytPlayerReady_<id>         -> bool
  window.ytPlayerState_<id>         -> number
  window.ytPlayerPosition_<id>      -> seconds
  window.ytPlayerDuration_<id>      -> seconds
  window.ytPlayerError_<id>         -> error payload
*/

use crate::video_player::VideoPlayerError;

/// Escape a string to be safely embedded inside single-quoted JS string literals.
fn escape_js_str(input: &str) -> String {
    input.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Helpers for the global control/keyboard handler used by the player.
pub mod global {
    /// Script to initialize a global keyboard handler that stores last key in `window.lastVideoPlayerKey`.
    ///
    /// - Prevents default for known shortcuts when focus is not on inputs/contentEditable.
    /// - Uses capture phase to intercept early (mirrors existing behavior).
    pub fn attach_keyboard_handler() -> String {
        r#"
        (function() {
            if (!window.videoPlayerKeyboardHandler) {
                window.videoPlayerKeyboardHandler = function(event) {
                    const key = event.key;
                    const shortcuts = [
                        ' ', 'k', 'j', 'l', 'ArrowLeft', 'ArrowRight',
                        'ArrowUp', 'ArrowDown', 'm', 'f', 'Escape',
                        '0','1','2','3','4','5','6','7','8','9'
                    ];

                    if (shortcuts.includes(key)) {
                        const ae = document.activeElement;
                        const isInputFocused = ae && (
                            ae.tagName === 'INPUT' ||
                            ae.tagName === 'TEXTAREA' ||
                            ae.isContentEditable === true ||
                            ae.getAttribute('contenteditable') === 'true'
                        );

                        if (!isInputFocused) {
                            event.preventDefault();
                            event.stopPropagation();
                            window.lastVideoPlayerKey = key;
                            return false;
                        }
                    }
                };
                document.addEventListener('keydown', window.videoPlayerKeyboardHandler, true);
            }
        })();
        "#
        .to_string()
    }

    /// Returns a small script that pops and returns the last stored key as a string OR null.
    pub fn pop_last_keyboard_key() -> String {
        r#"
        (function() {
            if (window.lastVideoPlayerKey) {
                const k = window.lastVideoPlayerKey;
                window.lastVideoPlayerKey = null;
                return k;
            }
            return null;
        })()
        "#
        .to_string()
    }

    /// Initializes a per-player state blob under `window.cpVideoState[<player_id>]`.
    pub fn init_cp_state(player_id: &str) -> String {
        let id = super::escape_js_str(player_id);
        format!(
            r#"
            (function() {{
                if (!window.cpVideoState) window.cpVideoState = {{}};
                if (!window.cpVideoState['{id}']) {{
                    window.cpVideoState['{id}'] = {{
                        currentTime: 0,
                        duration: 0,
                        volume: 1.0,
                        muted: false,
                        paused: true
                    }};
                }}
            }})()
            "#
        )
    }

    /// Set the active player context ({ kind: 'local' | 'youtube', id: string })
    pub fn set_active_player(kind: &str, player_id: &str) -> String {
        let k = super::escape_js_str(kind);
        let id = super::escape_js_str(player_id);
        format!(
            r#"
            (function() {{
                window.cpActivePlayer = {{ kind: '{k}', id: '{id}' }};
            }})()
            "#
        )
    }

    /// Install a keyboard action dispatcher at window.cpHandleVideoKey(key)
    /// Returns a script that defines the function if not already present.
    pub fn keyboard_action_handler() -> String {
        r#"
        (function() {
            if (!window.cpHandleVideoKey) {
                function clamp(v, min, max) { return Math.max(min, Math.min(max, v)); }
                function getLocalEl(id) { return document.getElementById(id); }
                function getYt(id) { return window['ytPlayerInstance_' + id] || null; }

                window.cpHandleVideoKey = function(key) {
                    const ap = window.cpActivePlayer;
                    if (!ap || !ap.kind || !ap.id) return false;

                    const k = (key || '').toString();
                    const kind = ap.kind;

                    // SPACE and 'k' => toggle play/pause
                    if (k === ' ' || k.toLowerCase() === 'k') {
                        if (kind === 'local') {
                            const el = getLocalEl(ap.id);
                            if (!el) return false;
                            if (el.paused) { el.play().catch(()=>{}); } else { el.pause(); }
                            return true;
                        } else if (kind === 'youtube') {
                            const p = getYt(ap.id);
                            if (!p) return false;
                            try {
                                const state = typeof p.getPlayerState === 'function' ? p.getPlayerState() : 2;
                                if (state === 1) { p.pauseVideo && p.pauseVideo(); }
                                else { p.playVideo && p.playVideo(); }
                            } catch(_) {}
                            return true;
                        }
                    }

                    // Seek relative (J/L => 10s, Arrows => 5s)
                    if (k.toLowerCase() === 'j' || k === 'ArrowLeft') {
                        const delta = (k.toLowerCase() === 'j') ? -10 : -5;
                        if (kind === 'local') {
                            const el = getLocalEl(ap.id);
                            if (!el) return false;
                            try { el.currentTime = clamp((el.currentTime||0) + delta, 0, el.duration||1e9); } catch(_){}
                            return true;
                        } else if (kind === 'youtube') {
                            const p = getYt(ap.id);
                            if (!p) return false;
                            try {
                                const cur = (p.getCurrentTime && p.getCurrentTime()) || 0;
                                const dur = (p.getDuration && p.getDuration()) || 1e9;
                                p.seekTo && p.seekTo(clamp(cur + delta, 0, dur), true);
                            } catch(_){}
                            return true;
                        }
                    }

                    if (k.toLowerCase() === 'l' || k === 'ArrowRight') {
                        const delta = (k.toLowerCase() === 'l') ? 10 : 5;
                        if (kind === 'local') {
                            const el = getLocalEl(ap.id);
                            if (!el) return false;
                            try { el.currentTime = clamp((el.currentTime||0) + delta, 0, el.duration||1e9); } catch(_){}
                            return true;
                        } else if (kind === 'youtube') {
                            const p = getYt(ap.id);
                            if (!p) return false;
                            try {
                                const cur = (p.getCurrentTime && p.getCurrentTime()) || 0;
                                const dur = (p.getDuration && p.getDuration()) || 1e9;
                                p.seekTo && p.seekTo(clamp(cur + delta, 0, dur), true);
                            } catch(_){}
                            return true;
                        }
                    }

                    // Volume Up/Down (0..1 for local; 0..100 for YT)
                    if (k === 'ArrowUp' || k === 'ArrowDown') {
                        const up = (k === 'ArrowUp');
                        if (kind === 'local') {
                            const el = getLocalEl(ap.id);
                            if (!el) return false;
                            let v = typeof el.volume === 'number' ? el.volume : 1.0;
                            v = clamp(v + (up ? 0.1 : -0.1), 0, 1);
                            el.volume = v;
                            el.muted = (v <= 0);
                            return true;
                        } else if (kind === 'youtube') {
                            const p = getYt(ap.id);
                            if (!p) return false;
                            try {
                                let v = (p.getVolume && p.getVolume()) || 100;
                                v = clamp(v + (up ? 10 : -10), 0, 100);
                                p.setVolume && p.setVolume(v);
                                if (v <= 0 && p.mute) p.mute();
                                if (v > 0 && p.isMuted && p.isMuted() && p.unMute) p.unMute();
                            } catch(_){}
                            return true;
                        }
                    }

                    // Toggle mute (M)
                    if (k.toLowerCase() === 'm') {
                        if (kind === 'local') {
                            const el = getLocalEl(ap.id);
                            if (!el) return false;
                            el.muted = !el.muted;
                            return true;
                        } else if (kind === 'youtube') {
                            const p = getYt(ap.id);
                            if (!p) return false;
                            try { if (p.isMuted && p.isMuted()) p.unMute && p.unMute(); else p.mute && p.mute(); } catch(_){}
                            return true;
                        }
                    }

                    // Digit seek (0..9 => 0%..90%)
                    if (/^[0-9]$/.test(k)) {
                        const digit = parseInt(k, 10);
                        const pct = digit / 10.0;
                        if (kind === 'local') {
                            const el = getLocalEl(ap.id);
                            if (!el) return false;
                            const dur = (typeof el.duration === 'number' && isFinite(el.duration)) ? el.duration : 0;
                            if (dur > 0) { el.currentTime = clamp(dur * pct, 0, dur); }
                            return true;
                        } else if (kind === 'youtube') {
                            const p = getYt(ap.id);
                            if (!p) return false;
                            try {
                                const dur = (p.getDuration && p.getDuration()) || 0;
                                if (dur > 0) { p.seekTo && p.seekTo(clamp(dur * pct, 0, dur), true); }
                            } catch(_){}
                            return true;
                        }
                    }

                    // Fullscreen (F) and Escape are handled natively by the Rust side
                    if (k.toLowerCase() === 'f' || k === 'Escape') {
                        return false;
                    }

                    return false;
                };
            }
        })();
        "#
        .to_string()
    }
}

/// HTML5 <video> control and sync helpers (Local video).
pub mod local {
    use super::escape_js_str;

    /// Update the global cpVideoState for this player from the DOM video element.
    /// Returns JSON.stringify(state) or null if element not found.
    pub fn sync_state(player_id: &str) -> String {
        let id = escape_js_str(player_id);
        format!(
            r#"
            (function() {{
                if (!window.cpVideoState) window.cpVideoState = {{}};
                if (!window.cpVideoState['{id}']) {{
                    window.cpVideoState['{id}'] = {{ currentTime: 0, duration: 0, volume: 1.0, muted: false, paused: true }};
                }}
                const st = window.cpVideoState['{id}'];
                const el = document.getElementById('{id}');
                if (!el) return null;

                st.currentTime = Number(el.currentTime || 0);
                st.duration = Number(isFinite(el.duration) ? (el.duration || 0) : 0);
                st.volume = (typeof el.volume === 'number') ? el.volume : 1.0;
                st.muted = !!el.muted;
                st.paused = !!el.paused;

                return JSON.stringify(st);
            }})()
            "#
        )
    }

    /// Play the video element.
    pub fn play(player_id: &str) -> String {
        let id = escape_js_str(player_id);
        format!(
            r#"(function() {{
                const el = document.getElementById('{id}');
                if (el && el.paused) {{
                    el.play().catch(e => console.error('Play failed:', e));
                }}
            }})()"#
        )
    }

    /// Pause the video element.
    pub fn pause(player_id: &str) -> String {
        let id = escape_js_str(player_id);
        format!(
            r#"(function() {{
                const el = document.getElementById('{id}');
                if (el && !el.paused) {{
                    el.pause();
                }}
            }})()"#
        )
    }

    /// Toggle play/pause for the video element.
    pub fn play_pause(player_id: &str) -> String {
        let id = escape_js_str(player_id);
        format!(
            r#"(function() {{
                const el = document.getElementById('{id}');
                if (!el) return;
                if (el.paused) {{
                    el.play().catch(e => console.error('Play failed:', e));
                }} else {{
                    el.pause();
                }}
            }})()"#
        )
    }

    /// Seek to an absolute time (seconds).
    pub fn seek_to(player_id: &str, position_seconds: f64) -> String {
        let id = escape_js_str(player_id);
        let pos = position_seconds.max(0.0);
        format!(
            r#"(function() {{
                const el = document.getElementById('{id}');
                if (!el) return;
                try {{ el.currentTime = {pos}; }} catch(_e) {{}}
            }})()"#
        )
    }

    /// Seek relative to the current time (seconds).
    pub fn seek_relative(player_id: &str, delta_seconds: f64) -> String {
        let id = escape_js_str(player_id);
        let d = delta_seconds;
        format!(
            r#"(function() {{
                const el = document.getElementById('{id}');
                if (!el) return;
                try {{
                    const dur = (typeof el.duration === 'number' && isFinite(el.duration)) ? el.duration : 1e9;
                    el.currentTime = Math.max(0, Math.min(dur, (el.currentTime || 0) + ({d})));
                }} catch(_e) {{}}
            }})()"#
        )
    }

    /// Set volume (0.0..1.0) and muted flag when <= 0.
    pub fn set_volume(player_id: &str, volume: f64) -> String {
        let id = escape_js_str(player_id);
        let v = volume.clamp(0.0, 1.0);
        let muted = (v <= 0.0) as i32;
        format!(
            r#"(function() {{
                const el = document.getElementById('{id}');
                if (!el) return;
                el.volume = {v};
                el.muted = !!({muted});
            }})()"#
        )
    }

    /// Explicitly set muted flag.
    pub fn set_muted(player_id: &str, muted: bool) -> String {
        let id = escape_js_str(player_id);
        let m = if muted { "true" } else { "false" };
        format!(
            r#"(function() {{
                const el = document.getElementById('{id}');
                if (!el) return;
                el.muted = {m};
            }})()"#
        )
    }
}

/// YouTube IFrame API helpers.
pub mod youtube {
    use super::escape_js_str;

    /// Load the YouTube IFrame API if not already loaded. Sets `window.ytAPIReady = true` on ready.
    pub fn load_api() -> String {
        r#"
        (function() {
            if (!window.YT) {
                const tag = document.createElement('script');
                tag.src = 'https://www.youtube.com/iframe_api';
                const firstScriptTag = document.getElementsByTagName('script')[0];
                if (firstScriptTag && firstScriptTag.parentNode) {
                    firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
                } else {
                    document.head.appendChild(tag);
                }
                window.onYouTubeIframeAPIReady = function() {
                    window.ytAPIReady = true;
                };
            } else {
                window.ytAPIReady = true;
            }
        })();
        "#
        .to_string()
    }

    /// Create a YouTube player inside the container with `player_div_id`, and optionally bind a playlist.
    /// Mirrors the current component's global variable conventions for state propagation.
    pub fn create_player(player_div_id: &str, video_id: &str, playlist_id: Option<&str>) -> String {
        let pid = escape_js_str(player_div_id);
        let vid = escape_js_str(video_id);
        let plist = playlist_id.map(escape_js_str);

        let playlist_param = plist
            .map(|p| format!(", list: '{}'", p))
            .unwrap_or_default();

        format!(
            r#"
            (function() {{
                function _create() {{
                    if (window.YT && window.YT.Player) {{
                        try {{
                            window.ytPlayer_{pid} = new YT.Player('{pid}', {{
                                height: '100%',
                                width: '100%',
                                videoId: '{vid}'{playlist_param},
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
                                        window.ytPlayerReady_{pid} = true;
                                        window.ytPlayerInstance_{pid} = event.target;
                                    }},
                                    'onStateChange': function(event) {{
                                        window.ytPlayerState_{pid} = event.data;
                                        if (event && event.target) {{
                                            try {{
                                                window.ytPlayerPosition_{pid} = Number(event.target.getCurrentTime() || 0);
                                                window.ytPlayerDuration_{pid} = Number(event.target.getDuration() || 0);
                                            }} catch(_e) {{}}
                                        }}
                                    }},
                                    'onError': function(event) {{
                                        window.ytPlayerError_{pid} = event && event.data ? String(event.data) : 'unknown';
                                        console.error('YouTube error {pid}:', event);
                                    }}
                                }}
                            }});
                        }} catch (e) {{
                            window.ytPlayerError_{pid} = 'Player creation failed: ' + e;
                            console.error('YouTube player creation failed ({pid}):', e);
                        }}
                    }} else {{
                        setTimeout(_create, 100);
                    }}
                }}
                if (window.ytAPIReady) {{
                    _create();
                }} else {{
                    var _chk = setInterval(function() {{
                        if (window.ytAPIReady) {{
                            clearInterval(_chk);
                            _create();
                        }}
                    }}, 100);
                    setTimeout(function() {{
                        if (!window.ytAPIReady) {{
                            clearInterval(_chk);
                            window.ytPlayerError_{pid} = 'YouTube API load timeout';
                            console.error('YouTube API failed to load in time for {pid}');
                        }}
                    }}, 10000);
                }}
            }})();
            "#
        )
    }

    /// Get a JSON string with current YouTube state for the given player id, or null.
    pub fn get_state_json(player_div_id: &str) -> String {
        let pid = escape_js_str(player_div_id);
        format!(
            r#"
            (function() {{
                var inst = window.ytPlayerInstance_{pid};
                if (!inst) return null;
                var state = {{
                    current_time: 0.0,
                    duration: 0.0,
                    volume: 1.0,
                    is_muted: false,
                    playback_rate: 1.0,
                    player_state: (typeof window.ytPlayerState_{pid} === 'number') ? window.ytPlayerState_{pid} : -1,
                    quality: null
                }};
                try {{
                    state.current_time = Number(inst.getCurrentTime() || 0);
                }} catch(_e) {{}}
                try {{
                    state.duration = Number(inst.getDuration() || 0);
                }} catch(_e) {{}}
                try {{
                    state.volume = Number((inst.getVolume() || 100) / 100);
                }} catch(_e) {{}}
                try {{
                    state.is_muted = !!inst.isMuted();
                }} catch(_e) {{}}
                try {{
                    state.playback_rate = Number(inst.getPlaybackRate() || 1.0);
                }} catch(_e) {{}}
                try {{
                    var q = inst.getPlaybackQuality && inst.getPlaybackQuality();
                    state.quality = q ? String(q) : null;
                }} catch(_e) {{}}
                return JSON.stringify(state);
            }})()
            "#
        )
    }

    /// Play the YouTube player if ready.
    pub fn play(player_div_id: &str) -> String {
        let pid = escape_js_str(player_div_id);
        format!(
            r#"(function() {{
                var p = window.ytPlayerInstance_{pid};
                if (p && p.playVideo) p.playVideo();
            }})()"#
        )
    }

    /// Pause the YouTube player if ready.
    pub fn pause(player_div_id: &str) -> String {
        let pid = escape_js_str(player_div_id);
        format!(
            r#"(function() {{
                var p = window.ytPlayerInstance_{pid};
                if (p && p.pauseVideo) p.pauseVideo();
            }})()"#
        )
    }

    /// Seek to an absolute time (seconds).
    pub fn seek_to(player_div_id: &str, position_seconds: f64) -> String {
        let pid = escape_js_str(player_div_id);
        let pos = if position_seconds.is_finite() && position_seconds >= 0.0 {
            position_seconds
        } else {
            0.0
        };
        format!(
            r#"(function() {{
                var p = window.ytPlayerInstance_{pid};
                if (p && p.seekTo) p.seekTo({pos}, true);
            }})()"#
        )
    }

    /// Set volume (0.0..1.0). YouTube expects 0..100.
    pub fn set_volume(player_div_id: &str, volume: f64) -> String {
        let pid = escape_js_str(player_div_id);
        let v = volume.clamp(0.0, 1.0) * 100.0;
        format!(
            r#"(function() {{
                var p = window.ytPlayerInstance_{pid};
                if (!p) return;
                if (p.setVolume) p.setVolume({v});
                if (p.isMuted && p.isMuted() && p.unMute && {v} > 0) p.unMute();
                if (p.mute && {v} <= 0) p.mute();
            }})()"#
        )
    }

    /// Set playback rate (e.g., 0.25, 0.5, 1, 1.25, 1.5, 2).
    pub fn set_playback_rate(player_div_id: &str, rate: f64) -> String {
        let pid = escape_js_str(player_div_id);
        let r = if rate.is_finite() { rate } else { 1.0 };
        format!(
            r#"(function() {{
                var p = window.ytPlayerInstance_{pid};
                if (p && p.setPlaybackRate) p.setPlaybackRate({r});
            }})()"#
        )
    }

    /// Seek relative to the current time (seconds).
    pub fn seek_relative(player_div_id: &str, delta_seconds: f64) -> String {
        let pid = escape_js_str(player_div_id);
        let d = if delta_seconds.is_finite() {
            delta_seconds
        } else {
            0.0
        };
        format!(
            r#"(function() {{
                var p = window.ytPlayerInstance_{pid};
                if (!p) return;
                try {{
                    var cur = (p.getCurrentTime && p.getCurrentTime()) || 0;
                    var dur = (p.getDuration && p.getDuration()) || 1e9;
                    if (p.seekTo) p.seekTo(Math.max(0, Math.min(dur, cur + ({d}))), true);
                }} catch(_e) {{}}
            }})()"#
        )
    }

    /// Toggle mute state.
    pub fn toggle_mute(player_div_id: &str) -> String {
        let pid = escape_js_str(player_div_id);
        format!(
            r#"(function() {{
                var p = window.ytPlayerInstance_{pid};
                if (!p) return;
                try {{
                    if (p.isMuted && p.isMuted()) {{ p.unMute && p.unMute(); }}
                    else {{ p.mute && p.mute(); }}
                }} catch(_e) {{}}
            }})()"#
        )
    }
}

/// Simple trait to abstract executing JavaScript against the desktop webview.
/// This keeps call sites decoupled from the concrete desktop APIs.
pub trait JsExecutor {
    fn eval_js(&self, script: &str) -> Result<(), VideoPlayerError>;
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
impl JsExecutor for dioxus_desktop::DesktopContext {
    fn eval_js(&self, script: &str) -> Result<(), VideoPlayerError> {
        self.webview
            .evaluate_script(script)
            .map_err(|e| VideoPlayerError::WebViewError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_js_str() {
        let s = "id'with\\quotes";
        let esc = escape_js_str(s);
        assert_eq!(esc, "id\\'with\\\\quotes");
    }

    #[test]
    fn test_local_scripts_contain_id() {
        let id = "cp-video-123";
        let s1 = local::play(id);
        let s2 = local::pause(id);
        let s3 = local::seek_to(id, 12.5);
        assert!(s1.contains(id));
        assert!(s2.contains(id));
        assert!(s3.contains(id));
    }

    #[test]
    fn test_youtube_scripts_contain_id() {
        let id = "yt-div-9";
        let create = youtube::create_player(id, "dQw4w9WgXcQ", None);
        let play = youtube::play(id);
        let pause = youtube::pause(id);
        let state = youtube::get_state_json(id);
        assert!(create.contains(id));
        assert!(play.contains(id));
        assert!(pause.contains(id));
        assert!(state.contains(id));
    }

    #[test]
    fn test_global_keyboard_scripts() {
        let a = global::attach_keyboard_handler();
        let p = global::pop_last_keyboard_key();
        assert!(a.contains("addEventListener('keydown'"));
        assert!(p.contains("lastVideoPlayerKey"));
    }
}
