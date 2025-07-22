use dioxus::prelude::*;
use crate::types::Course;
use crate::ui::components::card::{Card, CardVariant, ActionItem, BadgeData};
use crate::ui::components::export_format_dialog::ExportFormatDialog;
use crate::ui::hooks::{use_course_progress, use_course_manager, use_modal_manager};
use super::CourseActions;

#[derive(Props, PartialEq, Clone)]
pub struct CourseCardProps {
    pub course: Course,
    pub index: usize,
}

/// Clean course card component with separated concerns
#[component]
pub fn CourseCard(props: CourseCardProps) -> Element {
    let course_manager = use_course_manager();
    let (progress, status, badge_color) = use_course_progress(props.course.id);
    let export_dialog = use_modal_manager(false);
    let edit_modal = use_modal_manager(false);
    let delete_modal = use_modal_manager(false);
    
    // Calculate duration string
    let duration = props.course.structure.as_ref().map(|s| {
        let secs = s.aggregate_total_duration().as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }).unwrap_or_else(|| "N/A".to_string());
    
    // Create actions for the card
    let actions = create_course_actions(&props.course, &course_manager, &export_dialog, &edit_modal, &delete_modal);
    
    // Create badges for the card
    let badges = vec![
        BadgeData {
            label: status.clone(),
            color: badge_color.clone(),
        }
    ];

    // Handle export with format selection
    let handle_export_with_format = {
        let backend = crate::ui::backend_adapter::use_backend_adapter();
        let course_id = props.course.id;
        
        move |format| {
            let backend = backend.clone();
            let course_id = course_id;
            
            spawn(async move {
                crate::ui::components::toast::toast::info(&format!("Preparing {} export...", format));
                
                // Use the progress version for better user feedback
                match backend.export_course_with_progress(
                    course_id,
                    format,
                    |progress, message| {
                        // Update toast with progress information
                        let progress_percent = (progress * 100.0).round() as u8;
                        crate::ui::components::toast::toast::info(&format!("{} ({}%)", message, progress_percent));
                    }
                ).await {
                    Ok(export_result) => {
                        // Save the exported data to a file
                        match backend.save_export_data(export_result).await {
                            Ok(path) => {
                                crate::ui::components::toast::toast::success(&format!(
                                    "Course exported successfully! Saved to: {}",
                                    path.display()
                                ));
                            },
                            Err(e) => {
                                if e.to_string().contains("cancelled") {
                                    crate::ui::components::toast::toast::info("Export cancelled");
                                } else {
                                    crate::ui::components::toast::toast::error(&format!("Failed to save export: {}", e));
                                }
                            }
                        }
                    },
                    Err(e) => {
                        crate::ui::components::toast::toast::error(&format!("Failed to export course: {}", e));
                    }
                }
            });
        }
    };

    rsx! {
        Card {
            variant: CardVariant::Course {
                video_count: props.course.raw_titles.len(),
                duration: duration,
                progress: progress,
            },
            title: props.course.name.clone(),
            actions: Some(actions),
            badges: Some(badges),
            hover_effect: Some(true),
            on_click: Some(EventHandler::new({
                let course_manager = course_manager.clone();
                let course_id = props.course.id;
                move |_| {
                    course_manager.navigate_to_course.call(course_id);
                }
            })),
        }
        
        // Course actions (modals, etc.)
        CourseActions { 
            course: props.course.clone(),
            edit_modal_open: edit_modal.is_open,
            delete_modal_open: delete_modal.is_open,
            on_edit_close: move |_| edit_modal.close.call(()),
            on_delete_close: move |_| delete_modal.close.call(()),
        }
        
        // Export format dialog
        ExportFormatDialog {
            open: export_dialog.is_open,
            on_close: move |_| export_dialog.close.call(()),
            on_export: handle_export_with_format,
            title: Some(format!("Export Course: {}", props.course.name)),
        }
    }
}

/// Create action items for course card
fn create_course_actions(
    course: &Course, 
    course_manager: &crate::ui::hooks::use_courses::CourseManager,
    export_dialog: &crate::ui::hooks::use_modals::ModalManager,
    edit_modal: &crate::ui::hooks::use_modals::ModalManager,
    delete_modal: &crate::ui::hooks::use_modals::ModalManager,
) -> Vec<ActionItem> {
    let course_id = course.id;
    let course_manager = course_manager.clone();
    
    vec![
        ActionItem {
            label: "View Plan".to_string(),
            icon: None,
            on_select: Some(EventHandler::new(move |_| {
                course_manager.navigate_to_course.call(course_id);
            })),
            disabled: false,
        },
        ActionItem {
            label: "Edit Course".to_string(),
            icon: None,
            on_select: Some(EventHandler::new({
                let edit_modal = edit_modal.clone();
                move |_| {
                    edit_modal.open.call(());
                }
            })),
            disabled: false,
        },
        ActionItem {
            label: "Structure Course".to_string(),
            icon: None,
            on_select: Some(EventHandler::new({
                let backend = crate::ui::backend_adapter::use_backend_adapter();
                let course_manager = course_manager.clone();
                move |_| {
                    let backend = backend.clone();
                    let course_manager = course_manager.clone();
                    spawn(async move {
                        crate::ui::components::toast::toast::info("Analyzing course structure...");
                        
                        // Use the progress version for better user feedback
                        match backend.structure_course_with_progress(
                            course_id,
                            |progress, message| {
                                // Update toast with progress information
                                let progress_percent = (progress * 100.0).round() as u8;
                                crate::ui::components::toast::toast::info(&format!("{} ({}%)", message, progress_percent));
                            }
                        ).await {
                            Ok(course) => {
                                crate::ui::components::toast::toast::success(&format!(
                                    "Course structured successfully! Created {} modules with {} sections.",
                                    course.structure.as_ref().map(|s| s.modules.len()).unwrap_or(0),
                                    course.structure.as_ref().map(|s| s.modules.iter().map(|m| m.sections.len()).sum::<usize>()).unwrap_or(0)
                                ));
                                
                                // Refresh the course manager to show updated structure
                                course_manager.refresh.call(());
                            },
                            Err(e) => {
                                crate::ui::components::toast::toast::error(&format!("Failed to structure course: {}", e));
                            }
                        }
                    });
                }
            })),
            disabled: course.raw_titles.is_empty() || course.structure.is_some(),
        },
        ActionItem {
            label: "Export".to_string(),
            icon: None,
            on_select: Some(EventHandler::new({
                let export_dialog = export_dialog.clone();
                move |_| {
                    export_dialog.open.call(());
                }
            })),
            disabled: false,
        },
        ActionItem {
            label: "Delete".to_string(),
            icon: None,
            on_select: Some(EventHandler::new({
                let delete_modal = delete_modal.clone();
                move |_| {
                    delete_modal.open.call(());
                }
            })),
            disabled: false,
        },
    ]
}