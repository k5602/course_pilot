use crate::types::Course;
use crate::ui::{
    BadgeData, Card, CardVariant, DropdownTrigger, ExportFormatDialog, UnifiedDropdown,
    create_course_actions, toast_helpers, use_course_manager, use_course_progress,
    use_modal_manager,
};
use dioxus::prelude::*;

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
    let duration = props
        .course
        .structure
        .as_ref()
        .map(|s| {
            let secs = s.aggregate_total_duration().as_secs();
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            format!("{hours}h {mins}m")
        })
        .unwrap_or_else(|| "N/A".to_string());

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
            let plan_manager = crate::ui::hooks::use_plan_manager();
            let course_manager = course_manager.clone();
            move |_| {
                let plan_manager = plan_manager.clone();
                let course_manager = course_manager.clone();
                spawn(async move {
                    toast_helpers::info("Creating study plan...");

                    // Create default plan settings
                    let settings = crate::types::PlanSettings {
                        start_date: chrono::Utc::now(),
                        sessions_per_week: 3,
                        session_length_minutes: 60,
                        include_weekends: false,
                        advanced_settings: None,
                    };

                    // Call the callback (which handles the async work and toast messages internally)
                    plan_manager.generate_plan.call((props.course.id, settings));
                    course_manager.refresh.call(());
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
            let analytics_manager = crate::ui::hooks::use_analytics_manager();
            let course_manager = course_manager.clone();
            move |_| {
                let analytics_manager = analytics_manager.clone();
                let course_manager = course_manager.clone();
                spawn(async move {
                    toast_helpers::info("Structuring course content...");

                    // Call the callback (which handles the async work and toast messages internally)
                    analytics_manager.structure_course.call(props.course.id);
                    course_manager.refresh.call(());
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
    let badges = vec![BadgeData {
        label: status.clone(),
        color: badge_color.clone(),
    }];

    // Handle export with format selection
    let handle_export_with_format = {
        let export_manager = crate::ui::hooks::use_export_manager();
        let course_id = props.course.id;

        move |format| {
            let export_manager = export_manager.clone();
            let course_id = course_id;

            spawn(async move {
                toast_helpers::info(format!("Preparing {format} export..."));

                // Call the callback (which handles the async work and toast messages internally)
                export_manager.export_course_with_progress.call((
                    course_id,
                    format,
                    Box::new(|progress, message| {
                        // Update toast with progress information
                        let progress_percent = (progress * 100.0).round() as u8;
                        toast_helpers::info(format!("{message} ({progress_percent}%)"));
                    }),
                ));
            });
        }
    };

    rsx! {
        Card {
            variant: CardVariant::Course {
                video_count: props.course.raw_titles.len(),
                duration,
                progress,
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
