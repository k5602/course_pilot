use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// A simple, reusable dropdown component for Dioxus.
/// - `label`: The button label (RSX/Element).
/// - `children`: The dropdown menu items (RSX/Element).
/// - `open`: Optional, controls open state externally (default: false).
/// - `icon`: Optional icon to display in the label.
/// - `class`: Optional, additional classes for the container.
/// - Animates dropdown open/close with dioxus-motion.
#[component]
pub fn AppDropdown(
    label: Element,
    children: Element,
    #[props(optional)] open: Option<bool>,
    #[props(optional)] icon: Option<Element>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let mut is_open = use_signal(|| open.unwrap_or(false));

    let container_class = format!("relative inline-block {}", class.as_deref().unwrap_or(""));

    // Animate dropdown open/close
    let mut scale = use_motion(0.98f32);
    let mut opacity = use_motion(0.0f32);

    use_effect(move || {
        if is_open() {
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

    let dropdown_style = format!(
        "transform: scale({}); opacity: {}; transition: transform 0.2s, opacity 0.2s;",
        scale.get_value(),
        opacity.get_value()
    );

    rsx! {
        div { class: "{container_class}",
            button {
                class: "btn btn-sm btn-outline flex items-center gap-2",
                onclick: move |_| is_open.set(!is_open()),
                if let Some(icon) = &icon {
                    span { class: "mr-1 flex items-center", {icon.clone()} }
                }
                {label.clone()}
                span { class: "ml-1", "▼" }
            }
            if is_open() {
                div {
                    class: "absolute z-50 mt-2 w-48 rounded-md shadow-lg bg-base-200 ring-1 ring-black ring-opacity-5 focus:outline-none",
                    style: "min-width: 10rem; {dropdown_style}",
                    {children.clone()}
                }
            }
        }
    }
}
