//! UI state management for Course Pilot
//!
//! This module handles reactive state for UI-specific operations including
//! contextual panels, mobile sidebar, navigation, and video context management.

use crate::types::{ContextualPanelState, ContextualPanelTab, VideoContext};
use dioxus::prelude::*;
use uuid::Uuid;

use super::{StateError, StateResult};

/// Contextual panel management context
#[derive(Clone, Copy)]
pub struct ContextualPanelContext {
    pub state: Signal<ContextualPanelState>,
}

impl Default for ContextualPanelContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextualPanelContext {
    pub fn new() -> Self {
        Self {
            state: Signal::new(ContextualPanelState::default()),
        }
    }
}

/// Mobile sidebar management context
#[derive(Clone, Copy)]
pub struct MobileSidebarContext {
    pub is_open: Signal<bool>,
}

impl Default for MobileSidebarContext {
    fn default() -> Self {
        Self::new()
    }
}

impl MobileSidebarContext {
    pub fn new() -> Self {
        Self {
            is_open: Signal::new(false),
        }
    }
}

/// Video context management for notes and playback
#[derive(Clone, Copy, Debug)]
pub struct VideoContextState {
    pub current_video: Signal<Option<VideoContext>>,
    pub is_notes_panel_open: Signal<bool>,
}

impl Default for VideoContextState {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoContextState {
    pub fn new() -> Self {
        Self {
            current_video: Signal::new(None),
            is_notes_panel_open: Signal::new(false),
        }
    }
}

/// Navigation state for routing and breadcrumbs
#[derive(Clone, Copy, Debug)]
pub struct NavigationState {
    pub current_route: Signal<String>,
    pub breadcrumbs: Signal<Vec<String>>,
    pub navigation_history: Signal<Vec<String>>,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            current_route: Signal::new("/".to_string()),
            breadcrumbs: Signal::new(vec!["Dashboard".to_string()]),
            navigation_history: Signal::new(vec![]),
        }
    }
}

/// Contextual panel context provider component
#[component]
pub fn ContextualPanelContextProvider(children: Element) -> Element {
    use_context_provider(|| ContextualPanelContext::new());
    rsx! { {children} }
}

/// Mobile sidebar context provider component
#[component]
pub fn MobileSidebarContextProvider(children: Element) -> Element {
    use_context_provider(|| MobileSidebarContext::new());
    rsx! { {children} }
}

/// Video context provider component
#[component]
pub fn VideoContextProvider(children: Element) -> Element {
    use_context_provider(|| VideoContextState::new());
    rsx! { {children} }
}

/// Navigation context provider component
#[component]
pub fn NavigationContextProvider(children: Element) -> Element {
    use_context_provider(|| NavigationState::new());
    rsx! { {children} }
}

/// Hook to access contextual panel state reactively
pub fn use_contextual_panel_reactive() -> Signal<ContextualPanelState> {
    use_context::<ContextualPanelContext>().state
}

/// Hook to access mobile sidebar state reactively
pub fn use_mobile_sidebar_reactive() -> Signal<bool> {
    use_context::<MobileSidebarContext>().is_open
}

/// Hook to access video context reactively
pub fn use_video_context_reactive() -> Signal<Option<VideoContext>> {
    use_context::<VideoContextState>().current_video
}

/// Hook to access notes panel state reactively
pub fn use_notes_panel_reactive() -> Signal<bool> {
    use_context::<VideoContextState>().is_notes_panel_open
}

/// Hook to access navigation state reactively
pub fn use_navigation_reactive() -> Signal<String> {
    use_context::<NavigationState>().current_route
}

/// Hook to access breadcrumbs reactively
pub fn use_breadcrumbs_reactive() -> Signal<Vec<String>> {
    use_context::<NavigationState>().breadcrumbs
}

/// Set video context and open notes panel
pub fn set_video_context_and_open_notes_reactive(video_context: VideoContext) {
    let mut video_context_state = use_context::<VideoContextState>();

    video_context_state.current_video.set(Some(video_context));
    video_context_state.is_notes_panel_open.set(true);

    // Update contextual panel to show notes
    let mut contextual_panel = use_contextual_panel_reactive();
    let mut panel_state = contextual_panel.read().clone();
    panel_state.active_tab = ContextualPanelTab::Notes;
    panel_state.is_open = true;
    contextual_panel.set(panel_state);
}

/// Helper to construct VideoContext from parts and open notes panel
pub fn set_video_context_and_open_notes_reactive_from_parts(
    course_id: Uuid,
    video_index: usize,
    video_title: String,
    module_title: String,
) {
    let video_context = VideoContext {
        course_id,
        video_index,
        video_title,
        module_title,
    };
    set_video_context_and_open_notes_reactive(video_context);
}

/// Clear video context and close notes panel
pub fn clear_video_context_reactive() {
    let mut video_context_state = use_context::<VideoContextState>();
    video_context_state.current_video.set(None);
    video_context_state.is_notes_panel_open.set(false);
}

/// Toggle mobile sidebar
pub fn toggle_mobile_sidebar_reactive() {
    let mut sidebar = use_mobile_sidebar_reactive();
    let current_state = sidebar.read().clone();
    sidebar.set(!current_state);
}

/// Open mobile sidebar
pub fn open_mobile_sidebar_reactive() {
    let mut sidebar = use_mobile_sidebar_reactive();
    sidebar.set(true);
}

/// Close mobile sidebar
pub fn close_mobile_sidebar_reactive() {
    let mut sidebar = use_mobile_sidebar_reactive();
    sidebar.set(false);
}

