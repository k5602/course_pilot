use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    #[props(optional)]
    pub html_for: Option<String>,
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

/// Custom Label component using native <label> and CSS
#[component]
pub fn Label(props: LabelProps) -> Element {
    rsx! {
        label {
            class: props.class.clone().unwrap_or_else(|| "label".to_string()),
            r#for: props.html_for.clone().unwrap_or_default(),
            {props.children}
        }
    }
}

/// Demo for the Label component
#[component]
pub(super) fn Demo() -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/label/style.css")
        }
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/input/style.css")
        }

        div {
            style: "display: flex; flex-direction: column; gap: .5rem;",
            Label {
                class: "label".to_string(),
                html_for: "name".to_string(),
                "Name"
            }

            input {
                class: "input",
                id: "name",
                placeholder: "Enter your name"
            }
        }
    }
}
