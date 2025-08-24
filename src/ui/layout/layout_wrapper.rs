use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use super::{ContextualPanel, Sidebar};
use crate::state::{use_contextual_panel_reactive, use_mobile_sidebar_reactive};
use crate::types::Route;
use crate::ui::TopBar;
use crate::ui::{Breadcrumbs, DeepLinkingHandler};

/// Layout wrapper that can be used within Router context
#[component]
pub fn LayoutWrapper(children: Element) -> Element {
    let current_route = use_route::<Route>();
    let mobile_sidebar = use_mobile_sidebar_reactive();
    let contextual_panel = use_contextual_panel_reactive();
    let sidebar_open_mobile = *mobile_sidebar.read();
    let panel_is_open = contextual_panel.read().is_open;

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

    // Calculate margins for main content area
    let sidebar_margin = if is_sidebar_hovered() {
        "ml-45"
    } else {
        "ml-20"
    };
    let panel_margin = if panel_is_open { "md:mr-96" } else { "md:mr-0" };

    let main_class = format!(
        "flex-1 flex flex-col overflow-hidden {sidebar_margin} {panel_margin} transition-all duration-300"
    );

    rsx! {
        div {
            class: "h-screen w-screen bg-base-100 font-sans transition-colors duration-300",
            div {
                class: "h-full flex flex-row",

                Sidebar {
                    current_route: current_route.clone(),
                    is_mobile_open: sidebar_open_mobile,
                    is_hovered: is_sidebar_hovered(),
                    on_hover: move |hover_state| is_sidebar_hovered.set(hover_state),
                    on_width_change: move |_width| {
                        // Width change handler for future sidebar animations
                    },
                }

                // Main content area
                main {
                    class: "{main_class} bg-base-100",
                    style: "{main_content_style}",

                    // Deep linking handler for route verification
                    DeepLinkingHandler {}

                    TopBar {}
                    Breadcrumbs { current_route: current_route }

                    div {
                        class: "flex-1 overflow-y-auto",
                        {children}
                    }
                }

                ContextualPanel {}
            }
        }
    }
}
