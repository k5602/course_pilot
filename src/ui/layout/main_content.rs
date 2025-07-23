use crate::types::Route;
use crate::ui::components::top_bar::TopBar;
use crate::ui::dashboard::Dashboard;
use crate::ui::navigation::Breadcrumbs;
use crate::ui::plan_view::PlanView;
use dioxus::prelude::*;

const MAIN_BG: &str = "bg-base-100";

#[derive(Props, PartialEq, Clone)]
pub struct MainContentProps {
    pub current_route: Route,
    pub panel_is_open: bool,
}

/// Clean main content area with routing
#[component]
pub fn MainContent(props: MainContentProps) -> Element {
    let margin_right = if props.panel_is_open {
        "md:mr-96"
    } else {
        "md:mr-0"
    };

    rsx! {
        main {
            class: "flex-1 {margin_right} overflow-y-auto {MAIN_BG} transition-all duration-300",

            TopBar {}
            Breadcrumbs { current_route: props.current_route }

            div {
                class: "flex-1",
                {render_route_content(props.current_route)}
            }
        }
    }
}

/// Render content based on current route
fn render_route_content(route: Route) -> Element {
    match route {
        Route::Dashboard => rsx!(Dashboard {}),
        Route::PlanView(course_id) => rsx!(PlanView {
            course_id: course_id
        }),
        Route::Settings => rsx! {
            div {
                class: "p-8",
                h1 { class: "text-3xl font-bold mb-4", "Settings" }
                p { class: "text-base-content/70", "Configure your Course Pilot preferences here." }
            }
        },
        Route::AddCourse => rsx! {
            div {
                class: "p-8",
                h1 { class: "text-3xl font-bold mb-4", "Add Course" }
                p { class: "text-base-content/70", "Add a new course to your collection." }
            }
        },
        #[cfg(debug_assertions)]
        Route::ToastTest => rsx! {
            div {
                class: "p-8",
                h1 { class: "text-3xl font-bold mb-4", "Toast Test" }
                p { class: "text-base-content/70", "Test toast notifications." }
            }
        },
    }
}
