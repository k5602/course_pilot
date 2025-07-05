use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    #[props(optional)]
    pub r#type: Option<String>,
    #[props(optional)]
    pub placeholder: Option<String>,
    #[props(optional)]
    pub value: Option<String>,
    #[props(optional)]
    pub oninput: Option<EventHandler<FormEvent>>,
    #[props(optional)]
    pub label: Option<String>,
    #[props(optional)]
    pub id: Option<String>,
    #[props(optional, default = false)]
    pub disabled: bool,
    #[props(optional)]
    pub min: Option<String>,
    #[props(optional)]
    pub max: Option<String>,
    #[props(optional)]
    pub step: Option<String>,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/input/style.css")
        }
        {props.label.as_ref().map(|label| rsx!(
            label {
                r#for: props.id.as_deref().unwrap_or(""),
                "{label}"
            }
        ))}
        input {
            class: "input",
            r#type: props.r#type.as_deref().unwrap_or("text"),
            placeholder: props.placeholder.as_deref().unwrap_or(""),
            value: props.value.clone().unwrap_or_default(),
            id: props.id.as_deref().unwrap_or(""),
            disabled: props.disabled,
            min: props.min.as_deref().unwrap_or(""),
            max: props.max.as_deref().unwrap_or(""),
            step: props.step.as_deref().unwrap_or(""),
            oninput: move |evt| {
                if let Some(handler) = &props.oninput {
                    handler.call(evt);
                }
            }
        }
    }
}

#[component]
pub(super) fn Demo() -> Element {
    let mut input_value = use_signal(String::new);

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("/src/ui/components/input/style.css")
        }

        div {
            display: "flex",
            flex_direction: "column",
            gap: "1rem",

            // Basic text input
            Input {
                placeholder: "Enter your name",
                value: Some(input_value.read().clone()),
                oninput: move |evt: FormEvent| input_value.set(evt.data().value())
            }

            // Email input
            Input {
                r#type: "email",
                placeholder: "Enter your email"
            }

            // Password input
            Input {
                r#type: "password",
                placeholder: "Enter your password"
            }

            // Search input
            Input {
                r#type: "search",
                placeholder: "Search courses..."
            }

            div {
                p { "Current input value: {input_value}" }
            }
        }
    }
}
