//! Dashboard page - Course overview and progress

use dioxus::prelude::*;

use crate::ui::Route;
use crate::ui::custom::CourseCard;
use crate::ui::state::AppState;

/// Dashboard showing all courses and overall progress.
#[component]
pub fn Dashboard() -> Element {
    let state = use_context::<AppState>();
    let courses = state.courses.read();

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

            // Status cards
            div {
                class: "grid grid-cols-3 gap-4 mb-6",
                StatusCard {
                    label: "YouTube API",
                    status: state.has_youtube(),
                }
                StatusCard {
                    label: "Gemini AI",
                    status: state.has_gemini(),
                }
                StatusCard {
                    label: "Backend",
                    status: state.has_backend(),
                }
            }

            // Course grid
            if courses.is_empty() {
                div {
                    class: "text-center py-12 bg-base-200 rounded-lg",
                    p { class: "text-xl mb-2", "No courses yet" }
                    p { class: "text-base-content/60", "Import a YouTube playlist to get started" }
                    Link {
                        to: Route::Settings {},
                        class: "btn btn-outline mt-4",
                        "Configure API Keys"
                    }
                }
            } else {
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    for course in courses.iter() {
                        CourseCard {
                            id: course.id().as_uuid().to_string(),
                            name: course.name().to_string(),
                            module_count: 0,  // TODO: get from backend
                            completed_modules: 0,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatusCard(label: &'static str, status: bool) -> Element {
    let (icon, color) = if status { ("✓", "text-success") } else { ("✗", "text-error") };

    rsx! {
        div {
            class: "bg-base-200 p-4 rounded-lg flex items-center gap-3",
            span { class: "{color} text-xl", "{icon}" }
            span { "{label}" }
        }
    }
}
