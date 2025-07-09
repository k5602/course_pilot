use dioxus::prelude::*;

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
    /// Optional actions (e.g., buttons) for the modal footer.
    #[props(optional)]
    actions: Option<Element>,
    /// Modal content.
    children: Element,
) -> Element {
    if !open {
        return rsx! {};
    }

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
                onclick: move |evt| evt.stop_propagation(),
                {if let Some(title) = &title {
                    rsx!(h3 { class: "font-bold text-lg mb-2", "{title}" })
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
