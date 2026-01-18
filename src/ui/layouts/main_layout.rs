//! Main three-panel layout: Sidebar | Content | Right Panel

use dioxus::prelude::*;

use crate::ui::Route;
use crate::ui::custom::{OnboardingTour, RightPanel, Sidebar};
use crate::ui::state::AppState;

/// Main application layout with three panels.
#[component]
pub fn MainLayout() -> Element {
    let state = use_context::<AppState>();

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
            if *state.right_panel_visible.read() {
                RightPanel {}
            }

            OnboardingTour {}
        }
    }
}
