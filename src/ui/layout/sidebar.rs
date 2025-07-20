use dioxus::prelude::*;
use crate::types::Route;
use crate::ui::components::sidebar_nav::SidebarNav;
use crate::ui::theme_unified::ThemeToggleButton;
use crate::ui::hooks::use_app_state;

// Layout constants
const SIDEBAR_WIDTH_DESKTOP: &str = "w-20";
const SIDEBAR_WIDTH_EXPANDED: &str = "w-56";
const SIDEBAR_BG: &str = "bg-base-200 bg-opacity-70 backdrop-blur-md border-r border-base-300";

#[derive(Props, PartialEq, Clone)]
pub struct SidebarProps {
    pub current_route: Route,
    pub is_mobile_open: bool,
    pub is_hovered: bool,
    pub on_hover: EventHandler<bool>,
}

/// Clean sidebar component with navigation and theme toggle
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let mut app_state = use_app_state();
    
    let sidebar_width = if props.is_hovered {
        SIDEBAR_WIDTH_EXPANDED
    } else {
        SIDEBAR_WIDTH_DESKTOP
    };

    let mobile_translate = if props.is_mobile_open {
        "translate-x-0"
    } else {
        "-translate-x-full"
    };

    let on_route_change = EventHandler::new({
        let mut app_state = app_state.clone();
        move |new_route: Route| {
            app_state.write().current_route = new_route;
        }
    });

    rsx! {
        // Mobile backdrop
        if props.is_mobile_open {
            div {
                class: "fixed inset-0 bg-black/30 z-10 md:hidden",
                onclick: move |_| app_state.write().sidebar_open_mobile = false,
            }
        }

        // Sidebar navigation
        nav {
            class: "{sidebar_width} {SIDEBAR_BG} glass flex flex-col items-stretch py-4 space-y-4 fixed left-0 top-0 bottom-0 z-20 transition-all duration-300 md:translate-x-0 {mobile_translate}",
            onmouseenter: move |_| props.on_hover.call(true),
            onmouseleave: move |_| props.on_hover.call(false),
            
            SidebarNav { 
                current_route: props.current_route, 
                on_route_change: on_route_change, 
                is_expanded: props.is_hovered || props.is_mobile_open 
            }
            
            div { class: "flex-1" }
            
            ThemeToggleButton { 
                icon_only: !(props.is_hovered || props.is_mobile_open) 
            }
        }
    }
}