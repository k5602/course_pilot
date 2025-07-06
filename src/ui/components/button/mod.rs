//! Enhanced Button Component with Unified Theme System and Accessibility
//!
//! This module provides a comprehensive button component that follows modern
//! accessibility standards and integrates seamlessly with the unified theme system.
//!
//! Features:
//! - Consistent API with proper type safety
//! - Full accessibility support (ARIA, keyboard navigation)
//! - Loading states and disabled states
//! - Icon support with proper positioning
//! - Responsive design with mobile-first approach
//! - Theme-aware styling with CSS variables
//! - Performance optimized with minimal re-renders

use dioxus::events::Key;
use dioxus::prelude::*;

/// Button size variants
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl ButtonSize {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Small => "button--sm",
            Self::Medium => "button--md",
            Self::Large => "button--lg",
        }
    }
}

/// Button style variants
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

impl ButtonVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Primary => "button--primary",
            Self::Secondary => "button--secondary",
            Self::Outline => "button--outline",
            Self::Ghost => "button--ghost",
            Self::Destructive => "button--destructive",
        }
    }
}

/// Button type for semantic HTML
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonType {
    Button,
    Submit,
    Reset,
}

impl Default for ButtonType {
    fn default() -> Self {
        Self::Button
    }
}

impl ButtonType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Submit => "submit",
            Self::Reset => "reset",
        }
    }
}

/// Icon position within button
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IconPosition {
    Left,
    Right,
    Only,
}

/// Button component properties
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Button content (text, icons, etc.)
    pub children: Element,

    /// Click event handler
    #[props(optional)]
    pub onclick: Option<EventHandler<()>>,

    /// Button style variant
    #[props(optional, default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,

    /// Button size
    #[props(optional, default = ButtonSize::Medium)]
    pub size: ButtonSize,

    /// Button type for form submission
    #[props(optional, default = ButtonType::Button)]
    pub button_type: ButtonType,

    /// Whether the button is disabled
    #[props(optional, default = false)]
    pub disabled: bool,

    /// Whether the button is in loading state
    #[props(optional, default = false)]
    pub loading: bool,

    /// Whether the button should take full width
    #[props(optional, default = false)]
    pub full_width: bool,

    /// Icon element to display
    #[props(optional)]
    pub icon: Option<Element>,

    /// Position of the icon
    #[props(optional, default = IconPosition::Left)]
    pub icon_position: IconPosition,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,

    /// Accessibility label for screen readers
    #[props(optional)]
    pub aria_label: Option<String>,

    /// Accessibility description
    #[props(optional)]
    pub aria_describedby: Option<String>,

    /// Whether button is pressed (for toggle buttons)
    #[props(optional)]
    pub aria_pressed: Option<bool>,

    /// Tab index for keyboard navigation
    #[props(optional)]
    pub tabindex: Option<i32>,

    /// Form ID this button belongs to
    #[props(optional)]
    pub form: Option<String>,

    /// Custom data attributes
    #[props(optional)]
    pub data_testid: Option<String>,

    /// Tooltip text
    #[props(optional)]
    pub title: Option<String>,
}

