use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    #[props(optional)]
    pub checked: Option<bool>,
    #[props(optional)]
    pub onchange: Option<EventHandler<FormEvent>>,
    #[props(optional)]
    pub disabled: Option<bool>,
    #[props(optional)]
    pub aria_label: Option<String>,
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub name: Option<String>,
    #[props(optional)]
    pub id: Option<String>,
}

/// Custom Checkbox component using native input and SVG indicator
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let checked = props.checked.unwrap_or(false);
    let disabled = props.disabled.unwrap_or(false);

    rsx! {
        label {
            class: props.class.clone().unwrap_or_else(|| "checkbox".to_string()),
            input {
                r#type: "checkbox",
                checked: checked,
                disabled: disabled,
                name: props.name.clone().unwrap_or_default(),
                id: props.id.clone().unwrap_or_default(),
                aria_label: props.aria_label.clone().unwrap_or_default(),
                onchange: move |evt| {
                    if let Some(handler) = &props.onchange {
                        handler.call(evt);
                    }
                }
            }
            span {
                class: "checkbox-indicator",
                if checked {
                    svg {
                        class: "checkbox-check-icon",
                        view_box: "0 0 24 24",
                        xmlns: "http://www.w3.org/2000/svg",
                        path { d: "M5 13l4 4L19 7" }
                    }
                }
            }
        }
    }
}

/// Demo for the Checkbox component
#[component]
pub(super) fn Demo() -> Element {
    let mut checked = use_signal(|| false);

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/checkbox/style.css")
        }
        div {
            style: "display: flex; flex-direction: column; gap: 1rem;",
            Checkbox {
                checked: *checked.read(),
                onchange: move |evt: FormEvent| {
                    let value = evt.value().parse::<bool>().unwrap_or(false);
                    checked.set(value);
                },
                class: "checkbox".to_string(),
                name: "tos-check".to_string(),
                aria_label: "Demo Checkbox".to_string()
            }
            span { "Checked: {checked}" }
        }
    }
}
