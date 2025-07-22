use dioxus::prelude::*;
use crate::ui::components::modal_confirmation::{CircularProgress, Badge, ActionMenu, DropdownItem};
use crate::ui::components::toast::toast;
use crate::ui::backend_adapter::Backend;
use crate::export::ExportFormat;

#[derive(Props, PartialEq, Clone)]
pub struct PlanHeaderProps {
    pub plan_id: uuid::Uuid,
    pub progress: u8,
    pub completed_sections: usize,
    pub total_sections: usize,
}

/// Clean plan header component with progress and actions
#[component]
pub fn PlanHeader(props: PlanHeaderProps) -> Element {
    let backend = use_context::<std::sync::Arc<Backend>>();
    
    let actions = vec![
        DropdownItem {
            label: "Export as JSON".to_string(),
            icon: None,
            on_select: Some(EventHandler::new({
                let backend = backend.clone();
                let plan_id = props.plan_id;
                move |_| {
                    let backend = backend.clone();
                    spawn(async move {
                        toast::info("Exporting plan as JSON...");
                        match backend.export_plan(plan_id, ExportFormat::Json).await {
                            Ok(export_result) => {
                                match backend.save_export_data(export_result).await {
                                    Ok(file_path) => {
                                        toast::success(&format!("Plan exported successfully to {}", file_path.display()));
                                    },
                                    Err(e) => {
                                        toast::error(&format!("Failed to save export: {}", e));
                                    }
                                }
                            },
                            Err(e) => {
                                toast::error(&format!("Export failed: {}", e));
                            }
                        }
                    });
                }
            })),
            children: None,
            disabled: false,
        },
        DropdownItem {
            label: "Export as CSV".to_string(),
            icon: None,
            on_select: Some(EventHandler::new({
                let backend = backend.clone();
                let plan_id = props.plan_id;
                move |_| {
                    let backend = backend.clone();
                    spawn(async move {
                        toast::info("Exporting plan as CSV...");
                        match backend.export_plan(plan_id, ExportFormat::Csv).await {
                            Ok(export_result) => {
                                match backend.save_export_data(export_result).await {
                                    Ok(file_path) => {
                                        toast::success(&format!("Plan exported successfully to {}", file_path.display()));
                                    },
                                    Err(e) => {
                                        toast::error(&format!("Failed to save export: {}", e));
                                    }
                                }
                            },
                            Err(e) => {
                                toast::error(&format!("Export failed: {}", e));
                            }
                        }
                    });
                }
            })),
            children: None,
            disabled: false,
        },
        DropdownItem {
            label: "Export as PDF".to_string(),
            icon: None,
            on_select: Some(EventHandler::new({
                let backend = backend.clone();
                let plan_id = props.plan_id;
                move |_| {
                    let backend = backend.clone();
                    spawn(async move {
                        toast::info("Exporting plan as PDF...");
                        match backend.export_plan(plan_id, ExportFormat::Pdf).await {
                            Ok(export_result) => {
                                match backend.save_export_data(export_result).await {
                                    Ok(file_path) => {
                                        toast::success(&format!("Plan exported successfully to {}", file_path.display()));
                                    },
                                    Err(e) => {
                                        toast::error(&format!("Failed to save export: {}", e));
                                    }
                                }
                            },
                            Err(e) => {
                                toast::error(&format!("Export failed: {}", e));
                            }
                        }
                    });
                }
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