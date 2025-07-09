use dioxus::prelude::*;

/// DaisyUI-based reusable Button component.
/// - All props are function arguments per Dioxus 0.6+ idioms.
/// - Children are passed as RSX content.
#[component]
pub fn Button(
    /// Optional click handler.
    #[props(optional)]
    onclick: Option<EventHandler<MouseEvent>>,
    /// DaisyUI color (primary, secondary, accent, etc.)
    #[props(optional)]
    color: Option<String>,
    /// DaisyUI size (sm, md, lg)
    #[props(optional)]
    size: Option<String>,
    /// DaisyUI variant (outline, ghost, link, etc.)
    #[props(optional)]
    variant: Option<String>,
    /// Additional Tailwind/DaisyUI classes.
    #[props(optional)]
    class: Option<String>,
    /// Disabled state.
    #[props(optional)]
    disabled: Option<bool>,
    /// Button label or content.
    children: Element,
) -> Element {
    let mut classes = vec!["btn".to_string()];
    if let Some(color) = &color {
        classes.push(format!("btn-{}", color));
    }
    if let Some(size) = &size {
        classes.push(format!("btn-{}", size));
    }
    if let Some(variant) = &variant {
        classes.push(format!("btn-{}", variant));
    }
    if let Some(extra) = &class {
        classes.push(extra.clone());
    }
    if disabled.unwrap_or(false) {
        classes.push("btn-disabled".to_string());
    }

    rsx! {
        button {
            class: classes.join(" "),
            disabled: disabled.unwrap_or(false),
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            {children}
        }
    }
}
