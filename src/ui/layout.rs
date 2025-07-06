//! Layout Component with Unified Theme System
//!
//! This module provides the main application layout including:
//! - Responsive sidebar navigation
//! - App bar with theme toggle
//! - Main content area
//! - Proper state management
//! - Accessibility features
//! - Mobile-first responsive design

use crate::types::{AppState, Route};
use crate::ui::navigation::handle_navigation_with_fallback;
use crate::ui::theme_unified::{ThemeProvider, ThemeToggle};
use dioxus::events::SerializedMouseData;
use dioxus::prelude::*;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Sidebar navigation item
#[derive(Clone, Debug)]
struct NavItem {
    label: &'static str,
    icon: &'static str,
    route: Route,
    aria_label: &'static str,
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        label: "Dashboard",
        icon: "🏠",
        route: Route::Dashboard,
        aria_label: "Navigate to dashboard",
    },
    NavItem {
        label: "Add Course",
        icon: "➕",
        route: Route::AddCourse,
        aria_label: "Add new course",
    },
];

/// Layout state for managing UI preferences
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutState {
    pub sidebar_collapsed: bool,
    pub mobile_nav_open: bool,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_collapsed: true,
            mobile_nav_open: false,
        }
    }
}

/// Hook for layout state management
pub fn use_layout_state() -> Signal<LayoutState> {
    use_context::<Signal<LayoutState>>()
}

/// Sidebar Navigation Component
#[component]
fn Sidebar() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut layout_state = use_layout_state();
    let current_route = app_state.read().current_route.clone();

    let toggle_sidebar = move |_| {
        let mut state = layout_state.read().clone();
        state.sidebar_collapsed = !state.sidebar_collapsed;
        layout_state.set(state);

        // Persist to local storage
        save_layout_preferences(&layout_state.read());
    };

    let mut close_mobile_nav = move |_| {
        let mut state = layout_state.read().clone();
        state.mobile_nav_open = false;
        layout_state.set(state);
    };

    let mut handle_nav = move |route: Route| {
        handle_navigation_with_fallback(app_state, route);
        // Use SerializedMouseData::default() as dummy mouse data per Dioxus event API
        let dummy_mouse_data = Rc::new(MouseData::new(SerializedMouseData::default()));
        close_mobile_nav(MouseEvent::new(dummy_mouse_data, false));
    };

    let is_collapsed = layout_state.read().sidebar_collapsed;
    let is_mobile_open = layout_state.read().mobile_nav_open;

    rsx! {
        nav {
            class: format!(
                "sidebar {} {}",
                if is_collapsed { "sidebar--collapsed" } else { "sidebar--expanded" },
                if is_mobile_open { "sidebar--mobile-open" } else { "" }
            ),
            role: "navigation",
            aria_label: "Main navigation",

            // Sidebar header
            div { class: "sidebar__header",
                button {
                    class: "sidebar__toggle",
                    onclick: toggle_sidebar,
                    aria_label: if is_collapsed { "Expand sidebar" } else { "Collapse sidebar" },
                    aria_expanded: (!is_collapsed).to_string(),
                    title: if is_collapsed { "Expand sidebar" } else { "Collapse sidebar" },

                    if is_collapsed { "»" } else { "«" }
                }

                if !is_collapsed {
                    div { class: "sidebar__brand",
                        span { class: "sidebar__brand-icon", "🎓" }
                        span { class: "sidebar__brand-text", "Course Pilot" }
                    }
                }
            }

            // Navigation items
            ul { class: "sidebar__nav",
                for nav_item in NAV_ITEMS.iter() {
                    li { class: "sidebar__nav-item",
                        button {
                            class: format!(
                                "sidebar__nav-button {}",
                                if current_route == nav_item.route { "sidebar__nav-button--active" } else { "" }
                            ),
                            onclick: {
                                let route = nav_item.route.clone();
                                move |_| handle_nav(route.clone())
                            },
                            aria_label: nav_item.aria_label,
                            aria_current: if current_route == nav_item.route { "page" } else { "false" },
                            title: if is_collapsed { nav_item.label } else { "" },

                            span { class: "sidebar__nav-icon", "{nav_item.icon}" }

                            if !is_collapsed {
                                span { class: "sidebar__nav-label", "{nav_item.label}" }
                            }
                        }
                    }
                }
            }
        }

        // Mobile overlay
        if is_mobile_open {
            div {
                class: "sidebar__overlay",
                onclick: close_mobile_nav,
                aria_hidden: "true"
            }
        }
    }
}

