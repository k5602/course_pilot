use dioxus::prelude::*;
use dioxus_motion::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl Default for ButtonSize {
    fn default() -> Self {
        ButtonSize::Medium
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(optional, default = "primary".to_string())]
    pub button_type: String,
    #[props(optional, default = false)]
    pub disabled: bool,
    #[props(optional)]
    pub aria_label: Option<String>,
    #[props(optional)]
    pub tabindex: Option<String>,
    pub children: Element,
    #[props(optional)]
    pub size: Option<ButtonSize>,
}

impl ButtonProps {
    pub fn size_class(&self) -> &'static str {
        match self.size.unwrap_or(ButtonSize::Medium) {
            ButtonSize::Small => "button--sm",
            ButtonSize::Medium => "button--md",
            ButtonSize::Large => "button--lg",
        }
    }
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let ButtonProps {
        button_type,
        size,
        disabled,
        aria_label,
        children,
        ..
    } = props;
    let size_class = match size.unwrap_or(ButtonSize::Medium) {
        ButtonSize::Small => "button--sm",
        ButtonSize::Medium => "button--md",
        ButtonSize::Large => "button--lg",
    };
    let scale = use_motion(1.0f32);

    let on_mouse_down = {
        let mut scale = scale.clone();
        move |_| {
            scale.animate_to(
                0.97,
                AnimationConfig::new(AnimationMode::Spring(Spring {
                    stiffness: 300.0,
                    damping: 20.0,
                    mass: 1.0,
                    velocity: 0.0,
                })),
            );
        }
    };

    let on_mouse_up = {
        let mut scale = scale.clone();
        move |_| {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(Spring {
                    stiffness: 300.0,
                    damping: 20.0,
                    mass: 1.0,
                    velocity: 0.0,
                })),
            );
        }
    };

    let on_mouse_enter = {
        let mut scale = scale.clone();
        move |_| {
            scale.animate_to(
                1.03,
                AnimationConfig::new(AnimationMode::Spring(Spring {
                    stiffness: 300.0,
                    damping: 20.0,
                    mass: 1.0,
                    velocity: 0.0,
                })),
            );
        }
    };

    let on_mouse_leave = {
        let mut scale = scale.clone();
        move |_| {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(Spring {
                    stiffness: 300.0,
                    damping: 20.0,
                    mass: 1.0,
                    velocity: 0.0,
                })),
            );
        }
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/button/style.css")
        }

        button {
            class: format!("button {} {}", button_type, size_class),
            disabled: disabled,
            aria_label: aria_label.unwrap_or_default(),
            tabindex: props.tabindex.unwrap_or_default(),
            style: format!("transform: scale({}); transition: transform 0.1s cubic-bezier(0.4,0,0.2,1);", scale.get_value()),
            onmousedown: on_mouse_down,
            onmouseup: on_mouse_up,
            onmouseenter: on_mouse_enter,
            onmouseleave: on_mouse_leave,
            onclick: move |evt| {
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },
            {children}
        }
    }
}

#[component]
pub(super) fn Demo() -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("/src/ui/components/button/style.css")
        }

        div {
            display: "flex",
            flex_direction: "column",
            gap: "0.5rem",

            Button {
                button_type: "primary",
                "Primary"
            }

            Button {
                button_type: "secondary",
                "Secondary"
            }

            Button {
                button_type: "destructive",
                onclick: move |_| {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Destructive action!")
                        .unwrap();
                },
                "Destructive"
            }

            Button {
                button_type: "outline",
                "Outline"
            }

            Button {
                button_type: "ghost",
                "Ghost"
            }
        }
    }
}
