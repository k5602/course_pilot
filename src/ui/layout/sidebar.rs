use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaBook, FaGauge, FaGear};
use uuid::Uuid;

use crate::types::Route;
use crate::ui::hooks::use_app_state;
use crate::ui::theme_unified::ThemeToggleButton;

// Navigation items configuration
const NAV_ITEMS: &[NavItem] = if cfg!(debug_assertions) {
    &[
        NavItem {
            icon: IconData::Dashboard,
            label: "Dashboard",
            route: Route::Dashboard,
        },
        NavItem {
            icon: IconData::LibraryBooks,
            label: "All Courses",
            // TODO: This should lead to a dedicated 'all courses' view, not a specific plan.
            // For now, we'll use a placeholder ID. A better approach would be a new Route variant.
            route: Route::PlanView(Uuid::nil()),
        },
        NavItem {
            icon: IconData::Settings,
            label: "Settings",
            route: Route::Settings,
        },
    ]
} else {
    &[
        NavItem {
            icon: IconData::Dashboard,
            label: "Dashboard",
            route: Route::Dashboard,
        },
        NavItem {
            icon: IconData::LibraryBooks,
            label: "All Courses",
            route: Route::PlanView(Uuid::nil()),
        },
        NavItem {
            icon: IconData::Settings,
            label: "Settings",
            route: Route::Settings,
        },
    ]
};

#[derive(Props, PartialEq, Clone)]
pub struct SidebarProps {
    pub current_route: Route,
    pub is_mobile_open: bool,
    pub is_hovered: bool,
    pub on_hover: EventHandler<bool>,
    pub on_width_change: EventHandler<f32>,
}

/// DaisyUI-styled sidebar with native hover expansion
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let mut app_state = use_app_state();

    let mobile_translate = if props.is_mobile_open {
        "translate-x-0"
    } else {
        "-translate-x-full"
    };

    let on_route_change = EventHandler::new({
        let mut app_state = app_state;
        move |new_route: Route| {
            app_state.write().current_route = new_route;
        }
    });

    rsx! {
        // Mobile backdrop overlay (DaisyUI drawer-overlay)
        if props.is_mobile_open {
            div {
                class: "drawer-overlay fixed inset-0 z-10 md:hidden bg-black/50",
                onclick: move |_| app_state.write().sidebar_open_mobile = false,
            }
        }

        // DaisyUI-styled sidebar with native width transitions
        aside {
            class: format!(
                "fixed top-0 left-0 z-20 h-screen transition-all duration-300 ease-in-out {}",
                if props.is_hovered || props.is_mobile_open { "w-60" } else { "w-16" }
            ),
            onmouseenter: move |_| {
                props.on_hover.call(true);
            },
            onmouseleave: move |_| {
                props.on_hover.call(false);
            },

            div {
                class: format!(
                    "h-full bg-base-200 flex flex-col shadow-lg overflow-hidden transition-transform duration-300 md:translate-x-0 {}",
                    mobile_translate
                ),

                // Navigation menu using DaisyUI menu component
                SidebarNav {
                    current_route: props.current_route,
                    on_route_change: on_route_change,
                    is_expanded: props.is_hovered || props.is_mobile_open
                }

                // Spacer to push theme toggle to bottom
                div { class: "flex-1" }

                // Theme toggle at bottom
                div {
                    class: "p-4",
                    ThemeToggleButton {
                        icon_only: !(props.is_hovered || props.is_mobile_open)
                    }
                }
            }
        }
    }
}

/// DaisyUI menu-based navigation component
#[component]
fn SidebarNav(
    current_route: Route,
    on_route_change: EventHandler<Route>,
    is_expanded: bool,
) -> Element {
    rsx! {
        ul {
            class: "menu bg-base-200 w-full p-2",
            {NAV_ITEMS.iter().map(|item| {
                let is_active = std::mem::discriminant(&item.route) == std::mem::discriminant(&current_route);

                rsx! {
                    SidebarNavItem {
                        icon: item.icon.render(),
                        label: item.label,
                        active: is_active,
                        is_expanded: is_expanded,
                        on_click: move |_| on_route_change.call(item.route),
                    }
                }
            })}
        }
    }
}

/// DaisyUI menu item with tooltip support
#[component]
fn SidebarNavItem(
    icon: Element,
    label: &'static str,
    active: bool,
    is_expanded: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let active_class = if active { "menu-active" } else { "" };
    let tooltip_class = if !is_expanded {
        "tooltip tooltip-right"
    } else {
        ""
    };

    rsx! {
        li {
            class: "w-full",
            a {
                class: "flex items-center gap-3 {active_class} {tooltip_class}",
                "data-tip": if !is_expanded { label } else { "" },
                onclick: move |evt| on_click.call(evt),

                {icon}

                if is_expanded {
                    span {
                        class: "text-sm font-medium",
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Data structure for navigation items
struct NavItem {
    icon: IconData,
    label: &'static str,
    route: Route,
}

/// Icon data enum for sidebar navigation
#[derive(Clone, Copy, PartialEq)]
enum IconData {
    Dashboard,
    LibraryBooks,
    Settings,
}

impl IconData {
    fn render(self) -> Element {
        match self {
            IconData::Dashboard => rsx!(Icon {
                icon: FaGauge,
                class: "w-5 h-5",
            }),
            IconData::LibraryBooks => rsx!(Icon {
                icon: FaBook,
                class: "w-5 h-5",
            }),
            IconData::Settings => rsx!(Icon {
                icon: FaGear,
                class: "w-5 h-5",
            }),
        }
    }
}
