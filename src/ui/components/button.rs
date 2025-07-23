use dioxus::prelude::*;
use dioxus_motion::prelude::*;

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
    /// Optional icon to display at the start of the button.
    #[props(optional)]
    icon: Option<Element>,
    /// Button label or content.
    children: Element,
) -> Element {
    let mut classes = vec!["btn".to_string()];
    if let Some(color) = &color {
        classes.push(format!("btn-{color}"));
    }
    if let Some(size) = &size {
        classes.push(format!("btn-{size}"));
    }
    if let Some(variant) = &variant {
        classes.push(format!("btn-{variant}"));
    }
    if let Some(extra) = &class {
        classes.push(extra.clone());
    }
    if disabled.unwrap_or(false) {
        classes.push("btn-disabled".to_string());
    }

    let mut scale = use_motion(0.95f32);
    use_effect(move || {
        scale.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 120.0,
                damping: 16.0,
                mass: 1.0,
                velocity: 0.0,
            })),
        );
    });
    let style = format!(
        "transform: scale({}); transition: transform 0.2s;",
        scale.get_value()
    );

    rsx! {
        button {
            class: classes.join(" "),
            style: "{style}",
            disabled: disabled.unwrap_or(false),
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            if let Some(icon) = &icon {
                span { class: "mr-2 flex items-center", {icon.clone()} }
            }
            {children}
        }
    }
}
