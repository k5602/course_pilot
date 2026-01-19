//! Presence synchronization component.
//!
//! Wraps the `use_presence_sync` hook in a tiny component so it can be placed
//! inside the Router hierarchy without cluttering layout code.

use dioxus::prelude::*;

use crate::ui::hooks::use_presence_sync;
use crate::ui::state::AppState;

/// Initializes Discord Rich Presence synchronization.
///
/// Place this inside a Router descendant (e.g., `MainLayout`) so `use_route`
/// is valid.
#[component]
pub fn PresenceSync() -> Element {
    let state = use_context::<AppState>();
    use_presence_sync(state.backend.clone());
    rsx! {}
}
