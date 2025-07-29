//! Comprehensive test suites for Course Pilot UI components
//! 
//! This module provides centralized testing for all UI components with proper field-specific tests.
//! Tests are organized by component type and cover rendering, props validation, event handling, and edge cases.

use dioxus::prelude::*;
use dioxus_ssr::render;
use dioxus_core::Mutations;

// Import components to test
use course_pilot::ui::components::base::{
    BaseButton, BaseButtonProps, BaseCard, BaseCardProps, BaseModal, BaseModalProps,
    BasePage, BasePageProps,
};
use course_pilot::ui::components::{
    ProgressRing, Badge, ProgressBar,
};

/// Helper function to render a component with proper VirtualDom setup
fn render_component<P: Clone + 'static>(
    component: fn(P) -> Element,
    props: P,
) -> String {
    let mut dom = VirtualDom::new_with_props(component, props);
    let mut mutations = Mutations::default();
    let _ = dom.rebuild(&mut mutations);
    render(&dom)
}

/// Helper function to create a dummy event handler for testing
fn dummy_event_handler<T: 'static>() -> EventHandler<T> {
    EventHandler::new(|_| {})
}

/// Centralized test suite for BaseCard component
#[cfg(test)]
mod base_card_tests {
    use super::*;

    /// Test BaseCard basic rendering
    #[test]
    fn test_basic_rendering() {
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: None,
                subtitle: None,
                children: rsx! { p { "Basic content" } },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(html.contains("Basic content"));
        assert!(html.contains("card"));
        assert!(html.contains("bg-base-100"));
        assert!(html.contains("shadow-xl"));
    }

    /// Test BaseCard title field
    #[test]
    fn test_title_field() {
        // Test with title
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: Some("Test Title".to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(html.contains("Test Title"));
        assert!(html.contains("card-title"));

        // Test without title
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: None,
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(!html.contains("card-title"));
    }

    /// Test BaseCard subtitle field
    #[test]
    fn test_subtitle_field() {
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: Some("Title".to_string()),
                subtitle: Some("Test Subtitle".to_string()),
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(html.contains("Test Subtitle"));
        assert!(html.contains("text-base-content/70"));
    }

    /// Test BaseCard variant field
    #[test]
    fn test_variant_field() {
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: None,
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card-compact",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(html.contains("card-compact"));
    }

    /// Test BaseCard class field
    #[test]
    fn test_class_field() {
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: None,
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card",
                class: "custom-test-class",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(html.contains("custom-test-class"));
    }

    /// Test BaseCard actions field
    #[test]
    fn test_actions_field() {
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: None,
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: Some(rsx! {
                    button { class: "btn btn-primary", "Action 1" }
                    button { class: "btn btn-secondary", "Action 2" }
                }),
                header_actions: None,
            },
        );

        assert!(html.contains("card-actions"));
        assert!(html.contains("Action 1"));
        assert!(html.contains("Action 2"));
    }

    /// Test BaseCard header_actions field
    #[test]
    fn test_header_actions_field() {
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: Some("Title".to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: Some(rsx! {
                    button { class: "btn btn-ghost btn-sm", "⋮" }
                }),
            },
        );

        assert!(html.contains("⋮"));
        assert!(html.contains("flex-shrink-0"));
    }
}

/// Centralized test suite for BaseButton component
#[cfg(test)]
mod base_button_tests {
    use super::*;

    /// Test BaseButton basic rendering
    #[test]
    fn test_basic_rendering() {
        let html = render_component(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Click me" },
                onclick: None,
                color: None,
                size: None,
                variant: None,
                class: "",
                disabled: false,
                icon: None,
                loading: false,
                button_type: "button",
            },
        );

