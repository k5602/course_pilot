//! Root App component

use std::sync::Arc;

use dioxus::prelude::*;

use crate::application::{AppConfig, AppContext};
use crate::ui::Route;
use crate::ui::state::AppState;

/// Root application component.
#[component]
pub fn App() -> Element {
    // Initialize backend on first render
    let app_state = use_signal(|| {
        // Load config from environment
        let config = AppConfig::from_env();

        // Try to create backend context
        match AppContext::new(config) {
            Ok(ctx) => AppState::with_backend(Arc::new(ctx)),
            Err(e) => {
                log::error!("Failed to initialize backend: {}", e);
                AppState::new()
            },
        }
    });

    // Provide global state
    use_context_provider(move || app_state.read().clone());

    rsx! {
        // Tailwind CSS + DaisyUI (built output)
        document::Link {
            rel: "stylesheet",
            href: asset!("/assets/tailwind.out.css"),
        }

        // dx-components theme CSS
        document::Link {
            rel: "stylesheet",
            href: asset!("/assets/dx-components-theme.css"),
        }

        Router::<Route> {}
    }
}
