//! Onboarding tour overlay component.

use dioxus::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::UpdatePreferencesInput;
use crate::ui::state::AppState;

/// onboarding tour shown on first run.
#[component]
pub fn OnboardingTour() -> Element {
    let state = use_context::<AppState>();
    let current_step = use_signal(|| 0usize);
    let loaded = use_signal(|| false);
    let error_msg = use_signal(|| None::<String>);

    {
        let mut state = state.clone();
        let backend = state.backend.clone();
        let mut loaded = loaded;
        let mut error_msg = error_msg;
        use_effect(move || {
            if *loaded.read() {
                return;
            }
            loaded.set(true);

            let Some(ctx) = backend.as_ref() else {
                return;
            };

            let use_case = ServiceFactory::preferences(ctx);
            match use_case.load() {
                Ok(prefs) => {
                    state.onboarding_completed.set(prefs.onboarding_completed());
                },
                Err(e) => {
                    error_msg.set(Some(format!("Failed to load preferences: {}", e)));
                },
            }
        });
    }

    if *state.onboarding_completed.read() {
        return rsx! {};
    }

    let steps = [
        Step {
            title: "Import your first course",
            body: "Open the Dashboard and click “Import Playlist” to pull in a YouTube playlist or video.",
        },
        Step {
            title: "Take notes as you watch",
            body: "Open any video and use the Notes tab to capture ideas. Notes save automatically and render as Markdown.",
        },
        Step {
            title: "Ask the AI companion",
            body: "Add a Gemini API key in Settings to unlock summaries, quizzes, and chat about the current video.",
        },
    ];

    let current_step_value = *current_step.read();
    let total_steps = steps.len();
    let step_index = current_step_value + 1;
    let is_first_step = current_step_value == 0;
    let is_last_step = step_index == total_steps;
    let step = steps.get(current_step_value).cloned().unwrap_or(steps[0]);

    let mut on_close = {
        let mut state = state.clone();
        let mut error_msg = error_msg;
        move || {
            complete_onboarding(&mut state, &mut error_msg);
        }
    };

    let mut on_next = {
        let mut state = state.clone();
        let mut error_msg = error_msg;
        let mut current_step = current_step;
        move || {
            let current_value = *current_step.read();
            let next = current_value.saturating_add(1);
            if next >= total_steps {
                complete_onboarding(&mut state, &mut error_msg);
            } else {
                current_step.set(next);
            }
        }
    };

    let mut on_back = {
        let mut current_step = current_step;
        move || {
            let current_value = *current_step.read();
            current_step.set(current_value.saturating_sub(1));
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm",
            div {
                class: "w-full max-w-lg rounded-2xl bg-base-100 p-6 shadow-2xl border border-base-300",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xl font-bold", "Welcome to Course Pilot" }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: move |_| on_close(),
                        "Skip"
                    }
                }

                div { class: "space-y-2",
                    p { class: "text-xs uppercase tracking-widest text-base-content/50", "Step {step_index} of {total_steps}" }
                    h3 { class: "text-lg font-semibold", "{step.title}" }
                    p { class: "text-sm text-base-content/70 leading-relaxed", "{step.body}" }
                }

                div { class: "mt-4",
                    progress {
                        class: "progress progress-primary w-full",
                        value: "{step_index}",
                        max: "{total_steps}",
                    }
                }

                if let Some(ref err) = *error_msg.read() {
                    div { class: "mt-4 text-xs text-error", "{err}" }
                }

                div { class: "mt-6 flex items-center justify-between",
                    button {
                        class: "btn btn-ghost btn-sm",
                        disabled: is_first_step,
                        onclick: move |_| on_back(),
                        "Back"
                    }
                    button {
                        class: "btn btn-primary btn-sm",
                        onclick: move |_| on_next(),
                        if is_last_step { "Finish" } else { "Next" }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Step {
    title: &'static str,
    body: &'static str,
}

fn complete_onboarding(state: &mut AppState, error_msg: &mut Signal<Option<String>>) {
    state.onboarding_completed.set(true);

    let Some(ctx) = state.backend.clone() else {
        return;
    };

    let use_case = ServiceFactory::preferences(&ctx);
    match use_case.load() {
        Ok(prefs) => {
            let input = UpdatePreferencesInput {
                ml_boundary_enabled: prefs.ml_boundary_enabled(),
                cognitive_limit_minutes: prefs.cognitive_limit_minutes(),
                right_panel_visible: prefs.right_panel_visible(),
                onboarding_completed: true,
            };
            if let Err(e) = use_case.update(input) {
                error_msg.set(Some(format!("Failed to save onboarding state: {}", e)));
            }
        },
        Err(e) => {
            error_msg.set(Some(format!("Failed to load preferences: {}", e)));
        },
    }
}
