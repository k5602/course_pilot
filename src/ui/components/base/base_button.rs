use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// Props for the BaseButton component
#[derive(Props, PartialEq, Clone)]
pub struct BaseButtonProps {
    /// Button content
    pub children: Element,

    /// Click handler
    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    /// DaisyUI color (primary, secondary, accent, etc.)
    #[props(optional)]
    pub color: Option<String>,

    /// DaisyUI size (sm, md, lg)
    #[props(optional)]
    pub size: Option<String>,

    /// DaisyUI variant (outline, ghost, link, etc.)
    #[props(optional)]
    pub variant: Option<String>,

    /// Additional CSS classes
    #[props(default = "")]
    pub class: &'static str,

    /// Disabled state
    #[props(default = false)]
    pub disabled: bool,

    /// Optional icon to display at the start of the button
    #[props(optional)]
    pub icon: Option<Element>,

    /// Loading state
    #[props(default = false)]
    pub loading: bool,

    /// Button type
    #[props(default = "button")]
    pub button_type: &'static str,
}

/// Generic BaseButton component using DaisyUI styling
/// Provides consistent button styling with configurable variants
#[component]
pub fn BaseButton(props: BaseButtonProps) -> Element {
    let mut classes = vec!["btn".to_string()];

    if let Some(color) = &props.color {
        classes.push(format!("btn-{color}"));
    }
    if let Some(size) = &props.size {
        classes.push(format!("btn-{size}"));
    }
    if let Some(variant) = &props.variant {
        classes.push(format!("btn-{variant}"));
    }
    if !props.class.is_empty() {
        classes.push(props.class.to_string());
    }
    if props.disabled {
        classes.push("btn-disabled".to_string());
    }
    if props.loading {
        classes.push("loading".to_string());
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

    let style = format!("transform: scale({}); transition: transform 0.2s;", scale.get_value());

    rsx! {
        button {
            class: classes.join(" "),
            style: "{style}",
            disabled: props.disabled || props.loading,
            r#type: props.button_type,
            onclick: move |evt| {
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },

            if props.loading {
                span { class: "loading loading-spinner loading-sm mr-2" }
            } else if let Some(icon) = &props.icon {
                span { class: "mr-2 flex items-center", {icon.clone()} }
            }

            {props.children}
        }
    }
}
