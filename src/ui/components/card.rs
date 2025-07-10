use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// DaisyUI Card component.
/// - Applies DaisyUI card classes.
/// - Supports title, subtitle, icon, footer, and custom content.
/// - Animates scale/opacity on mount (dioxus-motion).
#[component]
pub fn Card(
    #[props(optional)] class: Option<String>,
    #[props(optional)] title: Option<String>,
    #[props(optional)] subtitle: Option<String>,
    #[props(optional)] icon: Option<Element>,
    #[props(optional)] footer: Option<Element>,
    children: Element,
) -> Element {
    let class = class.as_deref().unwrap_or("");
    let mut scale = use_motion(0.95f32);
    let mut opacity = use_motion(0.0f32);

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
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let style = format!(
        "transform: scale({}); opacity: {}; transition: transform 0.3s, opacity 0.3s;",
        scale.get_value(),
        opacity.get_value()
    );

    rsx! {
        div {
            class: "card bg-base-100 shadow-md {class}",
            style: "{style}",
            div {
                class: "card-body",
                if let Some(title) = &title {
                    h2 { class: "card-title flex items-center gap-2",
                        if let Some(icon) = &icon {
                            {icon.clone()}
                        }
                        "{title}"
                    }
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
