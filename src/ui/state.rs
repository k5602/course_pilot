use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::application::AppContext;
use crate::domain::ports::UserPreferencesRepository;
use crate::domain::value_objects::VideoQuality;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RightPanelTab {
    #[default]
    Notes,
    AiChat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Clone)]
pub struct AppState {
    pub backend: Option<Arc<AppContext>>,
    pub sidebar_collapsed: bool,
    pub right_panel_tab: RightPanelTab,
    pub right_panel_visible: bool,
    pub right_panel_width: f64,
    pub onboarding_completed: bool,
    pub chat_history_by_video: HashMap<String, Vec<ChatMessage>>,
    pub notes: HashMap<String, String>,
    pub current_video_id: Option<String>,
    pub current_course_id: Option<String>,
    pub current_quiz_id: Option<String>,
    pub last_video_by_course: HashMap<String, String>,
    pub preferred_quality: VideoQuality,
    pub session_quality: VideoQuality,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            backend: None,
            sidebar_collapsed: false,
            right_panel_tab: RightPanelTab::default(),
            right_panel_visible: true,
            right_panel_width: 320.0,
            onboarding_completed: false,
            chat_history_by_video: HashMap::new(),
            notes: HashMap::new(),
            current_video_id: None,
            current_course_id: None,
            current_quiz_id: None,
            last_video_by_course: HashMap::new(),
            preferred_quality: VideoQuality::P720,
            session_quality: VideoQuality::P720,
        }
    }

    pub fn with_backend(backend: Arc<AppContext>) -> Self {
        let mut state = Self::new();
        let quality = backend
            .preferences_repo
            .load("default")
            .ok()
            .flatten()
            .map(|p| p.preferred_quality())
            .unwrap_or(VideoQuality::P720);
        state.backend = Some(backend);
        state.preferred_quality = quality;
        state.session_quality = quality;
        state
    }

    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    pub fn has_gemini(&self) -> bool {
        self.backend.as_ref().map(|b| b.has_llm()).unwrap_or(false)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedState = Rc<RefCell<AppState>>;

pub fn new_shared_state() -> SharedState {
    let config = crate::application::AppConfig::from_env();
    let state = match crate::application::AppContext::new(config) {
        Ok(ctx) => AppState::with_backend(Arc::new(ctx)),
        Err(e) => {
            log::error!("Failed to initialize backend: {}", e);
            AppState::new()
        },
    };
    Rc::new(RefCell::new(state))
}
