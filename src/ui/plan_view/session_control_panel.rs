use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaCalendarDays, FaClock, FaGear, FaRotateRight};
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;


use crate::types::{Plan, PlanSettings};
use crate::ui::components::toast::toast;

#[derive(Props, PartialEq, Clone)]
pub struct SessionControlPanelProps {
    pub plan: Plan,
    pub on_settings_change: EventHandler<PlanSettings>,
}

/// Session control panel for plan customization and scheduling
#[component]
pub fn SessionControlPanel(props: SessionControlPanelProps) -> Element {
    let mut is_expanded = use_signal(|| false);
    let mut sessions_per_week = use_signal(|| props.plan.settings.sessions_per_week);
    let mut session_length = use_signal(|| props.plan.settings.session_length_minutes);
    let mut include_weekends = use_signal(|| props.plan.settings.include_weekends);
    
    // Animation for panel expansion
    let mut panel_height = use_motion(0.0f32);
    let mut panel_opacity = use_motion(0.0f32);

    use_effect(move || {
        if is_expanded() {
            panel_height.animate_to(
                200.0,
                AnimationConfig::new(AnimationMode::Spring(Spring::default())),
            );
            panel_opacity.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Tween(Tween::default())),
            );
        } else {
            panel_height.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(Spring::default())),
            );
            panel_opacity.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Tween(Tween::default())),
            );
        }
    });

    let panel_style = use_memo(move || {
        format!(
            "max-height: {}px; opacity: {}; overflow: hidden;",
            panel_height.get_value(),
            panel_opacity.get_value()
        )
    });

    let toggle_panel = move |_| {
        is_expanded.set(!is_expanded());
    };

    let apply_settings = move |_| {
        let new_settings = PlanSettings {
            start_date: props.plan.settings.start_date,
            sessions_per_week: sessions_per_week(),
            session_length_minutes: session_length(),
            include_weekends: include_weekends(),
        };
        
        props.on_settings_change.call(new_settings);
        
        spawn(async move {
            toast::success("Plan settings updated successfully!");
        });
    };

    let reset_settings = move |_| {
        sessions_per_week.set(props.plan.settings.sessions_per_week);
        session_length.set(props.plan.settings.session_length_minutes);
        include_weekends.set(props.plan.settings.include_weekends);
        
        spawn(async move {
            toast::info("Settings reset to current plan values");
        });
    };

    rsx! {
        div { class: "card bg-base-100 border border-base-300 mb-6",
            div { class: "card-body p-4",
                // Header with toggle button
                div { 
                    class: "flex items-center justify-between cursor-pointer",
                    onclick: toggle_panel,
                    
                    div { class: "flex items-center gap-3",
                        Icon { icon: FaGear, class: "w-5 h-5 text-primary" }
                        h3 { class: "text-lg font-semibold", "Session Controls" }
                    }
                    
                    div { class: "flex items-center gap-2 text-sm text-base-content/60",
                        span { "{sessions_per_week()} sessions/week" }
                        span { "•" }
                        span { "{session_length()} min each" }
                        
                        button { 
                            class: "btn btn-ghost btn-sm btn-circle ml-2",
                            span { 
                                class: "w-4 h-4 flex items-center justify-center",
                                if is_expanded() { "▲" } else { "▼" }
                            }
                        }
                    }
                }
                
                // Expandable controls panel
                div { 
                    class: "transition-all duration-300",
                    style: "{panel_style}",
                    
                    div { class: "pt-4 space-y-4",
                        // Sessions per week control
                        fieldset { class: "fieldset",
                            legend { class: "fieldset-legend flex items-center gap-2",
                                Icon { icon: FaCalendarDays, class: "w-4 h-4" }
                                "Sessions per Week"
                            }
                            
                            div { class: "flex items-center gap-4",
                                input { 
                                    type: "range",
                                    class: "range range-primary flex-1",
                                    min: "1",
                                    max: "14",
                                    value: "{sessions_per_week()}",
                                    step: "1",
                                    oninput: move |evt| {
                                        if let Ok(value) = evt.value().parse::<u8>() {
                                            sessions_per_week.set(value);
                                        }
                                    }
                                }
                                
                                div { class: "badge badge-primary badge-lg font-mono",
                                    "{sessions_per_week()}"
                                }
                            }
                            
                            div { class: "flex justify-between text-xs text-base-content/60 mt-1",
                                span { "1" }
                                span { "7" }
                                span { "14" }
                            }
                        }
                        
                        // Session length control
                        fieldset { class: "fieldset",
                            legend { class: "fieldset-legend flex items-center gap-2",
                                Icon { icon: FaClock, class: "w-4 h-4" }
                                "Session Length (minutes)"
                            }
                            
                            div { class: "flex items-center gap-4",
                                input { 
                                    type: "range",
                                    class: "range range-secondary flex-1",
                                    min: "15",
                                    max: "180",
                                    value: "{session_length()}",
                                    step: "15",
                                    oninput: move |evt| {
                                        if let Ok(value) = evt.value().parse::<u32>() {
                                            session_length.set(value);
                                        }
                                    }
                                }
                                
                                div { class: "badge badge-secondary badge-lg font-mono",
                                    "{session_length()}m"
                                }
                            }
                            
                            div { class: "flex justify-between text-xs text-base-content/60 mt-1",
                                span { "15m" }
                                span { "90m" }
                                span { "180m" }
                            }
                        }
                        
                        // Weekend inclusion toggle
                        fieldset { class: "fieldset",
                            legend { class: "fieldset-legend", "Schedule Options" }
                            
                            label { class: "label cursor-pointer justify-start gap-3",
                                input { 
                                    type: "checkbox",
                                    class: "toggle toggle-accent",
                                    checked: include_weekends(),
                                    onchange: move |evt| {
                                        include_weekends.set(evt.checked());
                                    }
                                }
                                span { class: "label-text", "Include weekends in schedule" }
                            }
                        }
                        
                        // Action buttons
                        div { class: "flex gap-2 pt-2",
                            button { 
                                class: "btn btn-primary btn-sm flex-1",
                                onclick: apply_settings,
                                Icon { icon: FaRotateRight, class: "w-4 h-4" }
                                "Apply Changes"
                            }
                            
                            button { 
                                class: "btn btn-ghost btn-sm",
                                onclick: reset_settings,
                                "Reset"
                            }
                        }
                    }
                }
            }
        }
    }
}