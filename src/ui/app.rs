//! Root App component

use std::sync::Arc;

use dioxus::prelude::*;

use crate::application::{AppConfig, AppContext};
use crate::infrastructure::embed_relay::EmbedRelayServer;
use crate::infrastructure::media_relay::MediaRelayServer;
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

    let relay = use_signal(|| None::<EmbedRelayServer>);
    let media_relay = use_signal(|| None::<MediaRelayServer>);

    {
        let mut app_state = app_state;
        let mut relay = relay;
        use_effect(move || {
            if relay.read().is_some() {
                return;
            }

            match EmbedRelayServer::start() {
                Ok(server) => {
                    app_state
                        .write()
                        .youtube_embed_relay_url
                        .set(Some(server.base_url().to_string()));
                    relay.set(Some(server));
                },
                Err(e) => {
                    log::error!("Failed to start embed relay: {}", e);
                },
            }
        });
    }

    {
        let mut app_state = app_state;
        let mut media_relay = media_relay;
        use_effect(move || {
            if media_relay.read().is_some() {
                return;
            }

            match MediaRelayServer::start() {
                Ok(server) => {
                    app_state
                        .write()
                        .local_media_relay_url
                        .set(Some(server.base_url().to_string()));
                    media_relay.set(Some(server));
                },
                Err(e) => {
                    log::error!("Failed to start media relay: {}", e);
                },
            }
        });
    }

    // Provide global state
    use_context_provider(move || app_state.read().clone());

    rsx! {
        // Tailwind CSS + DaisyUI (built output)
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.out.css") }

        // dx-components theme CSS
        document::Link {
            rel: "stylesheet",
            href: asset!("/assets/dx-components-theme.css"),
        }

        Router::<Route> {}
    }
}
