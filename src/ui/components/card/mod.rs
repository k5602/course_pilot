//! Enhanced Card Component with Unified Theme System
//!
//! This module provides a comprehensive card component that follows modern
//! design principles and integrates seamlessly with the unified theme system.
//!
//! Features:
//! - Multiple card variants (elevated, outlined, filled)
//! - Flexible composition with header, content, and actions
//! - Full accessibility support
//! - Hover and focus interactions
//! - Loading and disabled states
//! - Image and media support
//! - Responsive design
//! - Theme-aware styling

use dioxus::prelude::*;

/// Card variant styles
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardVariant {
    Elevated,
    Outlined,
    Filled,
}

impl Default for CardVariant {
    fn default() -> Self {
        Self::Elevated
    }
}

impl CardVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Elevated => "card--elevated",
            Self::Outlined => "card--outlined",
            Self::Filled => "card--filled",
        }
    }
}

/// Card size variants
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardSize {
    Small,
    Medium,
    Large,
}

impl Default for CardSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl CardSize {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Small => "card--sm",
            Self::Medium => "card--md",
            Self::Large => "card--lg",
        }
    }
}

/// Card interaction behavior
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardInteraction {
    None,
    Clickable,
    Hoverable,
}

impl Default for CardInteraction {
    fn default() -> Self {
        Self::None
    }
}

/// Main card component properties
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Card content
    pub children: Element,

    /// Card variant
    #[props(optional, default = CardVariant::Elevated)]
    pub variant: CardVariant,

    /// Card size
    #[props(optional, default = CardSize::Medium)]
    pub size: CardSize,

    /// Interaction behavior
    #[props(optional, default = CardInteraction::None)]
    pub interaction: CardInteraction,

    /// Whether the card is clickable
    #[props(optional, default = false)]
    pub clickable: bool,

    /// Click event handler
    #[props(optional)]
    pub onclick: Option<EventHandler<()>>,

    /// Whether the card is disabled
    #[props(optional, default = false)]
    pub disabled: bool,

    /// Whether the card is in loading state
    #[props(optional, default = false)]
    pub loading: bool,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,

    /// Accessibility label
    #[props(optional)]
    pub aria_label: Option<String>,

    /// Accessibility description
    #[props(optional)]
    pub aria_describedby: Option<String>,

    /// Role for accessibility
    #[props(optional)]
    pub role: Option<String>,

    /// Tab index for keyboard navigation
    #[props(optional)]
    pub tabindex: Option<i32>,

    /// Custom data attributes
    #[props(optional)]
    pub data_testid: Option<String>,

    /// Tooltip text
    #[props(optional)]
    pub title: Option<String>,
}

/// Card header component properties
#[derive(Props, Clone, PartialEq)]
pub struct CardHeaderProps {
    /// Header content
    pub children: Element,

    /// Header title
    #[props(optional)]
    pub title: Option<String>,

    /// Header subtitle
    #[props(optional)]
    pub subtitle: Option<String>,

    /// Header avatar/icon
    #[props(optional)]
    pub avatar: Option<Element>,

    /// Header action button
    #[props(optional)]
    pub action: Option<Element>,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,
}

/// Card content component properties
#[derive(Props, Clone, PartialEq)]
pub struct CardContentProps {
    /// Content
    pub children: Element,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,

    /// Whether content should be padded
    #[props(optional, default = true)]
    pub padded: bool,
}

/// Card actions component properties
#[derive(Props, Clone, PartialEq)]
pub struct CardActionsProps {
    /// Actions content
    pub children: Element,

    /// Actions alignment
    #[props(optional, default = "end".to_string())]
    pub align: String,

    /// Whether actions are full width
    #[props(optional, default = false)]
    pub full_width: bool,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,
}

/// Card media component properties
#[derive(Props, Clone, PartialEq)]
pub struct CardMediaProps {
    /// Media source (image URL)
    pub src: String,

    /// Alt text for image
    pub alt: String,

    /// Media height
    #[props(optional, default = "200px".to_string())]
    pub height: String,

    /// Object fit behavior
    #[props(optional, default = "cover".to_string())]
    pub object_fit: String,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,

    /// Loading behavior
    #[props(optional, default = "lazy".to_string())]
    pub loading: String,
}

