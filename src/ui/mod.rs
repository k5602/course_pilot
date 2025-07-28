//! UI Module for Course Pilot
//!
//! This module provides the complete user interface layer including components,
//! hooks, layout, and state management functionality.

pub mod app_root;
pub mod components;
pub mod courses;
pub mod dashboard;
pub mod error_handling;
pub mod hooks;
pub mod layout;
pub mod navigation;
pub mod notes_panel;
pub mod plan_view;
pub mod routes;
pub mod settings;
pub mod state;
pub mod state_management;
pub mod theme_unified;

// Re-export main application components
pub use app_root::AppRoot;
pub use notes_panel::{NotesPanel, NotesPanelMode};
pub use theme_unified::{AppTheme, ThemeContext, ThemeToggleButton, use_theme_context};

// Re-export error handling utilities
pub use error_handling::{handle_ui_error, use_error_handler};

// Re-export commonly used components for convenience
pub use components::{
    BaseButton, BaseCard, BaseList, BaseModal, BasePage,
    ProgressBar, ProgressRing, ToastContainer,
    ImportModal, ImportSource, ImportSettings, Badge,
    Card, CardVariant, BadgeData, ActionItem,
    DropdownItem, DropdownTrigger, UnifiedDropdown, create_course_actions,
    ExportFormatDialog, SearchHistory, TagInput, TopBar, BreadcrumbItem,
};

// Re-export toast functionality
pub use components::{Toast, ToastVariant, show_toast, use_toast_manager, provide_toast_manager, toast_helpers};

// Re-export modal functionality  
pub use components::modal::{Modal, ModalVariant, confirmation_modal, form_modal, alert_modal};

// Re-export layout components
pub use layout::{AppShell, ContextualPanel, Sidebar};

// Re-export navigation components
pub use navigation::{
    Breadcrumbs, RouteGuard, RouteGuardResult, RouteGuardManager, RouteGuardProvider, use_route_guard,
    DeepLinkingHandler, DeepLinkingManager, use_deep_linking
};

#[cfg(debug_assertions)]
pub use navigation::DeepLinkingTester;

// Re-export dashboard components
pub use dashboard::Dashboard;

// Re-export courses components
pub use courses::{AllCoursesView, CourseCard, CourseGrid, CourseActions};

// Re-export plan view components
pub use plan_view::{PlanView, PlanHeader, SessionControlPanel, SessionList, PlanChecklist};

// Re-export settings components
pub use settings::{SettingsView, GeneralSettings, APIKeysSettings, CourseDefaultSettings};

// Re-export commonly used hooks
pub use hooks::{
    use_app_state, use_backend, use_course_manager, use_courses_resource,
    use_export_manager, use_import_manager, use_modal_manager,
    use_navigation_manager, use_notes_manager, use_plan_manager, use_plan_resource,
    use_settings_manager, use_form_manager, use_course_progress,
    use_toggle_plan_item_action, AnalyticsManager, CourseManager, ExportManager,
    ImportManager, NotesManager, PlanManager, SettingsManager,
};

// Re-export state management
pub use state::{provide_app_contexts, use_sidebar_mobile};
pub use state_management::{
    use_loading_state, use_error_state, use_search_state, 
    use_pagination_state, use_selection_state, use_validation_state,
    use_debounced_state, use_async_operation_state,
    PaginationState, AsyncOperationState,
};