/// Enhanced Button Component
#[component]
pub fn Button(props: ButtonProps) -> Element {
    // Destructure props for easier access
    let ButtonProps {
        children,
        onclick,
        variant,
        size,
        button_type,
        disabled,
        loading,
        full_width,
        icon,
        icon_position,
        class,
        aria_label,
        aria_describedby,
        aria_pressed,
        tabindex,
        form,
        data_testid,
        title,
    } = props;

    // Build CSS classes
    let mut classes = vec!["button", variant.as_class(), size.as_class()];

    if full_width {
        classes.push("button--full-width");
    }

    if loading {
        classes.push("button--loading");
    }

    if icon.is_some() && matches!(icon_position, IconPosition::Only) {
        classes.push("button--icon-only");
    }

    // Add custom classes
    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }

    let class_string = classes.join(" ");

    // Determine if button should be disabled
    let is_disabled = disabled || loading;

    // Handle keyboard navigation
    let handle_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        let is_space = matches!(key, Key::Character(ref s) if s == " ");
        if key == Key::Enter || is_space {
            if !is_disabled {
                if let Some(handler) = &onclick {
                    handler.call(());
                }
            }
        }
    };

    // Handle click events
    let handle_click = move |_| {
        if !is_disabled {
            if let Some(handler) = &onclick {
                handler.call(());
            }
        }
    };

    // Render icon with proper positioning
    let render_icon = |position: IconPosition| {
        if let Some(icon_elem) = &icon {
            if icon_position == position {
                let icon_class = match position {
                    IconPosition::Left => "button__icon button__icon--left",
                    IconPosition::Right => "button__icon button__icon--right",
                    IconPosition::Only => "button__icon",
                };

                rsx! {
                    span { class: icon_class, {icon_elem.clone()} }
                }
            } else {
                rsx! {}
            }
        } else {
            rsx! {}
        }
    };

    // Render loading spinner
    let render_spinner = || {
        if loading {
            rsx! {
                span {
                    class: "button__spinner",
                    aria_hidden: "true",
                    role: "status"
                }
            }
        } else {
            rsx! {}
        }
    };

    // Render button content
    let render_content = || {
        if matches!(icon_position, IconPosition::Only) {
            rsx! {
                span { class: "button__content", {render_icon(IconPosition::Only)} }
            }
        } else {
            rsx! {
                span { class: "button__content",
                    {render_icon(IconPosition::Left)}
                    {children}
                    {render_icon(IconPosition::Right)}
                }
            }
        }
    };

    rsx! {
        // Include CSS styles
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/button/style.css")
        }

        button {
            // Core attributes
            r#type: button_type.as_str(),
            class: class_string,
            disabled: is_disabled,

            // Event handlers
            onclick: handle_click,
            onkeydown: handle_keydown,

            // Accessibility attributes
            aria_label: aria_label.unwrap_or_default(),
            aria_describedby: aria_describedby.unwrap_or_default(),
            aria_pressed: aria_pressed.map(|p| p.to_string()).unwrap_or_default(),
            aria_disabled: is_disabled.to_string(),
            aria_busy: loading.to_string(),

            // Form attributes
            form: form.unwrap_or_default(),

            // Navigation attributes
            tabindex: tabindex.unwrap_or(0),

            // Meta attributes
            title: title.unwrap_or_default(),
            "data-testid": data_testid.unwrap_or_default(),

            // Render content
            {render_content()}
            {render_spinner()}
        }

        // Additional CSS for full-width and responsive behavior
        style { dangerous_inner_html: r#"
            .button--full-width {{
                width: 100%;
            }}

            .button__content {{
                display: flex;
                align-items: center;
                justify-content: center;
                gap: inherit;
            }}

            .button--loading .button__content {{
                opacity: 0;
            }}

            /* High contrast mode support */
            @media (prefers-contrast: high) {{
                .button {{
                    border-width: 2px;
                }}

                .button:focus-visible {{
                    outline: 3px solid currentColor;
                    outline-offset: 2px;
                }}
            }}

            /* Reduced motion support */
            @media (prefers-reduced-motion: reduce) {{
                .button {{
                    transition: none;
                }}

                .button:hover {{
                    transform: none;
                }}

                .button:active {{
                    transform: none;
                }}

                .button__spinner {{
                    animation: none;
                }}
            }}
        "# }
    }
}

/// Button Group Component for grouping related buttons
#[derive(Props, Clone, PartialEq)]
pub struct ButtonGroupProps {
    pub children: Element,

    #[props(optional, default = false)]
    pub vertical: bool,

    #[props(optional, default = false)]
    pub connected: bool,

    #[props(optional)]
    pub class: Option<String>,

    #[props(optional)]
    pub aria_label: Option<String>,

    #[props(optional)]
    pub role: Option<String>,
}

#[component]
pub fn ButtonGroup(props: ButtonGroupProps) -> Element {
    let ButtonGroupProps {
        children,
        vertical,
        connected,
        class,
        aria_label,
        role,
    } = props;

    let mut classes = vec!["button-group"];

    if vertical {
        classes.push("button-group--vertical");
    }

    if connected {
        classes.push("button-group--connected");
    }

    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }

    let class_string = classes.join(" ");

    rsx! {
        div {
            class: class_string,
            role: role.as_deref().unwrap_or("group"),
            aria_label: aria_label.unwrap_or_default(),
            {children}
        }
    }
}

/// Icon Button - specialized button for icon-only use cases
#[component]
pub fn IconButton(
    icon: Element,
    #[props(optional)] onclick: Option<EventHandler<()>>,
    #[props(optional, default = ButtonVariant::Ghost)] variant: ButtonVariant,
    #[props(optional, default = ButtonSize::Medium)] size: ButtonSize,
    #[props(optional, default = false)] disabled: bool,
    #[props(optional, default = false)] loading: bool,
    #[props(optional)] aria_label: Option<String>,
    #[props(optional)] class: Option<String>,
    #[props(optional)] title: Option<String>,
) -> Element {
    rsx! {
        Button {
            variant,
            size,
            disabled,
            loading,
            icon: Some(icon),
            icon_position: IconPosition::Only,
            onclick,
            aria_label: aria_label.clone(),
            class,
            title,
            span { class: "sr-only", {aria_label.as_ref().map(|s| s.as_str()).unwrap_or("Button")} }
        }
    }
}

/// Loading Button - button with built-in loading state management
#[component]
pub fn LoadingButton(
    children: Element,
    #[props(optional)] onclick: Option<EventHandler<()>>,
    #[props(optional, default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(optional, default = ButtonSize::Medium)] size: ButtonSize,
    #[props(optional, default = false)] loading: bool,
    #[props(optional)] loading_text: Option<String>,
    #[props(optional)] class: Option<String>,
    #[props(optional)] aria_label: Option<String>,
) -> Element {
    rsx! {
        Button {
            variant,
            size,
            loading,
            onclick,
            class,
            aria_label,
            if loading && loading_text.is_some() {
                {loading_text.unwrap()}
            } else {
                {children}
            }
        }
    }
}