/// Main Card Component
#[component]
pub fn Card(props: CardProps) -> Element {
    let CardProps {
        children,
        variant,
        size,
        interaction,
        clickable,
        onclick,
        disabled,
        loading,
        class,
        aria_label,
        aria_describedby,
        role,
        tabindex,
        data_testid,
        title,
    } = props;

    // Build CSS classes
    let mut classes = vec!["card", variant.as_class(), size.as_class()];

    let is_interactive = clickable
        || onclick.is_some()
        || matches!(
            interaction,
            CardInteraction::Clickable | CardInteraction::Hoverable
        );

    if is_interactive {
        classes.push("card--interactive");
    }

    if disabled {
        classes.push("card--disabled");
    }

    if loading {
        classes.push("card--loading");
    }

    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }

    let class_string = classes.join(" ");

    // Handle keyboard events for interactive cards
    let handle_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        let is_space = matches!(key, Key::Character(ref s) if s == " ");
        if is_interactive && !disabled && (key == Key::Enter || is_space) {
            if let Some(handler) = &onclick {
                handler.call(());
            }
        }
    };

    // Handle click events
    let handle_click = move |_| {
        if is_interactive && !disabled && !loading {
            if let Some(handler) = &onclick {
                handler.call(());
            }
        }
    };

    // Determine the appropriate HTML element
    let element_role = if is_interactive {
        role.as_deref().unwrap_or("button")
    } else {
        role.as_deref().unwrap_or("article")
    };

    rsx! {
        // Include CSS styles
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/card/style.css")
        }

        div {
            class: class_string,
            role: element_role,

            // Accessibility attributes
            aria_label: aria_label.unwrap_or_default(),
            aria_describedby: aria_describedby.unwrap_or_default(),
            aria_disabled: if is_interactive { disabled.to_string() } else { String::new() },
            aria_busy: loading.to_string(),

            // Navigation attributes
            tabindex: if is_interactive && !disabled { tabindex.unwrap_or(0) } else { -1 },

            // Event handlers
            onclick: handle_click,
            onkeydown: handle_keydown,

            // Meta attributes
            title: title.unwrap_or_default(),
            "data-testid": data_testid.unwrap_or_default(),

            // Loading overlay
            if loading {
                div { class: "card__loading-overlay",
                    div { class: "card__spinner",
                        aria_hidden: "true",
                        role: "status"
                    }
                    span { class: "sr-only", "Loading..." }
                }
            }

            // Card content
            {children}
        }
    }
}

/// Card Header Component
#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let CardHeaderProps {
        children,
        title,
        subtitle,
        avatar,
        action,
        class,
    } = props;

    let mut classes = vec!["card__header"];
    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }
    let class_string = classes.join(" ");

    rsx! {
        div { class: class_string,
            // Avatar section
            if let Some(avatar_elem) = avatar {
                div { class: "card__header-avatar",
                    {avatar_elem}
                }
            }

            // Text content
            div { class: "card__header-content",
                if let Some(title_text) = title {
                    h3 { class: "card__header-title", "{title_text}" }
                }
                if let Some(subtitle_text) = subtitle {
                    p { class: "card__header-subtitle", "{subtitle_text}" }
                }
                {children}
            }

            // Action section
            if let Some(action_elem) = action {
                div { class: "card__header-action",
                    {action_elem}
                }
            }
        }
    }
}

/// Card Content Component
#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    let CardContentProps {
        children,
        class,
        padded,
    } = props;

    let mut classes = vec!["card__content"];
    if !padded {
        classes.push("card__content--no-padding");
    }
    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }
    let class_string = classes.join(" ");

    rsx! {
        div { class: class_string,
            {children}
        }
    }
}

/// Card Actions Component
#[component]
pub fn CardActions(props: CardActionsProps) -> Element {
    let CardActionsProps {
        children,
        align,
        full_width,
        class,
    } = props;

    let mut classes = vec!["card__actions"];
    let align_class = format!("card__actions--{}", align);
    classes.push(&align_class);

    if full_width {
        classes.push("card__actions--full-width");
    }

    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }
    let class_string = classes.join(" ");

    rsx! {
        div { class: class_string,
            {children}
        }
    }
}