        assert!(html.contains("Click me"));
        assert!(html.contains("btn"));
        assert!(html.contains("type=\"button\""));
    }

    /// Test BaseButton color field
    #[test]
    fn test_color_field() {
        let colors = vec!["primary", "secondary", "accent", "success", "warning", "error"];
        
        for color in colors {
            let html = render_component(
                BaseButton,
                BaseButtonProps {
                    children: rsx! { "Button" },
                    onclick: None,
                    color: Some(color.to_string()),
                    size: None,
                    variant: None,
                    class: "",
                    disabled: false,
                    icon: None,
                    loading: false,
                    button_type: "button",
                },
            );

            assert!(html.contains(&format!("btn-{}", color)));
        }
    }

    /// Test BaseButton size field
    #[test]
    fn test_size_field() {
        let sizes = vec!["xs", "sm", "md", "lg"];
        
        for size in sizes {
            let html = render_component(
                BaseButton,
                BaseButtonProps {
                    children: rsx! { "Button" },
                    onclick: None,
                    color: None,
                    size: Some(size.to_string()),
                    variant: None,
                    class: "",
                    disabled: false,
                    icon: None,
                    loading: false,
                    button_type: "button",
                },
            );

            assert!(html.contains(&format!("btn-{}", size)));
        }
    }

    /// Test BaseButton variant field
    #[test]
    fn test_variant_field() {
        let variants = vec!["outline", "ghost", "link"];
        
        for variant in variants {
            let html = render_component(
                BaseButton,
                BaseButtonProps {
                    children: rsx! { "Button" },
                    onclick: None,
                    color: None,
                    size: None,
                    variant: Some(variant.to_string()),
                    class: "",
                    disabled: false,
                    icon: None,
                    loading: false,
                    button_type: "button",
                },
            );

            assert!(html.contains(&format!("btn-{}", variant)));
        }
    }

    /// Test BaseButton disabled field
    #[test]
    fn test_disabled_field() {
        let html = render_component(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Disabled" },
                onclick: None,
                color: None,
                size: None,
                variant: None,
                class: "",
                disabled: true,
                icon: None,
                loading: false,
                button_type: "button",
            },
        );

        assert!(html.contains("btn-disabled"));
        assert!(html.contains("disabled"));
    }

    /// Test BaseButton loading field
    #[test]
    fn test_loading_field() {
        let html = render_component(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Loading" },
                onclick: None,
                color: None,
                size: None,
                variant: None,
                class: "",
                disabled: false,
                icon: None,
                loading: true,
                button_type: "button",
            },
        );

        assert!(html.contains("loading"));
        assert!(html.contains("loading-spinner"));
    }

    /// Test BaseButton icon field
    #[test]
    fn test_icon_field() {
        let html = render_component(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "With Icon" },
                onclick: None,
                color: None,
                size: None,
                variant: None,
                class: "",
                disabled: false,
                icon: Some(rsx! { span { "🔍" } }),
                loading: false,
                button_type: "button",
            },
        );

        assert!(html.contains("🔍"));
        assert!(html.contains("mr-2"));
    }

    /// Test BaseButton button_type field
    #[test]
    fn test_button_type_field() {
        let html = render_component(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Submit" },
                onclick: None,
                color: None,
                size: None,
                variant: None,
                class: "",
                disabled: false,
                icon: None,
                loading: false,
                button_type: "submit",
            },
        );

        assert!(html.contains("type=\"submit\""));
    }
}

/// Note: BaseModal tests are skipped because BaseModal uses dioxus-motion animations
/// and event handlers that require a Dioxus runtime context, which is not available
/// in unit tests. BaseModal functionality should be tested through integration tests instead.
/// 
/// The BaseModal component includes:
/// - Motion animations for scale and opacity
/// - Event handlers for close functionality
/// - Dynamic styling based on props
/// 
/// These features require a full Dioxus runtime environment to test properly.

/// Note: BaseList tests are skipped because BaseListProps contains Box<dyn Fn> 
/// which cannot be cloned, making it incompatible with VirtualDom::new_with_props.
/// BaseList functionality should be tested through integration tests instead.

/// Centralized test suite for BasePage component
#[cfg(test)]
mod base_page_tests {
    use super::*;

