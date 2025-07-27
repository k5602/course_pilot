pub mod use_analytics;
pub mod use_app_state;
pub mod use_backend;
pub mod use_courses;
pub mod use_export;
pub mod use_import;
pub mod use_modals;
pub mod use_navigation;
pub mod use_notes;
pub mod use_plan_actions;
pub mod use_plans;
pub mod use_settings;

// Re-export commonly used hooks
pub use use_analytics::{use_analytics_manager, use_ai_recommendations, AnalyticsManager};
pub use use_app_state::{use_app_state, use_backend_adapter};
pub use use_backend::{use_backend, Backend};
pub use use_courses::{use_course_manager, use_courses_resource, use_course_resource, CourseManager, use_course_progress, use_course_management};
pub use use_export::{use_export_course_action, use_export_notes_action, use_export_plan_action, use_export_manager, ExportManager};
pub use use_import::{use_folder_validation, use_import_manager, FolderValidation, ImportManager};
pub use use_modals::{use_form_manager, use_modal_manager};
pub use use_navigation::{BreadcrumbItem, use_navigation_manager};
pub use use_notes::{
    use_all_notes_resource, use_notes_with_video_index_resource, use_notes_manager, NotesManager,
    use_save_note_action, use_delete_note_action,
};
pub use use_plan_actions::{use_plan_resource, use_toggle_plan_item_action};
pub use use_plans::{use_plan_manager, use_plan_resource as use_plans_resource, ProgressInfo, PlanManager};
pub use use_settings::{use_settings_manager, use_settings_resource, use_api_key_manager, SettingsManager};
