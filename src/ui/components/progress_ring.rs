use dioxus::prelude::*;

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
    let percent = if max == 0 {
        0.0
    } else {
        (value as f32 / max as f32) * 100.0
    };
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
        let rendered = dioxus_ssr::render(&dom);
        assert!(rendered.contains("radial-progress"));
        assert!(rendered.contains("text-accent"));
        assert!(rendered.contains("75"));
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
        let rendered = dioxus_ssr::render(&dom);
        assert!(rendered.contains("Done"));
        assert!(rendered.contains("radial-progress"));
        assert!(rendered.contains("text-success"));
    }
}
