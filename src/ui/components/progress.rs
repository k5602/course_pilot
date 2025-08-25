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

/// DaisyUI radial-progress ProgressRing component for Dioxus.
/// - `value`: Current progress value (0..=max)
/// - `max`: Maximum value (default: 100)
/// - `color`: DaisyUI color class (e.g., "primary", "accent", "success", etc.)
/// - `size`: Optional pixel size (default: 48)
/// - `thickness`: Optional border thickness (default: 4)
/// - `label`: Optional label/child element inside the ring
#[component]
pub fn ProgressRing(
    value: u32,
    #[props(optional)] max: Option<u32>,
    #[props(optional)] color: Option<String>,
    #[props(optional)] size: Option<u32>,
    #[props(optional)] thickness: Option<u32>,
    #[props(optional)] label: Option<Element>,
) -> Element {
    let max = max.unwrap_or(100);
    let percent = if max == 0 { 0.0 } else { (value as f32 / max as f32) * 100.0 };
    let color_class = color.as_deref().unwrap_or("primary");
    let size_px = size.unwrap_or(48);
    let thickness_px = thickness.unwrap_or(4);

    // DaisyUI radial-progress uses CSS variables for value and thickness
    let style = format!(
        "--value: {}; --size: {}px; --thickness: {}px;",
        percent.round(),
        size_px,
        thickness_px
    );

    rsx! {
        div {
            class: format!("radial-progress text-{}", color_class),
            style: "{style}",
            // Show label if provided, else show percent rounded
            if let Some(label) = label {
                {label}
            } else {
                "{percent.round() as u32}%"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_ring_renders_with_value_and_color() {
        let props = ProgressRingProps {
            value: 75,
            max: Some(100),
            color: Some("accent".to_string()),
            size: Some(64),
            thickness: Some(6),
            label: None,
        };
        let mut dom = VirtualDom::new_with_props(ProgressRing, props);
        let mut mutations = dioxus_core::NoOpMutations;
        dom.rebuild(&mut mutations);
        let rendered = dioxus_ssr::render(&dom);
        assert!(!rendered.is_empty());
        assert!(rendered.contains("radial-progress"));
    }

    #[test]
    fn progress_ring_renders_with_label() {
        let label = rsx! { span { "Done" } };
        let props = ProgressRingProps {
            value: 100,
            max: Some(100),
            color: Some("success".to_string()),
            size: Some(48),
            thickness: Some(4),
            label: Some(label),
        };
        let mut dom = VirtualDom::new_with_props(ProgressRing, props);
        let mut mutations = dioxus_core::NoOpMutations;
        dom.rebuild(&mut mutations);
        let rendered = dioxus_ssr::render(&dom);
        assert!(!rendered.is_empty());
        assert!(rendered.contains("radial-progress"));
    }
}
