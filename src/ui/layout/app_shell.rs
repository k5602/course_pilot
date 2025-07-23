use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use super::{ContextualPanel, MainContent, Sidebar};
use crate::ui::hooks::use_app_state;

/// Clean app shell with proper component separation
#[component]
pub fn AppShell() -> Element {
    let app_state = use_app_state();
    let current_route = app_state.read().current_route;
    let sidebar_open_mobile = app_state.read().sidebar_open_mobile;
    let panel_is_open = app_state.read().contextual_panel.is_open;

    // Animation state
    let mut is_sidebar_hovered = use_signal(|| false);
    let mut main_opacity = use_motion(0.0f32);
    let mut main_y = use_motion(-16.0f32);

    // Animate main content entrance
    use_effect(move || {
        main_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
        main_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let main_content_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px);",
            main_opacity.get_value(),
            main_y.get_value()
        )
    });

    let main_class = format!(
        "flex-1 flex flex-col overflow-hidden {}",
        if is_sidebar_hovered() {
            "ml-45"
        } else {
            "ml-20"
        }
    );

    rsx! {
        div {
            class: "h-screen w-screen bg-base-100 font-sans transition-colors duration-300",
            div {
                class: "h-full flex flex-row",

                Sidebar {
                    current_route: current_route,
                    is_mobile_open: sidebar_open_mobile,
                    is_hovered: is_sidebar_hovered(),
                    on_hover: move |hover_state| is_sidebar_hovered.set(hover_state),
                    on_width_change: move |_width| {
                        // Width change handler for future sidebar animations
                    },
                }

                div {
                    class: "{main_class}",
                    style: "{main_content_style}",
                    MainContent {
                        current_route: current_route,
                        panel_is_open: panel_is_open,
                    }
                }

                ContextualPanel {}
            }
        }
    }
}
