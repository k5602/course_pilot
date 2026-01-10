//! Course view page - modules and videos

use dioxus::prelude::*;

use crate::ui::Route;
use crate::ui::custom::VideoItem;

/// Detailed course view with modules accordion.
#[component]
pub fn CourseView(course_id: String) -> Element {
    // TODO: Load course from backend

    rsx! {
        div {
            class: "p-6",

            // Back button
            Link {
                to: Route::CourseList {},
                class: "btn btn-ghost btn-sm mb-4",
                "← Back to Courses"
            }

            // Course header
            h1 { class: "text-2xl font-bold mb-2", "Course: {course_id}" }

            // Progress bar
            div {
                class: "w-full max-w-md bg-base-300 rounded-full h-3 mb-6",
                div {
                    class: "bg-primary h-3 rounded-full",
                    style: "width: 40%",
                }
            }

            // Modules accordion
            div {
                class: "space-y-4",

                // Module 1
                div {
                    class: "collapse collapse-arrow bg-base-200",
                    input { r#type: "checkbox", checked: true }
                    div {
                        class: "collapse-title font-medium",
                        "Module 1: Introduction"
                    }
                    div {
                        class: "collapse-content",
                        VideoItem {
                            course_id: course_id.clone(),
                            video_id: "dQw4w9WgXcQ".to_string(),
                            title: "Getting Started with Rust".to_string(),
                            duration_secs: 720,
                            is_completed: true,
                        }
                        VideoItem {
                            course_id: course_id.clone(),
                            video_id: "abc123".to_string(),
                            title: "Variables and Types".to_string(),
                            duration_secs: 540,
                            is_completed: false,
                        }
                    }
                }

                // Module 2
                div {
                    class: "collapse collapse-arrow bg-base-200",
                    input { r#type: "checkbox" }
                    div {
                        class: "collapse-title font-medium",
                        "Module 2: Ownership"
                    }
                    div {
                        class: "collapse-content",
                        VideoItem {
                            course_id: course_id.clone(),
                            video_id: "def456".to_string(),
                            title: "Understanding Ownership".to_string(),
                            duration_secs: 900,
                            is_completed: false,
                        }
                    }
                }
            }
        }
    }
}
