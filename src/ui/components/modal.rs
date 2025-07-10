use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// DaisyUI-based reusable Modal component for Dioxus Desktop.
/// Provides a flexible modal dialog with customizable content, actions, and theming.
/// Usage: <Modal open=... on_close=...>{ ... }</Modal>
#[component]
pub fn Modal(
    /// Whether the modal is open.
    open: bool,
    /// Callback to close the modal.
    #[props(optional)]
    on_close: Option<EventHandler<()>>,
    /// Optional modal title.
    #[props(optional)]
    title: Option<String>,
    /// Optional icon to display in the title.
    #[props(optional)]
    icon: Option<Element>,
    /// Optional actions (e.g., buttons) for the modal footer.
    #[props(optional)]
    actions: Option<Element>,
    /// Modal content.
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

    let style = format!(
        "transform: scale({}); opacity: {}; transition: transform 0.3s, opacity 0.3s;",
        scale.get_value(),
        opacity.get_value()
    );

    rsx! {
        // Modal overlay
        div {
            class: "fixed inset-0 z-40 flex items-center justify-center bg-black/40",
            onclick: move |_| {
                if let Some(handler) = &on_close {
                    handler.call(());
                }
            },
            // Prevent click propagation to overlay from modal content
            div {
                class: "modal-box bg-base-100 shadow-xl relative max-w-lg w-full mx-4",
                style: "{style}",
                onclick: move |evt| evt.stop_propagation(),
                {if let Some(title) = &title {
                    rsx!(h3 { class: "font-bold text-lg mb-2 flex items-center gap-2",
                        if let Some(icon) = &icon {
                            {icon.clone()}
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
                    {if let Some(actions) = &actions {
                        actions.clone()
                    } else {
                        rsx! {
                            button {
                                class: "btn btn-sm btn-ghost",
                                onclick: move |_| {
                                    if let Some(handler) = &on_close {
                                        handler.call(());
                                    }
                                },
                                "Close"
                            }
                        }
                    }}
                }
            }
        }
    }
}
