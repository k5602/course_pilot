use crate::types::{ContextualPanelTab, Route};
use crate::ui::hooks::use_app_state;
use crate::ui::notes_panel::NotesPanel;
use dioxus::prelude::*;
use dioxus_motion::prelude::*;

const CONTEXT_PANEL_WIDTH: &str = "w-0 md:w-96";
const CONTEXT_PANEL_BG: &str =
    "bg-base-200 bg-opacity-90 backdrop-blur-md border-l border-base-300";

/// Clean contextual panel component
#[component]
pub fn ContextualPanel() -> Element {
    let mut app_state = use_app_state();
    let is_open = app_state.read().contextual_panel.is_open;
    let active_tab = app_state.read().contextual_panel.active_tab;
    let current_route = app_state.read().current_route;

    // Determine current course_id from route
    let current_course_id = match current_route {
        Route::PlanView(course_id) => Some(course_id),
        _ => None,
    };

    // Animation
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

            // Tab navigation
            div {
                role: "tablist",
                class: "tabs tabs-boxed p-2 bg-transparent",

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

            // Tab content
            div {
                class: "flex-1 overflow-y-auto",
                {render_tab_content(active_tab, current_course_id)}
            }
        }
    }
}

/// Render tab content based on active tab
fn render_tab_content(active_tab: ContextualPanelTab, course_id: Option<uuid::Uuid>) -> Element {
    match active_tab {
        ContextualPanelTab::Notes => rsx!(NotesPanel {
            course_id: course_id
        }),
        ContextualPanelTab::Player => rsx! {
            div {
                class: "p-4",
                h2 { class: "text-lg font-semibold", "Video Player" }
                p { class: "text-base-content/70", "Player will be implemented in a future phase." }
            }
        },
    }
}
