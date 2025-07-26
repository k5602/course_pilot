//! Modern State Management System
//!
//! This module provides reactive state management using modern Dioxus patterns
//! with focused context providers and signal-based reactivity.

pub mod courses;
pub mod imports;
pub mod notes;
pub mod contextual_panel;

use dioxus::prelude::*;
use crate::types::AppState;

/// Initialize all state contexts in the app root
pub fn provide_app_contexts(initial_state: AppState) {
    // Provide individual state contexts
    courses::provide_courses_context(initial_state.courses);
    imports::provide_imports_context(initial_state.active_import);
    notes::provide_notes_context(initial_state.notes);
    contextual_panel::provide_contextual_panel_context(initial_state.contextual_panel);
    
    // Provide sidebar state
    use_context_provider(|| Signal::new(initial_state.sidebar_open_mobile));
}

/// Hook to get sidebar mobile state
pub fn use_sidebar_mobile() -> Signal<bool> {
    use_context::<Signal<bool>>()
}