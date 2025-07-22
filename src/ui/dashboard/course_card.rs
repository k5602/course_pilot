use dioxus::prelude::*;
use crate::types::Course;
use crate::ui::components::card::{Card, CardVariant, BadgeData};
use crate::ui::components::unified_dropdown::{UnifiedDropdown, DropdownTrigger, create_course_actions};
use crate::ui::components::export_format_dialog::ExportFormatDialog;
use crate::ui::hooks::{use_course_progress, use_course_manager, use_modal_manager};
use crate::ui::dashboard::course_actions::CourseActions;

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
    
    // Create dropdown actions for the course
    let dropdown_actions = create_course_actions(
        props.course.id,
        props.course.structure.is_some(),
        !props.course.raw_titles.is_empty(),
        EventHandler::new({
            let course_manager = course_manager.clone();
            let course_id = props.course.id;
            move |_| {
                course_manager.navigate_to_course.call(course_id);
            }
        }),
        EventHandler::new({
            let backend = crate::ui::backend_adapter::use_backend_adapter();
            let course_manager = course_manager.clone();
            move |_| {
                let backend = backend.clone();
                let course_manager = course_manager.clone();
                spawn(async move {
                    crate::ui::components::toast::toast::info("Creating study plan...");
                    
                    // Create default plan settings
                    let settings = crate::types::PlanSettings {
                        start_date: chrono::Utc::now(),
                        sessions_per_week: 3,
                        session_length_minutes: 60,
                        include_weekends: false,
                    };
                    
                    match backend.generate_plan(props.course.id, settings).await {
                        Ok(_plan) => {
                            crate::ui::components::toast::toast::success("Study plan created successfully!");
                            course_manager.refresh.call(());
                        },
                        Err(e) => {
                            crate::ui::components::toast::toast::error(&format!("Failed to create study plan: {}", e));
                        }
                    }
                });
            }
        }),
        EventHandler::new({
            let edit_modal = edit_modal.clone();
            move |_| {
                edit_modal.open.call(());
            }
        }),
        EventHandler::new({
            let backend = crate::ui::backend_adapter::use_backend_adapter();
            let course_manager = course_manager.clone();
            move |_| {
                let backend = backend.clone();
                let course_manager = course_manager.clone();
                spawn(async move {
                    crate::ui::components::toast::toast::info("Structuring course content...");
                    
                    match backend.structure_course(props.course.id).await {
                        Ok(_) => {
                            crate::ui::components::toast::toast::success("Course structured successfully!");
                            course_manager.refresh.call(());
                        },
                        Err(e) => {
                            crate::ui::components::toast::toast::error(&format!("Failed to structure course: {}", e));
                        }
                    }
                });
            }
        }),
        EventHandler::new({
            let export_dialog = export_dialog.clone();
            move |_| {
                export_dialog.open.call(());
            }
        }),
        EventHandler::new({
            let delete_modal = delete_modal.clone();
            move |_| {
                delete_modal.open.call(());
            }
        }),
    );
    
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
            badges: Some(badges),
            hover_effect: Some(true),
            on_click: Some(EventHandler::new({
                let course_manager = course_manager.clone();
                let course_id = props.course.id;
                move |_| {
                    course_manager.navigate_to_course.call(course_id);
                }
            })),
            content: Some(rsx! {
                // Add the unified dropdown as custom content with proper event isolation
                div {
                    class: "flex justify-end mt-2",
                    // Additional wrapper to ensure click isolation
                    onclick: move |evt| {
                        evt.stop_propagation();
                    },
                    UnifiedDropdown {
                        items: dropdown_actions,
                        trigger: DropdownTrigger::DotsMenu,
                        position: "dropdown-end".to_string(),
                    }
                }
            }),
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

