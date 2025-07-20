use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaChevronRight;
use dioxus_free_icons::Icon;
use crate::types::Route;

#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub route: Option<Route>,
    pub active: bool,
}

#[component]
pub fn Breadcrumb(items: Vec<BreadcrumbItem>) -> Element {
    if items.is_empty() {
        return rsx! { div {} };
    }

    let mut app_state = crate::ui::hooks::use_app_state();

    rsx! {
        nav {
            class: "breadcrumbs text-sm px-4 py-2 bg-base-200/50",
            ul {
                class: "flex items-center space-x-2",
                {items.iter().enumerate().map(|(idx, item)| {
                    let is_last = idx == items.len() - 1;
                    let item_route = item.route.clone();
                    
                    rsx! {
                        li {
                            key: "{idx}",
                            class: "flex items-center",
                            if let Some(route) = item_route {
                                if !is_last {
                                    button {
                                        class: "link link-hover text-base-content/70 hover:text-base-content",
                                        onclick: move |_| {
                                            app_state.write().current_route = route;
                                        },
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
                })}
            }
        }
    }
}

/// Helper function to generate breadcrumbs based on current route
pub fn generate_breadcrumbs(current_route: Route, courses: &[crate::types::Course]) -> Vec<BreadcrumbItem> {
    match current_route {
        Route::Dashboard => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: None,
                active: true,
            }
        ],
        Route::PlanView(course_id) => {
            let course_name = courses
                .iter()
                .find(|c| c.id == course_id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown Course".to_string());
            
            vec![
                BreadcrumbItem {
                    label: "Dashboard".to_string(),
                    route: Some(Route::Dashboard),
                    active: false,
                },
                BreadcrumbItem {
                    label: course_name,
                    route: None,
                    active: true,
                }
            ]
        },
        Route::Settings => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard),
                active: false,
            },
            BreadcrumbItem {
                label: "Settings".to_string(),
                route: None,
                active: true,
            }
        ],
        Route::AddCourse => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard),
                active: false,
            },
            BreadcrumbItem {
                label: "Add Course".to_string(),
                route: None,
                active: true,
            }
        ],
        #[cfg(debug_assertions)]
        Route::ToastTest => vec![
            BreadcrumbItem {
                label: "Dashboard".to_string(),
                route: Some(Route::Dashboard),
                active: false,
            },
            BreadcrumbItem {
                label: "Toast Test".to_string(),
                route: None,
                active: true,
            }
        ],
    }
}