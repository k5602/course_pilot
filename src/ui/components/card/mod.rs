use dioxus::prelude::*;
use dioxus_motion::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let mut scale = use_motion(1.0f32);

    rsx! {
        div {
            class: "card",
            style: format!("transform: scale({}); transition: transform 0.15s cubic-bezier(0.4,0,0.2,1);", scale.get_value()),
            onmouseenter: move |_| {
                scale.animate_to(
                    1.03,
                    AnimationConfig::new(AnimationMode::Spring(Spring {
                        stiffness: 200.0,
                        damping: 15.0,
                        mass: 1.0,
                        velocity: 0.0,
                    }))
                );
            },
            onmouseleave: move |_| {
                scale.animate_to(
                    1.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring {
                        stiffness: 200.0,
                        damping: 15.0,
                        mass: 1.0,
                        velocity: 0.0,
                    }))
                );
            },
            onclick: move |evt| {
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },
            {props.children}
        }
    }
}

// Demo for Card usage
#[component]
pub(super) fn Demo() -> Element {
    rsx! {
        div {
            Card {
                onclick: move |_| {
                    println!("Card clicked!");
                },
                div {
                    h3 { "Clickable Card" }
                    p { "This card responds to clicks and shows hover effects." }
                    p { "Click me to see the interaction!" }
                    div {
                        style: "margin-top: 0.5rem; font-size: 0.875rem; color: #666;"
                    }
                }
            }
        }
    }
}
