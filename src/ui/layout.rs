use crate::types::{AppState, Route};
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
    use crate::ui::{AddCourseDialog, CourseDashboard, PlanView};
    let mut app_state = use_context::<Signal<AppState>>();
    let current_route = app_state.read().current_route.clone();

    // Theme toggle (light/dark)
    let mut dark_mode = use_signal(|| false);

    // Sidebar collapsed state - collapsed by default
    let mut sidebar_collapsed = use_signal(|| true);

    // Sidebar width
    let sidebar_width = if *sidebar_collapsed.read() {
        "60px"
    } else {
        "220px"
    };

    // Handle navigation
    let mut on_nav = move |route: Route| {
        app_state.write().current_route = route;
    };

    // App bar branding
    let brand = "Course Pilot";

    // Only one rsx! block allowed per component. Move all style and UI into a single rsx! block.
    let theme_vars = if *dark_mode.read() {
        r#"
        :root {
            --bg: #181a20;
            --fg: #f5f5f5;
            --sidebar-bg: #23272f;
            --sidebar-fg: #fff;
            --sidebar-active: #3f51b5;
            --appbar-bg: #23272f;
            --appbar-fg: #fff;
        }
        "#
    } else {
        r#"
        :root {
            --bg: #f5f5f5;
            --fg: #23272f;
            --sidebar-bg: #23272f;
            --sidebar-fg: #fff;
            --sidebar-active: #3f51b5;
            --appbar-bg: #fff;
            --appbar-fg: #23272f;
        }
        "#
    };

    rsx! {
        style { "{theme_vars}" }
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
                        background: var(--bg, #f5f5f5);
                        color: var(--fg, #222);
                        transition: background 0.2s, color 0.2s;
                    }}
                    .sidebar {{
                        width: {};
                        background: var(--sidebar-bg, #23272f);
                        color: var(--sidebar-fg, #fff);
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
                        background: var(--sidebar-active, #3f51b5);
                        color: #fff;
                    }}
                    .sidebar-nav-item:hover {{
                        background: #2c3140;
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
                        background: #3335;
                    }}
                    .appbar {{
                        height: 56px;
                        background: var(--appbar-bg, #fff);
                        color: var(--appbar-fg, #23272f);
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
                        background: #eee;
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
                        button {
                            class: "theme-toggle-btn",
                            r#type: "button",
                            onclick: move |_| dark_mode.set(!dark_mode()),
                            aria_label: if *dark_mode.read() { "Switch to light mode" } else { "Switch to dark mode" },
                            if *dark_mode.read() { "🌙" } else { "☀️" }
                        }
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
                        Route::Dashboard => rsx! { CourseDashboard {} },
                        Route::AddCourse => rsx! { AddCourseDialog {} },
                        Route::PlanView(course_id) => rsx! { PlanView { course_id } }
                    }
                }
            }
        }
    }
}
