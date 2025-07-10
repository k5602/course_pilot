use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;

/// DaisyUI Tabs component (Dioxus idioms, 0.6+)
/// - Supports optional icons per tab (icons: Option<Vec<Element>>)
/// - Animates tab indicator on selection
#[component]
pub fn Tabs(
    tabs: Vec<String>,
    selected: usize,
    on_select: EventHandler<usize>,
    #[props(optional)] icons: Option<Vec<Element>>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let class = class.as_deref().unwrap_or("tabs tabs-boxed");

    let mut indicator_scale = use_motion(0.95f32);
    use_effect(move || {
        indicator_scale.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 180.0,
                damping: 18.0,
                mass: 1.0,
                velocity: 0.0,
            })),
        );
    });
    let indicator_style = format!(
        "transform: scale({}); transition: transform 0.2s;",
        indicator_scale.get_value()
    );

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
                            style: if idx == selected { indicator_style.clone() } else { "".to_string() },
                            onclick: move |_| on_select.call(idx),
                            if let Some(ref icons) = icons {
                                if let Some(icon) = icons.get(idx) {
                                    span { class: "mr-1 flex items-center", {icon.clone()} }
                                }
                            }
                            "{label}"
                        }
                    }
                })
            }
        }
    }
}
