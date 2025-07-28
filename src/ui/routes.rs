use dioxus::prelude::*;
use uuid::Uuid;

use crate::types::Route;

// Route components - these will be rendered by the router
#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();

    // Redirect to dashboard
    use_effect(move || {
        navigator.push(Route::Dashboard {});
    });

    rsx! {
        div { class: "p-8",
            "Redirecting to dashboard..."
        }
    }
}

#[component]
pub fn Dashboard() -> Element {
    rsx! { crate::ui::dashboard::Dashboard {} }
}

#[component]
pub fn PlanView(course_id: String) -> Element {
    // Convert string to UUID
    let course_uuid = match Uuid::parse_str(&course_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return rsx! {
                div { class: "p-8",
                    h1 { class: "text-3xl font-bold mb-4 text-error", "Invalid Course ID" }
                    p { class: "text-base-content/70", "The course ID '{course_id}' is not valid." }
                }
            };
        }
    };

    rsx! { crate::ui::plan_view::PlanView { course_id: course_uuid } }
}

#[component]
pub fn AllCourses() -> Element {
    rsx! { crate::ui::courses::AllCoursesView {} }
}

#[component]
pub fn Settings() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-3xl font-bold mb-4", "Settings" }
            p { class: "text-base-content/70", "Configure your Course Pilot preferences here." }
        }
    }
}

#[component]
pub fn AddCourse() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-3xl font-bold mb-4", "Add Course" }
            p { class: "text-base-content/70", "Add a new course to your collection." }
        }
    }
}

#[cfg(debug_assertions)]
#[component]
pub fn ToastTest() -> Element {
    rsx! {
        div {
            class: "p-8",
            h1 { class: "text-3xl font-bold mb-4", "Toast Test" }
            p { class: "text-base-content/70", "Test toast notifications." }
        }
    }
}
