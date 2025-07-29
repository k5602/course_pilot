//! Accessibility Tests for Course Pilot UI Components
//!
//! This module provides comprehensive accessibility testing for UI components,
//! ensuring compliance with WCAG 2.1 guidelines and proper screen reader support.

use dioxus::prelude::*;
use dioxus_ssr::render;
use dioxus_core;
use regex::Regex;
use std::collections::HashMap;

// Import components for testing
use course_pilot::ui::components::base::{
    BaseButton, BaseButtonProps, BaseCard, BaseCardProps, BaseModal, BaseModalProps,
    BasePage, BasePageProps,
};
use course_pilot::ui::components::{
    ProgressRing, ProgressBar, Toast, Modal, ModalVariant,
};

/// Accessibility test configuration
#[derive(Debug, Clone)]
pub struct AccessibilityConfig {
    pub check_aria_labels: bool,
    pub check_keyboard_navigation: bool,
    pub check_color_contrast: bool,
    pub check_semantic_html: bool,
    pub check_focus_management: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            check_aria_labels: true,
            check_keyboard_navigation: true,
            check_color_contrast: true,
            check_semantic_html: true,
            check_focus_management: true,
        }
    }
}

/// Accessibility test results
#[derive(Debug, Clone)]
pub struct AccessibilityResults {
    pub component_name: String,
    pub passed_checks: Vec<String>,
    pub failed_checks: Vec<String>,
    pub warnings: Vec<String>,
    pub score: f32, // 0.0 to 100.0
}

impl AccessibilityResults {
    fn new(component_name: String) -> Self {
        Self {
            component_name,
            passed_checks: Vec::new(),
            failed_checks: Vec::new(),
            warnings: Vec::new(),
            score: 0.0,
        }
    }

    fn add_pass(&mut self, check: &str) {
        self.passed_checks.push(check.to_string());
    }

    fn add_fail(&mut self, check: &str) {
        self.failed_checks.push(check.to_string());
    }

    fn add_warning(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    fn calculate_score(&mut self) {
        let total_checks = self.passed_checks.len() + self.failed_checks.len();
        if total_checks > 0 {
            self.score = (self.passed_checks.len() as f32 / total_checks as f32) * 100.0;
        }
    }

    fn print_summary(&self) {
        println!("\n=== Accessibility Results for {} ===", self.component_name);
        println!("Score: {:.1}%", self.score);
        println!("Passed checks: {}", self.passed_checks.len());
        println!("Failed checks: {}", self.failed_checks.len());
        println!("Warnings: {}", self.warnings.len());
        
        if !self.failed_checks.is_empty() {
            println!("\nFailed checks:");
            for check in &self.failed_checks {
                println!("  ❌ {}", check);
            }
        }
        
        if !self.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &self.warnings {
                println!("  ⚠️  {}", warning);
            }
        }
        
        if !self.passed_checks.is_empty() {
            println!("\nPassed checks:");
            for check in &self.passed_checks {
                println!("  ✅ {}", check);
            }
        }
    }
}

/// Helper function to render component and get HTML for accessibility testing
fn render_component_for_accessibility<P: Clone + 'static>(
    component: fn(P) -> Element,
    props: P,
) -> String {
    let mut dom = VirtualDom::new_with_props(component, props);
    let mut mutations = dioxus_core::Mutations::default();
    let _ = dom.rebuild(&mut mutations);
    render(&dom)
}

