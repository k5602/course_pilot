// Component modules - made public for accessibility
pub mod card;
pub mod checkbox;
pub mod input;
pub mod label;
pub mod progress;
pub mod radio_group;
pub mod skeleton;

// TODO: Add these modules once they are implemented
pub mod button;
// pub mod radio_group;
// pub mod label;
// pub mod alert_dialog;
pub mod alert_dialog;

// Re-export main components for easier importing
// Re-export local components
pub use card::Card;
pub use input::Input;

// Re-export local components directly
pub use crate::ui::components::checkbox::Checkbox;
pub use crate::ui::components::label::Label;
pub use crate::ui::components::progress::Progress;
pub use crate::ui::components::radio_group::{RadioGroup, RadioItem};

// TODO: Add these re-exports once the modules are implemented
pub use button::Button;
pub use skeleton::SkeletonLoader;
// pub use radio_group::{RadioGroup, RadioItem};
// pub use label::Label;
pub use alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
