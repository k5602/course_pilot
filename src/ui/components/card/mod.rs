use dioxus::events::Key;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub onclick: Option<EventHandler>,
    #[props(default = "filled".to_string())]
    pub variant: String, // "filled" | "outlined"
    #[props(default = 1)]
    pub elevation: u8, // 0-4
    #[props(optional)]
    pub color: Option<String>,
    #[props(optional)]
    pub background: Option<String>,
    /// Optional status badge (e.g., "Structured", "Unstructured")
    #[props(optional)]
    pub status_badge: Option<Element>,
    /// Optional meta/info row (e.g., modules, duration, difficulty)
    #[props(optional)]
    pub meta_row: Option<Element>,
    /// Optional sample content row
    #[props(optional)]
    pub sample_row: Option<Element>,
    /// Optional actions row (primary/secondary actions)
    #[props(optional)]
    pub actions_row: Option<Element>,
    // The children prop now correctly uses dioxus::prelude::Element
    pub children: Element,
}

#[component]
// The function now correctly returns dioxus::prelude::Element
pub fn Card(props: CardProps) -> Element {
    let is_clickable = props.onclick.is_some();
    let elevation = props.elevation.clamp(0, 4);

    let card_class = format!(
        "card card--{} card--elevation-{} {}",
        props.variant,
        elevation,
        if is_clickable { "card--clickable" } else { "" }
    );

    let mut style_str = String::new();
    if let Some(bg) = &props.background {
        style_str.push_str(&format!("--card-custom-bg: {};", bg));
    }
    if let Some(c) = &props.color {
        style_str.push_str(&format!("--card-custom-color: {};", c));
    }

    rsx! {
        div {
            class: "{card_class}",
            style: "{style_str}",
            tabindex: if is_clickable { "0" } else { "-1" },
            role: if is_clickable { "button" } else { "region" },
            aria_label: if is_clickable { Some("Course card, clickable") } else { Some("Course card") },
            aria_pressed: None::<bool>,
            aria_selected: None::<bool>,

            onclick: move |_| {
                if let Some(handler) = &props.onclick {
                    handler.call(());
                }
            },
            onkeydown: move |evt| {
                if is_clickable && (evt.key() == Key::Enter || evt.key() == Key::Character(" ".to_string())) {
                    if let Some(handler) = &props.onclick {
                        handler.call(());
                    }
                }
            },

            // Card content wrapper for spacing and vertical rhythm
            div {
                class: "card-content-visual",
                // Title row with status badge
                div {
                    class: "card-title-row",
                    h3 {
                        class: "card-title",
                        tabindex: "0",
                        aria_label: "Course title",
                        {props.children}
                    }
                    // Status badge (if any)
                    if let Some(badge) = &props.status_badge {
                        span {
                            role: "status",
                            aria_label: "Course status",
                            tabindex: "0",
                            style: "outline: none;",
                            {badge.clone()}
                        }
                    },
                }
                // Meta/info row (if any)
                if let Some(meta) = &props.meta_row {
                    div {
                        class: "card-meta-row",
                        role: "list",
                        aria_label: "Course meta information",
                        tabindex: "0",
                        style: "outline: none;",
                        {meta.clone()}
                    }
                },
                // Sample content row (if any)
                if let Some(sample) = &props.sample_row {
                    div {
                        class: "card-sample-row",
                        aria_label: "Sample course content",
                        tabindex: "0",
                        style: "outline: none;",
                        {sample.clone()}
                    }
                },
                // Actions row (if any)
                if let Some(actions) = &props.actions_row {
                    div {
                        class: "card-actions-row",
                        role: "group",
                        aria_label: "Course actions",
                        tabindex: "0",
                        style: "outline: none;",
                        {actions.clone()}
                    }
                },
            }
        }
    }
}

#[component]
// The function now correctly returns dioxus::prelude::Element
pub(super) fn Demo() -> Element {
    let mut is_dark = use_signal(|| false);

    rsx! {
        div {
            class: if is_dark() { "dark" } else { "" },
            div {
                class: "card-demo-container",
                div {
                    class: "demo-header",
                    h2 { "Elegant Card Components" },
                    button {
                        class: "theme-toggle",
                        onclick: move |_| is_dark.set(!is_dark()),
                        if is_dark() { "Switch to Light Mode" } else { "Switch to Dark Mode" }
                    }
                }

                div {
                    class: "card-demo-grid",
                    Card {
                        variant: "filled",
                        elevation: 2,
                        onclick: move |_| { println!("Filled Card clicked!"); },
                        div {
                            class: "card-content",
                            h3 { "Interactive Card" },
                            p { "Hover over me to see the magnetic glow effect." }
                        }
                    },
                    Card {
                        variant: "filled",
                        elevation: 2,
                        onclick: move |_| { println!("Image Card clicked!"); },
                        div {
                            class: "card-media",
                            img {
                                src: "https://placehold.co/600x400/6b73ff/FFFFFF?text=Dioxus",
                                alt: "Placeholder Image"
                            }
                        }
                        div {
                            class: "card-content",
                            h3 { "Card with Media" },
                            p { "Cards can seamlessly include images and other media." }
                        }
                    },
                    Card {
                        variant: "outlined",
                        elevation: 0,
                        div {
                            class: "card-content",
                            h3 { "Card with Actions" },
                            p { "The footer is perfect for buttons and links." }
                        }
                        div {
                            class: "card-actions",
                            button { class: "action-button", "Details" },
                            button { class: "action-button primary", "Confirm" }
                        }
                    }
                }
            }
        }
    }
}
