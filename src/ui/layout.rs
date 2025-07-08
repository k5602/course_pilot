use crate::ui::components::sidebar_nav::SidebarNav;
use crate::ui::components::top_bar::TopBar;
use crate::ui::dashboard::Dashboard;
use crate::ui::notes_panel::NotesPanel;
use crate::ui::plan_view::PlanView;
use course_pilot::types::{ContextualPanelTab, Route};
use crate::ui::theme_unified::{use_theme_context, ThemeToggleButton};
use crate::ui::hooks::use_app_state;
use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use dioxus_toast::ToastManager;

// Layout constants
#[allow(dead_code)]
const SIDEBAR_WIDTH_DESKTOP: &str = "w-15";
const SIDEBAR_WIDTH_EXPANDED: &str = "w-50";
const SIDEBAR_BG: &str = "bg-base-200 bg-opacity-70 backdrop-blur-md border-r border-base-300";
const MAIN_BG: &str = "bg-base-100";
const CONTEXT_PANEL_WIDTH: &str = "w-0 md:w-96";
const CONTEXT_PANEL_BG: &str =
    "bg-base-200 bg-opacity-90 backdrop-blur-md border-l border-base-300";

// AppShell: The root layout for the application
#[component]
pub fn AppShell() -> Element {
    let theme_ctx = use_theme_context();
    let mut app_state = use_app_state();
    let route = app_state.read().current_route;
    let sidebar_open_mobile = app_state.read().sidebar_open_mobile;
    let toast = use_context::<Signal<ToastManager>>();
    let mut is_hovered = use_signal(|| false);

    let mut main_opacity = use_motion(0.0f32);
    let mut main_y = use_motion(-16.0f32);

    use_effect(move || {
        main_opacity.animate_to(1.0, AnimationConfig::new(AnimationMode::Tween(Tween::default())));
        main_y.animate_to(0.0, AnimationConfig::new(AnimationMode::Tween(Tween::default())));
    });

    use_effect(move || {
        let theme = theme_ctx.theme.read();
        let theme_name = theme.as_str().to_string();
        log::info!("🎨 Applying theme: {}", theme_name);
        spawn(async move {
            let js = format!("console.log('Applying theme: {}'); document.documentElement.setAttribute('data-theme', '{}');", theme_name, theme_name);
            if let Err(e) = document::eval(&js).await {
                log::error!("Failed to apply theme: {:?}", e);
            }
        });
    });

    let main_content_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px);",
            main_opacity.get_value(),
            main_y.get_value()
        )
    });

    let main_class = format!(
        "flex-1 flex flex-col overflow-hidden {}",
        if is_hovered() { "ml-56" } else { "ml-20" }
    );

    rsx! {
        div {
            class: "h-screen w-screen bg-base-100 font-sans transition-colors duration-300",
            dioxus_toast::ToastFrame { manager: toast }
            div {
                class: "h-full flex flex-row",
                Sidebar { 
                    route: route, 
                    on_route_change: move |new_route| app_state.write().current_route = new_route, 
                    is_mobile_open: sidebar_open_mobile,
                    is_hovered: is_hovered(),
                    on_hover: move |hover_state| is_hovered.set(hover_state)
                }
                div {
                    class: "{main_class}",
                    style: "{main_content_style}",
                    TopBar {}
                    MainContent { route: route }
                }
                ContextualPanel {}
            }
        }
    }
}

