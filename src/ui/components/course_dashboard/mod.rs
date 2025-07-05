use crate::types::{AppState, Course, Route};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{
    MdCheckCircle, MdDelete, MdSchedule, MdSearch, MdTrendingUp,
};
use dioxus_free_icons::icons::md_av_icons::{MdLibraryBooks, MdMovie};
use dioxus_free_icons::icons::md_content_icons::{MdAdd, MdContentCopy, MdCreate};
use dioxus_free_icons::icons::md_device_icons::MdAccessTime;
use dioxus_motion::prelude::*;
use dioxus_toast::{ToastInfo, ToastManager};

fn count_structured_courses(courses: Vec<Course>) -> usize {
    courses.iter().filter(|c| c.is_structured()).count()
}

fn count_total_videos(courses: Vec<Course>) -> usize {
    courses.iter().map(|c| c.video_count()).sum()
}

fn format_date(date: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(date);

    if diff.num_days() == 0 {
        "today".to_string()
    } else if diff.num_days() == 1 {
        "yesterday".to_string()
    } else if diff.num_days() < 7 {
        format!("{} days ago", diff.num_days())
    } else if diff.num_days() < 30 {
        format!("{} weeks ago", diff.num_days() / 7)
    } else {
        date.format("%B %d, %Y").to_string()
    }
}

#[derive(Props, PartialEq, Clone)]
struct CourseCardProps {
    course: Course,
}

