//! exports for all reusable UI components in Course Pilot.
pub mod accordion;
pub mod analytics;
pub mod base;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod command_palette;
pub mod dropdown;
pub mod export_format_dialog;
pub mod gemini_chatbot;
pub mod import_modal;
pub mod markdown_renderer;
pub mod modal;
pub mod progress;
pub mod search_history;
pub mod tabs;
pub mod tag_input;
pub mod timer;
pub mod toast;
pub mod top_bar;
pub mod video_player_modal;

// exports for convenience
pub use analytics::{
    AIRecommendationsPanel, LastAccessedCourse, LearningAnalytics, PomodoroTimer, TodaysSessions,
    UpcomingDeadlines,
};
pub use base::{BaseButton, BaseCard, BaseList, BaseListItem, BaseModal, BasePage};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use card::{ActionItem, BadgeData, Card, CardVariant};
pub use dropdown::{DropdownItem, DropdownTrigger, UnifiedDropdown, create_course_actions};
pub use export_format_dialog::ExportFormatDialog;
pub use gemini_chatbot::GeminiChatbot;
pub use import_modal::{
    ImportModal, ImportModalProps, ImportPreview, ImportSettings, ImportSource, ImportVideoPreview,
};
pub use markdown_renderer::{ChatMarkdownRenderer, MarkdownRenderer};
pub use modal::{Badge, Modal, ModalVariant, alert_modal, confirmation_modal, form_modal};
pub use progress::{ProgressBar, ProgressRing};
pub use search_history::SearchHistory;
pub use tag_input::TagInput;
pub use timer::{PomodoroTimer as EnhancedPomodoroTimer, TimerSettings, TimerStatistics};
pub use toast::toast as toast_helpers;
pub use toast::{
    Toast, ToastContainer, ToastManager, ToastVariant, provide_toast_manager, show_toast,
    use_toast_manager,
};
pub use top_bar::TopBar;
pub use video_player_modal::{VideoPlayerModal, VideoPlayerModalManager, use_video_player_modal};
