//! exports for all reusable UI components in Course Pilot.
pub mod accordion;
pub mod button;
pub mod card;
pub mod command_palette;
pub mod course_card;
pub mod dropdown;
pub mod modal;
pub mod progress;
pub mod sidebar_nav;
pub mod tabs;
pub mod toast;
pub mod top_bar;

// Re-exports for convenience
pub use toast::ToastContainer;

// Add advanced UI primitives module
pub mod badge;
pub mod modal_confirmation;
pub mod progress_ring;
