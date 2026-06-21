use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::application::AppContext;
use crate::domain::value_objects::{UserId, VideoQuality};

pub const MAX_CHAT_HISTORY_PER_VIDEO: usize = 50;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RightPanelTab {
    #[default]
    Notes,
    AiChat,
}

#[derive(Clone)]
pub struct AppState {
    pub backend: Option<Arc<AppContext>>,
    pub sidebar_collapsed: bool,
    pub right_panel_tab: RightPanelTab,
    pub right_panel_visible: bool,
    pub right_panel_width: f64,
    pub onboarding_completed: bool,
    pub chat_history_by_video: HashMap<String, Vec<crate::application::use_cases::ChatMessageView>>,
    pub notes: HashMap<String, String>,
    pub current_video_id: Option<String>,
    pub current_course_id: Option<String>,
    pub current_quiz_id: Option<String>,
    pub last_video_by_course: HashMap<String, String>,
    pub preferred_quality: VideoQuality,
    pub session_quality: VideoQuality,
    pub boundary_batch_size: u32,
    pub cognitive_limit_minutes: u32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            backend: None,
            sidebar_collapsed: false,
            right_panel_tab: RightPanelTab::default(),
            right_panel_visible: false,
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
            boundary_batch_size: 5,
            cognitive_limit_minutes: 45,
        }
    }

    pub fn with_backend(backend: Arc<AppContext>) -> Self {
        let mut state = Self::new();
        let prefs = backend.preferences_repo.load(&UserId::new("default")).ok().flatten();
        if let Some(p) = &prefs {
            state.preferred_quality = p.preferred_quality();
            state.session_quality = p.preferred_quality();
            state.boundary_batch_size = p.boundary_batch_size();
            state.cognitive_limit_minutes = p.cognitive_limit_minutes();
            state.onboarding_completed = p.onboarding_completed();
            state.right_panel_visible = p.right_panel_visible();
            state.right_panel_width = p.right_panel_width() as f64;
        }
        state.backend = Some(backend);
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
