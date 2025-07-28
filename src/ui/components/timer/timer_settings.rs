use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PomodoroSettings {
    pub work_duration_minutes: u32,
    pub short_break_duration_minutes: u32,
    pub long_break_duration_minutes: u32,
    pub sessions_until_long_break: u32,
    pub auto_start_breaks: bool,
    pub auto_start_work: bool,
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
    pub volume: f32, // 0.0 to 1.0
    pub notification_title: String,
    pub work_notification_message: String,
    pub break_notification_message: String,
}

impl Default for PomodoroSettings {
    fn default() -> Self {
        Self {
            work_duration_minutes: 25,
            short_break_duration_minutes: 5,
            long_break_duration_minutes: 15,
            sessions_until_long_break: 4,
            auto_start_breaks: false,
            auto_start_work: false,
            notifications_enabled: true,
            sound_enabled: true,
            volume: 0.7,
            notification_title: "Course Pilot - Pomodoro Timer".to_string(),
            work_notification_message: "Time to focus! Your work session is starting.".to_string(),
            break_notification_message: "Great work! Time for a well-deserved break.".to_string(),
        }
    }
}

impl PomodoroSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.work_duration_minutes < 1 || self.work_duration_minutes > 120 {
            return Err("Work duration must be between 1 and 120 minutes".to_string());
        }

        if self.short_break_duration_minutes < 1 || self.short_break_duration_minutes > 30 {
            return Err("Short break duration must be between 1 and 30 minutes".to_string());
        }

        if self.long_break_duration_minutes < 1 || self.long_break_duration_minutes > 60 {
            return Err("Long break duration must be between 1 and 60 minutes".to_string());
        }

        if self.sessions_until_long_break < 2 || self.sessions_until_long_break > 10 {
            return Err("Sessions until long break must be between 2 and 10".to_string());
        }

        if !(0.0..=1.0).contains(&self.volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }
}

