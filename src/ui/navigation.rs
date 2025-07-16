/// Navigation and sidebar logic for Course Pilot UI.
/// Uses project Route enum for route management and sidebar state.
use crate::types::Route;
use dioxus::prelude::*;
use uuid::Uuid;

/// Sidebar navigation item definition.
#[derive(Clone)]
pub struct NavItem {
    pub label: &'static str,
    pub icon: Option<&'static str>, // Icon name or SVG path
    pub route: Route,
}

/// List of sidebar navigation items.
fn sidebar_nav_items() -> Vec<NavItem> {
    vec![
        NavItem {
            label: "Dashboard",
            icon: Some("dashboard"), // Replace with actual icon ref
            route: Route::Dashboard,
        },
        NavItem {
            label: "Planner",
            icon: Some("calendar"),
            route: Route::PlanView(Uuid::nil()),
        },
        NavItem {
            label: "Settings",
            icon: Some("settings"),
            route: Route::Settings,
        },
    ]
}

/// Hook for sidebar open/collapse state (expand on hover, collapse on mobile).

/// Main navigation component (to be used in layout).
#[component]
pub fn Navigation() -> Element {
    let nav_items = sidebar_nav_items();
    // Sidebar open/collapse state (expand on hover, collapse on mobile)
    let sidebar_open = use_signal(|| true);

    rsx! {
        nav {
            class: "sidebar-nav flex flex-col gap-2",
            ul {
                {nav_items.iter().cloned().enumerate().map(|(idx, item)| {
                    let label = item.label;
                    let icon = item.icon;
                    let sidebar_open = sidebar_open.clone();
                    let route = item.route.clone();
                    rsx! {
                        li {
                            key: "{idx}",
                            button {
                                class: "sidebar-link flex items-center gap-2 px-4 py-2 rounded hover:bg-base-200 transition-colors",
                                onclick: move |_| {
                                    let mut app_state = crate::ui::hooks::use_app_state();
                                    app_state.write().current_route = route.clone();
                                },
                                // Render icon if present
                                {icon.map(|icon| rsx!(
                                    span { class: "icon", "{icon}" }
                                ))},
                                // Show label only if sidebar is open
                                {
                                    if sidebar_open() {
                                        rsx!(span { "{label}" })
                                    } else {
                                        rsx!()
                                    }
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
