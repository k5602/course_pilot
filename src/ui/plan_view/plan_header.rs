use crate::export::ExportFormat;
use crate::ui::backend_adapter::Backend;
use crate::ui::components::modal_confirmation::{ActionMenu, DropdownItem};
use crate::ui::components::progress_ring::ProgressRing;
use crate::ui::components::toast::toast;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCheck, FaClock, FaFileCode, FaFileExport, FaFilePdf,
};

#[derive(Props, PartialEq, Clone)]
pub struct PlanHeaderProps {
    pub plan_id: uuid::Uuid,
    pub progress: u8,
    pub completed_sections: usize,
    pub total_sections: usize,
}

/// Enhanced plan header component with DaisyUI styling and accessibility
#[component]
pub fn PlanHeader(props: PlanHeaderProps) -> Element {
    let backend = use_context::<std::sync::Arc<Backend>>();

    // Export action handler with proper error handling
    let handle_export = move |format: ExportFormat, format_name: &str| {
        let backend = backend.clone();
        let plan_id = props.plan_id;
        let format_name = format_name.to_string();

        spawn(async move {
            toast::info(format!("Exporting plan as {}...", format_name));
            match backend.export_plan(plan_id, format).await {
                Ok(export_result) => match backend.save_export_data(export_result).await {
                    Ok(file_path) => {
                        toast::success(format!(
                            "Plan exported successfully to {}",
                            file_path.display()
                        ));
                    }
                    Err(e) => {
                        toast::error(format!("Failed to save export: {}", e));
                    }
                },
                Err(e) => {
                    toast::error(format!("Export failed: {}", e));
                }
            }
        });
    };

    let actions = vec![
        DropdownItem {
            label: "Export as JSON".to_string(),
            icon: Some(rsx! { Icon { icon: FaFileCode, class: "w-4 h-4" } }),
            on_select: Some(EventHandler::new({
                let handle_export = handle_export.clone();
                move |_| handle_export(ExportFormat::Json, "JSON")
            })),
            children: None,
            disabled: false,
        },
        DropdownItem {
            label: "Export as CSV".to_string(),
            icon: Some(rsx! { Icon { icon: FaFileExport, class: "w-4 h-4" } }),
            on_select: Some(EventHandler::new({
                let handle_export = handle_export.clone();
                move |_| handle_export(ExportFormat::Csv, "CSV")
            })),
            children: None,
            disabled: false,
        },
        DropdownItem {
            label: "Export as PDF".to_string(),
            icon: Some(rsx! { Icon { icon: FaFilePdf, class: "w-4 h-4" } }),
            on_select: Some(EventHandler::new({
                let handle_export = handle_export.clone();
                move |_| handle_export(ExportFormat::Pdf, "PDF")
            })),
            children: None,
            disabled: false,
        },
    ];

    rsx! {
        header {
            class: "card bg-base-100 shadow-sm border border-base-300 mb-6",
            role: "banner",
            "aria-label": "Study plan header",

            div {
                class: "card-body p-4",

                div {
                    class: "flex flex-col sm:flex-row items-start sm:items-center gap-4",

                    // Progress section with enhanced styling
                    div {
                        class: "flex items-center gap-4 flex-1",

                        // Progress ring with proper accessibility
                        div {
                            class: "flex-shrink-0",
                            role: "progressbar",
                            "aria-valuenow": "{props.progress}",
                            "aria-valuemin": "0",
                            "aria-valuemax": "100",
                            "aria-label": "Study plan progress: {props.progress}% complete",

                            ProgressRing {
                                value: props.progress as u32,
                                max: Some(100),
                                color: Some(if props.progress == 100 { "success" } else { "primary" }.to_string()),
                                size: Some(64),
                                thickness: Some(6),
                            }
                        }

                        // Progress details with responsive text
                        div {
                            class: "flex flex-col gap-1",

                            h2 {
                                class: "text-lg font-semibold text-base-content",
                                "Study Plan Progress"
                            }

                            p {
                                class: "text-sm text-base-content/70",
                                "{props.completed_sections} of {props.total_sections} sections completed"
                            }

                            // Status badge with improved styling
                            div {
                                class: "badge badge-lg gap-2",
                                class: if props.progress == 100 { "badge-success" } else { "badge-primary" },

                                if props.progress == 100 {
                                    Icon { icon: FaCheck, class: "w-3 h-3" }
                                } else {
                                    Icon { icon: FaClock, class: "w-3 h-3" }
                                }

                                span {
                                    if props.progress == 100 { "Completed" } else { "In Progress" }
                                }
                            }
                        }
                    }

                    // Actions section with enhanced styling
                    div {
                        class: "flex items-center gap-2 flex-shrink-0",

                        // Use ActionMenu for export functionality
                        ActionMenu {
                            actions: actions,
                            class: Some("btn-outline hover:btn-primary focus:btn-primary transition-colors duration-200".to_string()),
                        }
                    }
                }

                // Progress bar for visual enhancement
                div {
                    class: "mt-4",

                    div {
                        class: "flex justify-between items-center mb-1",

                        span {
                            class: "text-xs font-medium text-base-content/60",
                            "Overall Progress"
                        }

                        span {
                            class: "text-xs font-medium text-base-content/60",
                            "{props.progress}%"
                        }
                    }

                    progress {
                        class: "progress progress-primary w-full h-2",
                        class: if props.progress == 100 { "progress-success" } else { "progress-primary" },
                        value: "{props.progress}",
                        max: "100",
                        "aria-label": "Progress bar showing {props.progress}% completion",
                    }
                }
            }
        }
    }
}
