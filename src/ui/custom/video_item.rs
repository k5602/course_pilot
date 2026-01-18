//! Video item row in course view

use dioxus::prelude::*;

use crate::ui::Route;
 
/// A single video row with title, duration, and completion status.
#[component]
pub fn VideoItem(
    course_id: String,
    video_id: String,
    title: String,
    duration_secs: u32,
    is_completed: bool,
) -> Element {
    let duration_display = format_duration(duration_secs);
    let status_icon = if is_completed { "✓" } else { "○" };
    let status_class = if is_completed { "text-success" } else { "text-base-content/50" };

    rsx! {
        Link {
            to: Route::VideoPlayer {
                course_id: course_id.clone(),
                video_id: video_id.clone()
            },
            class: "flex items-center gap-3 p-3 rounded-lg hover:bg-base-200 transition-colors",

            // Completion status
            span {
                class: "text-lg {status_class}",
                "{status_icon}"
            }

            // Video info
            div {
                class: "flex-1 min-w-0",
                p {
                    class: "truncate font-medium",
                    "{title}"
                }
            }

            // Duration
            span {
                class: "text-sm text-base-content/60",
                "{duration_display}"
            }
        }
    }
}

fn format_duration(secs: u32) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    if mins >= 60 {
        let hours = mins / 60;
        let mins = mins % 60;
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}
