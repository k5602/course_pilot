use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// Props for the BaseCard component
#[derive(Props, PartialEq, Clone)]
pub struct BaseCardProps {
    /// Card title
    #[props(optional)]
    pub title: Option<String>,
    
    /// Card subtitle
    #[props(optional)]
    pub subtitle: Option<String>,
    
    /// Card content
    pub children: Element,
    
    /// DaisyUI card variant classes
    #[props(default = "card")]
    pub variant: &'static str,
    
    /// Additional CSS classes
    #[props(default = "")]
    pub class: &'static str,
    
    /// Enable hover effects
    #[props(default = true)]
    pub hover_effect: bool,
    
    /// Click handler
    #[props(optional)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    
    /// Action buttons in card footer
    #[props(optional)]
    pub actions: Option<Element>,
    
    /// Header actions (e.g., dropdown menu)
    #[props(optional)]
    pub header_actions: Option<Element>,
}

/// Generic BaseCard component using DaisyUI styling
/// Provides consistent card structure with configurable content
#[component]
pub fn BaseCard(props: BaseCardProps) -> Element {
    let card_classes = format!("{} {} bg-base-100 shadow-xl border border-base-300", props.variant, props.class);
    
    // Animation setup for hover effects
    let mut scale = use_motion(1.0f32);
    let mut y = use_motion(0.0f32);
    let mut opacity = use_motion(0.95f32);

    // Mount animation
    use_effect(move || {
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let card_style = use_memo(move || {
        format!(
            "transform: translateY({}px) scale({}); opacity: {}; transition: all 0.3s ease-in-out;",
            y.get_value(),
            scale.get_value(),
            opacity.get_value()
        )
    });

    rsx! {
        div {
            style: "{card_style}",
            class: "{card_classes}",
            onclick: move |evt| {
                if let Some(handler) = &props.on_click {
                    handler.call(evt);
                }
            },
            onmouseenter: move |_| {
                if props.hover_effect {
                    scale.animate_to(1.02, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                    y.animate_to(-5.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                }
            },
            onmouseleave: move |_| {
                if props.hover_effect {
                    scale.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                    y.animate_to(0.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                }
            },

            div {
                class: "card-body p-4",

                // Header with title and actions
                if props.title.is_some() || props.header_actions.is_some() {
                    div {
                        class: "flex justify-between items-start mb-2",
                        
                        // Title section
                        if let Some(title) = &props.title {
                            div {
                                class: "flex-1",
                                h2 {
                                    class: "card-title text-lg",
                                    "{title}"
                                }
                                
                                if let Some(subtitle) = &props.subtitle {
                                    p { 
                                        class: "text-base-content/70 text-sm mt-1", 
                                        "{subtitle}" 
                                    }
                                }
                            }
                        }
                        
                        // Header actions
                        if let Some(header_actions) = &props.header_actions {
                            div {
                                class: "flex-shrink-0",
                                {header_actions.clone()}
                            }
                        }
                    }
                }

                // Main content
                div {
                    class: if props.title.is_some() { "mt-4" } else { "" },
                    {props.children}
                }

                // Footer actions
                if let Some(actions) = &props.actions {
                    div {
                        class: "card-actions justify-end mt-4",
                        {actions.clone()}
                    }
                }
            }
        }
    }
}