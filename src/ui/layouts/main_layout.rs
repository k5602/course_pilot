//! Main three-panel layout: Sidebar | Content | Right Panel

use dioxus::prelude::*;

use crate::ui::Route;
use crate::ui::custom::{RightPanel, Sidebar};
use crate::ui::state::AppState;

/// Main application layout with three panels.
#[component]
pub fn MainLayout() -> Element {
    let _state = use_context::<AppState>();

    rsx! {
        div {
            class: "flex h-screen bg-base-100",

            // Left: Sidebar
            Sidebar {}

            // Center: Main content (router outlet)
            main {
                class: "flex-1 overflow-auto",
                Outlet::<Route> {}
            }

            // Right: Notes + AI Chat panel
            RightPanel {}
        }
    }
}
