use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaSquare};
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;

use crate::types::{Plan, PlanItem};
use crate::ui::components::modal_confirmation::Badge;
use crate::ui::hooks::use_toggle_plan_item_action;

#[derive(Props, PartialEq, Clone)]
pub struct PlanChecklistProps {
    pub plan: Plan,
}

/// Clean plan checklist component
#[component]
pub fn PlanChecklist(props: PlanChecklistProps) -> Element {
    rsx! {
        ul {
            class: "space-y-4",
            {props.plan.items.iter().enumerate().map(|(idx, item)| {
                rsx! {
                    PlanChecklistItem { 
                        key: "{idx}",
                        plan_id: props.plan.id,
                        item: item.clone(), 
                        item_index: idx 
                    }
                }
            })}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct PlanChecklistItemProps {
    pub plan_id: uuid::Uuid,
    pub item: PlanItem,
    pub item_index: usize,
}

/// Individual plan checklist item component
#[component]
pub fn PlanChecklistItem(props: PlanChecklistItemProps) -> Element {
    let toggle_completion = use_toggle_plan_item_action();
    let mut local_completed = use_signal(|| props.item.completed);
    
    // Sync local state with prop changes
    use_effect(move || {
        local_completed.set(props.item.completed);
    });
    
    let toggle_handler = {
        let plan_id = props.plan_id;
        let item_index = props.item_index;
        let mut local_completed = local_completed.clone();
        
        move |_| {
            let new_state = !local_completed();
            local_completed.set(new_state);
            toggle_completion(plan_id, item_index, new_state);
        }
    };

    // Animation
    let mut item_opacity = use_motion(0.0f32);
    let mut item_x = use_motion(-12.0f32);

    use_effect(move || {
        item_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        item_x.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let item_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateX({}px); transition: opacity 0.3s, transform 0.3s;",
            item_opacity.get_value(),
            item_x.get_value()
        )
    });

    let check_icon = if local_completed() {
        rsx! {
            Icon { icon: FaCheckDouble, class: "w-5 h-5 text-success" }
        }
    } else {
        rsx! {
            Icon { icon: FaSquare, class: "w-5 h-5 text-base-content/40" }
        }
    };

    let text_classes = if local_completed() {
        "line-through text-base-content/40"
    } else {
        "text-base-content"
    };

    rsx! {
        li {
            class: "flex items-center gap-3 px-2 py-2 rounded hover:bg-base-300 transition-colors cursor-pointer",
            style: "{item_style}",
            onclick: toggle_handler,
            
            {check_icon}
            
            span { 
                class: "flex-1 text-sm {text_classes}", 
                "{props.item.module_title} / {props.item.section_title}" 
            }
            
            Badge {
                label: if local_completed() { "Done".to_string() } else { "Pending".to_string() },
                color: Some(if local_completed() { "success".to_string() } else { "accent".to_string() }),
                class: Some("ml-2".to_string()),
            }
            
            span { 
                class: "text-xs text-base-content/60", 
                "{props.item.date.format(\"%Y-%m-%d\")}" 
            }
        }
    }
}