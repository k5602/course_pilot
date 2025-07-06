//! User Interface Module for Course Pilot
//!
//! This module provides the complete UI system for the Course Pilot application,
//! built with a unified theme system and consistent design patterns.
//!
//! ## Features
//!
//! - **Unified Theme System**: Consistent design tokens and theme variables
//! - **Component Library**: Reusable, accessible UI components
//! - **Layout System**: Responsive layout with sidebar navigation
//! - **Theme Toggle**: Light/dark mode support
//! - **Accessibility**: WCAG compliant components with proper ARIA support
//! - **Mobile-First**: Responsive design that works on all devices
//!
//! ## Usage
//!
//! ```rust
//! use crate::ui::{ThemeProvider, Layout};
//! use crate::ui::components::prelude::*;
//!
//! // Basic app structure
//! rsx! {
//!     ThemeProvider {
//!         Layout {}
//!     }
//! }
//!
//! // Using components
//! rsx! {
//!     Card {
//!         variant: CardVariant::Elevated,
//!
//!         CardHeader {
//!             title: Some("My Card".to_string())
//!         }
//!
//!         CardContent {
//!             Button {
//!                 variant: ButtonVariant::Primary,
//!                 "Click me"
//!             }
//!         }
//!     }
//! }
//! ```

// Core UI modules
pub mod components;
pub mod layout;
pub mod navigation;

// Theme system modules
// pub mod theme; // Legacy theme system removed
pub mod accessibility;
pub use components::error_boundary::ErrorBoundary;
pub mod theme_unified;

// Re-export the unified theme system as the primary theme
pub use theme_unified::{
    ThemeMode, ThemeProvider, ThemeToggle, get_theme_mode, is_dark_theme, use_theme,
};

// Re-export layout components
pub use layout::{Layout, LayoutState, use_layout_state};

// Re-export navigation utilities
pub use crate::types::Route;
pub use navigation::handle_navigation_with_fallback;

// Re-export all components for convenience
pub use components::{
    // Layout components
    ActionCard,
    // Application components
    AddCourseDialog,
    // Feedback components
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
    Card,
    CardActions,
    CardContent,
    CardHeader,
    CardInteraction,
    CardMedia,
    CardSize,
    CardVariant,
    // Form components
    Checkbox,
    // Utility components
    ContextMenu,
    HoverCard,
    IconButton,
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

/// UI Prelude - Import common UI components and utilities
pub mod prelude {
    pub use super::components::prelude::*;
    pub use super::{
        Layout, LayoutState, ThemeMode, ThemeProvider, ThemeToggle, use_layout_state, use_theme,
    };
}

/// Design system constants and utilities
pub mod design_system {
    /// Standard spacing scale (in rem)
    pub const SPACING: &[&str] = &[
        "0",    // 0px
        "0.25", // 4px
        "0.5",  // 8px
        "0.75", // 12px
        "1",    // 16px
        "1.25", // 20px
        "1.5",  // 24px
        "2",    // 32px
        "2.5",  // 40px
        "3",    // 48px
        "4",    // 64px
        "5",    // 80px
        "6",    // 96px
    ];

    /// Typography scale (in rem)
    pub const FONT_SIZES: &[(&str, &str)] = &[
        ("xs", "0.75"),   // 12px
        ("sm", "0.875"),  // 14px
        ("base", "1"),    // 16px
        ("lg", "1.125"),  // 18px
        ("xl", "1.25"),   // 20px
        ("2xl", "1.5"),   // 24px
        ("3xl", "1.875"), // 30px
        ("4xl", "2.25"),  // 36px
    ];

    /// Border radius scale (in rem)
    pub const RADIUS: &[(&str, &str)] = &[
        ("none", "0"),
        ("sm", "0.25"), // 4px
        ("md", "0.5"),  // 8px
        ("lg", "1"),    // 16px
        ("xl", "1.5"),  // 24px
        ("full", "9999px"),
    ];

    /// Breakpoints for responsive design (in px)
    pub const BREAKPOINTS: &[(&str, u32)] = &[
        ("sm", 640),
        ("md", 768),
        ("lg", 1024),
        ("xl", 1280),
        ("2xl", 1536),
    ];

    /// Z-index scale for layering
    pub const Z_INDEX: &[(&str, i32)] = &[
        ("auto", 0),
        ("base", 0),
        ("docked", 10),
        ("dropdown", 1000),
        ("sticky", 1100),
        ("banner", 1200),
        ("overlay", 1300),
        ("modal", 1400),
        ("popover", 1500),
        ("skipLink", 1600),
        ("toast", 1700),
        ("tooltip", 1800),
    ];
}

/// Utility functions for UI development
pub mod utils {
    use super::ThemeMode;

    /// Generate a CSS class string from optional classes
    pub fn class_names(classes: &[Option<&str>]) -> String {
        classes
            .iter()
            .filter_map(|c| *c)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Generate responsive classes based on breakpoint
    pub fn responsive_class(base: &str, breakpoint: &str, value: &str) -> String {
        if breakpoint == "base" {
            format!("{}-{}", base, value)
        } else {
            format!("{}:{}-{}", breakpoint, base, value)
        }
    }

    /// Get theme-aware CSS variable
    pub fn theme_var<'a>(theme: ThemeMode, light_var: &'a str, dark_var: &'a str) -> &'a str {
        match theme {
            ThemeMode::Light => light_var,
            ThemeMode::Dark => dark_var,
        }
    }

    /// Convert spacing index to CSS value
    pub fn spacing(index: usize) -> Option<&'static str> {
        super::design_system::SPACING.get(index).copied()
    }

    /// Convert font size name to CSS value
    pub fn font_size(name: &str) -> Option<&'static str> {
        super::design_system::FONT_SIZES
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, size)| *size)
    }

    /// Convert radius name to CSS value
    pub fn radius(name: &str) -> Option<&'static str> {
        super::design_system::RADIUS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, r)| *r)
    }
}

