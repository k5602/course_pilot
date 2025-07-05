use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupProps {
    pub name: String,
    pub value: String,
    pub onchange: EventHandler<String>,
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    provide_context(props.name.clone());
    provide_context(props.value.clone());
    provide_context(Callback::from(props.onchange.clone()));
    rsx! {
        div {
            class: props.class.clone().unwrap_or("radio-group".to_string()),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct RadioItemProps {
    pub value: String,
    pub index: usize,
    #[props(optional)]
    pub disabled: Option<bool>,
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn RadioItem(props: RadioItemProps) -> Element {
    // Get context from parent RadioGroup
    let name = use_context::<String>();
    let group_value = use_context::<String>();
    let onchange = use_context::<Callback<String>>();

    let checked = props.value == group_value;
    let disabled = props.disabled.unwrap_or(false);

    rsx! {
        label {
            class: props.class.clone().unwrap_or("radio-group".to_string()),
            input {
                r#type: "radio",
                name: name.clone(),
                value: props.value.clone(),
                checked: checked,
                disabled: disabled,
                onchange: move |_evt: dioxus::events::FormEvent| {
                    onchange.call(props.value.clone());
                }
            }
            {props.children}
        }
    }
}

#[component]
pub(super) fn Demo() -> Element {
    let mut selected = use_signal(|| "option1".to_string());

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/radio_group/style.css")
        }
        RadioGroup {
            name: "demo-radio".to_string(),
            value: selected().clone(),
            onchange: move |val| selected.set(val),
            class: "radio-group".to_string(),
            RadioItem {
                value: "option1".to_string(),
                index: 0,
                class: "radio-item".to_string(),
                "Blue"
            }
            RadioItem {
                value: "option2".to_string(),
                index: 1,
                class: "radio-item".to_string(),
                "Red"
            }
            RadioItem {
                value: "option3".to_string(),
                index: 2,
                class: "radio-item".to_string(),
                disabled: true,
                "Green"
            }
        }
        div { "Selected: {selected()}" }
    }
}
