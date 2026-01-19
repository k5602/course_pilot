//! Discord Presence Adapter
//!
//! Implements the `PresenceProvider` port using Discord Rich Presence.
//! This implementation is non-blocking and handles Discord IPC communication
//! in a background thread to prevent UI hangs.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{
    Arc,
    mpsc::{Sender, channel},
};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use log::{info, warn};

use crate::domain::ports::{Activity, PresenceProvider};

/// Default Client ID for Course Pilot.
const DEFAULT_CLIENT_ID: &str = "1346589201925341297";

/// Messages sent to the background presence worker.
enum PresenceMsg {
    Update(Activity),
    ChangeClientId(String),
    Clear,
    Shutdown,
}

/// Adapter for Discord Rich Presence.
pub struct DiscordPresenceAdapter {
    tx: Sender<PresenceMsg>,
    connected: Arc<AtomicBool>,
}
impl Default for DiscordPresenceAdapter {
    fn default() -> Self {
        Self::new()
    }
}
impl DiscordPresenceAdapter {
    /// Updates the client ID used for Discord Rich Presence.
    pub fn set_client_id(&self, client_id: String) {
        let _ = self.tx.send(PresenceMsg::ChangeClientId(client_id));
    }

    /// Creates a new Discord Presence adapter and starts the background worker.
    pub fn new() -> Self {
        let (tx, rx) = channel::<PresenceMsg>();
        let connected = Arc::new(AtomicBool::new(false));
        let connected_flag = connected.clone();

        let client_id = std::env::var("DISCORD_CLIENT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());

        if client_id == DEFAULT_CLIENT_ID {
            info!("Discord presence using default client id; set DISCORD_CLIENT_ID to override.");
        } else {
            info!("Discord presence using client id from DISCORD_CLIENT_ID.");
        }

        thread::spawn(move || {
            // Create the client directly.
            // DiscordIpcClient::new returns the client struct directly.
            let mut client_opt: Option<DiscordIpcClient> = Some(DiscordIpcClient::new(&client_id));

            let mut connected = false;
            let mut connection_attempts = 0u32;
            let start_time =
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

            // Attempt proactive connection on startup
            if let Some(ref mut c) = client_opt {
                match c.connect() {
                    Ok(_) => {
                        connected = true;
                        connected_flag.store(true, Ordering::SeqCst);
                        info!("Connected to Discord IPC on startup");
                    },
                    Err(e) => {
                        // Log at warn level for initial connection attempt
                        warn!(
                            "Initial Discord IPC connection failed: {}. Will retry on activity updates.",
                            e
                        );
                    },
                }
            }

            while let Ok(msg) = rx.recv() {
                match msg {
                    PresenceMsg::Shutdown => {
                        connected_flag.store(false, Ordering::SeqCst);
                        if let Some(mut c) = client_opt.take() {
                            let _ = c.close();
                        }
                        break;
                    },
                    PresenceMsg::ChangeClientId(new_id) => {
                        if let Some(mut c) = client_opt.take() {
                            let _ = c.close();
                        }
                        connected = false;
                        connected_flag.store(false, Ordering::SeqCst);
                        client_opt = Some(DiscordIpcClient::new(&new_id));
                        info!("Discord presence client ID changed to: {}", new_id);
                    },
                    PresenceMsg::Clear => {
                        if let Some(ref mut c) = client_opt {
                            if connected {
                                let _ = c.clear_activity();
                            }
                        }
                    },
                    PresenceMsg::Update(activity_type) => {
                        // Attempt connection if not connected
                        if !connected {
                            if let Some(ref mut c) = client_opt {
                                connection_attempts += 1;
                                match c.connect() {
                                    Ok(_) => {
                                        connected = true;
                                        connected_flag.store(true, Ordering::SeqCst);
                                        info!(
                                            "Connected to Discord IPC after {} attempt(s)",
                                            connection_attempts
                                        );
                                        connection_attempts = 0;
                                    },
                                    Err(e) => {
                                        connected_flag.store(false, Ordering::SeqCst);
                                        // Log first 3 attempts at warn level, then debug to avoid spam
                                        if connection_attempts <= 3 {
                                            warn!(
                                                "Discord IPC connection attempt {} failed: {}",
                                                connection_attempts, e
                                            );
                                        } else {
                                            log::debug!(
                                                "Discord IPC connection attempt {} failed: {}",
                                                connection_attempts,
                                                e
                                            );
                                        }
                                    },
                                }
                            }
                        }

                        if let Some(ref mut c) = client_opt {
                            if connected {
                                // Buffers must live outside match for Activity to borrow them
                                let mut details_buf: Option<String> = None;
                                let mut state_buf: Option<String> = None;

                                // Build activity without assets (assets require Discord Developer Portal setup)
                                let mut discord_act = activity::Activity::new()
                                    .timestamps(activity::Timestamps::new().start(start_time));

                                match activity_type {
                                    Activity::Idle => {
                                        discord_act = discord_act.details("Idle");
                                    },
                                    Activity::Dashboard => {
                                        discord_act = discord_act.details("In Dashboard");
                                    },
                                    Activity::BrowsingCourses => {
                                        discord_act = discord_act.details("Browsing Courses");
                                    },
                                    Activity::Watching { course_title, video_title } => {
                                        details_buf = Some(format!("Watching: {}", video_title));
                                        state_buf = Some(format!("Course: {}", course_title));
                                    },
                                    Activity::TakingExam { course_title, exam_title } => {
                                        details_buf = Some(format!("Taking Quiz: {}", exam_title));
                                        state_buf = Some(format!("Course: {}", course_title));
                                    },
                                    Activity::Settings => {
                                        discord_act = discord_act.details("Adjusting Settings");
                                    },
                                }

                                // Apply dynamic details/state after match if set
                                if let (Some(details), Some(state)) = (&details_buf, &state_buf) {
                                    discord_act = discord_act.details(details).state(state);
                                }

                                match c.set_activity(discord_act) {
                                    Ok(_) => {
                                        log::debug!("Discord presence updated successfully");
                                    },
                                    Err(e) => {
                                        warn!("Failed to update Discord presence: {}", e);
                                        // Connection might be lost
                                        connected = false;
                                        connected_flag.store(false, Ordering::SeqCst);
                                    },
                                }
                            }
                        }
                    },
                }
            }
        });

        Self { tx, connected }
    }
}

impl PresenceProvider for DiscordPresenceAdapter {
    fn update_activity(&self, activity: Activity) {
        let _ = self.tx.send(PresenceMsg::Update(activity));
    }

    fn clear_activity(&self) {
        let _ = self.tx.send(PresenceMsg::Clear);
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

impl Drop for DiscordPresenceAdapter {
    fn drop(&mut self) {
        self.connected.store(false, Ordering::SeqCst);
        let _ = self.tx.send(PresenceMsg::Shutdown);
    }
}

/// No-op provider for when Discord is disabled or unsupported.
pub struct NoopPresenceProvider;

impl PresenceProvider for NoopPresenceProvider {
    fn update_activity(&self, _activity: Activity) {}
    fn clear_activity(&self) {}
    fn is_connected(&self) -> bool {
        false
    }
}
