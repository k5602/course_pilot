//! Global Application State
//!
//! Combines UI state with backend AppContext.

use std::collections::HashMap;
use std::sync::Arc;

use dioxus::prelude::*;

use crate::application::AppContext;
use crate::domain::entities::{Course, Module, Video};

/// Which tab is active in the right panel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RightPanelTab {
    #[default]
    Notes,
    AiChat,
}

/// A chat message in the AI companion.
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

/// Global application state using Dioxus signals.
/// Integrates with backend AppContext for data access.
#[derive(Clone)]
pub struct AppState {
    // Backend context (wrapped in Arc for sharing)
    pub backend: Option<Arc<AppContext>>,

    // UI State
    pub sidebar_collapsed: Signal<bool>,
    pub right_panel_tab: Signal<RightPanelTab>,
    pub right_panel_visible: Signal<bool>,
    pub chat_history: Signal<Vec<ChatMessage>>,
    pub notes: Signal<HashMap<String, String>>,
    pub current_video_id: Signal<Option<String>>,
    pub youtube_embed_relay_url: Signal<Option<String>>,

    // Cached data from backend
    pub courses: Signal<Vec<Course>>,
    pub current_course: Signal<Option<Course>>,
    pub current_modules: Signal<Vec<Module>>,
    pub current_videos: Signal<Vec<Video>>,
}

impl AppState {
    /// Creates a new application state with default values.
    pub fn new() -> Self {
        Self {
            backend: None,
            sidebar_collapsed: Signal::new(false),
            right_panel_tab: Signal::new(RightPanelTab::default()),
            right_panel_visible: Signal::new(false),
            chat_history: Signal::new(Vec::new()),
            notes: Signal::new(HashMap::new()),
            current_video_id: Signal::new(None),
            youtube_embed_relay_url: Signal::new(None),
            courses: Signal::new(Vec::new()),
            current_course: Signal::new(None),
            current_modules: Signal::new(Vec::new()),
            current_videos: Signal::new(Vec::new()),
        }
    }

    /// Initialize with backend context.
    pub fn with_backend(backend: Arc<AppContext>) -> Self {
        Self { backend: Some(backend), ..Self::new() }
    }

    /// Check if backend is available.
    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    /// Check if Gemini API is configured.
    pub fn has_gemini(&self) -> bool {
        self.backend.as_ref().map(|b| b.has_llm()).unwrap_or(false)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
