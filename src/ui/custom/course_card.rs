//! Course card for dashboard

use dioxus::prelude::*;

use crate::ui::Route;

/// A card displaying course info on the dashboard.
#[component]
pub fn CourseCard(
    id: String,
    name: String,
    module_count: usize,
    completed_modules: usize,
) -> Element {
    let progress = if module_count > 0 {
        (completed_modules as f64 / module_count as f64 * 100.0) as u8
    } else {
        0
    };

    rsx! {
        Link {
            to: Route::CourseView { course_id: id },
            class: "card bg-base-200 hover:bg-base-300 transition-colors cursor-pointer",

            div {
                class: "card-body",

                h3 { class: "card-title text-lg", "{name}" }

                p {
                    class: "text-sm text-base-content/70",
                    "{completed_modules} / {module_count} modules"
                }

                // Progress bar
                div {
                    class: "w-full bg-base-300 rounded-full h-2 mt-2",
                    div {
                        class: "bg-primary h-2 rounded-full transition-all",
                        style: "width: {progress}%",
                    }
                }
            }
        }
    }
}
