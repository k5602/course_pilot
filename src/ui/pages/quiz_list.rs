//! Quiz list page

use dioxus::prelude::*;

use crate::ui::Route;

/// List of pending and completed quizzes.
#[component]
pub fn QuizList() -> Element {
    rsx! {
        div {
            class: "p-6",

            h1 { class: "text-2xl font-bold mb-6", "Quizzes" }

            div {
                class: "space-y-3",

                // Sample quiz items
                Link {
                    to: Route::QuizView { exam_id: "quiz-1".to_string() },
                    class: "flex items-center gap-3 p-4 bg-base-200 rounded-lg hover:bg-base-300 transition-colors",
                    span { class: "text-warning", "○" }
                    div {
                        class: "flex-1",
                        p { class: "font-medium", "Getting Started with Rust" }
                        p { class: "text-sm text-base-content/60", "5 questions" }
                    }
                    span { class: "text-sm", "Pending" }
                }

                div {
                    class: "flex items-center gap-3 p-4 bg-base-200 rounded-lg opacity-60",
                    span { class: "text-success", "✓" }
                    div {
                        class: "flex-1",
                        p { class: "font-medium", "Variables and Types" }
                        p { class: "text-sm text-base-content/60", "5 questions" }
                    }
                    span { class: "text-sm text-success", "Passed (80%)" }
                }
            }
        }
    }
}
