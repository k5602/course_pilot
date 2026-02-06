//! Main three-panel layout: Sidebar | Content | Right Panel

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::ui::Route;
use crate::ui::custom::{OnboardingTour, PresenceSync, RightPanel, Sidebar};

/// Main application layout with three panels.
#[component]
pub fn MainLayout() -> Element {
    rsx! {
        PresenceSync {}

        div { class: "flex h-screen bg-base-100",

            // Left: Sidebar
            Sidebar {}

            // Center: Main content (router outlet)
            main { class: "flex-1 overflow-auto", AnimatedOutlet::<Route> {} }

            // Right: Notes + AI Chat panel
            RightPanel {}

            OnboardingTour {}
        }
    }
}
