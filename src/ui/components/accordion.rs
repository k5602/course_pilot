use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;

/// Single accordion item (title + content + optional icon)
#[derive(Clone, PartialEq)]
pub struct AccordionItem {
    pub title: String,
    pub content: Element,
    pub icon: Option<Element>,
}

/// DaisyUI Accordion/Collapse component (Dioxus idioms, 0.6+)
/// - Supports optional icons per item
/// - Animates expand/collapse with dioxus-motion
#[component]
pub fn Accordion(
    items: Vec<AccordionItem>,
    #[props(optional)] default_open: Option<usize>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let mut open_idx = use_signal(|| default_open.unwrap_or(usize::MAX));
    let container_class = class.as_deref().unwrap_or("join join-vertical w-full");

    // Animation state for all items (indexed by idx)
    let scales: Vec<_> = (0..items.len()).map(|_| use_motion(0.98f32)).collect();
    let opacities: Vec<_> = (0..items.len()).map(|_| use_motion(0.0f32)).collect();

    // Run use_effect for each item at the top level
    for idx in 0..items.len() {
        let mut scale = scales[idx].clone();
        let mut opacity = opacities[idx].clone();
        let open_idx = open_idx.clone();
        use_effect(move || {
            if open_idx() == idx {
                scale.animate_to(
                    1.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring {
                        stiffness: 120.0,
                        damping: 16.0,
                        mass: 1.0,
                        velocity: 0.0,
                    })),
                );
                opacity.animate_to(
                    1.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring::default())),
                );
            } else {
                scale.reset();
                opacity.reset();
            }
        });
    }

    // Build all item elements in a Vec<Element>
    let item_elements: Vec<Element> = items.iter().enumerate().map(|(idx, item)| {
        let content_style = format!(
            "transform: scale({}); opacity: {}; transition: transform 0.2s, opacity 0.2s;",
            scales[idx].get_value(),
            opacities[idx].get_value()
        );
        rsx! {
            div {
                class: "collapse collapse-arrow bg-base-200",
                key: "{idx}",
                input {
                    r#type: "checkbox",
                    checked: open_idx() == idx,
                    onclick: move |_| open_idx.set(if open_idx() == idx { usize::MAX } else { idx }),
                    class: "peer",
                }
                div {
                    class: "collapse-title text-lg font-medium cursor-pointer select-none flex items-center gap-2",
                    if let Some(icon) = &item.icon {
                        {icon.clone()}
                    }
                    "{item.title}"
                }
                div {
                    class: "collapse-content",
                    style: "{content_style}",
                    { if open_idx() == idx { item.content.clone() } else { rsx!() } }
                }
            }
        }
    }).collect::<Vec<_>>();

    rsx! {
        div {
            class: "{container_class}",
            {item_elements.into_iter()}
        }
    }
}
