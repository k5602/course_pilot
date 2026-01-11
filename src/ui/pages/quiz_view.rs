//! Quiz view page - MCQ interface for taking and reviewing exams.

use dioxus::prelude::*;
use std::str::FromStr;

use crate::application::ServiceFactory;
use crate::application::use_cases::SubmitExamInput;
use crate::domain::ports::MCQuestion;
use crate::domain::value_objects::ExamId;
use crate::ui::Route;
use crate::ui::hooks::{use_load_exam, use_load_video};
use crate::ui::state::AppState;

/// Quiz with multiple choice questions.
#[component]
pub fn QuizView(exam_id: String) -> Element {
    let state = use_context::<AppState>();
    let backend = state.backend.clone();
    let nav = use_navigator();

    let exam_id_vo = match ExamId::from_str(&exam_id) {
        Ok(id) => id,
        Err(_) => return rsx! { div { class: "p-6 text-error", "Invalid Exam ID" } },
    };

    let exam = use_load_exam(backend.clone(), &exam_id_vo);
    let video = use_load_video(
        backend.clone(),
        &exam.read().as_ref().map(|e| e.video_id().clone()).unwrap_or_default(),
    );

    // UI State
    let mut current_index = use_signal(|| 0usize);
    let mut selected_option = use_signal(|| None::<usize>);
    let mut answers = use_signal(Vec::<usize>::new);
    let mut is_submitting = use_signal(|| false);
    let mut show_review = use_signal(|| false);

    // Sync answers from database if already taken
    use_effect(move || {
        if let Some(e) = exam.read().as_ref() {
            if e.is_taken() && answers.read().is_empty() {
                if let Some(json) = e.user_answers_json() {
                    if let Ok(loaded) = serde_json::from_str::<Vec<usize>>(json) {
                        answers.set(loaded);
                    }
                }
            }
        }
    });

    let exam_data = exam.read();
    let exam_ref = match exam_data.as_ref() {
        Some(e) => e,
        None => return rsx! { div { class: "p-6 animate-pulse", "Loading exam..." } },
    };

    let questions: Vec<MCQuestion> =
        serde_json::from_str(exam_ref.question_json()).unwrap_or_default();
    let total_questions = questions.len();

    // Handle exam completion
    if exam_ref.is_taken() && !show_review() {
        let score = exam_ref.score().unwrap_or(0.0) * 100.0;
        let passed = exam_ref.passed().unwrap_or(false);

        return rsx! {
            div { class: "p-6 max-w-2xl mx-auto",
                div { class: "bg-base-200 rounded-xl p-8 text-center shadow-lg",
                    h1 { class: "text-3xl font-bold mb-4", "Quiz Result" }
                    div {
                        class: if passed { "text-success text-6xl font-bold mb-4" } else { "text-error text-6xl font-bold mb-4" },
                        "{score:.0}%"
                    }
                    p { class: "text-xl mb-8 opacity-80",
                        if passed { "Congratulations! You've mastered this video." }
                        else { "Keep studying. You can retake the quiz after reviewing the content." }
                    }
                    div { class: "flex flex-col sm:flex-row justify-center gap-4",
                        button {
                            class: "btn btn-primary btn-lg",
                            onclick: move |_| { nav.push(Route::Dashboard {}); },
                            "Back to Dashboard"
                        }
                        button {
                            class: "btn btn-outline btn-lg",
                            onclick: move |_| show_review.set(true),
                            "Review Questions"
                        }
                        if !passed {
                            button {
                                class: "btn btn-ghost btn-lg",
                                onclick: move |_| {
                                    if let Some(v) = video.read().as_ref() {
                                        nav.push(Route::VideoPlayer {
                                            course_id: "unknown".to_string(),
                                            video_id: v.id().as_uuid().to_string()
                                        });
                                    }
                                },
                                "Watch Video Again"
                            }
                        }
                    }
                }
            }
        };
    }

    // Show detailed review mode
    if show_review() {
        return rsx! {
            div { class: "p-6 max-w-3xl mx-auto",
                div { class: "flex items-center justify-between mb-8",
                    h1 { class: "text-2xl font-bold", "Review: {video.read().as_ref().map(|v| v.title()).unwrap_or(\"...\")}" }
                    button {
                        class: "btn btn-sm btn-ghost",
                        onclick: move |_| show_review.set(false),
                        "← Back to Score"
                    }
                }

                div { class: "space-y-8",
                    for (idx, q) in questions.iter().enumerate() {
                        {
                            let user_answer = answers.read().get(idx).cloned();
                            let is_correct = user_answer == Some(q.correct_index);
                            let border_class = if answers.read().is_empty() {
                                "border-base-300"
                            } else if is_correct {
                                "border-success"
                            } else {
                                "border-error"
                            };

                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "bg-base-200 rounded-2xl p-6 shadow-sm border-l-4 {border_class}",
                                    p { class: "text-lg font-bold mb-4", "{idx + 1}. {q.question}" }

                                    div { class: "space-y-2 mb-4",
                                        for (opt_idx, opt) in q.options.iter().enumerate() {
                                            {
                                                let is_this_correct = opt_idx == q.correct_index;
                                                let is_user_choice = user_answer == Some(opt_idx);

                                                rsx! {
                                                    div {
                                                        key: "{opt_idx}",
                                                        class: if is_this_correct {
                                                            "p-3 rounded-lg bg-success/10 text-success border border-success/20 flex items-center gap-2"
                                                        } else if is_user_choice {
                                                            "p-3 rounded-lg bg-error/10 text-error border border-error/20 flex items-center gap-2"
                                                        } else {
                                                            "p-3 rounded-lg bg-base-300/50 opacity-60 flex items-center gap-2"
                                                        },
                                                        span { class: "font-mono text-xs w-4", "{opt_idx + 1}." }
                                                        span { "{opt}" }
                                                        if is_this_correct {
                                                            span { class: "ml-auto text-xs font-bold", "CORRECT" }
                                                        } else if is_user_choice {
                                                            span { class: "ml-auto text-xs font-bold", "YOUR CHOICE" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "mt-4 p-4 bg-base-100 rounded-xl border border-base-300",
                                        p { class: "text-xs font-bold uppercase tracking-widest opacity-40 mb-1", "Explanation" }
                                        p { class: "text-sm", "{q.explanation}" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "mt-12 text-center",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| { nav.push(Route::Dashboard {}); },
                        "Done Reviewing"
                    }
                }
            }
        };
    }

    if questions.is_empty() {
        return rsx! { div { class: "p-6 text-error", "This exam has no questions." } };
    }

    let current_q = &questions[current_index()];
    let backend_for_submit = backend.clone();
    let exam_id_for_submit = exam_id_vo.clone();

    let on_next = move |_| {
        if let Some(sel) = selected_option() {
            answers.write().push(sel);
            if current_index() + 1 < total_questions {
                current_index.set(current_index() + 1);
                selected_option.set(None);
            } else {
                // Submit!
                let backend_inner = backend_for_submit.clone();
                let exam_id_inner = exam_id_for_submit.clone();
                spawn(async move {
                    if let Some(ctx) = backend_inner.as_ref() {
                        if let Some(use_case) = ServiceFactory::take_exam(ctx) {
                            is_submitting.set(true);
                            let input = SubmitExamInput {
                                exam_id: exam_id_inner.clone(),
                                answers: answers.read().clone(),
                            };
                            let _ = use_case.submit(input);
                            is_submitting.set(false);
                            // The use_effect in use_load_exam will trigger a re-render when the DB updates
                        }
                    }
                });
            }
        }
    };

    rsx! {
        div { class: "p-6 max-w-2xl mx-auto",
            // Header
            h1 { class: "text-2xl font-bold mb-2", "Exam: {video.read().as_ref().map(|v| v.title()).unwrap_or(\"...\")}" }

            // Progress
            div { class: "flex items-center gap-4 mb-8",
                div { class: "flex-1 bg-base-300 rounded-full h-2.5 overflow-hidden",
                    div {
                        class: "bg-primary h-full transition-all duration-300",
                        style: "width: {(current_index() as f32 / total_questions as f32) * 100.0}%"
                    }
                }
                span { class: "text-sm font-medium whitespace-nowrap",
                    "Question {current_index() + 1} of {total_questions}"
                }
            }

            // Question Card
            div { class: "bg-base-200 rounded-2xl p-6 mb-8 shadow-sm",
                p { class: "text-xl font-semibold mb-6", "{current_q.question}" }

                div { class: "space-y-3",
                    for (i, option) in current_q.options.iter().enumerate() {
                        button {
                            key: "{i}",
                            class: if selected_option() == Some(i) {
                                "w-full text-left p-5 rounded-xl border-2 border-primary bg-primary/5 font-medium transition-all"
                            } else {
                                "w-full text-left p-5 rounded-xl border-2 border-transparent bg-base-300 hover:bg-base-100 transition-all"
                            },
                            onclick: move |_| selected_option.set(Some(i)),
                            div { class: "flex items-center gap-4",
                                span {
                                    class: if selected_option() == Some(i) {
                                        "w-6 h-6 rounded-full border-2 border-primary bg-primary flex items-center justify-center text-[10px] text-primary-content"
                                    } else {
                                        "w-6 h-6 rounded-full border-2 border-base-content/20 flex items-center justify-center"
                                    },
                                    if selected_option() == Some(i) { "✓" }
                                }
                                "{option}"
                            }
                        }
                    }
                }
            }

            // Navigation
            div { class: "flex justify-end",
                button {
                    class: "btn btn-primary btn-lg px-8",
                    disabled: selected_option().is_none() || is_submitting(),
                    onclick: on_next,
                    if is_submitting() {
                        span { class: "loading loading-spinner loading-sm" }
                    }
                    if current_index() + 1 < total_questions {
                        "Next Question"
                    } else {
                        "Finish Exam"
                    }
                }
            }
        }
    }
}