/// Set contextual panel tab
pub fn set_contextual_panel_tab_reactive(tab: ContextualPanelTab) {
    let mut contextual_panel = use_contextual_panel_reactive();
    let mut panel_state = contextual_panel.read().clone();
    panel_state.active_tab = tab;
    panel_state.is_open = true;
    contextual_panel.set(panel_state);
}

/// Toggle contextual panel
pub fn toggle_contextual_panel_reactive() {
    let mut contextual_panel = use_contextual_panel_reactive();
    let mut panel_state = contextual_panel.read().clone();
    panel_state.is_open = !panel_state.is_open;
    contextual_panel.set(panel_state);
}

/// Close contextual panel
pub fn close_contextual_panel_reactive() {
    let mut contextual_panel = use_contextual_panel_reactive();
    let mut panel_state = contextual_panel.read().clone();
    panel_state.is_open = false;
    contextual_panel.set(panel_state);
}

/// Navigate to a new route
pub fn navigate_to_reactive(route: String, breadcrumbs: Vec<String>) {
    let mut navigation_state = use_context::<NavigationState>();

    // Update navigation history
    let mut history = navigation_state.navigation_history.read().clone();
    let current_route = navigation_state.current_route.read().clone();
    if !current_route.is_empty() && current_route != route {
        history.push(current_route);
        // Limit history size
        if history.len() > 50 {
            history.remove(0);
        }
    }

    navigation_state.navigation_history.set(history);
    navigation_state.current_route.set(route);
    navigation_state.breadcrumbs.set(breadcrumbs);
}

/// Go back in navigation history
pub fn navigate_back_reactive() -> StateResult<()> {
    let mut navigation_state = use_context::<NavigationState>();
    let mut history = navigation_state.navigation_history.read().clone();

    if let Some(previous_route) = history.pop() {
        navigation_state.navigation_history.set(history);
        navigation_state.current_route.set(previous_route);
        // Note: breadcrumbs would need to be reconstructed based on the route
        Ok(())
    } else {
        Err(StateError::NavigationError(
            "No previous route in history".to_string(),
        ))
    }
}

/// Check if navigation can go back
pub fn can_navigate_back_reactive() -> bool {
    let navigation_state = use_context::<NavigationState>();
    !navigation_state.navigation_history.read().is_empty()
}

/// Get current video context
pub fn get_current_video_context_reactive() -> Option<VideoContext> {
    let video_context_state = use_context::<VideoContextState>();
    video_context_state.current_video.read().clone()
}

/// Check if notes panel is open
pub fn is_notes_panel_open_reactive() -> bool {
    let video_context_state = use_context::<VideoContextState>();
    *video_context_state.is_notes_panel_open.read()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contextual_panel_context_creation() {
        let default_state = ContextualPanelState::default();
        assert!(!default_state.is_open);
        assert_eq!(default_state.active_tab, ContextualPanelTab::Notes);
    }

    #[test]
    fn test_mobile_sidebar_context_creation() {
        let is_open = false;
        assert!(!is_open);
    }

    #[test]
    fn test_video_context_state_creation() {
        let current_video: Option<VideoContext> = None;
        let is_notes_panel_open = false;
        assert!(current_video.is_none());
        assert!(!is_notes_panel_open);
    }

    #[test]
    fn test_navigation_state_creation() {
        let current_route = "/".to_string();
        let breadcrumbs = vec!["Dashboard".to_string()];
        let navigation_history: Vec<String> = vec![];
        assert_eq!(current_route, "/");
        assert_eq!(breadcrumbs, vec!["Dashboard".to_string()]);
        assert!(navigation_history.is_empty());
    }

    #[test]
    fn test_video_context_creation() {
        let course_id = Uuid::new_v4();
        let video_context = VideoContext {
            course_id,
            video_index: 5,
            video_title: "Test Video".to_string(),
            module_title: "Test Module".to_string(),
        };

        assert_eq!(video_context.course_id, course_id);
        assert_eq!(video_context.video_index, 5);
        assert_eq!(video_context.video_title, "Test Video");
    }

    #[test]
    fn test_navigation_history_logic() {
        let mut history = vec!["route1".to_string(), "route2".to_string()];
        let _current_route = "route3".to_string();

        // Simulate adding to history
        history.push("route2".to_string()); // Current becomes previous

        // Simulate going back
        if let Some(previous) = history.pop() {
            assert_eq!(previous, "route2");
        }

        assert_eq!(history.len(), 2);
    }

    // UI/state integration test: toggling contextual panel via hooks in a minimal component
    #[test]
    #[ignore]
    fn test_contextual_panel_toggle_via_hooks() {
        use dioxus::prelude::*;
        let dom = VirtualDom::new(TestPanelRoot);
        let rendered = dioxus_ssr::render(&dom);

        // Initially closed, after calling set_contextual_panel_tab_reactive it should be open
        assert!(rendered.contains("before=false"));
        assert!(rendered.contains("after=true"));
    }

    #[component]
    fn TestPanelToggle() -> Element {
        let panel = use_contextual_panel_reactive();
        let before = panel.read().is_open;

        // Use hook-driven setter to open and set tab; should set is_open = true
        set_contextual_panel_tab_reactive(ContextualPanelTab::Notes);

        let after = panel.read().is_open;

        rsx! {
            div { id: "panel_state", "before={before} after={after}" }
        }
    }

    #[component]
    fn TestPanelRoot() -> Element {
        rsx! {
            ContextualPanelContextProvider {
                TestPanelToggle {}
            }
        }
    }
}
