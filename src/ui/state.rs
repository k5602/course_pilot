//! Global Application State

use dioxus::prelude::*;
use std::collections::HashMap;

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
#[derive(Clone)]
pub struct AppState {
    /// Whether the sidebar is collapsed.
    pub sidebar_collapsed: Signal<bool>,
    /// Current tab in right panel.
    pub right_panel_tab: Signal<RightPanelTab>,
    /// AI chat history for current video.
    pub chat_history: Signal<Vec<ChatMessage>>,
    /// Notes per video ID.
    pub notes: Signal<HashMap<String, String>>,
    /// Current video being watched (for context).
    pub current_video_id: Signal<Option<String>>,
}

impl AppState {
    /// Creates a new application state with default values.
    pub fn new() -> Self {
        Self {
            sidebar_collapsed: Signal::new(false),
            right_panel_tab: Signal::new(RightPanelTab::default()),
            chat_history: Signal::new(Vec::new()),
            notes: Signal::new(HashMap::new()),
            current_video_id: Signal::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
