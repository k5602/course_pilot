pub mod use_app_state;
pub mod use_courses;
pub mod use_modals;
pub mod use_navigation;
pub mod use_notes;
pub mod use_plan_actions;

// Re-export commonly used hooks
pub use use_app_state::{use_app_state, use_backend_adapter};
pub use use_courses::{
    use_course_manager, use_course_progress, use_course_reactive_hook, use_courses_reactive_hook,
};
pub use use_modals::{use_form_manager, use_modal_manager};
pub use use_navigation::{BreadcrumbItem, use_navigation_manager};
pub use use_notes::{
    use_all_notes_resource, use_delete_note_action, use_notes_with_video_index_resource,
    use_save_note_action,
};
pub use use_plan_actions::{use_plan_resource, use_toggle_plan_item_action};
