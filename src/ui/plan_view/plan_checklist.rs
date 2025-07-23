use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaCheckDouble, FaSquare};
use dioxus_motion::prelude::*;
use std::collections::HashMap;

use crate::types::{Plan, PlanItem};
use crate::ui::components::modal_confirmation::Badge;
use crate::ui::hooks::use_toggle_plan_item_action;

#[derive(Debug, Clone)]
pub struct ModuleGroup {
    title: String,
    items: Vec<(usize, PlanItem)>, // (original_index, item)
    total: usize,
    completed: usize,
    progress: f32,
}

#[derive(Props, PartialEq, Clone)]
pub struct PlanChecklistProps {
    pub plan: Plan,
}

/// Group plan items by module and calculate progress
fn group_items_by_module(items: &[PlanItem]) -> Vec<ModuleGroup> {
    let mut modules: HashMap<String, Vec<(usize, PlanItem)>> = HashMap::new();

    for (index, item) in items.iter().enumerate() {
        modules
            .entry(item.module_title.clone())
            .or_default()
            .push((index, item.clone()));
    }

    let mut module_groups: Vec<ModuleGroup> = modules
        .into_iter()
        .map(|(title, items)| {
            let total = items.len();
            let completed = items.iter().filter(|(_, item)| item.completed).count();
            let progress = if total > 0 {
                (completed as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            ModuleGroup {
                title,
                items,
                total,
                completed,
                progress,
            }
        })
        .collect();

    // Sort modules by title for consistent ordering
    module_groups.sort_by(|a, b| a.title.cmp(&b.title));
    module_groups
}

/// Clean plan checklist component with module accordion interface
#[component]
pub fn PlanChecklist(props: PlanChecklistProps) -> Element {
    let module_groups = group_items_by_module(&props.plan.items);

    // Animation for the entire accordion container
    let mut container_opacity = use_motion(0.0f32);
    let mut container_y = use_motion(20.0f32);

    use_effect(move || {
        container_opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Tween(Tween::default())),
        );
        container_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let container_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateY({}px);",
            container_opacity.get_value(),
            container_y.get_value()
        )
    });

    rsx! {
        div {
            class: "join join-vertical bg-base-100 w-full",
            style: "{container_style}",
            {module_groups.iter().enumerate().map(|(module_idx, module)| {
                rsx! {
                    ModuleAccordion {
                        key: "{module.title}",
                        plan_id: props.plan.id,
                        module: module.clone(),
                        module_index: module_idx,
                    }
                }
            })}
        }
    }
}

#[derive(Props, Clone)]
pub struct ModuleAccordionProps {
    pub plan_id: uuid::Uuid,
    pub module: ModuleGroup,
    pub module_index: usize,
}

impl PartialEq for ModuleAccordionProps {
    fn eq(&self, other: &Self) -> bool {
        self.plan_id == other.plan_id
            && self.module.title == other.module.title
            && self.module.total == other.module.total
            && self.module.completed == other.module.completed
            && self.module_index == other.module_index
    }
}

/// Module accordion component with progress indicator
#[component]
pub fn ModuleAccordion(props: ModuleAccordionProps) -> Element {
    let module_id = format!("module-{}-{}", props.plan_id, props.module_index);

    // Staggered animation for each module
    let mut module_opacity = use_motion(0.0f32);
    let mut module_x = use_motion(-20.0f32);

    use_effect({
        let module_index = props.module_index;
        move || {
            // Stagger animation based on module index
            let delay = module_index as f32 * 0.1;

            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis((delay * 1000.0) as u64))
                    .await;

                module_opacity.animate_to(
                    1.0,
                    AnimationConfig::new(AnimationMode::Tween(Tween::default())),
                );
                module_x.animate_to(
                    0.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring::default())),
                );
            });
        }
    });

    let module_style = use_memo(move || {
        format!(
            "opacity: {}; transform: translateX({}px);",
            module_opacity.get_value(),
            module_x.get_value()
        )
    });

    let progress_color = if props.module.progress >= 100.0 {
        "progress-success"
    } else if props.module.progress >= 50.0 {
        "progress-primary"
    } else {
        "progress-accent"
    };

    rsx! {
        div {
            class: "collapse collapse-arrow join-item border-base-300 border",
            style: "{module_style}",

            input {
                type: "checkbox",
                id: "{module_id}",
                name: "{module_id}",
                checked: true, // Start with modules expanded
            }

            div {
                class: "collapse-title font-semibold flex items-center justify-between pr-4",

                div { class: "flex items-center gap-3",
                    h3 { class: "text-lg font-semibold", "{props.module.title}" }
                    Badge {
                        label: format!("{}/{}", props.module.completed, props.module.total),
                        color: Some(if props.module.progress >= 100.0 { "success".to_string() } else { "primary".to_string() }),
                        class: Some("text-xs".to_string()),
                    }
                }

                div { class: "flex items-center gap-2",
                    progress {
                        class: "progress {progress_color} w-24 h-2",
                        value: "{props.module.progress}",
                        max: "100"
                    }
                    span { class: "text-sm text-base-content/60", "{props.module.progress:.0}%" }
                }
            }

            div {
                class: "collapse-content",
                ul {
                    class: "space-y-2 pt-2",
                    {props.module.items.iter().map(|(original_index, item)| {
                        rsx! {
                            PlanChecklistItem {
                                key: "{original_index}",
                                plan_id: props.plan_id,
                                item: item.clone(),
                                item_index: *original_index
                            }
                        }
                    })}
                }
            }
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
        let mut local_completed = local_completed;

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
            class: "flex items-center gap-3 px-3 py-3 rounded-lg hover:bg-base-200 transition-colors cursor-pointer border border-transparent hover:border-base-300",
            style: "{item_style}",
            onclick: toggle_handler,

            {check_icon}

            div { class: "flex-1 min-w-0",
                div {
                    class: "text-sm font-medium {text_classes} truncate",
                    "{props.item.section_title}"
                }
                div {
                    class: "text-xs text-base-content/60 mt-1",
                    "{props.item.date.format(\"%Y-%m-%d\")}"
                }
            }

            Badge {
                label: if local_completed() { "Done".to_string() } else { "Pending".to_string() },
                color: Some(if local_completed() { "success".to_string() } else { "accent".to_string() }),
                class: Some("text-xs shrink-0".to_string()),
            }
        }
    }
}