    /// Test BasePage title field
    #[test]
    fn test_title_field() {
        let html = render_component(
            BasePage,
            BasePageProps {
                title: Some("Page Title".to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                header_actions: None,
                breadcrumbs: None,
                class: "",
                max_width: "max-w-7xl",
                padded: true,
                background: "bg-base-100",
            },
        );

        assert!(html.contains("Page Title"));
        assert!(html.contains("text-3xl"));
        assert!(html.contains("font-bold"));
    }

    /// Test BasePage subtitle field
    #[test]
    fn test_subtitle_field() {
        let html = render_component(
            BasePage,
            BasePageProps {
                title: Some("Title".to_string()),
                subtitle: Some("Page subtitle".to_string()),
                children: rsx! { "Content" },
                header_actions: None,
                breadcrumbs: None,
                class: "",
                max_width: "max-w-7xl",
                padded: true,
                background: "bg-base-100",
            },
        );

        assert!(html.contains("Page subtitle"));
        assert!(html.contains("text-base-content/70"));
    }

    /// Test BasePage breadcrumbs field
    #[test]
    fn test_breadcrumbs_field() {
        let html = render_component(
            BasePage,
            BasePageProps {
                title: Some("Title".to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                header_actions: None,
                breadcrumbs: Some(rsx! {
                    nav { class: "breadcrumbs",
                        ul {
                            li { a { "Home" } }
                            li { "Current" }
                        }
                    }
                }),
                class: "",
                max_width: "max-w-7xl",
                padded: true,
                background: "bg-base-100",
            },
        );

        assert!(html.contains("breadcrumbs"));
        assert!(html.contains("Home"));
        assert!(html.contains("Current"));
    }

    /// Test BasePage header_actions field
    #[test]
    fn test_header_actions_field() {
        let html = render_component(
            BasePage,
            BasePageProps {
                title: Some("Title".to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                header_actions: Some(rsx! {
                    button { class: "btn btn-primary", "Action" }
                }),
                breadcrumbs: None,
                class: "",
                max_width: "max-w-7xl",
                padded: true,
                background: "bg-base-100",
            },
        );

        assert!(html.contains("Action"));
        assert!(html.contains("flex-shrink-0"));
    }

    /// Test BasePage max_width field
    #[test]
    fn test_max_width_field() {
        let html = render_component(
            BasePage,
            BasePageProps {
                title: Some("Title".to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                header_actions: None,
                breadcrumbs: None,
                class: "",
                max_width: "max-w-4xl",
                padded: true,
                background: "bg-base-100",
            },
        );

        assert!(html.contains("max-w-4xl"));
    }

    /// Test BasePage background field
    #[test]
    fn test_background_field() {
        let html = render_component(
            BasePage,
            BasePageProps {
                title: Some("Title".to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                header_actions: None,
                breadcrumbs: None,
                class: "",
                max_width: "max-w-7xl",
                padded: true,
                background: "bg-base-200",
            },
        );

        assert!(html.contains("bg-base-200"));
    }
}

/// Centralized test suite for ProgressRing component
#[cfg(test)]
mod progress_ring_tests {
    use super::*;

    /// Test ProgressRing value field
    #[test]
    fn test_value_field() {
        let html = render_component(
            ProgressRing,
            course_pilot::ui::components::progress::ProgressRingProps {
                value: 75,
                max: Some(100),
                color: Some("primary".to_string()),
                size: Some(48),
                thickness: Some(4),
                label: None,
            },
        );

        assert!(html.contains("75%"));
        assert!(html.contains("radial-progress"));
    }

    /// Test ProgressRing color field
    #[test]
    fn test_color_field() {
        let colors = vec!["primary", "secondary", "accent", "success"];
        
        for color in colors {
            let html = render_component(
                ProgressRing,
                course_pilot::ui::components::progress::ProgressRingProps {
                    value: 50,
                    max: Some(100),
                    color: Some(color.to_string()),
                    size: Some(48),
                    thickness: Some(4),
                    label: None,
                },
            );

            assert!(html.contains(&format!("text-{}", color)));
        }
    }

    /// Test ProgressRing with custom label
    #[test]
    fn test_label_field() {
        let html = render_component(
            ProgressRing,
            course_pilot::ui::components::progress::ProgressRingProps {
                value: 100,
                max: Some(100),
                color: Some("success".to_string()),
                size: Some(48),
                thickness: Some(4),
                label: Some(rsx! { span { "Done" } }),
            },
        );

        assert!(html.contains("Done"));
        assert!(!html.contains("100%")); // Should show label instead of percentage
    }
}

/// Centralized test suite for ProgressBar component
#[cfg(test)]
mod progress_bar_tests {
    use super::*;

    /// Test ProgressBar value field
    #[test]
    fn test_value_field() {
        let html = render_component(
            ProgressBar,
            course_pilot::ui::components::progress::ProgressBarProps {
                value: 60,
                label: None,
                color: None,
                class: None,
            },
        );

        assert!(html.contains("60%"));
        assert!(html.contains("progress"));
    }

    /// Test ProgressBar label field
    #[test]
    fn test_label_field() {
        let html = render_component(
            ProgressBar,
            course_pilot::ui::components::progress::ProgressBarProps {
                value: 80,
                label: Some("Loading...".to_string()),
                color: None,
                class: None,
            },
        );

        assert!(html.contains("Loading..."));
        assert!(html.contains("80%"));
    }

    /// Test ProgressBar color field
    #[test]
    fn test_color_field() {
        let html = render_component(
            ProgressBar,
            course_pilot::ui::components::progress::ProgressBarProps {
                value: 90,
                label: None,
                color: Some("success".to_string()),
                class: None,
            },
        );

        assert!(html.contains("progress-success"));
    }
}

/// Centralized test suite for Badge component
#[cfg(test)]
mod badge_tests {
    use super::*;

    /// Test Badge label field
    #[test]
    fn test_label_field() {
        let html = render_component(
            Badge,
            course_pilot::ui::components::modal::BadgeProps {
                label: "Test Badge".to_string(),
                color: None,
                class: None,
            },
        );

        assert!(html.contains("Test Badge"));
        assert!(html.contains("badge"));
    }

    /// Test Badge color field
    #[test]
    fn test_color_field() {
        let colors = vec!["primary", "secondary", "success", "warning", "error"];
        
        for color in colors {
            let html = render_component(
                Badge,
                course_pilot::ui::components::modal::BadgeProps {
                    label: "Badge".to_string(),
                    color: Some(color.to_string()),
                    class: None,
                },
            );

            assert!(html.contains(&format!("badge-{}", color)));
        }
    }

    /// Test Badge class field
    #[test]
    fn test_class_field() {
        let html = render_component(
            Badge,
            course_pilot::ui::components::modal::BadgeProps {
                label: "Custom Badge".to_string(),
                color: Some("primary".to_string()),
                class: Some("badge-lg custom-class".to_string()),
            },
        );

        assert!(html.contains("badge-lg"));
        assert!(html.contains("custom-class"));
    }
}

/// Test suite for component edge cases and error handling
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    /// Test components with empty strings
    #[test]
    fn test_empty_strings() {
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: Some("".to_string()),
                subtitle: Some("".to_string()),
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(html.contains("Content"));
    }

    /// Test components with very long strings
    #[test]
    fn test_long_strings() {
        let long_title = "A".repeat(200);
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: Some(long_title.clone()),
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        assert!(html.contains(&long_title));
        assert!(html.contains("Content"));
    }

    /// Test components with special characters
    #[test]
    fn test_special_characters() {
        let special_title = "Test & <script>alert('xss')</script> \"quotes\" 'apostrophes'";
        let html = render_component(
            BaseCard,
            BaseCardProps {
                title: Some(special_title.to_string()),
                subtitle: None,
                children: rsx! { "Content" },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: None,
                header_actions: None,
            },
        );

        // HTML should be properly escaped
        assert!(!html.contains("<script>"));
        assert!(html.contains("Content"));
    }

    /// Test progress components with boundary values
    #[test]
    fn test_progress_boundary_values() {
        // Test 0% progress
        let html = render_component(
            ProgressRing,
            course_pilot::ui::components::progress::ProgressRingProps {
                value: 0,
                max: Some(100),
                color: Some("primary".to_string()),
                size: Some(48),
                thickness: Some(4),
                label: None,
            },
        );

        assert!(html.contains("0%"));

        // Test 100% progress
        let html = render_component(
            ProgressRing,
            course_pilot::ui::components::progress::ProgressRingProps {
                value: 100,
                max: Some(100),
                color: Some("success".to_string()),
                size: Some(48),
                thickness: Some(4),
                label: None,
            },
        );

        assert!(html.contains("100%"));
    }
}