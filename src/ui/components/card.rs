use dioxus::prelude::*;

/// DaisyUI Card component.
/// - Applies DaisyUI card classes.
/// - Supports title, subtitle, footer, and custom content.
#[component]
pub fn Card(
    #[props(optional)] class: Option<String>,
    #[props(optional)] title: Option<String>,
    #[props(optional)] subtitle: Option<String>,
    #[props(optional)] footer: Option<Element>,
    children: Element,
) -> Element {
    let class = class.as_deref().unwrap_or("");
    rsx! {
        div {
            class: "card bg-base-100 shadow-md {class}",
            div {
                class: "card-body",
                if let Some(title) = &title {
                    h2 { class: "card-title", "{title}" }
                }
                if let Some(subtitle) = &subtitle {
                    p { class: "text-base-content/70 text-sm mb-2", "{subtitle}" }
                }
                {children}
            }
            if let Some(footer) = &footer {
                div { class: "card-actions justify-end p-4 pt-0", {footer.clone()} }
            }
        }
    }
}
