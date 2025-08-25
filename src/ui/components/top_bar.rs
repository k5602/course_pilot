use crate::state::{
    close_contextual_panel_reactive, set_contextual_panel_tab_reactive,
    toggle_mobile_sidebar_reactive, use_contextual_panel_reactive,
};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaBars, FaNoteSticky};

#[component]
pub fn TopBar() -> Element {
    let contextual_panel = use_contextual_panel_reactive();

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
                        toggle_mobile_sidebar_reactive();
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
            div {
                class: "tooltip tooltip-left",
                "data-tip": if contextual_panel.read().is_open { "Hide Notes Panel" } else { "Show Notes Panel" },
                button {
                    class: format!("btn btn-square {}",
                        if contextual_panel.read().is_open &&
                           contextual_panel.read().active_tab == crate::types::ContextualPanelTab::Notes {
                            "btn-primary"
                        } else {
                            "btn-ghost"
                        }
                    ),
                    onclick: move |_| {
                        let is_open = contextual_panel.read().is_open;
                        let active_tab = contextual_panel.read().active_tab;

                        if is_open && active_tab == crate::types::ContextualPanelTab::Notes {
                            close_contextual_panel_reactive();
                        } else {
                            set_contextual_panel_tab_reactive(crate::types::ContextualPanelTab::Notes);
                        }
                    },
                    Icon { icon: FaNoteSticky, class: "w-6 h-6" }
                }
            }
        }
    }
}