/// App Bar Component
#[component]
fn AppBar() -> Element {
    let mut layout_state = use_layout_state();
    let app_state = use_context::<Signal<AppState>>();
    let has_active_import = app_state.read().active_import.is_some();

    let toggle_mobile_nav = move |_| {
        let mut state = layout_state.read().clone();
        state.mobile_nav_open = !state.mobile_nav_open;
        layout_state.set(state);
    };

    rsx! {
        header { class: "appbar",
            div { class: "appbar__start",
                // Mobile menu button
                button {
                    class: "appbar__mobile-toggle",
                    onclick: toggle_mobile_nav,
                    aria_label: "Open navigation menu",
                    aria_expanded: layout_state.read().mobile_nav_open.to_string(),

                    span { class: "appbar__mobile-toggle-icon", "☰" }
                }

                // Brand
                div { class: "appbar__brand",
                    span { class: "appbar__brand-icon", "🎓" }
                    h1 { class: "appbar__brand-text", "Course Pilot" }
                }
            }

            div { class: "appbar__end",
                // Import status indicator
                if has_active_import {
                    div { class: "appbar__status",
                        span { class: "appbar__status-icon", "🔄" }
                        span { class: "appbar__status-text", "Import in progress" }
                    }
                }

                // Theme toggle
                ThemeToggle {}
            }
        }
    }
}

/// Main Content Area Component
#[component]
fn MainContent() -> Element {
    use crate::ui::{AddCourseDialog, PlanView, course_dashboard};
    let app_state = use_context::<Signal<AppState>>();
    let current_route = app_state.read().current_route.clone();

    rsx! {
        main { class: "main-content",
            role: "main",
            aria_label: "Main content",

            // Route-based content
            match current_route {
                Route::Dashboard => rsx! { course_dashboard {} },
                Route::AddCourse => rsx! { AddCourseDialog {} },
                Route::PlanView(course_id) => rsx! { PlanView { course_id } }
            }
        }
    }
}

/// Main Layout Component
#[component]
pub fn Layout() -> Element {
    let _app_state = use_context::<Signal<AppState>>();
    let layout_state = use_signal(|| load_layout_preferences());

    // Provide layout state context
    use_context_provider(|| layout_state);

    // Handle responsive behavior
    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        // Add resize listener for responsive behavior
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                handle_window_resize(&mut layout_state);
            }) as Box<dyn FnMut()>);

            let _ =
                window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());

            closure.forget(); // Keep closure alive
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    use_effect(move || {
        // No-op on desktop
    });

    rsx! {
        ThemeProvider {
            div { class: "layout",
                Sidebar {}

                div { class: "layout__main",
                    AppBar {}
                    MainContent {}
                }

                // Include layout styles
                style { dangerous_inner_html: LAYOUT_STYLES }
            }
        }
    }
}

