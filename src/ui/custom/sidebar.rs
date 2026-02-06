//! Sidebar navigation component

use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{MdAssignment, MdDashboard, MdSettings};
use dioxus_free_icons::icons::md_av_icons::MdPlaylistPlay;
use dioxus_motion::prelude::*;

use crate::domain::entities::SearchResultType;
use crate::ui::Route;
use crate::ui::hooks::use_search;
use crate::ui::state::AppState;

/// Collapsible sidebar with navigation links.
#[component]
pub fn Sidebar() -> Element {
    let mut state = use_context::<AppState>();
    let collapsed = *state.sidebar_collapsed.read();
    let mut search_query = use_signal(String::new);
    let results = {
        let query = search_query.read().clone();
        use_search(state.backend.clone(), query)
    };

    let toggle_sidebar = move |_| {
        state.sidebar_collapsed.set(!collapsed);
    };

    let width_class = if collapsed { "w-16" } else { "w-60" };

    rsx! {
        aside { class: "flex flex-col h-full bg-base-200 border-r border-base-300 transition-all duration-200 {width_class}",

            // Header with toggle
            div { class: "flex items-center justify-between p-4 border-b border-base-300",
                if !collapsed {
                    span { class: "font-bold text-lg", "Course Pilot" }
                }
                button { class: "btn btn-ghost btn-sm", onclick: toggle_sidebar, "☰" }
            }

            if !collapsed {
                div { class: "p-3 border-b border-base-300",
                    div { class: "relative",
                        input {
                            class: "input input-bordered w-full text-sm",
                            r#type: "text",
                            placeholder: "Search courses, videos, notes...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }

                    {
                        let results_list = results.data.read();
                        if !search_query.read().is_empty() {
                            if results_list.is_empty() {
                                rsx! {
                                    div { class: "mt-3 text-xs text-base-content/50", "No results" }
                                }
                            } else {
                                rsx! {
                                    div { class: "mt-3 space-y-2 max-h-56 overflow-auto",
                                        for result in results_list.iter() {
                                            {
                                                let label = match result.entity_type {
                                                    SearchResultType::Course => "Course",
                                                    SearchResultType::Video => "Video",
                                                    SearchResultType::Note => "Note",
                                                };
                                                let to = match result.entity_type {
                                                    SearchResultType::Course => {
                                                        Route::CourseView {
                                                            course_id: result.entity_id.clone(),
                                                        }
                                                    }
                                                    SearchResultType::Video => {
                                                        Route::VideoPlayer {
                                                            course_id: result.course_id.as_uuid().to_string(),
                                                            video_id: result.entity_id.clone(),
                                                        }
                                                    }
                                                    SearchResultType::Note => {
                                                        Route::CourseView {
                                                            course_id: result.course_id.as_uuid().to_string(),
                                                        }
                                                    }
                                                };
                                                rsx! {
                                                    Link {
                                                        to,
                                                        class: "block p-2 rounded-lg bg-base-100 hover:bg-base-300 transition-colors",
                                                        div { class: "text-xs text-base-content/50", "{label}" }
                                                        div { class: "text-sm font-medium truncate", "{result.title}" }
                                                        div { class: "text-xs text-base-content/60 truncate", "{result.snippet}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }

            // Navigation links
            nav { class: "flex-1 p-2 space-y-1",

                NavItem {
                    to: Route::Dashboard {},
                    icon: rsx! {
                        Icon { icon: MdDashboard, width: 20, height: 20 }
                    },
                    label: "Dashboard",
                    collapsed,
                }

                NavItem {
                    to: Route::CourseList {},
                    icon: rsx! {
                        Icon { icon: MdPlaylistPlay, width: 20, height: 20 }
                    },
                    label: "Courses",
                    collapsed,
                }

                NavItem {
                    to: Route::QuizList {},
                    icon: rsx! {
                        Icon { icon: MdAssignment, width: 20, height: 20 }
                    },
                    label: "Quizzes",
                    collapsed,
                }

                NavItem {
                    to: Route::Settings {},
                    icon: rsx! {
                        Icon { icon: MdSettings, width: 20, height: 20 }
                    },
                    label: "Settings",
                    collapsed,
                }
            }
        }
    }
}

#[component]
fn NavItem(to: Route, icon: Element, label: &'static str, collapsed: bool) -> Element {
    let scale = use_motion(1.0f32);
    let mut scale_for_enter = scale;
    let mut scale_for_leave = scale;

    let on_enter = move |_| {
        let config = AnimationConfig::new(AnimationMode::Spring(Spring {
            stiffness: 160.0,
            damping: 18.0,
            mass: 0.7,
            velocity: 0.0,
        }));
        scale_for_enter.animate_to(1.03, config);
    };

    let on_leave = move |_| {
        let config = AnimationConfig::new(AnimationMode::Spring(Spring {
            stiffness: 180.0,
            damping: 20.0,
            mass: 0.7,
            velocity: 0.0,
        }));
        scale_for_leave.animate_to(1.0, config);
    };

    rsx! {
        div {
            class: "rounded-lg",
            style: "transform: scale({scale.get_value()}); will-change: transform;",
            onmouseenter: on_enter,
            onmouseleave: on_leave,
            Link {
                to,
                class: "flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-base-300 transition-colors",
                {icon}
                if !collapsed {
                    span { "{label}" }
                }
            }
        }
    }
}
