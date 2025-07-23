use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheck, FaExclamation, FaXmark};
use dioxus_motion::prelude::*;

/// Modal variant types for different use cases
#[derive(Debug, Clone, PartialEq)]
pub enum ModalVariant {
    /// Standard modal dialog
    Standard,
    /// Confirmation dialog with confirm/cancel buttons
    Confirmation {
        message: String,
        confirm_label: String,
        cancel_label: String,
        confirm_color: String,
        on_confirm: Option<Callback<()>>,
        on_cancel: Option<Callback<()>>,
    },
    /// Form dialog with custom actions
    Form { actions: Element },
    /// Fullscreen modal for complex content
    Fullscreen,
    /// Alert dialog with single action
    Alert {
        message: String,
        action_label: String,
        action_color: String,
        on_action: Option<Callback<()>>,
    },
}

/// DaisyUI-based unified Modal component for Dioxus Desktop.
/// Supports multiple variants: Standard, Confirmation, Form, Fullscreen, Alert
/// Usage: <Modal variant=... open=... on_close=...>{ ... }</Modal>
#[component]
pub fn Modal(
    /// Modal variant type
    #[props(default = ModalVariant::Standard)]
    variant: ModalVariant,
    /// Whether the modal is open.
    open: bool,
    /// Callback to close the modal.
    #[props(optional)]
    on_close: Option<Callback<()>>,
    /// Optional modal title.
    #[props(optional)]
    title: Option<String>,
    /// Optional icon to display in the title.
    #[props(optional)]
    icon: Option<Element>,
    /// Optional size override (sm, md, lg, xl, full)
    #[props(optional)]
    size: Option<String>,
    /// Optional custom classes
    #[props(optional)]
    class: Option<String>,
    /// Modal content (not used for confirmation variant).
    children: Element,
) -> Element {
    if !open {
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

    // Determine modal classes based on variant and size
    let modal_box_class = match &variant {
        ModalVariant::Fullscreen => {
            "modal-box w-full h-full max-w-none max-h-none bg-base-100 shadow-xl"
        }
        _ => {
            let size_class = match size.as_deref().unwrap_or("md") {
                "sm" => "max-w-sm",
                "md" => "max-w-lg",
                "lg" => "max-w-2xl",
                "xl" => "max-w-4xl",
                "full" => "max-w-none w-full",
                _ => "max-w-lg",
            };
            &format!("modal-box bg-base-100 shadow-xl relative {size_class} w-full mx-4")
        }
    };

    let extra_class = class.as_deref().unwrap_or("");

    rsx! {
        // Modal overlay using DaisyUI classes
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/40",
            onclick: move |_| {
                if let Some(handler) = &on_close {
                    handler.call(());
                }
            },
            // Modal content box
            div {
                class: "{modal_box_class} {extra_class}",
                style: "{animation_style}",
                onclick: move |evt| evt.stop_propagation(),

                // Render content based on variant
                {render_modal_content(variant, title, icon, children)}
            }
        }
    }
}

