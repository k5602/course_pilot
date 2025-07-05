use dioxus::prelude::*;

/// Root dialog container
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogRootProps {
    pub open: bool,
    pub on_open_change: EventHandler<bool>,
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn AlertDialogRoot(props: AlertDialogRootProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: "position: fixed; inset: 0; z-index: 1000; background: rgba(0,0,0,0.4); display: flex; align-items: center; justify-content: center;",
            tabindex: "-1",
            aria_modal: "true",
            role: "dialog",
            if props.open {
                div {
                    onclick: move |_| props.on_open_change.call(false),
                    style: "position: absolute; inset: 0;",
                }
                div {
                    style: "position: relative; z-index: 1;",
                    {props.children}
                }
            }
        }
    }
}

/// Dialog content
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogContentProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn AlertDialogContent(props: AlertDialogContentProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: "background: white; border-radius: 8px; padding: 2rem; min-width: 320px; max-width: 90vw; box-shadow: 0 8px 32px rgba(0,0,0,0.2);",
            {props.children}
        }
    }
}

/// Dialog title
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogTitleProps {
    pub children: Element,
}

#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    rsx! {
        h2 {
            style: "font-size: 1.25rem; font-weight: 700; margin-bottom: 0.5rem;",
            {props.children}
        }
    }
}

/// Dialog description
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogDescriptionProps {
    pub children: Element,
}

#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    rsx! {
        p {
            style: "font-size: 1rem; color: #555; margin-bottom: 1.5rem;",
            {props.children}
        }
    }
}

/// Dialog actions container
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogActionsProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

#[component]
pub fn AlertDialogActions(props: AlertDialogActionsProps) -> Element {
    rsx! {
        div {
            class: props.class.clone().unwrap_or_default(),
            style: "display: flex; gap: 1rem; justify-content: flex-end;",
            {props.children}
        }
    }
}

/// Cancel button
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogCancelProps {
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    rsx! {
        button {
            class: props.class.clone().unwrap_or_default(),
            r#type: "button",
            onclick: move |evt| {
                if let Some(cb) = &props.on_click {
                    cb.call(evt);
                }
            },
            style: "padding: 0.5rem 1.25rem; border-radius: 4px; border: none; background: #eee; color: #333; font-weight: 500; cursor: pointer;",
            {props.children}
        }
    }
}

/// Confirm action button
#[derive(Props, PartialEq, Clone)]
pub struct AlertDialogActionProps {
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    rsx! {
        button {
            class: props.class.clone().unwrap_or_default(),
            r#type: "button",
            onclick: move |evt| {
                if let Some(cb) = &props.on_click {
                    cb.call(evt);
                }
            },
            style: "padding: 0.5rem 1.25rem; border-radius: 4px; border: none; background: #c00; color: #fff; font-weight: 500; cursor: pointer;",
            {props.children}
        }
    }
}