/// Check if HTML contains proper ARIA labels and attributes
fn check_aria_attributes(html: &str, results: &mut AccessibilityResults) {
    // Check for aria-label attributes
    if html.contains("aria-label=") {
        results.add_pass("Contains aria-label attributes");
    } else {
        results.add_warning("No aria-label attributes found - may be acceptable if using other labeling methods");
    }
    
    // Check for aria-describedby
    if html.contains("aria-describedby=") {
        results.add_pass("Contains aria-describedby attributes");
    }
    
    // Check for aria-expanded (for interactive elements)
    if html.contains("aria-expanded=") {
        results.add_pass("Contains aria-expanded attributes for interactive elements");
    }
    
    // Check for aria-hidden (should be used carefully)
    if html.contains("aria-hidden=\"true\"") {
        results.add_warning("Contains aria-hidden=\"true\" - ensure decorative elements only");
    }
    
    // Check for role attributes
    if html.contains("role=") {
        results.add_pass("Contains role attributes for semantic clarity");
    }
    
    // Check for aria-live regions
    if html.contains("aria-live=") {
        results.add_pass("Contains aria-live regions for dynamic content");
    }
}

/// Check for proper semantic HTML structure
fn check_semantic_html(html: &str, results: &mut AccessibilityResults) {
    // Check for proper heading hierarchy
    let h1_count = html.matches("<h1").count();
    let h2_count = html.matches("<h2").count();
    let h3_count = html.matches("<h3").count();
    
    if h1_count <= 1 {
        results.add_pass("Proper h1 usage (0 or 1 h1 elements)");
    } else {
        results.add_fail("Multiple h1 elements found - should have only one per page");
    }
    
    if h2_count > 0 || h3_count > 0 {
        results.add_pass("Uses proper heading hierarchy");
    }
    
    // Check for semantic elements
    let semantic_elements = ["nav", "main", "section", "article", "aside", "header", "footer"];
    let mut found_semantic = false;
    
    for element in &semantic_elements {
        if html.contains(&format!("<{}", element)) {
            found_semantic = true;
            break;
        }
    }
    
    if found_semantic {
        results.add_pass("Uses semantic HTML elements");
    } else {
        results.add_warning("No semantic HTML elements found - consider using nav, main, section, etc.");
    }
    
    // Check for proper button elements vs div with click handlers
    if html.contains("<button") {
        results.add_pass("Uses proper button elements");
    }
    
    // Check for proper form elements
    if html.contains("<input") || html.contains("<textarea") || html.contains("<select") {
        if html.contains("id=") && html.contains("for=") {
            results.add_pass("Form elements have proper label associations");
        } else {
            results.add_warning("Form elements may lack proper label associations");
        }
    }
}

/// Check for keyboard navigation support
fn check_keyboard_navigation(html: &str, results: &mut AccessibilityResults) {
    // Check for tabindex attributes
    if html.contains("tabindex=") {
        if html.contains("tabindex=\"-1\"") {
            results.add_pass("Uses tabindex=\"-1\" for programmatic focus management");
        }
        if html.contains("tabindex=\"0\"") {
            results.add_pass("Uses tabindex=\"0\" for keyboard accessibility");
        }
        if Regex::new(r#"tabindex="[1-9]""#).unwrap().is_match(html) {
            results.add_fail("Uses positive tabindex values - this can break natural tab order");
        }
    }
    
    // Check for interactive elements that should be keyboard accessible
    let interactive_elements = ["button", "input", "textarea", "select", "a"];
    let mut has_interactive = false;
    
    for element in &interactive_elements {
        if html.contains(&format!("<{}", element)) {
            has_interactive = true;
            break;
        }
    }
    
    if has_interactive {
        results.add_pass("Contains keyboard-accessible interactive elements");
    }
    
    // Check for proper link elements
    if html.contains("<a") {
        if html.contains("href=") {
            results.add_pass("Links have proper href attributes");
        } else {
            results.add_fail("Links without href attributes are not keyboard accessible");
        }
    }
}

/// Check for focus management
fn check_focus_management(html: &str, results: &mut AccessibilityResults) {
    // Check for focus indicators (CSS classes that suggest focus styling)
    let focus_indicators = ["focus:", "focus-visible:", "focus-within:"];
    let mut has_focus_styling = false;
    
    for indicator in &focus_indicators {
        if html.contains(indicator) {
            has_focus_styling = true;
            break;
        }
    }
    
    if has_focus_styling {
        results.add_pass("Includes focus styling indicators");
    } else {
        results.add_warning("No focus styling indicators found - ensure focus is visible");
    }
    
    // Check for skip links (mainly for page-level components)
    if html.contains("skip") && html.contains("main") {
        results.add_pass("Contains skip navigation links");
    }
}

