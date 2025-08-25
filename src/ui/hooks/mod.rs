pub mod use_analytics;

pub mod use_backend;
pub mod use_courses;
pub mod use_export;
pub mod use_gemini;
pub mod use_import;
pub mod use_modals;
pub mod use_navigation;
pub mod use_notes;
pub mod use_plan_actions;
pub mod use_plans;
pub mod use_settings;
pub mod use_timer_integration;
pub mod use_videoplayer;

// Re-export commonly used hooks
pub use use_analytics::{AnalyticsManager, use_ai_recommendations, use_analytics_manager};

pub use use_backend::{Backend, use_backend};
pub use use_courses::{
    CourseManager, use_course_management, use_course_manager, use_course_progress,
    use_course_resource, use_courses_resource,
};
pub use use_export::{
    ExportManager, use_export_course_action, use_export_manager, use_export_notes_action,
    use_export_plan_action,
};
pub use use_gemini::{GeminiManager, use_gemini_manager};
pub use use_import::{
    FolderValidation, ImportManager, LocalFolderPreview, use_folder_preview, use_folder_validation,
    use_import_manager,
};
pub use use_modals::{use_form_manager, use_modal_manager};
pub use use_navigation::{BreadcrumbItem, use_navigation_manager};
pub use use_notes::{
    NotesManager, use_all_notes_resource, use_delete_note_action, use_notes_manager,
    use_notes_with_video_index_resource, use_save_note_action,
};
pub use use_plan_actions::{use_plan_resource, use_toggle_plan_item_action};
pub use use_plans::{
    PlanManager, ProgressInfo, use_plan_manager, use_plan_resource as use_plans_resource,
};
pub use use_settings::{
    SettingsManager, use_api_key_manager, use_settings_manager, use_settings_resource,
};
pub use use_timer_integration::{TimerIntegration, use_timer_integration};
pub use use_videoplayer::{
    KeyboardShortcuts, VideoAnalytics, VideoPerformanceMetrics, VideoPlayerManager,
    use_video_analytics, use_video_focus, use_video_keyboard_shortcuts, use_video_performance,
    use_videoplayer,
};
