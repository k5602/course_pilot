//! Video player page

use dioxus::prelude::*;

use crate::ui::Route;
use crate::ui::custom::YouTubePlayer;
use crate::ui::state::AppState;

/// Video player with controls and completion actions.
#[component]
pub fn VideoPlayer(course_id: String, video_id: String) -> Element {
    let mut state = use_context::<AppState>();

    // Clone for use in different places
    let video_id_for_effect = video_id.clone();
    let video_id_for_player = video_id.clone();
    let video_id_for_quiz = video_id.clone();

    // Set current video for notes/chat context
    use_effect(move || {
        state.current_video_id.set(Some(video_id_for_effect.clone()));
    });

    rsx! {
        div {
            class: "p-6 h-full flex flex-col",

            // Back button
            Link {
                to: Route::CourseView { course_id: course_id.clone() },
                class: "btn btn-ghost btn-sm mb-4",
                "← Back to Course"
            }

            // Video player
            YouTubePlayer { video_id: video_id_for_player }

            // Video info
            div {
                class: "mt-4",
                h2 { class: "text-xl font-bold", "Video Title" }
                p { class: "text-sm text-base-content/60", "Module: Introduction" }
            }

            // Controls
            div {
                class: "flex gap-3 mt-4",
                button {
                    class: "btn btn-outline",
                    "⏮ Previous"
                }
                button {
                    class: "btn btn-outline",
                    "⏭ Next"
                }
                div { class: "flex-1" }
                button {
                    class: "btn btn-success",
                    "✓ Mark Complete"
                }
                Link {
                    to: Route::QuizView { exam_id: video_id_for_quiz },
                    class: "btn btn-primary",
                    "📝 Take Quiz"
                }
            }
        }
    }
}
