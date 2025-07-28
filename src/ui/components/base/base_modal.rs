use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// Props for the BaseModal component
#[derive(Props, PartialEq, Clone)]
pub struct BaseModalProps {
    /// Whether the modal is open
    pub open: bool,

    /// Modal title
    #[props(optional)]
    pub title: Option<String>,

    /// Modal content
    pub children: Element,

    /// Footer actions
    #[props(optional)]
    pub actions: Option<Element>,

    /// Close handler
    pub on_close: EventHandler<()>,

    /// Modal size variant
    #[props(default = "modal-box")]
    pub size: &'static str,

    /// Additional CSS classes
    #[props(default = "")]
    pub class: &'static str,

    /// Optional icon for the title
    #[props(optional)]
    pub icon: Option<Element>,

    /// Whether clicking backdrop closes modal
    #[props(default = true)]
    pub close_on_backdrop: bool,
}

/// Generic BaseModal component using DaisyUI styling
/// Provides consistent modal structure with configurable content
#[component]
pub fn BaseModal(props: BaseModalProps) -> Element {
    if !props.open {
        return rsx! {};
    }

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

    let animation_style = format!(
        "transform: scale({}); opacity: {}; transition: transform 0.3s, opacity 0.3s;",
        scale.get_value(),
        opacity.get_value()
    );

    let modal_box_class = format!(
        "{} bg-base-100 shadow-xl relative {}",
        props.size, props.class
    );

    rsx! {
        // Modal overlay using DaisyUI classes
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/40",
            onclick: move |_| {
                if props.close_on_backdrop {
                    props.on_close.call(());
                }
            },

            // Modal content box
            div {
                class: "{modal_box_class}",
                style: "{animation_style}",
                onclick: move |evt| evt.stop_propagation(),

                // Modal header
                if let Some(title) = &props.title {
                    div {
                        class: "flex justify-between items-center mb-4",

                        h3 {
                            class: "font-bold text-lg flex items-center gap-2",

                            if let Some(icon) = &props.icon {
                                {icon.clone()}
                            }

                            "{title}"
                        }

                        button {
                            class: "btn btn-sm btn-circle btn-ghost",
                            onclick: move |_| props.on_close.call(()),
                            "✕"
                        }
                    }
                }

                // Modal content
                div {
                    class: "py-2",
                    {props.children}
                }

                // Modal actions
                if let Some(actions) = &props.actions {
                    div {
                        class: "modal-action flex gap-2 mt-4",
                        {actions.clone()}
                    }
                }
            }
        }
    }
}
