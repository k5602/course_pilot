use dioxus::prelude::*;

/// A simple, reusable dropdown component for Dioxus.
/// - `label`: The button label (RSX/Element).
/// - `children`: The dropdown menu items (RSX/Element).
/// - `open`: Optional, controls open state externally (default: false).
/// - `class`: Optional, additional classes for the container.
#[component]
pub fn AppDropdown(
    label: Element,
    children: Element,
    #[props(optional)] open: Option<bool>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let mut is_open = use_signal(|| open.unwrap_or(false));

    let container_class = format!("relative inline-block {}", class.as_deref().unwrap_or(""));

    rsx! {
        div { class: "{container_class}",
            button {
                class: "btn btn-sm btn-outline flex items-center gap-2",
                onclick: move |_| is_open.set(!is_open()),
                {label.clone()}
                span { class: "ml-1", "▼" }
            }
            { if is_open() {
                rsx! {
                    div {
                        class: "absolute z-50 mt-2 w-48 rounded-md shadow-lg bg-base-200 ring-1 ring-black ring-opacity-5 focus:outline-none",
                        style: "min-width: 10rem;",
                        {children.clone()}
                    }
                }
            } else { rsx!{} } }
        }
    }
}
