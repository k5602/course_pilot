use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// SkeletonLoaderProps defines the shape, size, and style of the skeleton loader.
#[derive(Props, Clone, PartialEq)]
pub struct SkeletonLoaderProps {
    /// Width of the skeleton (e.g., "100%", "200px")
    #[props(optional, default = "100%".to_string())]
    pub width: String,
    /// Height of the skeleton (e.g., "1.5rem", "40px")
    #[props(optional, default = "1.5rem".to_string())]
    pub height: String,
    /// Border radius (e.g., "4px", "50%" for circle)
    #[props(optional, default = "8px".to_string())]
    pub border_radius: String,
    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,
    /// Optional style override
    #[props(optional)]
    pub style: Option<String>,
}

/// SkeletonLoader: a shimmering skeleton placeholder for async loading states.
#[component]
pub fn SkeletonLoader(props: SkeletonLoaderProps) -> Element {
    // Animate shimmer position from -100% to 100% in a loop
    let mut shimmer = use_motion(-100.0f32);

    use_effect(move || {
        shimmer.animate_to(
            100.0,
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 60.0,
                damping: 12.0,
                mass: 1.0,
                velocity: 0.0,
            }))
            .with_loop(LoopMode::Infinite),
        );
    });

    let shimmer_pos = shimmer.get_value();

    let style = format!(
        "position: relative; overflow: hidden; background: #e0e0e0; width: {}; height: {}; border-radius: {}; {}",
        props.width,
        props.height,
        props.border_radius,
        props.style.clone().unwrap_or_default()
    );

    let shimmer_style = format!(
        "position: absolute; top: 0; left: 0; height: 100%; width: 100%; \
        background: linear-gradient(90deg, transparent 0%, #f5f5f5 50%, transparent 100%); \
        transform: translateX({}%); transition: transform 0.2s linear;",
        shimmer_pos
    );

    rsx! {
        div {
            class: format!("skeleton-loader {}", props.class.clone().unwrap_or_default()),
            style: style,
            div {
                style: shimmer_style,
            }
        }
    }
}

/// Demo for SkeletonLoader usage
#[component]
pub(super) fn Demo() -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1rem; width: 300px;",
            SkeletonLoader {
                width: "100%".to_string(),
                height: "1.5rem".to_string(),
            }
            SkeletonLoader {
                width: "60%".to_string(),
                height: "1.5rem".to_string(),
                border_radius: "50px".to_string(),
            }
            SkeletonLoader {
                width: "40px".to_string(),
                height: "40px".to_string(),
                border_radius: "50%".to_string(),
            }
        }
    }
}
