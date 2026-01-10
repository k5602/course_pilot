//! Dashboard page - Course overview and progress

use dioxus::prelude::*;

use crate::ui::Route;
use crate::ui::custom::CourseCard;

/// Dashboard showing all courses and overall progress.
#[component]
pub fn Dashboard() -> Element {
    // TODO: Load courses from AppContext

    rsx! {
        div {
            class: "p-6",

            // Header
            div {
                class: "flex items-center justify-between mb-6",
                h1 { class: "text-2xl font-bold", "Dashboard" }
                Link {
                    to: Route::CourseList {},
                    class: "btn btn-primary",
                    "+ Import Playlist"
                }
            }

            // Course grid
            div {
                class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",

                // Placeholder courses
                CourseCard {
                    id: "demo-1".to_string(),
                    name: "Rust Programming".to_string(),
                    module_count: 5,
                    completed_modules: 2,
                }
                CourseCard {
                    id: "demo-2".to_string(),
                    name: "Machine Learning Basics".to_string(),
                    module_count: 8,
                    completed_modules: 0,
                }
            }

            // Empty state
            // div {
            //     class: "text-center py-12 text-base-content/50",
            //     p { "No courses yet" }
            //     p { class: "text-sm mt-2", "Import a YouTube playlist to get started" }
            // }
        }
    }
}