fn CourseCard(props: CourseCardProps) -> Element {
    let course = props.course;
    let mut app_state = use_context::<Signal<AppState>>();
    let formatted_date = format_date(course.created_at);
    let video_count = course.video_count();
    let is_structured = course.is_structured();
    let status_class = if is_structured {
        "structured"
    } else {
        "unstructured"
    };
    let mut show_delete_dialog = use_signal(|| false);
    let mut show_duplicate_dialog = use_signal(|| false);
    // Removed content expansion for compact grid layout
    let mut toast: Signal<ToastManager> = use_context();

    // Motion for icon buttons
    let mut scale_edit = use_motion(1.0f32);
    let mut scale_duplicate = use_motion(1.0f32);
    let mut scale_delete = use_motion(1.0f32);

    // Prepare course statistics
    let (modules_count, duration, difficulty) = if is_structured {
        if let Some(structure) = &course.structure {
            let modules = structure.modules.len();
            let duration_hours = structure.metadata.estimated_duration_hours.unwrap_or(0.0);
            let difficulty_str = structure
                .metadata
                .difficulty_level
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            (modules, duration_hours, difficulty_str)
        } else {
            (0, 0.0, "Unknown".to_string())
        }
    } else {
        (0, 0.0, "Pending".to_string())
    };

    // Compact content preview - show max 3 items
    let content_preview = if !course.raw_titles.is_empty() {
        let preview_items: Vec<_> = course
            .raw_titles
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, title)| {
                rsx!(
                    div { class: "course-content-item",
                        span { class: "course-content-number", "{i + 1}" }
                        span { class: "course-content-title-text", "{title}" }
                    }
                )
            })
            .collect();

        rsx!(
            div { class: "course-content-preview",
                div { class: "course-content-header",
                    h4 { class: "course-content-title", "Course Content" }
                }
                div { class: "course-content-list", {preview_items.into_iter()} }
                if course.raw_titles.len() > 3 {
                    div {
                        class: "course-content-item",
                        style: "font-style: italic; opacity: 0.7;",
                        span { class: "course-content-title-text", "... and {course.raw_titles.len() - 3} more videos" }
                    }
                }
            }
        )
    } else {
        rsx!(
            div { class: "course-content-preview",
                div { class: "course-content-header",
                    h4 { class: "course-content-title", "Course Content" }
                }
                div { class: "course-content-item",
                    span { class: "course-content-title-text", "No content available" }
                }
            }
        )
    };

    // Dialog components
    let delete_dialog = if *show_delete_dialog.read() {
        Some(rsx!(
            crate::ui::components::alert_dialog::AlertDialogRoot {
                open: true,
                on_open_change: move |open| {
                    show_delete_dialog.set(open)
                },
                class: "alert-dialog-backdrop",
                crate::ui::components::alert_dialog::AlertDialogContent {
                    class: "alert-dialog",
                    crate::ui::components::alert_dialog::AlertDialogTitle { "Delete Course" }
                    crate::ui::components::alert_dialog::AlertDialogDescription {
                        "Are you sure you want to delete this course? This action cannot be undone."
                    }
                    crate::ui::components::alert_dialog::AlertDialogActions {
                        crate::ui::components::alert_dialog::AlertDialogCancel {
                            class: "alert-dialog-cancel",
                            on_click: move |_| show_delete_dialog.set(false),
                            "Cancel"
                        }
                        crate::ui::components::alert_dialog::AlertDialogAction {
                            class: "alert-dialog-action",
                            on_click: {
                                let course_id = course.id;
                                move |_| {
                                    app_state.write().courses.retain(|c| c.id != course_id);
                                    show_delete_dialog.set(false);
                                    toast.write().popup(ToastInfo::simple("Course deleted"));
                                }
                            },
                            "Delete"
                        }
                    }
                }
            }
        ))
    } else {
        None
    };

    let duplicate_dialog = if *show_duplicate_dialog.read() {
        Some(rsx!(
            crate::ui::components::alert_dialog::AlertDialogRoot {
                open: true,
                on_open_change: move |open| show_duplicate_dialog.set(open),
                class: "alert-dialog-backdrop",
                crate::ui::components::alert_dialog::AlertDialogContent {
                    class: "alert-dialog",
                    crate::ui::components::alert_dialog::AlertDialogTitle { "Duplicate Course" }
                    crate::ui::components::alert_dialog::AlertDialogDescription {
                        "Do you want to duplicate this course? All course data will be copied."
                    }
                    crate::ui::components::alert_dialog::AlertDialogActions {
                        crate::ui::components::alert_dialog::AlertDialogCancel {
                            class: "alert-dialog-cancel",
                            on_click: move |_| show_duplicate_dialog.set(false),
                            "Cancel"
                        }
                        crate::ui::components::alert_dialog::AlertDialogAction {
                            class: "alert-dialog-action",
                            on_click: {
                                let course = course.clone();
                                move |_| {
                                    let mut new_course = course.clone();
                                    new_course.id = uuid::Uuid::new_v4();
                                    new_course.name = format!("{} (Copy)", new_course.name);
                                    app_state.write().courses.push(new_course);
                                    show_duplicate_dialog.set(false);
                                    toast.write().popup(ToastInfo::simple("Course duplicated"));
                                }
                            },
                            "Duplicate"
                        }
                    }
                }
            }
        ))
    } else {
        None
    };

    rsx! {
        div {
            class: format!("course-card {}", status_class),

            // Course Card Header
            div { class: "course-card-header",
                // Secondary actions (show on hover)
                div { class: "course-secondary-actions",
                    button {
                        class: "course-action-btn",
                        title: "Edit Course",
                        style: format!("transform: scale({});", scale_edit.get_value()),
                        onmouseenter: move |_| {
                            scale_edit.animate_to(1.1, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                        },
                        onmouseleave: move |_| {
                            scale_edit.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                        },
                        onclick: move |_| {
                            toast.write().popup(ToastInfo::simple("Edit not implemented"));
                        },
                        Icon { width: 12, height: 12, fill: "currentColor", icon: MdCreate }
                    }

                    button {
                        class: "course-action-btn",
                        title: "Duplicate Course",
                        style: format!("transform: scale({});", scale_duplicate.get_value()),
                        onmouseenter: move |_| {
                            scale_duplicate.animate_to(1.1, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                        },
                        onmouseleave: move |_| {
                            scale_duplicate.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                        },
                        onclick: move |_| show_duplicate_dialog.set(true),
                        Icon { width: 12, height: 12, fill: "currentColor", icon: MdContentCopy }
                    }

                    button {
                        class: "course-action-btn danger",
                        title: "Delete Course",
                        style: format!("transform: scale({});", scale_delete.get_value()),
                        onmouseenter: move |_| {
                            scale_delete.animate_to(1.1, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                        },
                        onmouseleave: move |_| {
                            scale_delete.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                        },
                        onclick: move |_| show_delete_dialog.set(true),
                        Icon { width: 12, height: 12, fill: "currentColor", icon: MdDelete }
                    }
                }

                div { class: "course-title-row",
                    h3 { class: "course-title", "{course.name}" }
                }

                div {
                    class: format!("course-status-badge {}", status_class),
                    if is_structured {
                        Icon { width: 12, height: 12, fill: "currentColor", icon: MdCheckCircle }
                    } else {
                        Icon { width: 12, height: 12, fill: "currentColor", icon: MdSchedule }
                    }
                    span { if is_structured { "Ready" } else { "Pending" } }
                }

                div { class: "course-meta-row",
                    div { class: "course-meta-item",
                        Icon { width: 12, height: 12, fill: "currentColor", icon: MdMovie }
                        span { "{video_count}" }
                    }
                    div { class: "course-meta-item",
                        Icon { width: 12, height: 12, fill: "currentColor", icon: MdAccessTime }
                        span { "Created {formatted_date}" }
                    }
                }
            }

            // Course Card Body
            div { class: "course-card-body",
                if is_structured {
                    div { class: "course-stats-grid",
                        div { class: "course-stat",
                            div { class: "course-stat-value", "{modules_count}" }
                            div { class: "course-stat-label", "Modules" }
                        }
                        div { class: "course-stat",
                            div { class: "course-stat-value", "{duration:.1}h" }
                            div { class: "course-stat-label", "Duration" }
                        }
                        div { class: "course-stat",
                            div { class: "course-stat-value", "{difficulty}" }
                            div { class: "course-stat-label", "Level" }
                        }
                    }
                } else {
                    div { class: "course-stats-grid",
                        div { class: "course-stat",
                            div { class: "course-stat-value", "—" }
                            div { class: "course-stat-label", "Modules" }
                        }
                        div { class: "course-stat",
                            div { class: "course-stat-value", "—" }
                            div { class: "course-stat-label", "Duration" }
                        }
                        div { class: "course-stat",
                            div { class: "course-stat-value", "Pending" }
                            div { class: "course-stat-label", "Analysis" }
                        }
                    }
                }

                // Content preview
                {content_preview}
            }

            // Course Card Footer
            div { class: "course-card-footer",
                button {
                    class: "course-primary-action",
                    onclick: {
                        let course_id = course.id;
                        move |_| {
                            app_state.write().current_route = Route::PlanView(course_id);
                        }
                    },
                    if is_structured { "View Plan" } else { "Structure Course" }
                }
            }
        }

        if let Some(dialog) = delete_dialog { {dialog} }
        if let Some(dialog) = duplicate_dialog { {dialog} }
    }
}

pub fn CourseDashboard() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut search_query = use_signal(String::new);
    let mut filter_structured = use_signal(|| false);

    let filtered_courses = use_memo(move || {
        let state = app_state.read();
        let query = search_query.read().to_lowercase();
        state
            .courses
            .iter()
            .filter(|course| {
                let matches_search =
                    query.is_empty() || course.name.to_lowercase().contains(&query);
                let matches_structure = if *filter_structured.read() {
                    course.is_structured()
                } else {
                    true
                };
                matches_search && matches_structure
            })
            .cloned()
            .collect::<Vec<Course>>()
    });

    // Animated stats
    let mut total_courses = use_motion(app_state.read().courses.len() as f32);
    let mut structured_courses =
        use_motion(count_structured_courses(app_state.read().courses.clone()) as f32);
    let mut total_videos = use_motion(count_total_videos(app_state.read().courses.clone()) as f32);

    use_effect(move || {
        let courses = app_state.read().courses.clone();
        total_courses.animate_to(
            courses.len() as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
        structured_courses.animate_to(
            count_structured_courses(courses.clone()) as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
        total_videos.animate_to(
            count_total_videos(courses) as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/course_dashboard/style.css")
        }

        div { class: "dashboard-container",
            // Header
            header { class: "dashboard-header",
                h1 { class: "dashboard-title", "Dashboard" }
                button {
                    class: "dashboard-add-btn",
                    onclick: move |_| {
                        app_state.write().current_route = Route::AddCourse;
                    },
                    Icon { width: 20, height: 20, fill: "currentColor", icon: MdAdd }
                    span { "Add Course" }
                }
            }

            // Enhanced Stats Section
            section { class: "stats-section",
                div { class: "stats-grid",
                    div { class: "stat-card",
                        div { class: "stat-header",
                            div { class: "stat-icon",
                                Icon { width: 24, height: 24, fill: "currentColor", icon: MdLibraryBooks }
                            }
                            div { class: "stat-content",
                                h3 { class: "stat-value", "{total_courses.get_value().round() as usize}" }
                                p { class: "stat-label", "Total Courses" }
                            }
                        }
                        div { class: "stat-trend",
                            Icon { width: 12, height: 12, fill: "currentColor", icon: MdTrendingUp }
                            span { "Growing" }
                        }
                    }

                    div { class: "stat-card",
                        div { class: "stat-header",
                            div { class: "stat-icon",
                                Icon { width: 24, height: 24, fill: "currentColor", icon: MdCheckCircle }
                            }
                            div { class: "stat-content",
                                h3 { class: "stat-value", "{structured_courses.get_value().round() as usize}" }
                                p { class: "stat-label", "Structured" }
                            }
                        }
                        div { class: "stat-trend",
                            Icon { width: 12, height: 12, fill: "currentColor", icon: MdTrendingUp }
                            span { "Ready to learn" }
                        }
                    }

                    div { class: "stat-card",
                        div { class: "stat-header",
                            div { class: "stat-icon",
                                Icon { width: 24, height: 24, fill: "currentColor", icon: MdMovie }
                            }
                            div { class: "stat-content",
                                h3 { class: "stat-value", "{total_videos.get_value().round() as usize}" }
                                p { class: "stat-label", "Total Videos" }
                            }
                        }
                        div { class: "stat-trend",
                            Icon { width: 12, height: 12, fill: "currentColor", icon: MdTrendingUp }
                            span { "Content available" }
                        }
                    }
                }
            }

            // Enhanced Search and Filter Controls
            section { class: "controls-section",
                div { class: "search-container",
                    div { class: "search-icon",
                        Icon { width: 20, height: 20, fill: "currentColor", icon: MdSearch }
                    }
                    input {
                        class: "search-input",
                        r#type: "search",
                        placeholder: "Search courses...",
                        value: search_query(),
                        oninput: move |evt| search_query.set(evt.value())
                    }
                }

                div { class: "filter-controls",
                    button {
                        class: if !*filter_structured.read() { "filter-pill active" } else { "filter-pill" },
                        onclick: move |_| filter_structured.set(false),
                        "All Courses"
                    }
                    button {
                        class: if *filter_structured.read() { "filter-pill active" } else { "filter-pill" },
                        onclick: move |_| filter_structured.set(true),
                        "Structured Only"
                    }
                }
            }

            // Course grid or empty state
            if filtered_courses().is_empty() && app_state.read().courses.is_empty() {
                section { class: "empty-state",
                    div { class: "empty-state-icon", "📚" }
                    h2 { class: "empty-state-title", "Welcome to Course Pilot!" }
                    p { class: "empty-state-description", "Start your learning journey by adding your first course from YouTube or local folders." }
                    button {
                        class: "dashboard-add-btn",
                        onclick: move |_| {
                            app_state.write().current_route = Route::AddCourse;
                        },
                        Icon { width: 20, height: 20, fill: "currentColor", icon: MdAdd }
                        span { "Add Your First Course" }
                    }
                }
            } else if filtered_courses().is_empty() {
                section { class: "empty-state",
                    div { class: "empty-state-icon", "🔍" }
                    h2 { class: "empty-state-title", "No courses found" }
                    p { class: "empty-state-description", "Try adjusting your search terms or filters to find what you're looking for." }
                }
            } else {
                section { class: "courses-section",
                    div { class: "courses-grid",
                        for course in filtered_courses() {
                            CourseCard { course }
                        }
                    }
                }
            }
        }
    }
}
