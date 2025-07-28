use dioxus::prelude::*;
use crate::types::{Course, Route};
use crate::ui::hooks::use_course_manager;
use crate::ui::components::ProgressRing;
use std::sync::Arc;

#[component]
pub fn LastAccessedCourse() -> Element {
    let course_manager = use_course_manager();
    
    // For now, we'll use the most recently created course as "last accessed"
    // In a real implementation, you'd track actual access times
    let last_course = course_manager.courses
        .iter()
        .max_by_key(|course| course.created_at)
        .cloned();

    match last_course {
        Some(course) => rsx! {
            CourseQuickAccess { course }
        },
        None => rsx! {
            div { class: "text-center py-8 text-base-content/60",
                div { class: "text-4xl mb-2", "📚" }
                p { "No courses available" }
                p { class: "text-sm mt-2", "Import your first course to get started!" }
                
                Link {
                    to: Route::AddCourse {},
                    class: "btn btn-primary btn-sm mt-4",
                    "Import Course"
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct CourseQuickAccessProps {
    course: Course,
}

#[component]
fn CourseQuickAccess(props: CourseQuickAccessProps) -> Element {
    let course = props.course.clone();
    let db = use_context::<Arc<crate::storage::Database>>();
    
    // Find plan for this course using resource
    let course_plan_resource = use_resource(move || {
        let db = db.clone();
        let course_id = course.id;
        async move {
            tokio::task::spawn_blocking(move || {
                crate::storage::get_plan_by_course_id(&db, &course_id)
            }).await.unwrap_or_else(|_| Ok(None))
        }
    });

    let course_plan = course_plan_resource.read_unchecked()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(|plan| plan.as_ref())
        .cloned();

    let progress_percentage = course_plan.as_ref()
        .map(|plan| plan.progress_percentage())
        .unwrap_or(0.0);

    let status_info = if course.is_structured() {
        if course_plan.is_some() {
            ("Ready to Study", "text-success", "▶️")
        } else {
            ("Create Study Plan", "text-warning", "📋")
        }
    } else {
        ("Structure Course", "text-info", "🏗️")
    };

    rsx! {
        div { class: "card bg-gradient-to-br from-primary/10 to-secondary/10 border border-primary/20",
            div { class: "card-body p-4",
                div { class: "flex items-center gap-4",
                    div { class: "flex-shrink-0",
                        ProgressRing {
                            value: progress_percentage as u32,
                            max: 100,
                            size: 48,
                            thickness: 4
                        }
                    }
                    
                    div { class: "flex-1 min-w-0",
                        h3 { class: "font-semibold text-lg truncate", "{course.name}" }
                        div { class: "flex items-center gap-2 text-sm text-base-content/70 mt-1",
                            span { "{course.video_count()} videos" }
                            span { "•" }
                            span { class: status_info.1, "{status_info.2} {status_info.0}" }
                        }
                        
                        if let Some(plan) = course_plan.as_ref() {
                            div { class: "text-xs text-base-content/50 mt-1",
                                "{plan.completed_sessions()}/{plan.total_sessions()} sessions completed"
                            }
                        }
                    }
                }
                
                div { class: "card-actions justify-end mt-4",
                    if course.is_structured() {
                        if course_plan.is_some() {
                            Link {
                                to: Route::PlanView { course_id: course.id.to_string() },
                                class: "btn btn-primary btn-sm",
                                "Continue Learning"
                            }
                        } else {
                            Link {
                                to: Route::PlanView { course_id: course.id.to_string() },
                                class: "btn btn-outline btn-sm",
                                "Create Plan"
                            }
                        }
                    } else {
                        Link {
                            to: Route::AllCourses {},
                            class: "btn btn-outline btn-sm",
                            "Structure Course"
                        }
                    }
                    
                    Link {
                        to: Route::AllCourses {},
                        class: "btn btn-ghost btn-sm",
                        "View All"
                    }
                }
            }
        }
    }
}