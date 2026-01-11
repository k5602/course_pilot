//! Quiz list page - Shows all generated exams and their status.

use dioxus::prelude::*;

use crate::domain::entities::Exam;
use crate::ui::Route;
use crate::ui::hooks::{use_load_all_exams, use_load_video};
use crate::ui::state::AppState;

/// List of pending and completed quizzes.
#[component]
pub fn QuizList() -> Element {
    let state = use_context::<AppState>();
    let exams = use_load_all_exams(state.backend.clone());

    rsx! {
        div {
            class: "p-6 max-w-4xl mx-auto",

            div { class: "flex justify-between items-center mb-8",
                h1 { class: "text-3xl font-bold", "My Quizzes" }
                span { class: "badge badge-primary", "{exams.read().len()} Total" }
            }

            if exams.read().is_empty() {
                div { class: "text-center py-20 bg-base-200 rounded-3xl border-2 border-dashed border-base-300",
                    div { class: "text-6xl mb-4", "📝" }
                    h2 { class: "text-xl font-semibold mb-2", "No quizzes yet" }
                    p { class: "text-base-content/60 max-w-md mx-auto",
                        "Quizzes are generated when you mark a video as mastered. Start learning to challenge yourself!"
                    }
                }
            } else {
                div {
                    class: "grid gap-4",
                    for exam in exams.read().iter() {
                        QuizItem {
                            key: "{exam.id().as_uuid()}",
                            exam: exam.clone()
                        }
                    }
                }
            }
        }
    }
}

/// Individual quiz item row.
#[component]
fn QuizItem(exam: Exam) -> Element {
    let state = use_context::<AppState>();
    let video = use_load_video(state.backend.clone(), exam.video_id());

    let video_title = match video.read().as_ref() {
        Some(v) => v.title().to_string(),
        None => "Video #".to_string() + &exam.video_id().as_uuid().to_string()[..8],
    };

    let score = exam.score().map(|s| (s * 100.0) as i32);
    let passed = exam.passed().unwrap_or(false);
    let is_taken = exam.is_taken();

    rsx! {
        Link {
            to: Route::QuizView { exam_id: exam.id().as_uuid().to_string() },
            class: "flex items-center gap-4 p-5 bg-base-200 rounded-2xl hover:bg-base-300 transition-all border border-transparent hover:border-primary/20 group",

            // Status Icon
            div {
                class: match (is_taken, passed) {
                    (true, true) => "w-12 h-12 rounded-xl bg-success/20 text-success flex items-center justify-center text-xl shadow-inner",
                    (true, false) => "w-12 h-12 rounded-xl bg-error/20 text-error flex items-center justify-center text-xl shadow-inner",
                    (false, _) => "w-12 h-12 rounded-xl bg-warning/20 text-warning flex items-center justify-center text-xl shadow-inner",
                },
                if is_taken {
                    if passed { "✓" } else { "✕" }
                } else {
                    "?"
                }
            }

            // Info
            div {
                class: "flex-1",
                h3 { class: "font-bold text-lg group-hover:text-primary transition-colors", "{video_title}" }
                div { class: "flex items-center gap-3 mt-1",
                    if let Some(s) = score {
                        span {
                            class: if passed { "text-success text-sm font-medium" } else { "text-error text-sm font-medium" },
                            "Score: {s}%"
                        }
                    } else {
                        span { class: "text-sm text-base-content/50", "Not attempted" }
                    }
                    span { class: "text-xs text-base-content/30", "•" }
                    span { class: "text-xs text-base-content/30", "Ref: {&exam.id().as_uuid().to_string()[..8]}" }
                }
            }

            // Badge/Action
            div { class: "text-right flex flex-col items-end gap-2",
                span {
                    class: if is_taken {
                        if passed { "badge badge-success badge-sm" } else { "badge badge-error badge-sm" }
                    } else {
                        "badge badge-warning badge-sm"
                    },
                    if is_taken {
                        if passed { "PASSED" } else { "FAILED" }
                    } else {
                        "PENDING"
                    }
                }
                span { class: "text-xs opacity-0 group-hover:opacity-100 transition-opacity text-primary font-bold",
                    if is_taken { "Review →" } else { "Take Quiz →" }
                }
            }
        }
    }
}
