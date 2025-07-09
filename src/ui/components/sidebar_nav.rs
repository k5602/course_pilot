use course_pilot::types::Route;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaBook, FaBug, FaGauge, FaGear};
use dioxus_free_icons::Icon;
use uuid::Uuid;

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

/// SidebarNav: Vertical navigation for the sidebar.
/// - Icon-only by default, expands to show labels on hover (desktop)
/// - Responsive: collapses to hamburger on small screens (future enhancement)
#[component]
pub fn SidebarNav(
    current_route: Route,
    on_route_change: EventHandler<Route>,
    is_expanded: bool,
) -> Element {
    rsx! {
        ul {
            class: "flex flex-col gap-2 w-full",
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

/// Data for a navigation item
struct NavItem {
    icon: IconData,
    label: &'static str,
    route: Route,
}

/// Supported routes (expand as needed)
/* Route enum is now defined in layout.rs as MainRoute */

/// IconData: Enum for supported icons
#[derive(Clone, Copy, PartialEq)]
enum IconData {
    Dashboard,
    LibraryBooks,
    Settings,
    Bug,
}

impl IconData {
    fn render(self) -> Element {
        match self {
            IconData::Bug => rsx! { Icon { icon: FaBug, width: 20, height: 20 } },
            IconData::Dashboard => rsx!(Icon {
                icon: FaGauge,
                class: "w-6 h-6",
            }),
            IconData::LibraryBooks => rsx!(Icon {
                icon: FaBook,
                class: "w-6 h-6",
            }),
            IconData::Settings => rsx!(Icon {
                icon: FaGear,
                class: "w-6 h-6",
            }),
        }
    }
}

/// SidebarNavItem: Single navigation item with icon and label
#[component]
fn SidebarNavItem(
    icon: Element,
    label: &'static str,
    active: bool,
    is_expanded: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    use dioxus_motion::prelude::*;
    let mut scale = use_motion(1.0f32);
    let mut bg_opacity = use_motion(if active { 1.0 } else { 0.0 });

    let base_class = "flex items-center gap-4 px-4 py-2 rounded-lg cursor-pointer transition-colors duration-200";
    let active_class = if active {
        "bg-primary text-primary-content"
    } else {
        "hover:bg-base-300"
    };

    use_effect(move || {
        if active {
            scale.animate_to(
                1.05,
                AnimationConfig::new(AnimationMode::Spring(Spring::default())),
            );
            bg_opacity.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Tween(Tween::default())),
            );
        } else {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(Spring::default())),
            );
            bg_opacity.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Tween(Tween::default())),
            );
        }
    });

    let style = use_memo(move || {
        format!(
            "transform: scale({}); background-color: rgba(var(--p), {}); transition: transform 0.2s, background-color 0.2s;",
            scale.get_value(),
            bg_opacity.get_value()
        )
    });

    rsx! {
        li {
            class: "w-full",
            button {
                class: "{base_class} {active_class} w-full",
                style: "{style}",
                onmouseenter: move |_| scale.animate_to(1.08, AnimationConfig::new(AnimationMode::Spring(Spring::default()))),
                onmouseleave: move |_| if !active { scale.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(Spring::default()))) },
                onclick: move |evt| on_click.call(evt),
                {icon}
                if is_expanded {
                    span { class: "text-sm font-medium", "{label}" }
                }
            }
        }
    }
}
