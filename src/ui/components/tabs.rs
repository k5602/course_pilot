use dioxus::prelude::*;

/// DaisyUI Tabs component (Dioxus idioms, 0.6+)
#[component]
pub fn Tabs(
    tabs: Vec<String>,
    selected: usize,
    on_select: EventHandler<usize>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let class = class.as_deref().unwrap_or("tabs tabs-boxed");

    rsx! {
        div { class: "{class}",
            {
                tabs.iter().enumerate().map(|(idx, label)| {
                    let tab_class = if idx == selected {
                        "tab tab-active"
                    } else {
                        "tab"
                    };
                    rsx! {
                        button {
                            key: "{idx}",
                            class: "{tab_class}",
                            onclick: move |_| on_select.call(idx),
                            "{label}"
                        }
                    }
                })
            }
        }
    }
}
