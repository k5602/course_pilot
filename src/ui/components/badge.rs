use dioxus::prelude::*;

/// DaisyUI Badge component for Dioxus.
/// - `label`: The badge text.
/// - `color`: DaisyUI color (e.g., "primary", "secondary", "accent", "info", "success", "warning", "error").
/// - `icon`: Optional icon (DioxusFreeIcons).
/// - `class`: Optional extra classes.
#[derive(Props, PartialEq, Clone)]
pub struct BadgeProps {
    pub label: String,
    #[props(optional)]
    pub color: Option<String>,
    #[props(optional)]
    pub icon: Option<Element>,
    #[props(optional)]
    pub class: Option<String>,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let color_class = props
        .color
        .as_deref()
        .map(|c| format!("badge-{}", c))
        .unwrap_or_else(|| "badge-neutral".to_string());

    let class = props.class.as_deref().unwrap_or("badge").to_string();

    rsx! {
        span {
            class: format!("badge {} {}", color_class, class),
            if let Some(icon) = &props.icon {
                span { class: "mr-1 flex items-center", {icon.clone()} }
            }
            {props.label.clone()}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;
    use dioxus::prelude::*;
    use dioxus_ssr::render;

    #[test]
    fn badge_renders_with_label_and_color() {
        let props = BadgeProps {
            label: "Test".to_string(),
            color: Some("primary".to_string()),
            icon: None,
            class: None,
        };
        let mut dom = VirtualDom::new_with_props(Badge, props);
        let mut to = Vec::new();
        dom.rebuild(&mut to);
        let rendered = render(&dom);
        assert!(rendered.contains("Test"));
        assert!(rendered.contains("badge-primary"));
    }

    #[test]
    fn badge_renders_with_icon() {
        let icon = rsx! { span { "icon" } };
        let props = BadgeProps {
            label: "WithIcon".to_string(),
            color: Some("success".to_string()),
            icon: Some(icon),
            class: None,
        };
        let mut dom = VirtualDom::new_with_props(Badge, props);
        let mut to = Vec::new();
        dom.rebuild(&mut to);
        let rendered = render(&dom);
        assert!(rendered.contains("WithIcon"));
        assert!(rendered.contains("badge-success"));
        assert!(rendered.contains("icon"));
    }
}
