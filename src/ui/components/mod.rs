//! UI Components Module
//!
//! This module contains all the reusable UI components for the Course Pilot application.
//! All components are built with the unified theme system and follow consistent design patterns.
//!
//! ## Component Categories
//!
//! ### Form Components
//! - `Input` - Enhanced input component with multiple variants and states
//! - `TextArea` - Multi-line text input component
//! - `SearchInput` - Specialized search input with icon
//! - `PasswordInput` - Password input with show/hide toggle
//! - `NumberInput` - Numeric input with validation
//! - `Checkbox` - Checkbox input component
//! - `RadioGroup` - Radio button group component
//! - `Label` - Form label component
//!
//! ### Layout Components
//! - `Card` - Flexible card component with multiple variants
//! - `CardHeader` - Card header with title, subtitle, and actions
//! - `CardContent` - Card content area
//! - `CardActions` - Card action buttons area
//! - `CardMedia` - Card media/image component
//!
//! ### Interactive Components
//! - `Button` - Enhanced button component with multiple variants
//! - `IconButton` - Icon-only button component
//! - `LoadingButton` - Button with loading state
//! - `SubmitButton` - Form submit button
//! - `ButtonGroup` - Group of related buttons
//!
//! ### Feedback Components
//! - `Progress` - Progress indicator component
//! - `SkeletonLoader` - Loading skeleton component
//! - `AlertDialog` - Alert and confirmation dialogs
//!
//! ### Application Components
//! - `AddCourseDialog` - Course creation dialog
//! - `CourseDashboard` - Main dashboard component
//! - `PlanView` - Course plan viewing component

// Core form components
pub mod checkbox;
pub mod input;
pub mod label;
pub mod radio_group;

// Layout components
pub mod card;

// Interactive components
pub mod button;

// Feedback components
pub mod alert_dialog;
pub mod progress;
pub mod skeleton;

// Specialized UI components
pub mod context_menu;
pub mod hover_card;

// Application-specific components
pub mod add_course_dialog;
pub mod course_dashboard;
pub mod plan_view;

// Error handling
pub mod error_boundary;

// Re-export all main components for easier importing

// Form Components
pub use checkbox::Checkbox;
pub use input::{
    Input, InputSize, InputState, InputType, InputVariant, NumberInput, PasswordInput, SearchInput,
    TextArea,
};
pub use label::Label;
pub use radio_group::{RadioGroup, RadioItem};

// Layout Components
pub use card::{
    ActionCard, Card, CardActions, CardContent, CardHeader, CardInteraction, CardMedia, CardSize,
    CardVariant, MediaCard, SimpleCard,
};

// Interactive Components
pub use button::{
    Button, ButtonGroup, ButtonSize, ButtonType, ButtonVariant, IconButton, LoadingButton,
    SubmitButton,
};

// Feedback Components
pub use alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
pub use progress::Progress;
pub use skeleton::SkeletonLoader;

// Application Components
pub use add_course_dialog::AddCourseDialog;
pub use course_dashboard::course_dashboard;
pub use plan_view::PlanView;

// Utility re-exports for specialized components
pub use context_menu::ContextMenu;
pub use hover_card::HoverCard;
pub use error_boundary::ErrorBoundary;

/// Prelude module for importing common components
pub mod prelude {
    pub use super::{
        ActionCard,
        // Application components
        AddCourseDialog,
        AlertDialogAction,
        AlertDialogActions,
        AlertDialogCancel,

        AlertDialogContent,
        AlertDialogDescription,
        AlertDialogRoot,
        AlertDialogTitle,
        // Interactive components
        Button,
        ButtonGroup,
        ButtonSize,
        ButtonType,

        ButtonVariant,
        // Layout components
        Card,
        CardActions,
        CardContent,
        CardHeader,
        CardInteraction,

        CardMedia,
        CardSize,
        CardVariant,
        Checkbox,
        ContextMenu,
        // Utility components
        HoverCard,
        IconButton,
        // Form components
        Input,
        InputSize,
        InputState,
        InputType,

        InputVariant,
        Label,
        LoadingButton,
        MediaCard,
        NumberInput,
        PasswordInput,
        PlanView,

        // Feedback components
        Progress,
        RadioGroup,
        RadioItem,
        SearchInput,
        SimpleCard,
        SkeletonLoader,
        SubmitButton,
        TextArea,
        course_dashboard,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_exports() {
        // This test ensures all main components are properly exported
        // and can be imported without issues

        // Form components
        let _input_variant = InputVariant::Outlined;
        let _input_size = InputSize::Medium;
        let _input_state = InputState::Normal;
        let _input_type = InputType::Text;

        // Layout components
        let _card_variant = CardVariant::Elevated;
        let _card_size = CardSize::Medium;
        let _card_interaction = CardInteraction::None;

        // Interactive components
        let _button_variant = ButtonVariant::Primary;
        let _button_size = ButtonSize::Medium;
        let _button_type = ButtonType::Button;
    }

    #[test]
    fn test_enum_defaults() {
        // Test that all enum defaults work as expected
        assert_eq!(InputVariant::default(), InputVariant::Outlined);
        assert_eq!(InputSize::default(), InputSize::Medium);
        assert_eq!(InputState::default(), InputState::Normal);
        assert_eq!(InputType::default(), InputType::Text);

        assert_eq!(CardVariant::default(), CardVariant::Elevated);
        assert_eq!(CardSize::default(), CardSize::Medium);
        assert_eq!(CardInteraction::default(), CardInteraction::None);

        assert_eq!(ButtonVariant::default(), ButtonVariant::Primary);
        assert_eq!(ButtonSize::default(), ButtonSize::Medium);
        assert_eq!(ButtonType::default(), ButtonType::Button);
    }
}