// Sidebar: Navigation and theme toggle
#[component]
fn Sidebar(route: Route, on_route_change: EventHandler<Route>, is_mobile_open: bool, is_hovered: bool, on_hover: EventHandler<bool>) -> Element {
    let mut app_state = use_app_state();

    let sidebar_width = if is_hovered { SIDEBAR_WIDTH_EXPANDED } else { SIDEBAR_WIDTH_DESKTOP };
    
    let mobile_translate = if is_mobile_open { "translate-x-0" } else { "-translate-x-full" };

    rsx! {
        // Backdrop for mobile
        if is_mobile_open {
            div {
                class: "fixed inset-0 bg-black/30 z-10 md:hidden",
                onclick: move |_| app_state.write().sidebar_open_mobile = false,
            }
        }

        // Sidebar Navigation
        nav {
            class: "{sidebar_width} {SIDEBAR_BG} glass flex flex-col items-stretch py-4 space-y-4 fixed left-0 top-0 bottom-0 z-20 transition-all duration-300 md:translate-x-0 {mobile_translate}",
            onmouseenter: move |_| on_hover.call(true),
            onmouseleave: move |_| on_hover.call(false),
            SidebarNav { current_route: route, on_route_change: on_route_change, is_expanded: is_hovered || is_mobile_open }
            div { class: "flex-1" }
            ThemeToggleButton {}
        }
    }
}

// MainContent: The central workspace area
#[component]
fn MainContent(route: Route) -> Element {
    let app_state = use_app_state();
    let panel_is_open = app_state.read().contextual_panel.is_open;
    
    let margin_right = if panel_is_open { "md:mr-96" } else { "md:mr-0" };

    rsx! {
        main {
            class: "flex-1 {margin_right} overflow-y-auto {MAIN_BG} transition-all duration-300",
            match route {
                Route::Dashboard => rsx!(Dashboard {}),
                Route::PlanView(course_id) => rsx!(PlanView { course_id: course_id }),
                Route::Settings => rsx! {
                    div {
                        class: "p-8",
                        h1 { class: "text-3xl font-bold mb-4", "Settings" }
                        p { class: "text-base-content/70", "Configure your Course Pilot preferences here." }
                    }
                },
                Route::AddCourse => rsx! { div { "Add Course UI - Not Implemented" } },
            }
        }
    }
}

// ContextualPanel: Slide-in panel for notes, player, etc.
#[component]
fn ContextualPanel() -> Element {
    let mut app_state = use_app_state();
    let is_open = app_state.read().contextual_panel.is_open;
    let active_tab = app_state.read().contextual_panel.active_tab;

    let mut panel_x = use_motion(if is_open { 0.0 } else { 100.0 });

    use_effect(move || {
        let config = AnimationConfig::new(AnimationMode::Spring(Spring::default()));
        if is_open {
            panel_x.animate_to(0.0, config);
        } else {
            panel_x.animate_to(100.0, config);
        }
    });

    let panel_style = use_memo(move || format!("transform: translateX({}%);", panel_x.get_value()));

    let container_class = format!(
        "{CONTEXT_PANEL_WIDTH} {CONTEXT_PANEL_BG} fixed right-0 top-0 bottom-0 z-30 transition-transform duration-300 hidden md:flex flex-col {}",
        if !is_open { "pointer-events-none" } else { "" }
    );

    rsx! {
        aside {
            class: "{container_class}",
            style: "{panel_style}",
            div { role: "tablist", class: "tabs tabs-boxed p-2 bg-transparent",
                a { 
                    role: "tab", 
                    class: if active_tab == ContextualPanelTab::Notes { "tab tab-active" } else { "tab" },
                    onclick: move |_| app_state.write().contextual_panel.active_tab = ContextualPanelTab::Notes,
                    "Notes"
                }
                a { 
                    role: "tab", 
                    class: if active_tab == ContextualPanelTab::Player { "tab tab-active" } else { "tab" },
                    onclick: move |_| app_state.write().contextual_panel.active_tab = ContextualPanelTab::Player,
                    "Player"
                }
            }
            div { class: "flex-1 overflow-y-auto",
                match active_tab {
                    ContextualPanelTab::Notes => rsx!(NotesPanel {}),
                    ContextualPanelTab::Player => rsx! {
                        div { class: "p-4",
                            h2 { class: "text-lg font-semibold", "Video Player" }
                            p { class: "text-base-content/70", "Player will be implemented in a future phase." }
                        }
                    }
                }
            }
        }
    }
}