/// Card Media Component
#[component]
pub fn CardMedia(props: CardMediaProps) -> Element {
    let CardMediaProps {
        src,
        alt,
        height,
        object_fit,
        class,
        loading,
    } = props;

    let mut classes = vec!["card__media"];
    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }
    let class_string = classes.join(" ");

    rsx! {
        div { class: class_string,
            style: format!("height: {};", height),
            img {
                src: src,
                alt: alt,
                loading: loading,
                style: format!("object-fit: {}; width: 100%; height: 100%;", object_fit),
                draggable: "false"
            }
        }
    }
}

/// Utility function to create a simple card with title and content
#[component]
pub fn SimpleCard(
    title: String,
    children: Element,
    #[props(optional)] variant: Option<CardVariant>,
    #[props(optional)] size: Option<CardSize>,
    #[props(optional)] onclick: Option<EventHandler<()>>,
    #[props(optional)] class: Option<String>,
) -> Element {
    rsx! {
        Card {
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            clickable: onclick.is_some(),
            onclick,
            class,

            CardHeader {
                title: Some(title)
            }

            CardContent {
                {children}
            }
        }
    }
}

/// Utility function to create an action card with buttons
#[component]
pub fn ActionCard(
    title: String,
    children: Element,
    actions: Element,
    #[props(optional)] variant: Option<CardVariant>,
    #[props(optional)] size: Option<CardSize>,
    #[props(optional)] class: Option<String>,
) -> Element {
    rsx! {
        Card {
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            class,

            CardHeader {
                title: Some(title)
            }

            CardContent {
                {children}
            }

            CardActions {
                {actions}
            }
        }
    }
}

/// Utility function to create a media card with image
#[component]
pub fn MediaCard(
    title: String,
    image_src: String,
    image_alt: String,
    children: Element,
    #[props(optional)] variant: Option<CardVariant>,
    #[props(optional)] size: Option<CardSize>,
    #[props(optional)] onclick: Option<EventHandler<()>>,
    #[props(optional)] actions: Option<Element>,
    #[props(optional)] class: Option<String>,
) -> Element {
    rsx! {
        Card {
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            clickable: onclick.is_some(),
            onclick,
            class,

            CardMedia {
                src: image_src,
                alt: image_alt
            }

            CardHeader {
                title: Some(title)
            }

            CardContent {
                {children}
            }

            if let Some(action_elements) = actions {
                CardActions {
                    {action_elements}
                }
            }
        }
    }
}

