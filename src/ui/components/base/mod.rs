//! Base Component System for Course Pilot
//! 
//! This module provides the foundational UI components that follow DaisyUI design patterns
//! and provide consistent APIs across the application.
//! 
//! ## Components
//! 
//! - [`BaseCard`] - Generic card container with variants
//! - [`BaseModal`] - Modal dialogs with multiple variants  
//! - [`BaseButton`] - Button component with DaisyUI styling
//! - [`BaseList`] - Generic list with configurable item rendering
//! - [`BasePage`] - Page layout with header, breadcrumbs, content
//! 
//! ## Documentation
//! 
//! - See `COMPONENT_STANDARDS.md` for API standards and patterns
//! - See `USAGE_EXAMPLES.md` for practical implementation examples
//! 
//! ## Design Principles
//! 
//! 1. **DaisyUI First** - All components use DaisyUI classes exclusively
//! 2. **Configurable Props** - Components configured through props without business logic
//! 3. **Consistent Animation** - Smooth animations using dioxus-motion
//! 4. **Accessibility** - Proper ARIA labels and keyboard navigation
//! 5. **Responsive Design** - Works seamlessly across desktop and mobile

pub mod base_card;
pub mod base_modal;
pub mod base_button;
pub mod base_list;
pub mod base_page;

pub use base_card::{BaseCard, BaseCardProps};
pub use base_modal::{BaseModal, BaseModalProps};
pub use base_button::{BaseButton, BaseButtonProps};
pub use base_list::{BaseList, BaseListProps, BaseListItem};
pub use base_page::{BasePage, BasePageProps};