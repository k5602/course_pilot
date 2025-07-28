use crate::types::Route;
use crate::ui::hooks::{BreadcrumbItem, use_navigation_manager};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaChevronRight, FaHouse};

/// Enhanced breadcrumb navigation component with router context integration
#[component]
pub fn Breadcrumbs(current_route: Route) -> Element {
    let nav_manager = use_navigation_manager();
    let navigator = use_navigator();

    // Always show at least a home breadcrumb
    let breadcrumbs = if nav_manager.breadcrumbs.is_empty() {
        vec![BreadcrumbItem {
            label: "Dashboard".to_string(),
            route: Some(Route::Dashboard {}),
            active: matches!(current_route, Route::Dashboard {}),
        }]
    } else {
        nav_manager.breadcrumbs
    };

    rsx! {
        nav {
            class: "breadcrumbs text-sm px-4 py-2 bg-base-200/50 border-b border-base-300",
            "aria-label": "Breadcrumb navigation",
            ul {
                class: "flex items-center space-x-2",

                // Home icon for first breadcrumb
                li {
                    class: "flex items-center",
                    if let Some(first_item) = breadcrumbs.first() {
                        if let Some(route) = &first_item.route {
                            Link {
                                to: route.clone(),
                                class: "link link-hover text-base-content/70 hover:text-base-content flex items-center gap-1",
                                "aria-label": "Go to {first_item.label}",
                                Icon { icon: FaHouse, class: "w-3 h-3" }
                                span { class: "hidden sm:inline", "{first_item.label}" }
                            }
                        } else {
                            span {
                                class: "text-base-content font-medium flex items-center gap-1",
                                Icon { icon: FaHouse, class: "w-3 h-3" }
                                span { class: "hidden sm:inline", "{first_item.label}" }
                            }
                        }
                    }

                    if breadcrumbs.len() > 1 {
                        Icon {
                            icon: FaChevronRight,
                            class: "w-3 h-3 mx-2 text-base-content/40"
                        }
                    }
                }

                // Remaining breadcrumbs
                {breadcrumbs.iter().skip(1).enumerate().map(|(idx, item)| {
                    let is_last = idx == breadcrumbs.len() - 2; // -2 because we skipped first
                    render_breadcrumb_item(item, idx + 1, breadcrumbs.len(), is_last, navigator)
                })}
            }
        }
    }
}

/// Render individual breadcrumb item with enhanced accessibility and navigation
fn render_breadcrumb_item(
    item: &BreadcrumbItem,
    idx: usize,
    _total: usize,
    is_last: bool,
    navigator: Navigator,
) -> Element {
    rsx! {
        li {
            key: "{idx}",
            class: "flex items-center",

            if let Some(route) = &item.route {
                if !is_last {
                    Link {
                        to: route.clone(),
                        class: "link link-hover text-base-content/70 hover:text-base-content transition-colors duration-200 max-w-32 truncate",
                        "aria-label": "Navigate to {item.label}",
                        title: "{item.label}",
                        "{item.label}"
                    }
                } else {
                    span {
                        class: "text-base-content font-medium max-w-32 truncate",
                        title: "{item.label}",
                        "{item.label}"
                    }
                }
            } else {
                if !is_last {
                    button {
                        class: "link link-hover text-base-content/70 hover:text-base-content transition-colors duration-200 max-w-32 truncate",
                        onclick: move |_| {
                            // For items without routes, try to navigate back
                            navigator.go_back();
                        },
                        "aria-label": "Go back to {item.label}",
                        title: "{item.label}",
                        "{item.label}"
                    }
                } else {
                    span {
                        class: "text-base-content font-medium max-w-32 truncate",
                        title: "{item.label}",
                        "{item.label}"
                    }
                }
            }

            if !is_last {
                Icon {
                    icon: FaChevronRight,
                    class: "w-3 h-3 mx-2 text-base-content/40 flex-shrink-0"
                }
            }
        }
    }
}