/// Common CSS utility classes as constants
pub mod css_utils {
    /// Flexbox utilities
    pub const FLEX: &str = "display: flex;";
    pub const FLEX_COL: &str = "display: flex; flex-direction: column;";
    pub const FLEX_CENTER: &str = "display: flex; align-items: center; justify-content: center;";
    pub const FLEX_BETWEEN: &str =
        "display: flex; align-items: center; justify-content: space-between;";

    /// Grid utilities
    pub const GRID: &str = "display: grid;";
    pub const GRID_COLS_1: &str = "grid-template-columns: repeat(1, minmax(0, 1fr));";
    pub const GRID_COLS_2: &str = "grid-template-columns: repeat(2, minmax(0,
 1fr));";
    pub const GRID_COLS_3: &str = "grid-template-columns: repeat(3, minmax(0, 1fr));";

    /// Common spacing
    pub const GAP_2: &str = "gap: 0.5rem;";
    pub const GAP_4: &str = "gap: 1rem;";
    pub const GAP_6: &str = "gap: 1.5rem;";

    /// Common padding
    pub const P_2: &str = "padding: 0.5rem;";
    pub const P_4: &str = "padding: 1rem;";
    pub const P_6: &str = "padding: 1.5rem;";

    /// Common margin
    pub const M_2: &str = "margin: 0.5rem;";
    pub const M_4: &str = "margin: 1rem;";
    pub const M_6: &str = "margin: 1.5rem;";

    /// Width utilities
    pub const W_FULL: &str = "width: 100%;";
    pub const H_FULL: &str = "height: 100%;";

    /// Text utilities
    pub const TEXT_CENTER: &str = "text-align: center;";
    pub const TEXT_LEFT: &str = "text-align: left;";
    pub const TEXT_RIGHT: &str = "text-align: right;";

    /// Border radius
    pub const ROUNDED: &str = "border-radius: 0.25rem;";
    pub const ROUNDED_MD: &str = "border-radius: 0.375rem;";
    pub const ROUNDED_LG: &str = "border-radius: 0.5rem;";
}

/// Type definitions for common UI patterns
pub mod types {
    /// Size variants used across components
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Size {
        Small,
        Medium,
        Large,
    }

    impl Default for Size {
        fn default() -> Self {
            Self::Medium
        }
    }

    /// Color variants for semantic meaning
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ColorVariant {
        Primary,
        Secondary,
        Success,
        Warning,
        Error,
        Info,
    }

    impl Default for ColorVariant {
        fn default() -> Self {
            Self::Primary
        }
    }

    /// Common positioning options
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Position {
        Top,
        Bottom,
        Left,
        Right,
        Center,
    }

    /// Loading state for async operations
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum LoadingState {
        Idle,
        Loading,
        Success,
        Error,
    }

    impl Default for LoadingState {
        fn default() -> Self {
            Self::Idle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_system_constants() {
        assert!(!design_system::SPACING.is_empty());
        assert!(!design_system::FONT_SIZES.is_empty());
        assert!(!design_system::RADIUS.is_empty());
        assert!(!design_system::BREAKPOINTS.is_empty());
        assert!(!design_system::Z_INDEX.is_empty());
    }

    #[test]
    fn test_utils_functions() {
        // Test class_names utility
        let classes = utils::class_names(&[Some("btn"), None, Some("primary")]);
        assert_eq!(classes, "btn primary");

        // Test spacing utility
        let spacing = utils::spacing(4);
        assert_eq!(spacing, Some("1"));

        // Test font_size utility
        let size = utils::font_size("lg");
        assert_eq!(size, Some("1.125"));

        // Test radius utility
        let radius = utils::radius("md");
        assert_eq!(radius, Some("0.5"));
    }

    #[test]
    fn test_type_defaults() {
        assert_eq!(types::Size::default(), types::Size::Medium);
        assert_eq!(types::ColorVariant::default(), types::ColorVariant::Primary);
        assert_eq!(types::LoadingState::default(), types::LoadingState::Idle);
    }
}
