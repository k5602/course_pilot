use crate::ui::hooks::use_app_state;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaBars, FaNoteSticky};

#[component]
pub fn TopBar() -> Element {
    let mut app_state = use_app_state();

    rsx! {
        div {
            class: "flex items-center justify-between p-2 bg-base-200 border-b border-base-300",

            // Left side: hamburger and title
            div {
                class: "flex items-center gap-2",
                // Hamburger button - only on mobile
                button {
                    class: "btn btn-ghost btn-square md:hidden",
                    onclick: move |_| {
                        let current_state = app_state.read().sidebar_open_mobile;
                        app_state.write().sidebar_open_mobile = !current_state;
                    },
                    Icon { icon: FaBars, class: "w-6 h-6" }
                },
                // Title
                h1 {
                    class: "text-lg font-bold",
                    "Course Pilot"
                }
            },

            // Right side: contextual panel toggle
            button {
                class: "btn btn-ghost btn-square",
                onclick: move |_| {
                    let is_open = app_state.read().contextual_panel.is_open;
                    app_state.write().contextual_panel.is_open = !is_open;
                },
                Icon { icon: FaNoteSticky, class: "w-6 h-6" }
            }
        }
    }
}
