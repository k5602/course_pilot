use crate::state::{set_contextual_panel_tab_reactive, use_contextual_panel_reactive};
use crate::types::{ContextualPanelTab, Route, VideoContext};
use crate::ui::components::GeminiChatbot;
use crate::ui::{NotesPanel, NotesPanelMode};
use dioxus::prelude::*;
use dioxus_motion::prelude::*;

const CONTEXT_PANEL_BG: &str = "bg-base-100 border-l border-base-300 shadow-lg";

/// Clean contextual panel component
#[component]
pub fn ContextualPanel() -> Element {
    let contextual_panel = use_contextual_panel_reactive();
    let is_open = contextual_panel.read().is_open;
    let active_tab = contextual_panel.read().active_tab;
    let video_context = contextual_panel.read().video_context.clone();
    let current_route = use_route::<Route>();

    // Debug logging
    log::info!("ContextualPanel render: is_open={is_open}, active_tab={active_tab:?}");

    // Determine current course_id from route
    let current_course_id = match current_route {
        Route::PlanView { course_id } => {
            // Parse course_id string to UUID
            uuid::Uuid::parse_str(&course_id).ok()
        },
        _ => None,
    };

    // Only render when open to avoid layout issues
    if !is_open {
        return rsx! { div { class: "hidden" } };
    }

    // Animation for smooth entrance
    let mut panel_opacity = use_motion(0.0f32);
    let mut panel_x = use_motion(100.0f32);

    use_effect(move || {
        let config = AnimationConfig::new(AnimationMode::Spring(Spring::default()));
        panel_opacity.animate_to(1.0, config.clone());
        panel_x.animate_to(0.0, config);
    });

    let panel_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateX({}px);",
            panel_opacity.get_value(),
            panel_x.get_value()
        )
    });

    let container_class = format!(
        "w-96 {CONTEXT_PANEL_BG} fixed right-0 top-0 bottom-0 z-30 flex flex-col overflow-hidden shadow-xl"
    );

    rsx! {
        aside {
            class: "{container_class}",
            style: "{panel_style}",

            // Tab navigation
            div {
                role: "tablist",
                class: "tabs tabs-boxed p-2 bg-base-200/50 border-b border-base-300",

                a {
                    role: "tab",
                    class: if active_tab == ContextualPanelTab::Notes {
                        "tab tab-active tab-bordered"
                    } else {
                        "tab hover:tab-active"
                    },
                    onclick: move |_| set_contextual_panel_tab_reactive(ContextualPanelTab::Notes),
                    "Notes"
                }

                a {
                    role: "tab",
                    class: if active_tab == ContextualPanelTab::Chatbot {
                        "tab tab-active tab-bordered"
                    } else {
                        "tab hover:tab-active"
                    },
                    onclick: move |_| set_contextual_panel_tab_reactive(ContextualPanelTab::Chatbot),
                    "Assistant"
                }
            }

            // Tab content
            div {
                class: "flex-1 overflow-y-auto p-2",
                {render_tab_content(active_tab, current_course_id, video_context)}
            }
        }
    }
}

/// Render tab content based on active tab
fn render_tab_content(
    active_tab: ContextualPanelTab,
    course_id: Option<uuid::Uuid>,
    video_context: Option<VideoContext>,
) -> Element {
    match active_tab {
        ContextualPanelTab::Notes => {
            let mode = match video_context {
                Some(ctx) => NotesPanelMode::VideoNotes(
                    ctx.course_id,
                    ctx.video_index,
                    ctx.video_title,
                    ctx.module_title,
                ),
                None => match course_id {
                    Some(id) => NotesPanelMode::CourseNotes(id),
                    None => NotesPanelMode::AllNotes,
                },
            };
            rsx!(NotesPanel { mode })
        },
        ContextualPanelTab::Chatbot => rsx! {
            GeminiChatbot {}
        },
    }
}
