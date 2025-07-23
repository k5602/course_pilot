use crate::types::{ContextualPanelTab, Route};
use crate::ui::hooks::use_app_state;
use crate::ui::notes_panel::{NotesPanel, NotesPanelMode};
use dioxus::prelude::*;
use dioxus_motion::prelude::*;

const CONTEXT_PANEL_BG: &str =
    "bg-base-100 border-l border-base-300 shadow-lg";

/// Clean contextual panel component
#[component]
pub fn ContextualPanel() -> Element {
    let mut app_state = use_app_state();
    let is_open = app_state.read().contextual_panel.is_open;
    let active_tab = app_state.read().contextual_panel.active_tab;
    let current_route = app_state.read().current_route;

    // Debug logging
    log::info!("ContextualPanel render: is_open={}, active_tab={:?}", is_open, active_tab);

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
        "w-96 {CONTEXT_PANEL_BG} fixed right-0 top-0 bottom-0 z-30 flex flex-col overflow-hidden"
    );

    rsx! {
        aside {
            class: "{container_class}",
            style: "{panel_style}",

            // Tab navigation - always render to keep hooks active
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

            // Tab content - always render to keep hooks active
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
        ContextualPanelTab::Notes => {
            let mode = match course_id {
                Some(id) => NotesPanelMode::CourseNotes(id),
                None => NotesPanelMode::AllNotes,
            };
            rsx!(NotesPanel { mode: mode })
        }
        ContextualPanelTab::Player => rsx! {
            div {
                class: "p-4",
                h2 { class: "text-lg font-semibold", "Video Player" }
                p { class: "text-base-content/70", "Player will be implemented in a future phase." }
            }
        },
    }
}