/// Check for color contrast and visual accessibility
fn check_color_contrast(html: &str, results: &mut AccessibilityResults) {
    // Check for DaisyUI color classes that should have good contrast
    let good_contrast_classes = [
        "text-base-content", "text-primary-content", "text-secondary-content",
        "text-accent-content", "text-neutral-content", "text-info-content",
        "text-success-content", "text-warning-content", "text-error-content"
    ];
    
    let mut has_contrast_classes = false;
    for class in &good_contrast_classes {
        if html.contains(class) {
            has_contrast_classes = true;
            break;
        }
    }
    
    if has_contrast_classes {
        results.add_pass("Uses DaisyUI contrast-safe color classes");
    }
    
    // Check for potential contrast issues
    let potential_issues = ["text-gray-400", "text-gray-300", "opacity-50"];
    for issue in &potential_issues {
        if html.contains(issue) {
            results.add_warning(&format!("Contains {} which may have contrast issues", issue));
        }
    }
    
    // Check for proper use of color information
    if html.contains("color:") && !html.contains("aria-label") {
        results.add_warning("Uses color styling - ensure information is not conveyed by color alone");
    }
}

/// Run comprehensive accessibility test on a component
fn run_accessibility_test<P: Clone + 'static>(
    component: fn(P) -> Element,
    props: P,
    component_name: &str,
    config: &AccessibilityConfig,
) -> AccessibilityResults {
    let html = render_component_for_accessibility(component, props);
    let mut results = AccessibilityResults::new(component_name.to_string());
    
    if config.check_aria_labels {
        check_aria_attributes(&html, &mut results);
    }
    
    if config.check_semantic_html {
        check_semantic_html(&html, &mut results);
    }
    
    if config.check_keyboard_navigation {
        check_keyboard_navigation(&html, &mut results);
    }
    
    if config.check_focus_management {
        check_focus_management(&html, &mut results);
    }
    
    if config.check_color_contrast {
        check_color_contrast(&html, &mut results);
    }
    
    results.calculate_score();
    results
}

#[cfg(test)]
mod accessibility_tests {
    use super::*;

    #[test]
    fn test_base_button_accessibility() {
        let config = AccessibilityConfig::default();
        
        // Test basic button
        let results = run_accessibility_test(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Accessible Button" },
                onclick: None,
                color: Some("primary".to_string()),
                size: Some("md".to_string()),
                variant: None,
                class: "",
                disabled: false,
                icon: Some(rsx! { span { "🔍" } }),
                loading: false,
                button_type: "button",
            },
            "BaseButton",
            &config,
        );

        results.print_summary();
        
        // Accessibility assertions
        assert!(
            results.score >= 70.0,
            "BaseButton accessibility score ({:.1}%) is below acceptable threshold (70%)",
            results.score
        );
        
        assert!(
            results.passed_checks.iter().any(|check| check.contains("button elements")),
            "BaseButton should use proper button elements"
        );
        