/// Utility function to create a submit button
#[component]
pub fn SubmitButton(
    children: Element,
    #[props(optional)] onclick: Option<EventHandler<()>>,
    #[props(optional, default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(optional, default = ButtonSize::Medium)] size: ButtonSize,
    #[props(optional, default = false)] disabled: bool,
    #[props(optional, default = false)] loading: bool,
    #[props(optional)] form: Option<String>,
    #[props(optional)] class: Option<String>,
) -> Element {
    rsx! {
        Button {
            button_type: ButtonType::Submit,
            variant,
            size,
            disabled,
            loading,
            onclick,
            form,
            class,
            {children}
        }
    }
}

/// Demo component showcasing all button variants and states
#[component]
pub(crate) fn Demo() -> Element {
    let mut loading_state = use_signal(|| false);
    let mut disabled_state = use_signal(|| false);

    let toggle_loading = move |_| {
        loading_state.set(!loading_state());
    };

    let toggle_disabled = move |_| {
        disabled_state.set(!disabled_state());
    };

    rsx! {
        div {
            style: "padding: 2rem; max-width: 800px; margin: 0 auto;",

            h2 { "Button Component Demo" }

            // Variant showcase
            section {
                h3 { "Variants" }
                ButtonGroup {
                    aria_label: "Button variants",
                    Button { variant: ButtonVariant::Primary, "Primary" }
                    Button { variant: ButtonVariant::Secondary, "Secondary" }
                    Button { variant: ButtonVariant::Outline, "Outline" }
                    Button { variant: ButtonVariant::Ghost, "Ghost" }
                    Button { variant: ButtonVariant::Destructive, "Destructive" }
                }
            }

            // Size showcase
            section {
                h3 { "Sizes" }
                ButtonGroup {
                    aria_label: "Button sizes",
                    Button { size: ButtonSize::Small, "Small" }
                    Button { size: ButtonSize::Medium, "Medium" }
                    Button { size: ButtonSize::Large, "Large" }
                }
            }

            // State showcase
            section {
                h3 { "States" }
                ButtonGroup {
                    aria_label: "Button states",
                    Button {
                        loading: *loading_state.read(),
                        onclick: toggle_loading,
                        if *loading_state.read() { "Loading..." } else { "Toggle Loading" }
                    }
                    Button {
                        disabled: *disabled_state.read(),
                        onclick: toggle_disabled,
                        "Toggle Disabled"
                    }
                    Button {
                        disabled: *disabled_state.read(),
                        if *disabled_state.read() { "Disabled" } else { "Enabled" }
                    }
                }
            }

            // Icon buttons
            section {
                h3 { "Icon Buttons" }
                ButtonGroup {
                    aria_label: "Icon buttons",
                    IconButton {
                        icon: rsx! { span { "🔍" } },
                        aria_label: "Search",
                        title: "Search"
                    }
                    IconButton {
                        icon: rsx! { span { "⚙️" } },
                        aria_label: "Settings",
                        title: "Settings"
                    }
                    IconButton {
                        icon: rsx! { span { "❤️" } },
                        aria_label: "Favorite",
                        title: "Add to favorites"
                    }
                }
            }

            // Full width button
            section {
                h3 { "Full Width" }
                Button {
                    full_width: true,
                    "Full Width Button"
                }
            }

            // Connected button group
            section {
                h3 { "Connected Button Group" }
                ButtonGroup {
                    connected: true,
                    aria_label: "Text formatting",
                    Button { variant: ButtonVariant::Outline, "Bold" }
                    Button { variant: ButtonVariant::Outline, "Italic" }
                    Button { variant: ButtonVariant::Outline, "Underline" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_size_classes() {
        assert_eq!(ButtonSize::Small.as_class(), "button--sm");
        assert_eq!(ButtonSize::Medium.as_class(), "button--md");
        assert_eq!(ButtonSize::Large.as_class(), "button--lg");
    }

    #[test]
    fn test_button_variant_classes() {
        assert_eq!(ButtonVariant::Primary.as_class(), "button--primary");
        assert_eq!(ButtonVariant::Secondary.as_class(), "button--secondary");
        assert_eq!(ButtonVariant::Outline.as_class(), "button--outline");
        assert_eq!(ButtonVariant::Ghost.as_class(), "button--ghost");
        assert_eq!(ButtonVariant::Destructive.as_class(), "button--destructive");
    }

    #[test]
    fn test_button_type_strings() {
        assert_eq!(ButtonType::Button.as_str(), "button");
        assert_eq!(ButtonType::Submit.as_str(), "submit");
        assert_eq!(ButtonType::Reset.as_str(), "reset");
    }

    #[test]
    fn test_defaults() {
        assert_eq!(ButtonSize::default(), ButtonSize::Medium);
        assert_eq!(ButtonVariant::default(), ButtonVariant::Primary);
        assert_eq!(ButtonType::default(), ButtonType::Button);
    }
}
