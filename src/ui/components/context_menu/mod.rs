use dioxus::prelude::*;
#[derive(Props, PartialEq, Clone)]
pub struct ContextMenuProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: "position: relative; display: inline-block;",
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ContextMenuTriggerProps {
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub style: Option<String>,
    pub children: Element,
}

#[component]
pub fn ContextMenuTrigger(props: ContextMenuTriggerProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: props.style.clone().unwrap_or_default(),
            tabindex: "0",
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ContextMenuContentProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn ContextMenuContent(props: ContextMenuContentProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: "position: absolute; top: 100%; left: 0; background: white; border: 1px solid #ddd; border-radius: 6px; min-width: 160px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); z-index: 100;",
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ContextMenuItemProps {
    #[props(optional)]
    pub class: Option<String>,
    pub value: String,
    pub index: usize,
    #[props(optional)]
    pub disabled: Option<bool>,
    pub on_select: EventHandler<String>,
    pub children: Element,
}

#[component]
pub fn ContextMenuItem(props: ContextMenuItemProps) -> Element {
    let is_disabled = props.disabled.unwrap_or(false);
    rsx! {
        div {
            class: format!(
                "{}{}",
                props.class.clone().unwrap_or_default(),
                if is_disabled { " context-menu-item-disabled" } else { "" }
            ),
            style: "padding: 0.5rem 1rem; cursor: pointer; user-select: none;",
            tabindex: if is_disabled { "-1" } else { "0" },
            aria_disabled: is_disabled,
            onclick: move |_| {
                if !is_disabled {
                    props.on_select.call(props.value.clone());
                }
            },
            {props.children}
        }
    }
}
#[component]
pub(super) fn Demo() -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("/src/components/context_menu/style.css"),
        }
        ContextMenu {
            ContextMenuTrigger {
                class: None,
                style: Some("padding: 20px; background: var(--primary-color); border: 1px dashed var(--primary-color-6); border-radius: .5rem; cursor: context-menu; user-select: none; text-align: center;".to_string()),
                "right click here"
            }
            ContextMenuContent { class: Some("context-menu-content".to_string()),
                ContextMenuItem {
                    class: Some("context-menu-item".to_string()),
                    value: "edit".to_string(),
                    index: 0usize,
                    on_select: move |value| {
                        tracing::info!("Selected item: {}", value);
                    },
                    "Edit"
                }
                ContextMenuItem {
                    class: Some("context-menu-item".to_string()),
                    value: "undo".to_string(),
                    index: 1usize,
                    disabled: Some(true),
                    on_select: move |value| {
                        tracing::info!("Selected item: {}", value);
                    },
                    "Undo"
                }
                ContextMenuItem {
                    class: Some("context-menu-item".to_string()),
                    value: "duplicate".to_string(),
                    index: 2usize,
                    on_select: move |value| {
                        tracing::info!("Selected item: {}", value);
                    },
                    "Duplicate"
                }
                ContextMenuItem {
                    class: Some("context-menu-item".to_string()),
                    value: "delete".to_string(),
                    index: 3usize,
                    on_select: move |value| {
                        tracing::info!("Selected item: {}", value);
                    },
                    "Delete"
                }
            }
        }
    }
}
