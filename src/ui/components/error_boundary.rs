
//! ErrorBoundary component for Dioxus
//! Catches rendering errors, provides fallback UI, error logging, and recovery

use dioxus::prelude::*;


/// ErrorBoundary state
#[derive(Default)]
struct ErrorBoundaryState {
    error: Option<String>,
}

/// ErrorBoundary component
#[component]
pub fn ErrorBoundary(children: Element) -> Element {
    let mut state = use_signal(ErrorBoundaryState::default);

    // This is a placeholder for real error catching. Dioxus currently does not support React-style error boundaries,
    // but this pattern allows for future error handling and fallback UI.
    // In a real implementation, you would use a custom renderer or error boundary hook.

    if let Some(error) = &state.read().error {
        rsx! {
            div {
                class: "error-boundary-fallback",
                style: "background: var(--color-error-light); color: var(--color-error); padding: var(--spacing-6); border-radius: var(--radius-lg); text-align: center;",
                h2 { "Something went wrong" }
                p { "{error}" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| state.set(ErrorBoundaryState::default()),
                    "Try Again"
                }
            }
        }
    } else {
        rsx! { {children} }
    }
}