/// Demo component showcasing all card variants and compositions
#[component]
pub(crate) fn Demo() -> Element {
    let mut interactive_count = use_signal(|| 0);

    let increment_count = move |_| {
        interactive_count.set(interactive_count() + 1);
    };

    rsx! {
        div {
            style: "padding: 2rem; max-width: 1200px; margin: 0 auto;",

            h2 { "Card Component Demo" }

            // Basic card variants
            section {
                h3 { "Card Variants" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    SimpleCard {
                        title: "Elevated Card".to_string(),
                        variant: Some(CardVariant::Elevated),
                        p { "This is an elevated card with shadow elevation." }
                    }

                    SimpleCard {
                        title: "Outlined Card".to_string(),
                        variant: Some(CardVariant::Outlined),
                        p { "This is an outlined card with border styling." }
                    }

                    SimpleCard {
                        title: "Filled Card".to_string(),
                        variant: Some(CardVariant::Filled),
                        p { "This is a filled card with background color." }
                    }
                }
            }

            // Card sizes
            section {
                h3 { "Card Sizes" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    SimpleCard {
                        title: "Small Card".to_string(),
                        size: Some(CardSize::Small),
                        p { "Compact card size." }
                    }

                    SimpleCard {
                        title: "Medium Card".to_string(),
                        size: Some(CardSize::Medium),
                        p { "Standard card size." }
                    }

                    SimpleCard {
                        title: "Large Card".to_string(),
                        size: Some(CardSize::Large),
                        p { "Spacious card size." }
                    }
                }
            }

            // Interactive cards
            section {
                h3 { "Interactive Cards" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    SimpleCard {
                        title: "Clickable Card".to_string(),
                        onclick: increment_count,
                        p {
                            "Click me! Count: {interactive_count()}"
                            br {}
                            "This card responds to clicks and keyboard navigation."
                        }
                    }

                    Card {
                        variant: CardVariant::Outlined,
                        loading: true,

                        CardHeader {
                            title: Some("Loading Card".to_string())
                        }

                        CardContent {
                            p { "This card is in a loading state." }
                        }
                    }

                    Card {
                        variant: CardVariant::Elevated,
                        disabled: true,

                        CardHeader {
                            title: Some("Disabled Card".to_string())
                        }

                        CardContent {
                            p { "This card is disabled and non-interactive." }
                        }
                    }
                }
            }

            // Complex card compositions
            section {
                h3 { "Complex Card Compositions" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    // Card with header, content, and actions
                    ActionCard {
                        title: "Action Card".to_string(),
                        variant: Some(CardVariant::Elevated),
                        actions: rsx! {
                            button { style: "margin-right: 0.5rem;", "Cancel" }
                            button { style: "background: var(--color-primary); color: white; border: none; padding: 0.5rem 1rem; border-radius: 0.25rem;", "Save" }
                        },

                        p { "This card includes action buttons at the bottom." }
                        p { "Actions are commonly used for forms or confirmation dialogs." }
                    }

                    // Card with avatar and action
                    Card {
                        variant: CardVariant::Outlined,

                        CardHeader {
                            title: Some("User Profile".to_string()),
                            subtitle: Some("Software Engineer".to_string()),
                            avatar: rsx! {
                                div {
                                    style: "width: 40px; height: 40px; border-radius: 50%; background: var(--color-primary); display: flex; align-items: center; justify-content: center; color: white; font-weight: bold;",
                                    "JD"
                                }
                            },
                            action: rsx! {
                                button {
                                    style: "background: none; border: none; cursor: pointer; font-size: 1.25rem;",
                                    "⋮"
                                }
                            }
                        }

                        CardContent {
                            p { "John Doe is a senior software engineer with expertise in web development and system architecture." }
                        }

                        CardActions {
                            button { style: "margin-right: 0.5rem;", "Message" }
                            button { style: "background: var(--color-primary); color: white; border: none; padding: 0.5rem 1rem; border-radius: 0.25rem;", "Connect" }
                        }
                    }

                    // Media card example
                    MediaCard {
                        title: "Beautiful Landscape".to_string(),
                        image_src: "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&h=200&fit=crop".to_string(),
                        image_alt: "Mountain landscape".to_string(),
                        variant: Some(CardVariant::Elevated),
                        actions: rsx! {
                            button { style: "margin-right: 0.5rem;", "Share" }
                            button { style: "background: var(--color-primary); color: white; border: none; padding: 0.5rem 1rem; border-radius: 0.25rem;", "Download" }
                        },

                        p { "A stunning mountain landscape captured during golden hour." }
                        p { "This media card showcases how to combine images with content and actions." }
                    }
                }
            }

            // Card without padding
            section {
                h3 { "Custom Layouts" }
                div { style: "max-width: 400px; margin-bottom: 2rem;",

                    Card {
                        variant: CardVariant::Outlined,

                        CardContent {
                            padded: false,
                            div { style: "padding: 1rem; border-bottom: 1px solid var(--border-primary);",
                                h4 { style: "margin: 0;", "Custom Layout" }
                            }
                            div { style: "padding: 1rem;",
                                p { style: "margin: 0;", "This card uses custom padding and borders for a unique layout." }
                            }
                        }

                        CardActions {
                            full_width: true,
                            button { style: "width: 100%; padding: 1rem; background: var(--color-primary); color: white; border: none; cursor: pointer;", "Full Width Action" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_variant_classes() {
        assert_eq!(CardVariant::Elevated.as_class(), "card--elevated");
        assert_eq!(CardVariant::Outlined.as_class(), "card--outlined");
        assert_eq!(CardVariant::Filled.as_class(), "card--filled");
    }

    #[test]
    fn test_card_size_classes() {
        assert_eq!(CardSize::Small.as_class(), "card--sm");
        assert_eq!(CardSize::Medium.as_class(), "card--md");
        assert_eq!(CardSize::Large.as_class(), "card--lg");
    }

    #[test]
    fn test_defaults() {
        assert_eq!(CardVariant::default(), CardVariant::Elevated);
        assert_eq!(CardSize::default(), CardSize::Medium);
        assert_eq!(CardInteraction::default(), CardInteraction::None);
    }
}
