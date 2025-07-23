//! exports for all reusable UI components in Course Pilot.
pub mod accordion;
pub mod breadcrumb;
pub mod button;
pub mod card;
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
pub use modal::Badge;
pub use dropdown::{DropdownItem, DropdownTrigger, UnifiedDropdown};
pub use import_modal::{ImportModal, ImportModalProps, ImportPreview, ImportSettings, ImportSource, ImportVideoPreview};
pub use progress::{ProgressBar, ProgressRing};
pub use search_history::SearchHistory;
pub use tag_input::TagInput;
pub use toast::ToastContainer;