/// Render modal content based on variant
fn render_modal_content(
    variant: ModalVariant,
    title: Option<String>,
    icon: Option<Element>,
    children: Element,
) -> Element {
    match variant {
        ModalVariant::Standard => {
            rsx! {
                {if let Some(title) = title {
                    rsx!(h3 { class: "font-bold text-lg mb-4 flex items-center gap-2",
                        if let Some(icon) = icon {
                            {icon}
                        }
                        "{title}"
                    })
                } else { rsx!{} }}
                div {
                    class: "py-2",
                    {children}
                }
            }
        }
        ModalVariant::Confirmation {
            message,
            confirm_label,
            cancel_label,
            confirm_color,
            on_confirm,
            on_cancel,
        } => {
            rsx! {
                h3 { class: "font-bold text-lg flex items-center gap-2 mb-4",
                    Icon { icon: FaExclamation, class: "text-warning w-5 h-5" }
                    {title.unwrap_or_else(|| "Confirm Action".to_string())}
                }
                p { class: "mb-6 text-base-content/80", "{message}" }
                div { class: "modal-action flex gap-2 justify-end",
                    button {
                        class: "btn btn-sm btn-ghost",
                        onclick: move |_| {
                            if let Some(cb) = on_cancel {
                                cb.call(());
                            }
                        },
                        Icon { icon: FaXmark, class: "w-4 h-4 mr-1" }
                        "{cancel_label}"
                    }
                    button {
                        class: "btn btn-sm btn-{confirm_color}",
                        onclick: move |_| {
                            if let Some(cb) = on_confirm {
                                cb.call(());
                            }
                        },
                        Icon { icon: FaCheck, class: "w-4 h-4 mr-1" }
                        "{confirm_label}"
                    }
                }
            }
        }
        ModalVariant::Form { actions } => {
            rsx! {
                {if let Some(title) = title {
                    rsx!(h3 { class: "font-bold text-lg mb-4 flex items-center gap-2",
                        if let Some(icon) = icon {
                            {icon}
                        }
                        "{title}"
                    })
                } else { rsx!{} }}
                div {
                    class: "py-2",
                    {children}
                }
                div {
                    class: "modal-action flex gap-2 mt-4",
                    {actions}
                }
            }
        }
        ModalVariant::Fullscreen => {
            rsx! {
                {if let Some(title) = title {
                    rsx!(
                        div { class: "flex items-center justify-between mb-4 pb-2 border-b border-base-300",
                            h3 { class: "font-bold text-xl flex items-center gap-2",
                                if let Some(icon) = icon {
                                    {icon}
                                }
                                "{title}"
                            }
                            button {
                                class: "btn btn-sm btn-ghost btn-circle",
                                Icon { icon: FaXmark, class: "w-4 h-4" }
                            }
                        }
                    )
                } else { rsx!{} }}
                div {
                    class: "flex-1 overflow-auto",
                    {children}
                }
            }
        }
        ModalVariant::Alert {
            message,
            action_label,
            action_color,
            on_action,
        } => {
            rsx! {
                h3 { class: "font-bold text-lg flex items-center gap-2 mb-4",
                    Icon { icon: FaExclamation, class: "text-info w-5 h-5" }
                    {title.unwrap_or_else(|| "Notice".to_string())}
                }
                p { class: "mb-6 text-base-content/80", "{message}" }
                div { class: "modal-action flex justify-end",
                    button {
                        class: "btn btn-sm btn-{action_color}",
                        onclick: move |_| {
                            if let Some(cb) = on_action {
                                cb.call(());
                            }
                        },
                        "{action_label}"
                    }
                }
            }
        }
    }
}

/// Helper function to create confirmation modal variant
pub fn confirmation_modal(
    message: impl Into<String>,
    confirm_label: impl Into<String>,
    cancel_label: impl Into<String>,
    confirm_color: impl Into<String>,
    on_confirm: Option<Callback<()>>,
    on_cancel: Option<Callback<()>>,
) -> ModalVariant {
    ModalVariant::Confirmation {
        message: message.into(),
        confirm_label: confirm_label.into(),
        cancel_label: cancel_label.into(),
        confirm_color: confirm_color.into(),
        on_confirm,
        on_cancel,
    }
}

/// Helper function to create form modal variant
pub fn form_modal(actions: Element) -> ModalVariant {
    ModalVariant::Form { actions }
}

/// Helper function to create alert modal variant
pub fn alert_modal(
    message: impl Into<String>,
    action_label: impl Into<String>,
    action_color: impl Into<String>,
    on_action: Option<Callback<()>>,
) -> ModalVariant {
    ModalVariant::Alert {
        message: message.into(),
        action_label: action_label.into(),
        action_color: action_color.into(),
        on_action,
    }
}

/// DaisyUI Badge component for Dioxus with simple interface.
/// - `label`: The badge text.
/// - `color`: Optional DaisyUI color (e.g., "primary", "secondary", "accent", "info", "success", "warning", "error").
/// - `class`: Optional extra classes.
#[component]
pub fn Badge(
    label: String,
    #[props(optional)] color: Option<String>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let color = color.as_deref().unwrap_or("primary");
    let class = class.as_deref().unwrap_or("");
    rsx! {
        span {
            class: format!("badge badge-{} {}", color, class),
            "{label}"
        }
    }
}

#[cfg(test)]
mod badge_tests {
    use super::*;
    use dioxus::prelude::*;
    use dioxus_ssr::render;

    #[test]
    fn badge_renders_with_label_and_color() {
        let props = BadgeProps {
            label: "Test".to_string(),
            color: Some("primary".to_string()),
            class: None,
        };
        let dom = VirtualDom::new_with_props(Badge, props);
        let rendered = dioxus_ssr::render(&dom);
        assert!(rendered.contains("Test"));
        assert!(rendered.contains("badge-primary"));
    }

    #[test]
    fn badge_renders_with_custom_class() {
        let props = BadgeProps {
            label: "Custom".to_string(),
            color: Some("success".to_string()),
            class: Some("badge-lg".to_string()),
        };
        let dom = VirtualDom::new_with_props(Badge, props);
        let rendered = dioxus_ssr::render(&dom);
        assert!(rendered.contains("Custom"));
        assert!(rendered.contains("badge-success"));
        assert!(rendered.contains("badge-lg"));
    }
}
