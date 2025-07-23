use dioxus::prelude::*;

/// DaisyUI Badge component for Dioxus with simple interface.
/// - `label`: The badge text.
/// - `color`: Optional DaisyUI color (e.g., "primary", "secondary", "accent", "info", "success", "warning", "error").
/// - `class`: Optional extra classes.
#[component]
pub fn Badge(
    label: String,
    #[props(optional)] color: Option<String>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let color = color.as_deref().unwrap_or("primary");
    let class = class.as_deref().unwrap_or("");
    rsx! {
        span {
            class: format!("badge badge-{} {}", color, class),
            "{label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;
    use dioxus_ssr::render;

    #[test]
    fn badge_renders_with_label_and_color() {
        let props = BadgeProps {
            label: "Test".to_string(),
            color: Some("primary".to_string()),
            class: None,
        };
        let dom = VirtualDom::new_with_props(Badge, props);
        let rendered = dioxus_ssr::render(&dom);
        assert!(rendered.contains("Test"));
        assert!(rendered.contains("badge-primary"));
    }

    #[test]
    fn badge_renders_with_custom_class() {
        let props = BadgeProps {
            label: "Custom".to_string(),
            color: Some("success".to_string()),
            class: Some("badge-lg".to_string()),
        };
        let dom = VirtualDom::new_with_props(Badge, props);
        let rendered = dioxus_ssr::render(&dom);
        assert!(rendered.contains("Custom"));
        assert!(rendered.contains("badge-success"));
        assert!(rendered.contains("badge-lg"));
    }
}
