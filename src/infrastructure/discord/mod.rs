//! Discord Presence Adapter
//!
//! Implements the `PresenceProvider` port using Discord Rich Presence.
//! This implementation is non-blocking and handles Discord IPC communication
//! in a background thread to prevent UI hangs.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{
    Arc,
    mpsc::{Receiver, Sender, channel},
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, to_value};
use uuid::Uuid;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use log::{info, warn};

use crate::domain::ports::{Activity, PresenceProvider};

/// Default Client ID for Course Pilot.
const DEFAULT_CLIENT_ID: &str = "1465702852373647452";

const UPDATE_INTERVAL: Duration = Duration::from_secs(2);
const CONNECT_BACKOFF_MIN: Duration = Duration::from_secs(2);
const CONNECT_BACKOFF_MAX: Duration = Duration::from_secs(60);
const WORKER_TICK: Duration = Duration::from_millis(250);

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

        let client_id = DEFAULT_CLIENT_ID.to_string();

        thread::spawn(move || {
            presence_worker(rx, connected_flag, client_id);
        });

        Self { tx, connected }
    }
}

fn presence_worker(rx: Receiver<PresenceMsg>, connected_flag: Arc<AtomicBool>, client_id: String) {
    let mut state = WorkerState::new(client_id);

    loop {
        match rx.recv_timeout(WORKER_TICK) {
            Ok(msg) => {
                if state.handle_message(msg, &connected_flag) {
                    break;
                }
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {},
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }

        state.tick(&connected_flag);
    }
}

struct WorkerState {
    client_id: String,
    client: Option<DiscordIpcClient>,
    connected: bool,
    pending: Option<Activity>,
    pending_clear: bool,
    last_sent: Option<Activity>,
    last_update: Option<Instant>,
    next_connect_attempt: Instant,
    backoff: Duration,
    app_start: SystemTime,
}

impl WorkerState {
    fn new(client_id: String) -> Self {
        Self {
            client_id,
            client: None,
            connected: false,
            pending: None,
            pending_clear: false,
            last_sent: None,
            last_update: None,
            next_connect_attempt: Instant::now(),
            backoff: CONNECT_BACKOFF_MIN,
            app_start: SystemTime::now(),
        }
    }

    fn handle_message(&mut self, msg: PresenceMsg, connected_flag: &Arc<AtomicBool>) -> bool {
        match msg {
            PresenceMsg::Shutdown => {
                log::debug!("Presence worker received shutdown");
                self.shutdown(connected_flag);
                true
            },
            PresenceMsg::ChangeClientId(new_id) => {
                log::debug!("Presence worker received client id change");
                self.reconfigure_client(new_id, connected_flag);
                false
            },
            PresenceMsg::Clear => {
                log::debug!("Presence worker received clear request");
                self.pending_clear = true;
                self.pending = None;
                false
            },
            PresenceMsg::Update(activity) => {
                log::debug!("Presence worker received update: {:?}", activity);
                if !self.connected || self.last_sent.as_ref() != Some(&activity) {
                    self.pending = Some(activity);
                }
                false
            },
        }
    }

    fn tick(&mut self, connected_flag: &Arc<AtomicBool>) {
        let should_connect = (self.pending.is_some() || self.pending_clear) && !self.connected;
        if should_connect && Instant::now() >= self.next_connect_attempt {
            self.try_connect(connected_flag);
        }

        if !self.connected {
            return;
        }

        if self.pending_clear && self.can_send_update() {
            if let Some(ref mut client) = self.client {
                log::debug!("Sending clear activity to Discord");
                if client.clear_activity().is_ok() {
                    log::debug!("Cleared Discord activity");
                    self.pending_clear = false;
                    self.last_sent = None;
                    self.last_update = Some(Instant::now());
                } else {
                    self.on_connection_error(connected_flag);
                }
            }
            return;
        }

        if let Some(activity) = self.pending.take() {
            if self.can_send_update() {
                if let Some(ref mut client) = self.client {
                    log::debug!("Sending Discord activity update: {:?}", activity);
                    let (details, state) = match &activity {
                        Activity::Idle => ("Idle".to_string(), "Course Pilot".to_string()),
                        Activity::Dashboard => {
                            ("In Dashboard".to_string(), "Course Pilot".to_string())
                        },
                        Activity::BrowsingCourses => {
                            ("Browsing Courses".to_string(), "Course Pilot".to_string())
                        },
                        Activity::Watching { course_title, video_title } => (
                            format!("Watching: {}", video_title),
                            format!("Course: {}", course_title),
                        ),
                        Activity::TakingExam { course_title, exam_title } => (
                            format!("Taking Quiz: {}", exam_title),
                            format!("Course: {}", course_title),
                        ),
                        Activity::Settings => {
                            ("Adjusting Settings".to_string(), "Course Pilot".to_string())
                        },
                    };

                    let start_ts = self
                        .app_start
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or_default();

                    let discord_act = activity::Activity::new()
                        .activity_type(activity::ActivityType::Playing)
                        .details(details.as_str())
                        .state(state.as_str())
                        .timestamps(activity::Timestamps::new().start(start_ts));

                    let activity_payload = match to_value(&discord_act) {
                        Ok(value) => value,
                        Err(e) => {
                            warn!("Failed to serialize Discord activity: {}", e);
                            self.pending = Some(activity);
                            self.on_connection_error(connected_flag);
                            return;
                        },
                    };

                    let payload = json!({
                        "cmd": "SET_ACTIVITY",
                        "args": {
                            "pid": std::process::id(),
                            "activity": activity_payload
                        },
                        "nonce": Uuid::new_v4().to_string()
                    });

                    match client.send(payload, 1) {
                        Ok(_) => {
                            match client.recv() {
                                Ok((opcode, payload)) => {
                                    log::debug!(
                                        "Discord IPC response opcode={} payload={}",
                                        opcode,
                                        payload
                                    );
                                },
                                Err(e) => {
                                    log::debug!("Discord IPC response unavailable: {}", e);
                                },
                            }
                            self.last_sent = Some(activity);
                            self.last_update = Some(Instant::now());
                        },
                        Err(e) => {
                            warn!("Failed to update Discord presence: {}", e);
                            self.pending = Some(activity);
                            self.on_connection_error(connected_flag);
                        },
                    }
                }
            } else {
                self.pending = Some(activity);
            }
        }
    }

    fn can_send_update(&self) -> bool {
        self.last_update.map(|last| last.elapsed() >= UPDATE_INTERVAL).unwrap_or(true)
    }

    fn try_connect(&mut self, connected_flag: &Arc<AtomicBool>) {
        if self.client.is_none() {
            self.client = Some(DiscordIpcClient::new(&self.client_id));
        }

        if let Some(ref mut client) = self.client {
            match client.connect() {
                Ok(_) => {
                    self.connected = true;
                    connected_flag.store(true, Ordering::SeqCst);
                    self.backoff = CONNECT_BACKOFF_MIN;
                    self.last_update = None;
                    if self.pending.is_none() && !self.pending_clear {
                        if let Some(last) = self.last_sent.clone() {
                            self.pending = Some(last);
                        }
                    }
                    info!("Connected to Discord IPC");
                },
                Err(e) => {
                    self.connected = false;
                    connected_flag.store(false, Ordering::SeqCst);
                    warn!("Discord IPC connection failed: {}", e);
                    self.schedule_backoff();
                },
            }
        }
    }

    fn schedule_backoff(&mut self) {
        self.backoff = self.backoff.checked_mul(2).unwrap_or(CONNECT_BACKOFF_MAX);
        if self.backoff > CONNECT_BACKOFF_MAX {
            self.backoff = CONNECT_BACKOFF_MAX;
        }
        self.next_connect_attempt = Instant::now() + self.backoff;
    }

    fn on_connection_error(&mut self, connected_flag: &Arc<AtomicBool>) {
        self.connected = false;
        connected_flag.store(false, Ordering::SeqCst);
        if let Some(mut client) = self.client.take() {
            let _ = client.close();
        }
        if self.pending.is_none() {
            if let Some(last) = self.last_sent.clone() {
                self.pending = Some(last);
            }
        }
        self.schedule_backoff();
    }

    fn reconfigure_client(&mut self, new_id: String, connected_flag: &Arc<AtomicBool>) {
        self.shutdown(connected_flag);
        self.client_id = new_id;
        self.client = None;
        self.connected = false;
        self.next_connect_attempt = Instant::now();
        self.backoff = CONNECT_BACKOFF_MIN;
    }

    fn shutdown(&mut self, connected_flag: &Arc<AtomicBool>) {
        self.connected = false;
        connected_flag.store(false, Ordering::SeqCst);
        if let Some(mut client) = self.client.take() {
            let _ = client.close();
        }
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