/// Handle window resize for responsive behavior
#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
fn handle_window_resize(layout_state: &mut Signal<LayoutState>) {
    if let Some(window) = web_sys::window() {
        let width = window
            .inner_width()
            .unwrap_or_default()
            .as_f64()
            .unwrap_or(0.0);

        let mut state = layout_state.read().clone();

        // Auto-collapse sidebar on mobile
        if width < 768.0 {
            state.sidebar_collapsed = true;
            state.mobile_nav_open = false;
        }

        layout_state.set(state);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_window_resize(_layout_state: &mut Signal<LayoutState>) {
    // No-op for desktop/native
}

#[cfg(not(target_arch = "wasm32"))]

/// Load layout preferences from localStorage
#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
fn load_layout_preferences() -> LayoutState {
    if let Some(window) = web_sys::window() {
        if let Some(storage) = window.local_storage().ok().flatten() {
            if let Some(data) = storage.get_item("course_pilot_layout").ok().flatten() {
                if let Ok(state) = serde_json::from_str::<LayoutState>(&data) {
                    return state;
                }
            }
        }
    }
    LayoutState::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_layout_preferences() -> LayoutState {
    LayoutState::default()
}

#[cfg(not(target_arch = "wasm32"))]

/// Save layout preferences to localStorage
#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
fn save_layout_preferences(state: &LayoutState) {
    if let Some(window) = web_sys::window() {
        if let Some(storage) = window.local_storage().ok().flatten() {
            if let Ok(data) = serde_json::to_string(state) {
                let _ = storage.set_item("course_pilot_layout", &data);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_layout_preferences(_state: &LayoutState) {
    // No-op for desktop/native
}

#[cfg(not(target_arch = "wasm32"))]

/// Layout component styles
const LAYOUT_STYLES: &str = r#"
/* === LAYOUT STYLES === */

.layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    overflow: hidden;
}

/* === SIDEBAR === */

.sidebar {
    background-color: var(--nav-bg);
    color: var(--nav-text);
    transition: width var(--transition-normal);
    box-shadow: var(--shadow-lg);
    z-index: 100;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
}

.sidebar--collapsed {
    width: 4rem; /* 64px */
}

.sidebar--expanded {
    width: 14rem; /* 224px */
}

.sidebar__header {
    display: flex;
    align-items: center;
    padding: var(--spacing-4);
    border-bottom: 1px solid var(--border-primary);
    min-height: 4rem;
}

.sidebar__toggle {
    background: none;
    border: none;
    color: var(--nav-text-secondary);
    cursor: pointer;
    font-size: 1.25rem;
    border-radius: var(--radius-md);
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
    flex-shrink: 0;
}

.sidebar__toggle:hover,
.sidebar__toggle:focus-visible {
    background-color: var(--nav-item-hover);
    color: var(--nav-text);
}

.sidebar__brand {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
    margin-left: var(--spacing-3);
    overflow: hidden;
}

.sidebar__brand-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
}

.sidebar__brand-text {
    font-weight: var(--font-weight-bold);
    font-size: var(--font-size-lg);
    white-space: nowrap;
    letter-spacing: 0.025em;
}

.sidebar__nav {
    flex: 1;
    list-style: none;
    margin: 0;
    padding: var(--spacing-2) 0;
    overflow-y: auto;
    overflow-x: hidden;
}

.sidebar__nav-item {
    margin: 0;
    padding: 0;
}

.sidebar__nav-button {
    width: 100%;
    background: none;
    border: none;
    color: var(--nav-text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
    padding: var(--spacing-3) var(--spacing-4);
    transition: all var(--transition-fast);
    text-align: left;
    font-size: var(--font-size-base);
    min-height: 3rem;
    position: relative;
}

.sidebar__nav-button:hover,
.sidebar__nav-button:focus-visible {
    background-color: var(--nav-item-hover);
    color: var(--nav-text);
}

.sidebar__nav-button--active {
    background-color: var(--nav-item-active);
    color: var(--nav-text);
    font-weight: var(--font-weight-medium);
}

.sidebar__nav-button--active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background-color: currentColor;
}

.sidebar__nav-icon {
    font-size: 1.25rem;
    flex-shrink: 0;
    width: 1.5rem;
    text-align: center;
}

.sidebar__nav-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* === MOBILE SIDEBAR === */

.sidebar__overlay {
    position: fixed;
    inset: 0;
    background-color: var(--bg-overlay);
    z-index: 90;
    display: none;
}

@media (max-width: 768px) {
    .sidebar {
        position: fixed;
        top: 0;
        left: 0;
        height: 100vh;
        width: 16rem;
        transform: translateX(-100%);
        z-index: 100;
    }

    .sidebar--mobile-open {
        transform: translateX(0);
    }

    .sidebar--mobile-open + .layout__main .sidebar__overlay {
        display: block;
    }

    .sidebar--collapsed {
        width: 16rem;
    }
}

/* === APP BAR === */

.appbar {
    height: 4rem;
    background-color: var(--appbar-bg);
    color: var(--appbar-text);
    border-bottom: 1px solid var(--appbar-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--spacing-6);
    box-shadow: var(--shadow-sm);
    position: sticky;
    top: 0;
    z-index: 50;
    min-height: 4rem;
}

.appbar__start {
    display: flex;
    align-items: center;
    gap: var(--spacing-4);
}

.appbar__mobile-toggle {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 1.25rem;
    border-radius: var(--radius-md);
    width: 2.5rem;
    height: 2.5rem;
    display: none;
    align-items: center;
    justify-content: center;
    transition: background-color var(--transition-fast);
}

.appbar__mobile-toggle:hover,
.appbar__mobile-toggle:focus-visible {
    background-color: var(--bg-secondary);
}

.appbar__brand {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
}

.appbar__brand-icon {
    font-size: 1.75rem;
}

.appbar__brand-text {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-bold);
    margin: 0;
    letter-spacing: 0.025em;
}

.appbar__end {
    display: flex;
    align-items: center;
    gap: var(--spacing-4);
}

.appbar__status {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    background-color: var(--color-warning-light);
    color: var(--color-warning-text);
    padding: var(--spacing-2) var(--spacing-3);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
}

.appbar__status-icon {
    animation: spin 2s linear infinite;
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

@media (max-width: 768px) {
    .appbar {
        padding: 0 var(--spacing-4);
    }

    .appbar__mobile-toggle {
        display: flex;
    }

    .appbar__brand-text {
        font-size: var(--font-size-lg);
    }
}

/* === MAIN LAYOUT === */

.layout__main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
}

.main-content {
    flex: 1;
    background-color: var(--bg-primary);
    padding: var(--spacing-6);
    overflow-y: auto;
    overflow-x: hidden;
}

@media (max-width: 768px) {
    .main-content {
        padding: var(--spacing-4);
    }
}

@media (max-width: 480px) {
    .main-content {
        padding: var(--spacing-3);
    }
}

/* === REDUCED MOTION === */

@media (prefers-reduced-motion: reduce) {
    .sidebar,
    .sidebar__nav-button,
    .appbar__mobile-toggle {
        transition: none;
    }

    .appbar__status-icon {
        animation: none;
    }
}

/* === HIGH CONTRAST === */

@media (prefers-contrast: high) {
    .sidebar {
        border-right: 2px solid var(--border-primary);
    }

    .appbar {
        border-bottom: 2px solid var(--border-primary);
    }

    .sidebar__nav-button:focus-visible,
    .sidebar__toggle:focus-visible,
    .appbar__mobile-toggle:focus-visible {
        outline: 2px solid currentColor;
        outline-offset: 2px;
    }
}

/* === PRINT STYLES === */

@media print {
    .sidebar,
    .appbar {
        display: none;
    }

    .layout__main {
        width: 100%;
    }

    .main-content {
        padding: 0;
        overflow: visible;
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_state_default() {
        let state = LayoutState::default();
        assert!(state.sidebar_collapsed);
        assert!(!state.mobile_nav_open);
    }

    #[test]
    fn test_nav_items_not_empty() {
        assert!(!NAV_ITEMS.is_empty());
        assert!(NAV_ITEMS.len() >= 2);
    }

    #[test]
    fn test_nav_item_properties() {
        for item in NAV_ITEMS {
            assert!(!item.label.is_empty());
            assert!(!item.icon.is_empty());
            assert!(!item.aria_label.is_empty());
        }
    }
}
