use dioxus::prelude::*;
use crate::ui::components::modal_confirmation::{CircularProgress, Badge, ActionMenu, DropdownItem};
use crate::ui::components::toast::toast;

#[derive(Props, PartialEq, Clone)]
pub struct PlanHeaderProps {
    pub progress: u8,
    pub completed_sections: usize,
    pub total_sections: usize,
}

/// Clean plan header component with progress and actions
#[component]
pub fn PlanHeader(props: PlanHeaderProps) -> Element {
    let actions = vec![
        DropdownItem {
            label: "Clear Plan".to_string(),
            icon: None,
            on_select: Some(EventHandler::new(|_| {
                toast::warning("Clear plan functionality will be implemented");
            })),
            children: None,
            disabled: false,
        },
        DropdownItem {
            label: "Export Plan".to_string(),
            icon: None,
            on_select: Some(EventHandler::new(|_| {
                toast::info("Export plan functionality will be implemented");
            })),
            children: None,
            disabled: false,
        },
    ];

    rsx! {
        div { 
            class: "flex items-center gap-4 mb-6",
            
            CircularProgress {
                value: props.progress,
                size: Some(56),
                color: Some("accent".to_string()),
                label: Some(format!("{}/{} Complete", props.completed_sections, props.total_sections)),
                class: Some("mr-2".to_string()),
            }
            
            Badge {
                label: if props.progress == 100 { 
                    "Completed".to_string() 
                } else { 
                    "In Progress".to_string() 
                },
                color: Some(if props.progress == 100 { 
                    "success".to_string() 
                } else { 
                    "accent".to_string() 
                }),
                class: Some("ml-2".to_string()),
            }
            
            ActionMenu {
                actions: actions,
                class: Some("ml-auto".to_string()),
            }
        }
    }
}