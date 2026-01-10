//! Quiz view page - MCQ interface

use dioxus::prelude::*;

/// Quiz with multiple choice questions.
#[component]
pub fn QuizView(exam_id: String) -> Element {
    let mut selected = use_signal(|| None::<usize>);

    rsx! {
        div {
            class: "p-6 max-w-2xl mx-auto",

            // Header
            h1 { class: "text-2xl font-bold mb-2", "Quiz" }
            p { class: "text-base-content/60 mb-6", "Video: {exam_id}" }

            // Progress
            div {
                class: "flex items-center gap-2 mb-6",
                span { class: "text-sm", "Question 1 of 5" }
                div {
                    class: "flex-1 bg-base-300 rounded-full h-2",
                    div {
                        class: "bg-primary h-2 rounded-full",
                        style: "width: 20%",
                    }
                }
            }

            // Question
            div {
                class: "bg-base-200 rounded-lg p-6 mb-6",
                p { class: "text-lg font-medium mb-4",
                    "What is the main advantage of Rust's ownership system?"
                }

                // Options
                div {
                    class: "space-y-3",
                    for (i, option) in [
                        "Memory safety without garbage collection",
                        "Faster compilation times",
                        "Simpler syntax",
                        "Better package management",
                    ].iter().enumerate() {
                        button {
                            class: if *selected.read() == Some(i) {
                                "w-full text-left p-4 rounded-lg border-2 border-primary bg-primary/10"
                            } else {
                                "w-full text-left p-4 rounded-lg border-2 border-base-300 hover:border-primary/50 transition-colors"
                            },
                            onclick: move |_| selected.set(Some(i)),
                            "{option}"
                        }
                    }
                }
            }

            // Navigation
            div {
                class: "flex justify-between",
                button { class: "btn btn-ghost", "Previous" }
                if selected.read().is_some() {
                    button { class: "btn btn-primary", "Next Question" }
                } else {
                    button { class: "btn btn-disabled", "Next Question" }
                }
            }
        }
    }
}
