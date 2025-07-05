use crate::ui::components::Button;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::{MdCheckCircle, MdDelete};
use dioxus_free_icons::icons::md_av_icons::{MdLibraryBooks, MdMovie};
use dioxus_free_icons::icons::md_content_icons::{MdContentCopy, MdCreate};
use dioxus_motion::prelude::*;
use dioxus_toast::{ToastInfo, ToastManager};
// Course Dashboard UI Component
//
// This component displays the main dashboard where users can view all their courses,
// see progress, and navigate to add new courses or view existing plans.

use crate::types::{AppState, Course, Route};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;

/// Helper: count structured courses
fn count_structured_courses(courses: Vec<Course>) -> usize {
    courses.iter().filter(|c| c.is_structured()).count()
}

/// Helper: count total videos
fn count_total_videos(courses: Vec<Course>) -> usize {
    courses.iter().map(|c| c.video_count()).sum()
}

/// Helper: format date
fn format_date(date: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(date);

    if duration.num_days() == 0 {
        "today".to_string()
    } else if duration.num_days() == 1 {
        "yesterday".to_string()
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_weeks() == 1 {
        "1 week ago".to_string()
    } else if duration.num_weeks() < 4 {
        format!("{} weeks ago", duration.num_weeks())
    } else {
        date.format("%b %d, %Y").to_string()
    }
}

/// Standalone CourseCard component
#[component]
fn CourseCard(course: Course) -> Element {
    use dioxus::prelude::*;
    let mut app_state = use_context::<Signal<AppState>>();
    let formatted_date = format_date(course.created_at);
    let video_count = course.video_count();
    let is_structured = course.is_structured();

    // Dialog state for confirmation modals
    let show_delete_dialog = use_signal(|| false);
    let show_duplicate_dialog = use_signal(|| false);

    // Toast manager
    let mut toast: Signal<ToastManager> = use_context();

    // Dialog RSX as Option<Element>
    let delete_dialog = if *show_delete_dialog.read() {
        Some(rsx!(
            crate::ui::components::alert_dialog::AlertDialogRoot {
                open: true,
                on_open_change: move |open| {
                    let mut show_delete_dialog = show_delete_dialog.clone();
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
                            on_click: move |_| {
                                let mut show_delete_dialog = show_delete_dialog.clone();
                                show_delete_dialog.set(false)
                            },
                            "Cancel"
                        }
                        crate::ui::components::alert_dialog::AlertDialogAction {
                            class: "alert-dialog-action",
                            on_click: {
                                let mut show_delete_dialog = show_delete_dialog.clone();
                                let mut app_state = app_state.clone();
                                let course_id = course.id;
                                let mut toast = toast.clone();
                                move |_| {
                                    // Remove course from state
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
                on_open_change: move |open| {
                    let mut show_duplicate_dialog = show_duplicate_dialog.clone();
                    show_duplicate_dialog.set(open)
                },
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
                            on_click: move |_| {
                                let mut show_duplicate_dialog = show_duplicate_dialog.clone();
                                show_duplicate_dialog.set(false)
                            },
                            "Cancel"
                        }
                        crate::ui::components::alert_dialog::AlertDialogAction {
                            class: "alert-dialog-action",
                            on_click: {
                                let mut show_duplicate_dialog = show_duplicate_dialog.clone();
                                let mut app_state = app_state.clone();
                                let course = course.clone();
                                let mut toast = toast.clone();
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

    // Motion for icon buttons
    let scale_edit = use_motion(1.0f32);
    let scale_duplicate = use_motion(1.0f32);
    let scale_delete = use_motion(1.0f32);

    // Prepare badges
    let mut badges = vec![];
    if is_structured {
        badges.push(rsx!(span { class: "course-card-badge", "Structured" }));
    } else {
        badges.push(rsx!(span { class: "course-card-badge unstructured", "Unstructured" }));
    }
    badges.push(rsx!(span { class: "course-card-badge", "{video_count} videos" }));

    // Prepare meta
    let meta = if is_structured {
        if let Some(structure) = &course.structure {
            let mut meta_vec = vec![rsx!(span { "📁 {structure.modules.len()} modules" })];
            if let Some(duration) = structure.metadata.estimated_duration_hours {
                meta_vec.push(rsx!(span { "⏱️ ~{duration:.1} hours" }));
            }
            if let Some(difficulty) = &structure.metadata.difficulty_level {
                meta_vec.push(rsx!(span { "📊 {difficulty} level" }));
            }
            meta_vec
        } else {
            vec![]
        }
    } else {
        vec![rsx!(span { class: "course-card-structure", "Course structure analysis pending..." })]
    };

    // Prepare sample content
    let mut sample_titles = vec![];
    for (i, title) in course.raw_titles.iter().take(3).enumerate() {
        sample_titles.push(rsx!(
            div {
                span { style: "color:#bbb; margin-right:0.4rem;", "{i + 1}." }
                span { "{title}" }
            }
        ));
    }
    if course.raw_titles.len() > 3 {
        sample_titles.push(rsx!(
            div { style: "color:#bbb;", "... and {course.raw_titles.len() - 3} more" }
        ));
    }

    rsx! {
        div {
            class: "course-card-root",
            tabindex: "0",
            onclick: {
                let course_id = course.id;
                move |_| {
                    app_state.write().current_route = Route::PlanView(course_id);
                }
            },
            onkeydown: move |evt| {
                use dioxus::events::Key;
                if matches!(evt.key(), Key::Enter) || evt.key() == Key::Character(" ".to_string()) {
                    let course_id = course.id;
                    app_state.write().current_route = Route::PlanView(course_id);
                }
            },

            div { class: "course-card-header",
                div {
                    class: "course-card-title",
                    "{course.name}"
                }
                div {
                    class: "course-card-date",
                    "Created {formatted_date}"
                }
            }

            div { class: "course-card-badges", { badges.into_iter() } }

            div { class: "course-card-meta", { meta.into_iter() } }

            div { class: "course-card-sample",
                div { style: "font-weight:500; color:#888; margin-bottom:0.2rem;", "Sample content:" }
                div { { sample_titles.into_iter() } }
            }

            div { class: "course-card-actions",
                Button {
                    onclick: {
                        let course_id = course.id;
                        move |evt: Event<MouseData>| {
                            evt.stop_propagation();
                            app_state.write().current_route = Route::PlanView(course_id);
                        }
                    },
                    if is_structured { "View Plan" } else { "Structure Course" }
                }
                // Edit action
                button {
                    class: "icon-action-btn",
                    title: "Edit Course",
                    aria_label: "Edit Course",
                    tabindex: "0",
                    style: format!("transform: scale({}); transition: transform 0.12s cubic-bezier(0.4,0,0.2,1);", scale_edit.get_value()),
                    onmouseenter: {
                        let scale_edit = scale_edit.clone();
                        move |_| scale_edit.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmouseleave: {
                        let scale_edit = scale_edit.clone();
                        move |_| scale_edit.clone().animate_to(
                            1.0,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmousedown: {
                        let scale_edit = scale_edit.clone();
                        move |_| scale_edit.clone().animate_to(
                            0.93,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmouseup: {
                        let scale_edit = scale_edit.clone();
                        move |_| scale_edit.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onfocus: {
                        let scale_edit = scale_edit.clone();
                        move |_| scale_edit.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onblur: {
                        let scale_edit = scale_edit.clone();
                        move |_| scale_edit.clone().animate_to(
                            1.0,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onclick: move |evt: Event<MouseData>| {
                        evt.stop_propagation();
                        toast.write().popup(ToastInfo::simple("Edit not implemented"));
                    },
                    Icon {
                        width: 22,
                        height: 22,
                        fill: "#666",
                        icon: MdCreate,
                    }
                }
                // Duplicate action
                button {
                    class: "icon-action-btn",
                    title: "Duplicate Course",
                    aria_label: "Duplicate Course",
                    tabindex: "0",
                    style: format!("transform: scale({}); transition: transform 0.12s cubic-bezier(0.4,0,0.2,1);", scale_duplicate.get_value()),
                    onmouseenter: {
                        let scale_duplicate = scale_duplicate.clone();
                        move |_| scale_duplicate.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmouseleave: {
                        let scale_duplicate = scale_duplicate.clone();
                        move |_| scale_duplicate.clone().animate_to(
                            1.0,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmousedown: {
                        let scale_duplicate = scale_duplicate.clone();
                        move |_| scale_duplicate.clone().animate_to(
                            0.93,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmouseup: {
                        let scale_duplicate = scale_duplicate.clone();
                        move |_| scale_duplicate.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onfocus: {
                        let scale_duplicate = scale_duplicate.clone();
                        move |_| scale_duplicate.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onblur: {
                        let scale_duplicate = scale_duplicate.clone();
                        move |_| scale_duplicate.clone().animate_to(
                            1.0,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onclick: move |evt: Event<MouseData>| {
                        evt.stop_propagation();
                        let mut show_duplicate_dialog = show_duplicate_dialog.clone();
                        show_duplicate_dialog.set(true);
                    },
                    Icon {
                        width: 22,
                        height: 22,
                        fill: "#666",
                        icon: MdContentCopy,
                    }
                }
                // Delete action
                button {
                    class: "icon-action-btn",
                    title: "Delete Course",
                    aria_label: "Delete Course",
                    tabindex: "0",
                    style: format!("transform: scale({}); transition: transform 0.12s cubic-bezier(0.4,0,0.2,1);", scale_delete.get_value()),
                    onmouseenter: {
                        let scale_delete = scale_delete.clone();
                        move |_| scale_delete.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmouseleave: {
                        let scale_delete = scale_delete.clone();
                        move |_| scale_delete.clone().animate_to(
                            1.0,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmousedown: {
                        let scale_delete = scale_delete.clone();
                        move |_| scale_delete.clone().animate_to(
                            0.93,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onmouseup: {
                        let scale_delete = scale_delete.clone();
                        move |_| scale_delete.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onfocus: {
                        let scale_delete = scale_delete.clone();
                        move |_| scale_delete.clone().animate_to(
                            1.12,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onblur: {
                        let scale_delete = scale_delete.clone();
                        move |_| scale_delete.clone().animate_to(
                            1.0,
                            AnimationConfig::new(AnimationMode::Spring(Spring {
                                stiffness: 300.0,
                                damping: 20.0,
                                mass: 1.0,
                                velocity: 0.0,
                            })),
                        )
                    },
                    onclick: move |evt: Event<MouseData>| {
                        evt.stop_propagation();
                        let mut show_delete_dialog = show_delete_dialog.clone();
                        show_delete_dialog.set(true);
                    },
                    Icon {
                        width: 22,
                        height: 22,
                        fill: "#c00",
                        icon: MdDelete,
                    }
                }
            }

            // Dialogs (conditionally rendered)
            {delete_dialog}
            {duplicate_dialog}
        }
    }
}

/// Main dashboard component that displays the list of courses
#[component]
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

    // Animated metrics with icons
    let mut total_courses = use_motion(app_state.read().courses.len() as f32);
    let mut structured_courses =
        use_motion(count_structured_courses(app_state.read().courses.clone()) as f32);
    let mut total_videos = use_motion(count_total_videos(app_state.read().courses.clone()) as f32);

    use_effect(move || {
        total_courses.animate_to(
            app_state.read().courses.len() as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 120.0,
                damping: 18.0,
                mass: 1.0,
                velocity: 0.0,
            })),
        );
        structured_courses.animate_to(
            count_structured_courses(app_state.read().courses.clone()) as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 120.0,
                damping: 18.0,
                mass: 1.0,
                velocity: 0.0,
            })),
        );
        total_videos.animate_to(
            count_total_videos(app_state.read().courses.clone()) as f32,
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 120.0,
                damping: 18.0,
                mass: 1.0,
                velocity: 0.0,
            })),
        );
    });

    let metrics = if !app_state.read().courses.is_empty() {
        vec![rsx!(
            div { class: "dashboard-metrics",
                div { class: "dashboard-metric-card",
                    Icon { width: 28, height: 28, fill: "#1976d2", icon: MdLibraryBooks }
                    div { class: "dashboard-metric-label", "Total Courses" }
                    div { class: "dashboard-metric-value", "{total_courses.get_value().round() as usize}" }
                }
                div { class: "dashboard-metric-card",
                    Icon { width: 28, height: 28, fill: "#43a047", icon: MdCheckCircle }
                    div { class: "dashboard-metric-label", "Structured" }
                    div { class: "dashboard-metric-value", "{structured_courses.get_value().round() as usize}" }
                }
                div { class: "dashboard-metric-card",
                    Icon { width: 28, height: 28, fill: "#fbc02d", icon: MdMovie }
                    div { class: "dashboard-metric-label", "Total Videos" }
                    div { class: "dashboard-metric-value", "{total_videos.get_value().round() as usize}" }
                }
            }
        )]
    } else {
        vec![]
    };

    // Prepare empty state
    let empty_state = if filtered_courses().len() == 0 && app_state.read().courses.is_empty() {
        vec![rsx!(
            div { class: "dashboard-empty-card",
                div { style: "font-size:2.5rem; margin-bottom:0.5rem;", "📚" }
                h2 { "Welcome to Course Pilot!" }
                p { "Start your learning journey by adding your first course." }
                Button {
                    onclick: move |_| {
                        app_state.write().current_route = Route::AddCourse;
                    },
                    "Add Your First Course"
                }
            }
        )]
    } else if filtered_courses().len() == 0 {
        vec![rsx!(
            div { class: "dashboard-empty-card",
                div { style: "font-size:2.2rem; margin-bottom:0.5rem;", "🔍" }
                h3 { "No courses found" }
                p { "Try adjusting your search terms or filters." }
            }
        )]
    } else {
        vec![]
    };

    // Prepare course grid
    let course_grid: Vec<_> = if filtered_courses().len() > 0 {
        let cards: Vec<_> = filtered_courses()
            .into_iter()
            .map(|course| rsx! { CourseCard { course: course } })
            .collect();
        vec![rsx!(div {
            class: "course-grid",
            { cards.into_iter() }
        })]
    } else {
        vec![]
    };

    rsx! {
    document::Link {
        rel: "stylesheet",
        href: asset!("src/ui/course_dashboard/style.css")
    }
    div { class: "course-dashboard",
        { metrics.into_iter() }
        // Search/filter bar
        div { class: "dashboard-search-row",
            input {
                class: "dashboard-search-input",
                r#type: "search",
                placeholder: "Search courses...",
                value: search_query(),
                oninput: move |evt| search_query.set(evt.value()),
                aria_label: "Search courses"
            }
            // Structured/unstructured/all toggle
            div { class: "dashboard-filter-toggle",
                label {
                    input {
                        r#type: "radio",
                        name: "filter",
                        checked: !*filter_structured.read(),
                        onchange: move |_| filter_structured.set(false),
                        aria_label: "Show all courses"
                    }
                    "All"
                }
                label {
                    input {
                        r#type: "radio",
                        name: "filter",
                        checked: *filter_structured.read(),
                        onchange: move |_| filter_structured.set(true),
                        aria_label: "Show only structured courses"
                    }
                    "Structured"
                }
            }
        }
            div { class: "dashboard-search-row",
                input {
                    class: "dashboard-search-input",
                    r#type: "search",
                    placeholder: "Search courses...",
                    value: search_query(),
                    oninput: move |evt| search_query.set(evt.value()),
                    aria_label: "Search courses"
                }
                div { class: "dashboard-filter-toggle",
                    label {
                        input {
                            r#type: "radio",
                            name: "filter",
                            checked: !*filter_structured.read(),
                            onchange: move |_| filter_structured.set(false),
                            aria_label: "Show all courses"
                        }
                        "All"
                    }
                    label {
                        input {
                            r#type: "radio",
                            name: "filter",
                            checked: *filter_structured.read(),
                            onchange: move |_| filter_structured.set(true),
                            aria_label: "Show only structured courses"
                        }
                        "Structured"
                    }
                }
            }
            { empty_state.into_iter() }
            { course_grid.into_iter() }
        }
    }
}