#[component]
pub fn TimerSettings(
    settings: PomodoroSettings,
    on_settings_change: EventHandler<PomodoroSettings>,
) -> Element {
    let mut local_settings = use_signal(|| settings.clone());
    let mut validation_error = use_signal(|| None::<String>);

    let handle_save = move |_| match local_settings().validate() {
        Ok(()) => {
            validation_error.set(None);
            on_settings_change.call(local_settings());
            crate::ui::components::toast::show_toast(
                "Timer settings saved successfully!".to_string(),
                crate::ui::components::toast::ToastVariant::Success,
            );
        }
        Err(error) => {
            validation_error.set(Some(error));
        }
    };

    let handle_reset = move |_| {
        local_settings.set(PomodoroSettings::default());
        validation_error.set(None);
    };

    rsx! {
        div { class: "space-y-6",
            // Duration Settings
            div { class: "card bg-base-100 border border-base-300",
                div { class: "card-body",
                    h3 { class: "card-title text-lg mb-4", "Timer Durations" }

                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text", "Work Duration (minutes)" }
                            }
                            input {
                                class: "input input-bordered",
                                r#type: "number",
                                min: "1",
                                max: "120",
                                value: "{local_settings().work_duration_minutes}",
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse::<u32>() {
                                        local_settings.with_mut(|s| s.work_duration_minutes = value);
                                    }
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text", "Short Break (minutes)" }
                            }
                            input {
                                class: "input input-bordered",
                                r#type: "number",
                                min: "1",
                                max: "30",
                                value: "{local_settings().short_break_duration_minutes}",
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse::<u32>() {
                                        local_settings.with_mut(|s| s.short_break_duration_minutes = value);
                                    }
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text", "Long Break (minutes)" }
                            }
                            input {
                                class: "input input-bordered",
                                r#type: "number",
                                min: "1",
                                max: "60",
                                value: "{local_settings().long_break_duration_minutes}",
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse::<u32>() {
                                        local_settings.with_mut(|s| s.long_break_duration_minutes = value);
                                    }
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text", "Sessions Until Long Break" }
                            }
                            input {
                                class: "input input-bordered",
                                r#type: "number",
                                min: "2",
                                max: "10",
                                value: "{local_settings().sessions_until_long_break}",
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse::<u32>() {
                                        local_settings.with_mut(|s| s.sessions_until_long_break = value);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Behavior Settings
            div { class: "card bg-base-100 border border-base-300",
                div { class: "card-body",
                    h3 { class: "card-title text-lg mb-4", "Behavior" }

                    div { class: "space-y-4",
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Auto-start breaks" }
                                input {
                                    class: "toggle toggle-primary",
                                    r#type: "checkbox",
                                    checked: local_settings().auto_start_breaks,
                                    onchange: move |evt| {
                                        local_settings.with_mut(|s| s.auto_start_breaks = evt.checked());
                                    }
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Auto-start work sessions" }
                                input {
                                    class: "toggle toggle-primary",
                                    r#type: "checkbox",
                                    checked: local_settings().auto_start_work,
                                    onchange: move |evt| {
                                        local_settings.with_mut(|s| s.auto_start_work = evt.checked());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Notification Settings
            div { class: "card bg-base-100 border border-base-300",
                div { class: "card-body",
                    h3 { class: "card-title text-lg mb-4", "Notifications" }

                    div { class: "space-y-4",
                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Enable desktop notifications" }
                                input {
                                    class: "toggle toggle-primary",
                                    r#type: "checkbox",
                                    checked: local_settings().notifications_enabled,
                                    onchange: move |evt| {
                                        local_settings.with_mut(|s| s.notifications_enabled = evt.checked());
                                    }
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label cursor-pointer",
                                span { class: "label-text", "Enable sound notifications" }
                                input {
                                    class: "toggle toggle-primary",
                                    r#type: "checkbox",
                                    checked: local_settings().sound_enabled,
                                    onchange: move |evt| {
                                        local_settings.with_mut(|s| s.sound_enabled = evt.checked());
                                    }
                                }
                            }
                        }

                        if local_settings().sound_enabled {
                            div { class: "form-control",
                                label { class: "label",
                                    span { class: "label-text", "Volume: {(local_settings().volume * 100.0) as u32}%" }
                                }
                                input {
                                    class: "range range-primary",
                                    r#type: "range",
                                    min: "0",
                                    max: "100",
                                    value: "{(local_settings().volume * 100.0) as u32}",
                                    oninput: move |evt| {
                                        if let Ok(value) = evt.value().parse::<u32>() {
                                            local_settings.with_mut(|s| s.volume = value as f32 / 100.0);
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text", "Work Session Message" }
                            }
                            textarea {
                                class: "textarea textarea-bordered",
                                placeholder: "Message shown when work session starts",
                                value: "{local_settings().work_notification_message}",
                                oninput: move |evt| {
                                    local_settings.with_mut(|s| s.work_notification_message = evt.value());
                                }
                            }
                        }

                        div { class: "form-control",
                            label { class: "label",
                                span { class: "label-text", "Break Session Message" }
                            }
                            textarea {
                                class: "textarea textarea-bordered",
                                placeholder: "Message shown when break session starts",
                                value: "{local_settings().break_notification_message}",
                                oninput: move |evt| {
                                    local_settings.with_mut(|s| s.break_notification_message = evt.value());
                                }
                            }
                        }
                    }
                }
            }

            // Validation Error
            if let Some(error) = validation_error() {
                div { class: "alert alert-error",
                    span { "{error}" }
                }
            }

            // Action Buttons
            div { class: "flex justify-end gap-2",
                button {
                    class: "btn btn-ghost",
                    onclick: handle_reset,
                    "Reset to Defaults"
                }

                button {
                    class: "btn btn-primary",
                    onclick: handle_save,
                    "Save Settings"
                }
            }
        }
    }
}
