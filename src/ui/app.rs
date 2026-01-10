//! Root App component

use dioxus::prelude::*;

use crate::ui::Route;
use crate::ui::state::AppState;

/// Root application component.
#[component]
pub fn App() -> Element {
    // Provide global state
    use_context_provider(AppState::new);

    rsx! {
        Router::<Route> {}
    }
}
