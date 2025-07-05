use crate::types::{AppState, Route};
use crate::ui::navigation::handle_navigation_with_fallback;
use crate::ui::theme::ThemeToggle;
use dioxus::prelude::*;

/// Sidebar navigation item
#[derive(Clone)]
struct NavItem {
    label: &'static str,
    icon: &'static str, // Unicode or SVG path for simplicity
    route: Route,
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        label: "Dashboard",
        icon: "🏠",
        route: Route::Dashboard,
    },
    NavItem {
        label: "Add Course",
        icon: "➕",
        route: Route::AddCourse,
    },
    // PlanView is dynamic, not shown in sidebar
];

/// Layout component: sidebar, app bar, main content
#[component]
pub fn Layout() -> Element {
    use crate::ui::{AddCourseDialog, PlanView, course_dashboard};
    let app_state = use_context::<Signal<AppState>>();
    let current_route = app_state.read().current_route.clone();

    // Sidebar collapsed state - collapsed by default
    let mut sidebar_collapsed = use_signal(|| true);

    // Load sidebar state from config file on mount
    use_effect(move || {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("course_pilot").join("ui_state.json");
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(ui_state) = serde_json::from_str::<serde_json::Value>(&contents) {
                    if let Some(collapsed) = ui_state.get("sidebar_collapsed") {
                        if let Some(state) = collapsed.as_bool() {
                            sidebar_collapsed.set(state);
                        }
                    }
                }
            }
        }
    });

    // Save sidebar state to config file whenever it changes
    use_effect(move || {
        let state = *sidebar_collapsed.read();
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("course_pilot");
            let _ = std::fs::create_dir_all(&config_path);
            let config_file = config_path.join("ui_state.json");

            let ui_state = serde_json::json!({
                "sidebar_collapsed": state
            });

            let _ = std::fs::write(
                &config_file,
                serde_json::to_string_pretty(&ui_state).unwrap_or_default(),
            );
        }
    });

    // Sidebar width
    let sidebar_width = if *sidebar_collapsed.read() {
        "60px"
    } else {
        "220px"
    };

    // Handle navigation with safe navigation system
    let on_nav = move |route: Route| {
        handle_navigation_with_fallback(app_state, route);
    };

    // App bar branding
    let brand = "Course Pilot";

    rsx! {
        style {
            {
                let sidebar_align = if *sidebar_collapsed.read() { "center" } else { "flex-start" };
                let sidebar_justify = if *sidebar_collapsed.read() { "center" } else { "space-between" };
                format!(
                    r#"
                    .layout-root {{
                        display: flex;
                        height: 100vh;
                        width: 100vw;
                        background: var(--bg);
                        color: var(--fg);
                        transition: background 0.2s, color 0.2s;
                    }}
                    .sidebar {{
                        width: {};
                        background: var(--sidebar-bg);
                        color: var(--sidebar-fg);
                        display: flex;
                        flex-direction: column;
                        align-items: {};
                        padding: 0.5rem 0;
                        transition: width 0.2s;
                        box-shadow: 2px 0 8px #0001;
                        z-index: 2;
                    }}
                    .sidebar-header {{
                        width: 100%;
                        display: flex;
                        align-items: center;
                        justify-content: {};
                        padding: 0 1rem 1rem 1rem;
                        font-weight: bold;
                        font-size: 1.2rem;
                        letter-spacing: 0.04em;
                        border-bottom: 1px solid #3334;
                    }}
                    .sidebar-nav {{
                        flex: 1;
                        width: 100%;
                        display: flex;
                        flex-direction: column;
                        gap: 0.25rem;
                        margin-top: 1rem;
                    }}
                    .sidebar-nav-item {{
                        display: flex;
                        align-items: center;
                        gap: 1rem;
                        width: 100%;
                        padding: 0.7rem 1.2rem;
                        font-size: 1rem;
                        border-radius: 0.5rem;
                        cursor: pointer;
                        background: none;
                        border: none;
                        color: inherit;
                        transition: background 0.15s, color 0.15s;
                        outline: none;
                    }}
                    .sidebar-nav-item.active, .sidebar-nav-item:focus {{
                        background: var(--sidebar-active);
                        color: var(--sidebar-fg);
                    }}
                    .sidebar-nav-item:hover {{
                        background: var(--sidebar-hover);
                    }}
                    .sidebar-toggle {{
                        margin: 1rem auto 0.5rem auto;
                        background: none;
                        border: none;
                        color: #bbb;
                        cursor: pointer;
                        font-size: 1.3rem;
                        border-radius: 50%;
                        width: 2.2rem;
                        height: 2.2rem;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        transition: background 0.15s;
                    }}
                    .sidebar-toggle:hover {{
                        background: var(--sidebar-hover);
                    }}
                    .appbar {{
                        height: 56px;
                        background: var(--appbar-bg);
                        color: var(--appbar-fg);
                        border-bottom: 1px solid var(--appbar-border);
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        padding: 0 2rem;
                        box-shadow: 0 2px 8px #0001;
                        position: sticky;
                        top: 0;
                        z-index: 1;
                    }}
                    .appbar-brand {{
                        font-size: 1.3rem;
                        font-weight: bold;
                        letter-spacing: 0.03em;
                        display: flex;
                        align-items: center;
                        gap: 0.7rem;
                    }}
                    .appbar-actions {{
                        display: flex;
                        align-items: center;
                        gap: 1.2rem;
                    }}
                    .theme-toggle-btn {{
                        background: none;
                        border: none;
                        color: inherit;
                        cursor: pointer;
                        font-size: 1.3rem;
                        border-radius: 50%;
                        width: 2.2rem;
                        height: 2.2rem;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        transition: background 0.15s;
                    }}
                    .theme-toggle-btn:hover {{
                        background: var(--bg-secondary);
                    }}
                    .main-content {{
                        flex: 1;
                        min-width: 0;
                        background: inherit;
                        padding: 2.5rem 2.5rem 2.5rem 2.5rem;
                        overflow-y: auto;
                        height: 100vh;
                    }}
                    .import-status-bar {{
                        background: #fffae6;
                        color: #7c5e00;
                        padding: 0.7rem 1.2rem;
                        border-radius: 0.5rem;
                        margin-bottom: 1.5rem;
                        font-size: 1rem;
                        display: flex;
                        align-items: center;
                        gap: 0.7rem;
                        box-shadow: 0 2px 8px #0001;
                    }}
                    @media (max-width: 900px) {{
                        .main-content {{
                            padding: 1rem;
                        }}
                        .appbar {{
                            padding: 0 1rem;
                        }}
                    }}
                    @media (max-width: 600px) {{
                        .sidebar {{
                            width: 0;
                            min-width: 0;
                            overflow: hidden;
                        }}
                        .main-content {{
                            padding: 0.5rem;
                        }}
                    }}
                    "#,
                    sidebar_width,
                    sidebar_align,
                    sidebar_justify
                )
            }
        }
        div { class: "layout-root",
            nav { class: "sidebar",
                div { class: "sidebar-header",
                    if !*sidebar_collapsed.read() {
                        span { "{brand}" }
                    }
                    button {
                        class: "sidebar-toggle",
                        r#type: "button",
                        onclick: move |_| sidebar_collapsed.set(!sidebar_collapsed()),
                        aria_label: "Collapse sidebar",
                        if *sidebar_collapsed.read() { "»" } else { "«" }
                    }
                }
                {
                    let mut nav_buttons = Vec::new();
                    for nav_item in NAV_ITEMS.iter() {
                        let is_active = current_route == nav_item.route;
                        nav_buttons.push(rsx! {
                            button {
                                class: if is_active { "sidebar-nav-item active" } else { "sidebar-nav-item" },
                                r#type: "button",
                                tabindex: "0",
                                onclick: {
                                    let route = nav_item.route.clone();
                                    move |_| on_nav(route.clone())
                                },
                                aria_label: nav_item.label,
                                span { style: "font-size:1.3rem;", "{nav_item.icon}" }
                                if !*sidebar_collapsed.read() {
                                    span { "{nav_item.label}" }
                                }
                            }
                        });
                    }
                    rsx! {
                        div { class: "sidebar-nav", { nav_buttons.into_iter() } }
                    }
                }
            }
            div { style: "flex:1; display:flex; flex-direction:column; min-width:0;",
                header { class: "appbar",
                    div { class: "appbar-brand",
                        "🎓"
                        span { "{brand}" }
                    }
                    div { class: "appbar-actions",
                        ThemeToggle {}
                    }
                }
                main { class: "main-content",
                    if app_state.read().active_import.is_some() {
                        div {
                            class: "import-status-bar",
                            "🔄 Import in progress... Check the dashboard for details."
                        }
                    }
                    match current_route {
                        Route::Dashboard => rsx! { course_dashboard {} },
                        Route::AddCourse => rsx! { AddCourseDialog {} },
                        Route::PlanView(course_id) => rsx! { PlanView { course_id } }
                    }
                }
            }
        }
    }
}
