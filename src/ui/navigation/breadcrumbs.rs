use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaChevronRight;
use dioxus_free_icons::Icon;
use crate::types::Route;
use crate::ui::hooks::{use_navigation_manager, BreadcrumbItem};

/// Clean breadcrumb navigation component
#[component]
pub fn Breadcrumbs(current_route: Route) -> Element {
    let nav_manager = use_navigation_manager();
    
    if nav_manager.breadcrumbs.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        nav {
            class: "breadcrumbs text-sm px-4 py-2 bg-base-200/50",
            ul {
                class: "flex items-center space-x-2",
                {nav_manager.breadcrumbs.iter().enumerate().map(|(idx, item)| {
                    render_breadcrumb_item(item, idx, nav_manager.breadcrumbs.len(), &nav_manager.navigate_to)
                })}
            }
        }
    }
}

/// Render individual breadcrumb item
fn render_breadcrumb_item(
    item: &BreadcrumbItem, 
    idx: usize, 
    total: usize, 
    navigate_to: &EventHandler<Route>
) -> Element {
    let is_last = idx == total - 1;
    let item_route = item.route.clone();
    let navigate_to = navigate_to.clone();
    
    rsx! {
        li {
            key: "{idx}",
            class: "flex items-center",
            
            if let Some(route) = item_route {
                if !is_last {
                    button {
                        class: "link link-hover text-base-content/70 hover:text-base-content",
                        onclick: move |_| navigate_to.call(route),
                        "{item.label}"
                    }
                } else {
                    span {
                        class: "text-base-content font-medium",
                        "{item.label}"
                    }
                }
            } else {
                span {
                    class: if is_last { "text-base-content font-medium" } else { "text-base-content/70" },
                    "{item.label}"
                }
            }
            
            if !is_last {
                Icon {
                    icon: FaChevronRight,
                    class: "w-3 h-3 mx-2 text-base-content/40"
                }
            }
        }
    }
}