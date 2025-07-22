use dioxus::prelude::*;
use crate::export::ExportFormat;
use crate::ui::components::modal::Modal;

#[derive(Props, PartialEq, Clone)]
pub struct ExportFormatDialogProps {
    pub open: bool,
    pub on_close: EventHandler<()>,
    pub on_export: EventHandler<ExportFormat>,
    #[props(optional)]
    pub title: Option<String>,
}

/// Dialog for selecting export format
#[component]
pub fn ExportFormatDialog(props: ExportFormatDialogProps) -> Element {
    let mut selected_format = use_signal(|| ExportFormat::Json);
    
    // Reset selected format when dialog opens
    use_effect({
        let mut selected_format = selected_format.clone();
        move || {
            if props.open {
                selected_format.set(ExportFormat::Json);
            }
        }
    });
    
    let handle_export = {
        let on_export = props.on_export.clone();
        let on_close = props.on_close.clone();
        let selected_format = selected_format.clone();
        
        move |_| {
            on_export.call(selected_format());
            on_close.call(());
        }
    };
    
    rsx! {
        Modal {
            open: props.open,
            on_close: props.on_close.clone(),
            title: props.title.clone().unwrap_or_else(|| "Select Export Format".to_string()),
            actions: rsx! {
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| props.on_close.call(()),
                    "Cancel"
                }
                button {
                    class: "btn btn-primary",
                    onclick: handle_export,
                    "Export"
                }
            },
            
            div { class: "space-y-4",
                div { class: "form-control",
                    label { class: "label cursor-pointer justify-start gap-4",
                        input {
                            r#type: "radio",
                            name: "export-format",
                            class: "radio radio-primary",
                            checked: selected_format() == ExportFormat::Json,
                            onchange: move |_| selected_format.set(ExportFormat::Json),
                        }
                        div {
                            div { class: "font-medium", "JSON Format" }
                            div { class: "text-sm opacity-70", "Complete data structure with all metadata" }
                        }
                    }
                }
                
                div { class: "form-control",
                    label { class: "label cursor-pointer justify-start gap-4",
                        input {
                            r#type: "radio",
                            name: "export-format",
                            class: "radio radio-primary",
                            checked: selected_format() == ExportFormat::Csv,
                            onchange: move |_| selected_format.set(ExportFormat::Csv),
                        }
                        div {
                            div { class: "font-medium", "CSV Format" }
                            div { class: "text-sm opacity-70", "Tabular format for spreadsheet applications" }
                        }
                    }
                }
                
                div { class: "form-control",
                    label { class: "label cursor-pointer justify-start gap-4",
                        input {
                            r#type: "radio",
                            name: "export-format",
                            class: "radio radio-primary",
                            checked: selected_format() == ExportFormat::Pdf,
                            onchange: move |_| selected_format.set(ExportFormat::Pdf),
                        }
                        div {
                            div { class: "font-medium", "PDF Format" }
                            div { class: "text-sm opacity-70", "Formatted document for printing and sharing" }
                        }
                    }
                }
                
                div { class: "alert alert-info mt-4",
                    svg {
                        class: "stroke-current shrink-0 h-6 w-6",
                        fill: "none",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        }
                    }
                    span { "You'll be prompted to choose where to save the exported file." }
                }
            }
        }
    }
}