//! Course list page

use dioxus::prelude::*;

use crate::ui::custom::CourseCard;

/// List of all imported courses.
#[component]
pub fn CourseList() -> Element {
    // TODO: Load from backend

    rsx! {
        div {
            class: "p-6",

            h1 { class: "text-2xl font-bold mb-6", "Courses" }

            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",

                CourseCard {
                    id: "demo-1".to_string(),
                    name: "Rust Programming".to_string(),
                    module_count: 5,
                    completed_modules: 2,
                }
            }
        }
    }
}
