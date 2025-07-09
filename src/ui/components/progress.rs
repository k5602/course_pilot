use dioxus::prelude::*;

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

    rsx! {
        div {
            class: "flex flex-col gap-1",
            if let Some(label) = &label {
                span { class: "text-xs opacity-70", "{label}" }
            }
            progress {
                class: "progress progress-{color_class} {extra_class}",
                value: "{value}",
                max: "100",
                aria_label: "progress bar",
            }
            span {
                class: "text-xs text-right opacity-60",
                "{value}%"
            }
        }
    }
}
