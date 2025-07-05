use dioxus::events::{Key, MouseEvent};
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

            // onmousemove: move |evt: MouseEvent| {
            //     if is_clickable {
            //         // We explicitly dereference evt.data to access the methods on the underlying MouseData.
            //         if let Some(element) = (*evt.data).target_element() {
            //             if let Ok(rect) = element.get_bounding_client_rect() {
            //                 let x = (*evt.data).client_x() as f64 - rect.left();
            //                 let y = (*evt.data).client_y() as f64 - rect.top();

            //                 let current_style = element.get_attribute("style").unwrap_or_default();
            //                 let base_style = current_style.split(" --mouse-x").next().unwrap_or(&current_style);
            //                 let new_style = format!(
            //                     "{} --mouse-x: {}px; --mouse-y: {}px;",
            //                     base_style.trim(),
            //                     x,
            //                     y
            //                 );
            //                 let _ = element.set_attribute("style", &new_style);
            //             }
            //         }
            //     }
            // },

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

            {props.children}
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