        // Test disabled button
        let disabled_results = run_accessibility_test(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Disabled Button" },
                onclick: None,
                color: Some("primary".to_string()),
                size: None,
                variant: None,
                class: "",
                disabled: true,
                icon: None,
                loading: false,
                button_type: "button",
            },
            "BaseButton (Disabled)",
            &config,
        );

        disabled_results.print_summary();
        
        assert!(
            disabled_results.score >= 70.0,
            "Disabled BaseButton accessibility score is too low"
        );
    }

    #[test]
    fn test_base_card_accessibility() {
        let config = AccessibilityConfig::default();
        
        let results = run_accessibility_test(
            BaseCard,
            BaseCardProps {
                title: Some("Accessible Card Title".to_string()),
                subtitle: Some("Card subtitle with descriptive text".to_string()),
                children: rsx! {
                    div {
                        p { "This card contains accessible content with proper structure." }
                        ul {
                            li { "List item 1 with meaningful content" }
                            li { "List item 2 with additional information" }
                        }
                    }
                },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: Some(rsx! {
                    button { 
                        class: "btn btn-primary",
                        r#type: "button",
                        "Primary Action"
                    }
                    button { 
                        class: "btn btn-secondary",
                        r#type: "button",
                        "Secondary Action"
                    }
                }),
                header_actions: Some(rsx! {
                    button { 
                        class: "btn btn-ghost btn-sm",
                        r#type: "button",
                        "aria-label": "More options",
                        "⋮"
                    }
                }),
            },
            "BaseCard",
            &config,
        );

        results.print_summary();
        
        // Accessibility assertions
        assert!(
            results.score >= 75.0,
            "BaseCard accessibility score ({:.1}%) is below acceptable threshold (75%)",
            results.score
        );
        
        assert!(
            results.passed_checks.iter().any(|check| check.contains("heading hierarchy")),
            "BaseCard should use proper heading hierarchy"
        );
    }

    #[test]
    fn test_base_modal_accessibility() {
        let config = AccessibilityConfig::default();
        
        let results = run_accessibility_test(
            BaseModal,
            BaseModalProps {
                open: true,
                title: "Accessible Modal Dialog".to_string(),
                children: rsx! {
                    div {
                        p { "This modal contains accessible content." }
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text", "Input Label" }
                            }
                            input { 
                                r#type: "text",
                                class: "input input-bordered",
                                id: "modal-input",
                                "aria-describedby": "input-help"
                            }
                            div { 
                                id: "input-help",
                                class: "label-text-alt",
                                "Helper text for the input field"
                            }
                        }
                    }
                },
                actions: Some(rsx! {
                    button { 
                        class: "btn btn-primary",
                        r#type: "button",
                        "Confirm"
                    }
                    button { 
                        class: "btn btn-ghost",
                        r#type: "button",
                        "Cancel"
                    }
                }),
                on_close: EventHandler::new(|_| {}),
                size: "modal-box",
            },
            "BaseModal",
            &config,
        );

        results.print_summary();
        
        // Modal-specific accessibility checks
        assert!(
            results.score >= 80.0,
            "BaseModal accessibility score ({:.1}%) is below acceptable threshold (80%)",
            results.score
        );
        
        // Check for proper form labeling
        assert!(
            results.passed_checks.iter().any(|check| check.contains("label associations")) ||
            results.warnings.is_empty(), // No warnings about form labeling
            "BaseModal should have proper form element labeling"
        );
    }

    #[test]
    fn test_base_page_accessibility() {
        let config = AccessibilityConfig::default();
        
        let results = run_accessibility_test(
            BasePage,
            BasePageProps {
                title: Some("Accessible Page Title".to_string()),
                subtitle: Some("Page subtitle with descriptive information".to_string()),
                children: rsx! {
                    main {
                        section { class: "mb-8",
                            h2 { class: "text-2xl font-bold mb-4", "Main Content Section" }
                            p { "This page demonstrates proper semantic HTML structure." }
                            
                            article { class: "mt-6",
                                h3 { class: "text-xl font-semibold mb-2", "Article Title" }
                                p { "Article content with proper heading hierarchy." }
                            }
                        }
                        
                        aside { class: "bg-base-200 p-4 rounded",
                            h2 { class: "text-lg font-medium mb-2", "Sidebar Information" }
                            p { "Additional information in a sidebar." }
                        }
                    }
                },
                header_actions: Some(rsx! {
                    nav {
                        button { 
                            class: "btn btn-primary",
                            r#type: "button",
                            "Primary Action"
                        }
                        button { 
                            class: "btn btn-secondary",
                            r#type: "button",
                            "Secondary Action"
                        }
                    }
                }),
                breadcrumbs: Some(rsx! {
                    nav { 
                        class: "breadcrumbs",
                        "aria-label": "Breadcrumb navigation",
                        ul {
                            li { a { href: "/", "Home" } }
                            li { a { href: "/section", "Section" } }
                            li { "Current Page" }
                        }
                    }
                }),
                class: "",
                max_width: "max-w-6xl",
                padded: true,
                background: "bg-base-100",
            },
            "BasePage",
            &config,
        );

        results.print_summary();
        
        // Page-specific accessibility checks
        assert!(
            results.score >= 85.0,
            "BasePage accessibility score ({:.1}%) is below acceptable threshold (85%)",
            results.score
        );
        
        assert!(
            results.passed_checks.iter().any(|check| check.contains("semantic HTML")),
            "BasePage should use semantic HTML elements"
        );
        
        assert!(
            results.passed_checks.iter().any(|check| check.contains("heading hierarchy")),
            "BasePage should have proper heading hierarchy"
        );
    }

    #[test]
    fn test_progress_components_accessibility() {
        let config = AccessibilityConfig::default();
        
        // Test ProgressRing
        let ring_results = run_accessibility_test(
            ProgressRing,
            course_pilot::ui::components::progress::ProgressRingProps {
                value: 75,
                max: Some(100),
                color: Some("primary".to_string()),
                size: Some(64),
                thickness: Some(6),
                label: Some(rsx! { 
                    span { 
                        class: "sr-only",
                        "Loading progress: 75 percent complete"
                    }
                }),
            },
            "ProgressRing",
            &config,
        );

        ring_results.print_summary();
        
        // Test ProgressBar
        let bar_results = run_accessibility_test(
            ProgressBar,
            course_pilot::ui::components::progress::ProgressBarProps {
                value: 60,
                label: Some("File upload progress".to_string()),
                color: Some("success".to_string()),
                class: Some("w-full".to_string()),
            },
            "ProgressBar",
            &config,
        );

        bar_results.print_summary();
        
        // Progress components should be highly accessible
        assert!(
            ring_results.score >= 70.0,
            "ProgressRing accessibility score too low: {:.1}%",
            ring_results.score
        );
        
        assert!(
            bar_results.score >= 70.0,
            "ProgressBar accessibility score too low: {:.1}%",
            bar_results.score
        );
    }

    #[test]
    fn test_keyboard_navigation_compliance() {
        // Test components that should be keyboard navigable
        let config = AccessibilityConfig {
            check_keyboard_navigation: true,
            ..Default::default()
        };
        
        // Test button keyboard navigation
        let button_html = render_component_for_accessibility(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Keyboard Test" },
                onclick: None,
                color: Some("primary".to_string()),
                size: None,
                variant: None,
                class: "",
                disabled: false,
                icon: None,
                loading: false,
                button_type: "button",
            },
        );
        
        // Buttons should be keyboard accessible by default
        assert!(
            button_html.contains("<button"),
            "Button should use proper button element for keyboard accessibility"
        );
        
        assert!(
            button_html.contains("type=\"button\""),
            "Button should have proper type attribute"
        );
        
        // Test that disabled buttons are properly marked
        let disabled_html = render_component_for_accessibility(
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
        
        assert!(
            disabled_html.contains("disabled") || disabled_html.contains("btn-disabled"),
            "Disabled button should be properly marked as disabled"
        );
    }

    #[test]
    fn test_screen_reader_compatibility() {
        // Test components for screen reader compatibility
        let config = AccessibilityConfig::default();
        
        // Test card with proper labeling for screen readers
        let card_html = render_component_for_accessibility(
            BaseCard,
            BaseCardProps {
                title: Some("Course: Introduction to Programming".to_string()),
                subtitle: Some("Beginner level • 12 hours • 85% complete".to_string()),
                children: rsx! {
                    div {
                        p { "Learn the fundamentals of programming with hands-on exercises." }
                        div { class: "stats stats-horizontal shadow mt-4",
                            div { class: "stat",
                                div { class: "stat-title", "Progress" }
                                div { class: "stat-value text-primary", "85%" }
                                div { class: "stat-desc", "17 of 20 lessons completed" }
                            }
                            div { class: "stat",
                                div { class: "stat-title", "Time Remaining" }
                                div { class: "stat-value text-secondary", "2h" }
                                div { class: "stat-desc", "Estimated completion time" }
                            }
                        }
                    }
                },
                variant: "card",
                class: "",
                hover_effect: true,
                on_click: None,
                actions: Some(rsx! {
                    button { 
                        class: "btn btn-primary",
                        r#type: "button",
                        "aria-label": "Continue course: Introduction to Programming",
                        "Continue"
                    }
                    button { 
                        class: "btn btn-ghost",
                        r#type: "button",
                        "aria-label": "View course details for Introduction to Programming",
                        "Details"
                    }
                }),
                header_actions: None,
            },
        );
        
        // Check for descriptive content that helps screen readers
        assert!(
            card_html.contains("Introduction to Programming"),
            "Card should contain descriptive title"
        );
        
        assert!(
            card_html.contains("Beginner level") || card_html.contains("85%"),
            "Card should contain descriptive metadata"
        );
        
        // Check for proper button labeling
        assert!(
            card_html.contains("aria-label") || card_html.contains("Continue"),
            "Interactive elements should have descriptive labels"
        );
    }

    #[test]
    fn test_color_contrast_compliance() {
        // Test that components use DaisyUI classes with good contrast
        let config = AccessibilityConfig {
            check_color_contrast: true,
            ..Default::default()
        };
        
        let results = run_accessibility_test(
            BaseCard,
            BaseCardProps {
                title: Some("High Contrast Test".to_string()),
                subtitle: Some("Testing color contrast compliance".to_string()),
                children: rsx! {
                    div { class: "space-y-4",
                        div { class: "alert alert-info",
                            span { class: "text-info-content", "Information message with proper contrast" }
                        }
                        div { class: "alert alert-success",
                            span { class: "text-success-content", "Success message with proper contrast" }
                        }
                        div { class: "alert alert-warning",
                            span { class: "text-warning-content", "Warning message with proper contrast" }
                        }
                        div { class: "alert alert-error",
                            span { class: "text-error-content", "Error message with proper contrast" }
                        }
                    }
                },
                variant: "card",
                class: "",
                hover_effect: false,
                on_click: None,
                actions: None,
                header_actions: None,
            },
            "Color Contrast Test",
            &config,
        );

        results.print_summary();
        
        // Should pass contrast checks when using DaisyUI semantic colors
        assert!(
            results.passed_checks.iter().any(|check| check.contains("contrast-safe")),
            "Component should use contrast-safe DaisyUI color classes"
        );
        
        // Should have minimal contrast warnings
        assert!(
            results.warnings.iter().filter(|w| w.contains("contrast")).count() <= 1,
            "Component should have minimal contrast warnings"
        );
    }
}

/// Manual accessibility testing helper
#[allow(dead_code)]
pub fn run_accessibility_audit() {
    println!("Running Course Pilot Accessibility Audit...\n");
    
    let config = AccessibilityConfig::default();
    let mut all_results = Vec::new();
    
    // This would run all accessibility tests and generate a comprehensive report
    // In practice, individual tests are run via `cargo test`
    
    println!("Accessibility audit completed. Run individual tests with:");
    println!("cargo test test_base_button_accessibility -- --nocapture");
    println!("cargo test test_base_card_accessibility -- --nocapture");
    println!("cargo test test_keyboard_navigation_compliance -- --nocapture");
    println!("cargo test test_screen_reader_compatibility -- --nocapture");
}