//! Discord presence health indicator component.

use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

use crate::ui::state::AppState;

/// Shows a connection status indicator for Discord Rich Presence.
#[component]
pub fn PresenceHealth() -> Element {
    let state = use_context::<AppState>();

    let is_connected = use_signal(|| false);
    let polling_started = use_signal(|| false);

    {
        let backend = state.backend.clone();
        let mut polling_started = polling_started;
        use_effect(move || {
            if *polling_started.read() {
                return;
            }
            polling_started.set(true);

            let backend_inner = backend.clone();
            let mut is_connected_inner = is_connected;
            spawn(async move {
                loop {
                    let connected = backend_inner
                        .as_ref()
                        .map(|ctx| ctx.presence.is_connected())
                        .unwrap_or(false);
                    is_connected_inner.set(connected);
                    sleep(Duration::from_secs(2)).await;
                }
            });
        });
    }

    let (badge_class, label) = if *is_connected.read() {
        ("badge badge-success", "Discord: Connected")
    } else {
        ("badge badge-warning", "Discord: Disconnected")
    };

    rsx! {
        span { class: "{badge_class}", "{label}" }
    }
}
