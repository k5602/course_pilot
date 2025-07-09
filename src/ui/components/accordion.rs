use dioxus::prelude::*;

/// Single accordion item (title + content)
#[derive(Clone, PartialEq)]
pub struct AccordionItem {
    pub title: String,
    pub content: Element,
}

/// DaisyUI Accordion/Collapse component (Dioxus idioms, 0.6+)
#[component]
pub fn Accordion(
    items: Vec<AccordionItem>,
    #[props(optional)] default_open: Option<usize>,
    #[props(optional)] class: Option<String>,
) -> Element {
    // use_signal is the idiomatic state hook in Dioxus 0.6+
    let mut open_idx = use_signal(|| default_open.unwrap_or(usize::MAX));
    let container_class = class.as_deref().unwrap_or("join join-vertical w-full");

    rsx! {
        div {
            class: "{container_class}",
            for (idx, item) in items.iter().enumerate() {
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
                        class: "collapse-title text-lg font-medium cursor-pointer select-none",
                        "{item.title}"
                    }
                    div {
                        class: "collapse-content",
                        { if open_idx() == idx { item.content.clone() } else { rsx!() } }
                    }
                }
            }
        }
    }
}
