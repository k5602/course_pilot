use dioxus::prelude::*;

#[derive(PartialEq, Clone, Copy)]
pub enum ContentSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Props, PartialEq, Clone)]
pub struct HoverCardProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn HoverCard(props: HoverCardProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: "position: relative; display: inline-block;",
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct HoverCardTriggerProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn HoverCardTrigger(props: HoverCardTriggerProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            tabindex: "0",
            style: "cursor: pointer;",
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct HoverCardContentProps {
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub side: Option<ContentSide>,
    pub children: Element,
}

#[component]
pub fn HoverCardContent(props: HoverCardContentProps) -> Element {
    let side_style = match props.side.unwrap_or(ContentSide::Top) {
        ContentSide::Top => "bottom: 100%; left: 50%; transform: translateX(-50%);",
        ContentSide::Bottom => "top: 100%; left: 50%; transform: translateX(-50%);",
        ContentSide::Left => "right: 100%; top: 50%; transform: translateY(-50%);",
        ContentSide::Right => "left: 100%; top: 50%; transform: translateY(-50%);",
    };
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: format!("position: absolute; min-width: 200px; background: white; border: 1px solid #ddd; border-radius: 6px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); padding: 1rem; z-index: 100; {}", side_style),
            {props.children}
        }
    }
}

#[component]
pub(super) fn Demo() -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("/src/components/hover_card/style.css"),
        }
        div {
            style: "padding: 50px; display: flex; flex-direction: row; flex-wrap: wrap; gap: 40px; justify-content: center; align-items: center;",
            HoverCard { class: Some("hover-card".to_string()),
                HoverCardTrigger { class: Some("hover-card-trigger".to_string()),
                    i { "Dioxus" }
                }
                HoverCardContent { class: Some("hover-card-content".to_string()), side: Some(ContentSide::Bottom),
                    div {
                        padding: "1rem",
                        "Dioxus is"
                        i { " the " }
                        "Rust framework for building fullstack web, desktop, and mobile apps. Iterate with live hotreloading, add server functions, and deploy in record time."
                    }
                }
            }
        }
    }
}
