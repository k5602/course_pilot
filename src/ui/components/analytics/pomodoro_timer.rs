use dioxus::prelude::*;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum TimerState {
    Stopped,
    Running,
    Paused,
}

#[derive(Clone, Copy, PartialEq)]
enum TimerMode {
    Work,    // 25 minutes
    Break,   // 5 minutes
}

impl TimerMode {
    fn duration(&self) -> Duration {
        match self {
            TimerMode::Work => Duration::from_secs(25 * 60),  // 25 minutes
            TimerMode::Break => Duration::from_secs(5 * 60),  // 5 minutes
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            TimerMode::Work => "Focus Time",
            TimerMode::Break => "Break Time",
        }
    }
    
    fn icon(&self) -> &'static str {
        match self {
            TimerMode::Work => "🍅",
            TimerMode::Break => "☕",
        }
    }
    
    fn next(&self) -> Self {
        match self {
            TimerMode::Work => TimerMode::Break,
            TimerMode::Break => TimerMode::Work,
        }
    }
}

#[component]
pub fn PomodoroTimer() -> Element {
    let mut timer_state = use_signal(|| TimerState::Stopped);
    let mut timer_mode = use_signal(|| TimerMode::Work);
    let mut remaining_time = use_signal(|| TimerMode::Work.duration());
    let mut completed_pomodoros = use_signal(|| 0u32);

    // Timer effect
    use_effect(move || {
        if timer_state() == TimerState::Running {
            spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    
                    if timer_state() != TimerState::Running {
                        break;
                    }
                    
                    let current_remaining = remaining_time();
                    if current_remaining <= Duration::from_secs(1) {
                        // Timer finished
                        timer_state.set(TimerState::Stopped);
                        
                        if timer_mode() == TimerMode::Work {
                            completed_pomodoros.set(completed_pomodoros() + 1);
                        }
                        
                        // Switch to next mode
                        let next_mode = timer_mode().next();
                        timer_mode.set(next_mode);
                        remaining_time.set(next_mode.duration());
                        
                        // Show notification (in a real app, you'd use system notifications)
                        crate::ui::toast_helpers::success(
                            format!("{} completed! Starting {}", 
                                timer_mode().label(), 
                                next_mode.label()
                            )
                        );
                        
                        break;
                    } else {
                        remaining_time.set(current_remaining - Duration::from_secs(1));
                    }
                }
            });
        }
    });

    let handle_start_pause = move |_| {
        match timer_state() {
            TimerState::Stopped | TimerState::Paused => {
                timer_state.set(TimerState::Running);
            }
            TimerState::Running => {
                timer_state.set(TimerState::Paused);
            }
        }
    };

    let handle_reset = move |_| {
        timer_state.set(TimerState::Stopped);
        remaining_time.set(timer_mode().duration());
    };

    let handle_mode_switch = move |_| {
        timer_state.set(TimerState::Stopped);
        let new_mode = timer_mode().next();
        timer_mode.set(new_mode);
        remaining_time.set(new_mode.duration());
    };

    let minutes = remaining_time().as_secs() / 60;
    let seconds = remaining_time().as_secs() % 60;
    let progress = 1.0 - (remaining_time().as_secs() as f32 / timer_mode().duration().as_secs() as f32);

    rsx! {
        div { class: "card bg-base-100 border border-base-300",
            div { class: "card-body p-4 text-center",
                div { class: "flex items-center justify-center gap-2 mb-4",
                    span { class: "text-2xl", "{timer_mode().icon()}" }
                    h3 { class: "font-semibold", "{timer_mode().label()}" }
                }
                
                // Timer display
                div { class: "relative mb-6",
                    div { 
                        class: "radial-progress text-primary",
                        style: "--value:{(progress * 100.0) as i32}; --size:8rem; --thickness:4px;",
                        div { class: "text-2xl font-mono font-bold",
                            "{minutes:02}:{seconds:02}"
                        }
                    }
                }
                
                // Controls
                div { class: "flex justify-center gap-2 mb-4",
                    button {
                        class: format!(
                            "btn btn-sm {}",
                            match timer_state() {
                                TimerState::Running => "btn-warning",
                                _ => "btn-primary"
                            }
                        ),
                        onclick: handle_start_pause,
                        match timer_state() {
                            TimerState::Running => "Pause",
                            _ => "Start"
                        }
                    }
                    
                    button {
                        class: "btn btn-sm btn-ghost",
                        onclick: handle_reset,
                        "Reset"
                    }
                    
                    button {
                        class: "btn btn-sm btn-outline",
                        onclick: handle_mode_switch,
                        "Switch Mode"
                    }
                }
                
                // Stats
                div { class: "text-sm text-base-content/70",
                    "Completed Pomodoros: {completed_pomodoros()}"
                }
                
                if timer_state() == TimerState::Running {
                    div { class: "text-xs text-primary mt-2",
                        "Timer is running..."
                    }
                }
            }
        }
    }
}