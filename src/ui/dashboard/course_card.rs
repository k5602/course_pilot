use dioxus::prelude::*;
use crate::types::Course;
use crate::ui::components::card::{Card, CardVariant, ActionItem, BadgeData};
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
    
    // Calculate duration string
    let duration = props.course.structure.as_ref().map(|s| {
        let secs = s.aggregate_total_duration().as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }).unwrap_or_else(|| "N/A".to_string());
    
    // Create actions for the card
    let actions = create_course_actions(&props.course, &course_manager);
    
    // Create badges for the card
    let badges = vec![
        BadgeData {
            label: status.clone(),
            color: badge_color.clone(),
        }
    ];

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
        }
    }
}

/// Create action items for course card
fn create_course_actions(
    course: &Course, 
    course_manager: &crate::ui::hooks::use_courses::CourseManager,
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
            on_select: Some(EventHandler::new(move |_| {
                // Edit functionality is handled by CourseActions component
                crate::ui::components::toast::toast::info("Edit course functionality");
            })),
            disabled: false,
        },
        ActionItem {
            label: "Export".to_string(),
            icon: None,
            on_select: Some(EventHandler::new(move |_| {
                crate::ui::components::toast::toast::info("Export functionality will be implemented in task 3");
            })),
            disabled: false,
        },
        ActionItem {
            label: "Delete".to_string(),
            icon: None,
            on_select: Some(EventHandler::new(move |_| {
                // Delete functionality is handled by CourseActions component
                crate::ui::components::toast::toast::info("Delete course functionality");
            })),
            disabled: false,
        },
    ]
}