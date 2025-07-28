//! exports for all reusable UI components in Course Pilot.
pub mod accordion;
pub mod analytics;
pub mod base;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod clustering_settings;
pub mod command_palette;
pub mod dropdown;
pub mod export_format_dialog;
pub mod import_modal;
pub mod modal;
pub mod progress;
pub mod search_history;
pub mod tabs;
pub mod tag_input;
pub mod toast;
pub mod top_bar;

// exports for convenience
pub use analytics::{LearningAnalytics, AIRecommendationsPanel, TodaysSessions, LastAccessedCourse, UpcomingDeadlines, PomodoroTimer, ClusteringInsights};
pub use base::{BaseCard, BaseModal, BaseButton, BaseList, BasePage, BaseListItem};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use card::{Card, CardVariant, ActionItem, BadgeData};
pub use clustering_settings::{ABTestResults, ClusteringSettings, ManualAdjustmentInterface};
pub use dropdown::{DropdownItem, DropdownTrigger, UnifiedDropdown, create_course_actions};
pub use export_format_dialog::ExportFormatDialog;
pub use import_modal::{
    ImportModal, ImportModalProps, ImportPreview, ImportSettings, ImportSource, ImportVideoPreview,
};
pub use modal::{Modal, ModalVariant, Badge, confirmation_modal, form_modal, alert_modal};
pub use progress::{ProgressBar, ProgressRing};
pub use search_history::SearchHistory;
pub use tag_input::TagInput;
pub use toast::{Toast, ToastContainer, ToastManager, ToastVariant, show_toast, use_toast_manager, provide_toast_manager};
pub use toast::toast as toast_helpers;
pub use top_bar::TopBar;
