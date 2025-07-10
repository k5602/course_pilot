use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// DaisyUI Progress Bar component (Dioxus idioms, 0.6+)
#[component]
pub fn ProgressBar(
    /// Current progress value (0-100).
    value: u8,
    /// Optional label to display above the bar.
    #[props(optional)]
    label: Option<String>,
    /// DaisyUI color (e.g., "primary", "secondary", "accent", etc.).
    #[props(optional)]
    color: Option<String>,
    /// Optional extra classes for customization.
    #[props(optional)]
    class: Option<String>,
) -> Element {
    let color_class = color.as_deref().unwrap_or("primary");
    let extra_class = class.as_deref().unwrap_or("");
    let value = value.clamp(0, 100);

    // Animate the progress value
    let mut animated_value = use_motion(value as f32);
    use_effect(move || {
        animated_value.animate_to(
            value as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 120.0,
                damping: 18.0,
                mass: 1.0,
                velocity: 0.0,
            })),
        );
    });

    let display_value = animated_value.get_value().round().clamp(0.0, 100.0);

    rsx! {
        div {
            class: "flex flex-col gap-1",
            if let Some(label) = &label {
                span { class: "text-xs opacity-70", "{label}" }
            }
            progress {
                class: "progress progress-{color_class} {extra_class}",
                value: "{display_value}",
                max: "100",
                aria_label: "progress bar",
            }
            span {
                class: "text-xs text-right opacity-60",
                "{display_value as u8}%"
            }
        }
    }
}
