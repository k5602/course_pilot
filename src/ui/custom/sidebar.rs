//! Sidebar navigation component

use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{MdAssignment, MdDashboard, MdSettings};
use dioxus_free_icons::icons::md_av_icons::MdPlaylistPlay;

use crate::ui::Route;
use crate::ui::state::AppState;

/// Collapsible sidebar with navigation links.
#[component]
pub fn Sidebar() -> Element {
    let mut state = use_context::<AppState>();
    let collapsed = *state.sidebar_collapsed.read();

    let toggle_sidebar = move |_| {
        state.sidebar_collapsed.set(!collapsed);
    };

    let width_class = if collapsed { "w-16" } else { "w-60" };

    rsx! {
        aside {
            class: "flex flex-col h-full bg-base-200 border-r border-base-300 transition-all duration-200 {width_class}",

            // Header with toggle
            div {
                class: "flex items-center justify-between p-4 border-b border-base-300",
                if !collapsed {
                    span { class: "font-bold text-lg", "Course Pilot" }
                }
                button {
                    class: "btn btn-ghost btn-sm",
                    onclick: toggle_sidebar,
                    "☰"
                }
            }

            // Navigation links
            nav {
                class: "flex-1 p-2 space-y-1",

                NavItem {
                    to: Route::Dashboard {},
                    icon: rsx! { Icon { icon: MdDashboard, width: 20, height: 20 } },
                    label: "Dashboard",
                    collapsed: collapsed,
                }

                NavItem {
                    to: Route::CourseList {},
                    icon: rsx! { Icon { icon: MdPlaylistPlay, width: 20, height: 20 } },
                    label: "Courses",
                    collapsed: collapsed,
                }

                NavItem {
                    to: Route::QuizList {},
                    icon: rsx! { Icon { icon: MdAssignment, width: 20, height: 20 } },
                    label: "Quizzes",
                    collapsed: collapsed,
                }

                NavItem {
                    to: Route::Settings {},
                    icon: rsx! { Icon { icon: MdSettings, width: 20, height: 20 } },
                    label: "Settings",
                    collapsed: collapsed,
                }
            }
        }
    }
}

#[component]
fn NavItem(to: Route, icon: Element, label: &'static str, collapsed: bool) -> Element {
    rsx! {
        Link {
            to: to,
            class: "flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-base-300 transition-colors",
            {icon}
            if !collapsed {
                span { "{label}" }
            }
        }
    }
}
