//! Course view page - modules and videos

use dioxus::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;

use crate::application::{
    ServiceFactory,
    use_cases::{MoveVideoInput, PlanSessionInput, UpdateCourseInput, UpdateModuleTitleInput},
};
use crate::domain::entities::{Module, Tag, Video};
use crate::domain::ports::{CourseRepository, TagRepository, VideoRepository};
use crate::domain::value_objects::{CourseId, ModuleId, SessionPlan, TagId};
use crate::ui::Route;
use crate::ui::actions::export_course_notes_with_dialog;
use crate::ui::custom::{ErrorAlert, PageSkeleton, TagBadge, TagInput, VideoItem};
use crate::ui::hooks::{
    use_load_course_state, use_load_course_tags, use_load_modules_state, use_load_tags,
    use_load_videos_by_course_state,
};
use crate::ui::state::AppState;

/// Detailed course view with modules accordion.
#[component]
pub fn CourseView(course_id: String) -> Element {
    let state = use_context::<AppState>();
    let nav = use_navigator();

    {
        let mut state = state.clone();
        use_effect(move || {
            state.right_panel_visible.set(false);
            state.current_video_id.set(None);
        });
    }

    // Parse course ID
    let course_id_parsed = CourseId::from_str(&course_id);
    let course_id_effective = course_id_parsed.clone().unwrap_or_else(|_| CourseId::new());

    // Load course and modules
    let (course, course_state) = use_load_course_state(state.backend.clone(), &course_id_effective);

    let (modules, modules_state): (Signal<Vec<Module>>, _) =
        use_load_modules_state(state.backend.clone(), &course_id_effective);

    let (all_videos, videos_state): (Signal<Vec<Video>>, _) =
        use_load_videos_by_course_state(state.backend.clone(), &course_id_effective);

    let course_tags = use_load_course_tags(state.backend.clone(), &course_id_effective);

    let all_tags = use_load_tags(state.backend.clone());

    let total_videos = all_videos.read().len();
    let completed_videos = all_videos.read().iter().filter(|v| v.is_completed()).count();
    let progress = if total_videos > 0 {
        (completed_videos as f32 / total_videos as f32) * 100.0
    } else {
        0.0
    };

    if *course_state.is_loading.read() && course.read().is_none() {
        return rsx! {
            div { class: "p-6", PageSkeleton {} }
        };
    }

    // State for modals
    let mut show_delete_modal = use_signal(|| false);
    let mut show_session_modal = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);
    let is_exporting = use_signal(|| false);
    let export_status = use_signal(|| None::<(bool, String)>);
    let mut session_plans = use_signal(Vec::<SessionPlan>::new);
    let active_plan_day = use_signal(|| None::<u32>);
    let mut cognitive_limit = use_signal(|| 45u32);

    // Course editing state
    let mut edit_mode = use_signal(|| false);
    let mut edit_name = use_signal(String::new);
    let mut edit_description = use_signal(String::new);
    let edit_status = use_signal(|| None::<(bool, String)>);

    // Tag management state
    let mut selected_tag_id = use_signal(String::new);
    let tag_status = use_signal(|| None::<(bool, String)>);

    // Module boundary editing toggle
    let mut boundary_edit_mode = use_signal(|| false);

    // Sync edit fields with current course when not editing
    {
        let mut edit_name = edit_name;
        let mut edit_description = edit_description;
        let edit_mode = edit_mode;
        use_effect(move || {
            if *edit_mode.read() {
                return;
            }
            if let Some(c) = course.read().as_ref() {
                edit_name.set(c.name().to_string());
                edit_description.set(c.description().unwrap_or("").to_string());
            }
        });
    }

    // Delete course handler
    let backend_for_delete = state.backend.clone();
    let course_id_for_delete = course_id_parsed.clone();
    let on_delete_confirm = move |_| {
        if let Ok(ref cid) = course_id_for_delete {
            if let Some(ref ctx) = backend_for_delete {
                is_deleting.set(true);
                match ctx.course_repo.delete(cid) {
                    Ok(_) => {
                        log::info!("Course deleted successfully");
                        nav.push(Route::CourseList {});
                    },
                    Err(e) => {
                        log::error!("Failed to delete course: {}", e);
                        is_deleting.set(false);
                    },
                }
            }
        }
    };

    // Session planning handler
    let backend_for_session = state.backend.clone();
    let course_id_for_session = course_id_parsed.clone();
    let mut active_plan_day_for_session = active_plan_day;
    let on_plan_sessions = move |_| {
        if let Ok(ref cid) = course_id_for_session {
            if let Some(ref ctx) = backend_for_session {
                let use_case = ServiceFactory::plan_session(ctx);
                let input = PlanSessionInput {
                    course_id: cid.clone(),
                    cognitive_limit_minutes: *cognitive_limit.read(),
                };
                match use_case.execute(input) {
                    Ok(plans) => {
                        session_plans.set(plans);
                        active_plan_day_for_session.set(None);
                    },
                    Err(e) => log::error!("Failed to plan sessions: {}", e),
                }
            }
        }
    };

    // Export notes handler
    let backend_for_export = state.backend.clone();
    let course_id_for_export = course_id_parsed.clone();
    let mut export_status_for_export = export_status;
    let is_exporting_for_export = is_exporting;
    let on_export_notes = move |_| {
        if let Ok(ref cid) = course_id_for_export {
            let backend = backend_for_export.clone();
            let course_id = cid.clone();
            let mut export_status_for_export = export_status_for_export;
            let mut is_exporting_for_export = is_exporting_for_export;
            spawn(async move {
                is_exporting_for_export.set(true);
                match export_course_notes_with_dialog(backend, course_id).await {
                    Ok(path) => {
                        export_status_for_export
                            .set(Some((true, format!("Notes exported to {}", path))));
                    },
                    Err(e) => {
                        if e != "Save cancelled" {
                            export_status_for_export.set(Some((false, e)));
                        }
                    },
                }
                is_exporting_for_export.set(false);
            });
        } else {
            export_status_for_export.set(Some((false, "Invalid course ID".to_string())));
        }
    };

    // Course update handler
    let backend_for_update = state.backend.clone();
    let course_id_for_update = course_id_parsed.clone();
    let mut course_for_update = course;
    let edit_name_for_update = edit_name;
    let edit_description_for_update = edit_description;
    let mut edit_status_for_update = edit_status;
    let mut edit_mode_for_update = edit_mode;
    let on_save_course = move |_| {
        let name = edit_name_for_update.read().trim().to_string();
        if name.is_empty() {
            edit_status_for_update.set(Some((false, "Course name cannot be empty.".to_string())));

            return;
        }

        if let Ok(ref cid) = course_id_for_update {
            if let Some(ref ctx) = backend_for_update {
                let use_case = ServiceFactory::update_course(ctx);
                let description = {
                    let read_guard = edit_description_for_update.read();
                    let desc = read_guard.trim();
                    if desc.is_empty() { None } else { Some(desc.to_string()) }
                };
                let input =
                    UpdateCourseInput { course_id: cid.clone(), name: name.clone(), description };

                match use_case.execute(input) {
                    Ok(_) => {
                        edit_status_for_update.set(Some((true, "Course updated.".to_string())));
                        if let Ok(updated) = ctx.course_repo.find_by_id(cid) {
                            course_for_update.set(updated);
                        }
                        edit_mode_for_update.set(false);
                    },
                    Err(e) => {
                        edit_status_for_update
                            .set(Some((false, format!("Failed to update course: {}", e))));
                    },
                }
            }
        }
    };

    // Tag management handlers
    let backend_for_create_tag = state.backend.clone();
    let course_id_for_create_tag = course_id_parsed.clone();
    let mut course_tags_for_create = course_tags;
    let mut all_tags_for_create = all_tags;
    let mut tag_status_for_create = tag_status;
    let on_create_tag = move |name: String| {
        let trimmed = name.trim().to_string();
        if trimmed.is_empty() {
            tag_status_for_create.set(Some((false, "Tag name cannot be empty.".to_string())));
            return;
        }

        if let Ok(ref cid) = course_id_for_create_tag {
            if let Some(ref ctx) = backend_for_create_tag {
                let tag = Tag::new(TagId::new(), trimmed);
                if let Err(e) = ctx.tag_repo.save(&tag) {
                    tag_status_for_create
                        .set(Some((false, format!("Failed to create tag: {}", e))));
                    return;
                }
                if let Err(e) = ctx.tag_repo.add_to_course(cid, tag.id()) {
                    tag_status_for_create
                        .set(Some((false, format!("Failed to attach tag: {}", e))));
                    return;
                }
                if let Ok(updated) = ctx.tag_repo.find_by_course(cid) {
                    course_tags_for_create.set(updated);
                }
                if let Ok(all) = ctx.tag_repo.find_all() {
                    all_tags_for_create.set(all);
                }
                tag_status_for_create.set(Some((true, "Tag added.".to_string())));
            }
        }
    };

    let backend_for_attach_tag = state.backend.clone();
    let course_id_for_attach_tag = course_id_parsed.clone();
    let mut course_tags_for_attach = course_tags;
    let mut tag_status_for_attach = tag_status;
    let mut selected_tag_for_attach = selected_tag_id;
    let on_attach_tag = move |_| {
        let tag_id_value = selected_tag_for_attach.read().trim().to_string();
        if tag_id_value.is_empty() {
            tag_status_for_attach.set(Some((false, "Select a tag to add.".to_string())));
            return;
        }
        let tag_id = match TagId::from_str(&tag_id_value) {
            Ok(id) => id,
            Err(_) => {
                tag_status_for_attach.set(Some((false, "Invalid tag selection.".to_string())));
                return;
            },
        };

        if let Ok(ref cid) = course_id_for_attach_tag {
            if let Some(ref ctx) = backend_for_attach_tag {
                if let Err(e) = ctx.tag_repo.add_to_course(cid, &tag_id) {
                    tag_status_for_attach
                        .set(Some((false, format!("Failed to attach tag: {}", e))));
                    return;
                }
                if let Ok(updated) = ctx.tag_repo.find_by_course(cid) {
                    course_tags_for_attach.set(updated);
                }
                selected_tag_for_attach.set(String::new());
                tag_status_for_attach.set(Some((true, "Tag added.".to_string())));
            }
        }
    };

    let ordered_videos = all_videos.read().clone();

    rsx! {
        div { class: "p-6",

            if let Some(ref err) = *course_state.error.read() {
                ErrorAlert { message: err.clone(), on_dismiss: None }
            }
            if let Some(ref err) = *modules_state.error.read() {
                ErrorAlert { message: err.clone(), on_dismiss: None }
            }
            if let Some(ref err) = *videos_state.error.read() {
                ErrorAlert { message: err.clone(), on_dismiss: None }
            }

            if let Some((is_success, ref msg)) = *export_status.read() {
                div { class: if is_success { "alert alert-success mb-4" } else { "alert alert-error mb-4" },
                    "{msg}"
                }
            }

            // Back button and actions row
            div { class: "flex flex-wrap justify-between items-center mb-4 gap-3",
                Link { to: Route::CourseList {}, class: "btn btn-ghost btn-sm", "← Back to Courses" }

                div { class: "flex gap-2 flex-wrap",
                    button {
                        class: "btn btn-ghost btn-sm text-primary hover:bg-primary/10",
                        onclick: move |_| {
                            session_plans.set(Vec::new());
                            show_session_modal.set(true);
                        },
                        "📅 Plan Study Sessions"
                    }
                    button {
                        class: "btn btn-ghost btn-sm",
                        onclick: on_export_notes,
                        disabled: *is_exporting.read(),
                        if *is_exporting.read() {
                            span { class: "loading loading-spinner loading-sm" }
                        } else {
                            "📤 Export Notes"
                        }
                    }
                    button {
                        class: if *boundary_edit_mode.read() { "btn btn-outline btn-sm" } else { "btn btn-ghost btn-sm" },
                        onclick: move |_| {
                            let current = *boundary_edit_mode.read();
                            boundary_edit_mode.set(!current);
                        },
                        if *boundary_edit_mode.read() {
                            "✅ Editing Boundaries"
                        } else {
                            "✂️ Edit Boundaries"
                        }
                    }
                    button {
                        class: "btn btn-ghost btn-sm text-error hover:bg-error/10",
                        onclick: move |_| show_delete_modal.set(true),
                        "🗑️ Delete"
                    }
                }
            }

            // Course header
            if let Some(ref c) = *course.read() {
                div { class: "mb-4",

                    if *edit_mode.read() {
                        div { class: "space-y-3",
                            input {
                                class: "input input-bordered w-full",
                                r#type: "text",
                                placeholder: "Course name",
                                value: "{edit_name}",
                                oninput: move |e| edit_name.set(e.value()),
                            }
                            textarea {
                                class: "textarea textarea-bordered w-full",
                                placeholder: "Course description (optional)",
                                value: "{edit_description}",
                                oninput: move |e| edit_description.set(e.value()),
                            }
                            div { class: "flex gap-2",
                                button {
                                    class: "btn btn-primary btn-sm",
                                    onclick: on_save_course,
                                    "Save"
                                }
                                button {
                                    class: "btn btn-ghost btn-sm",
                                    onclick: {
                                        // Clone course data before the closure
                                        let course_name = c.name().to_string();
                                        let course_desc = c.description().unwrap_or("").to_string();
                                        move |_| {
                                            edit_mode.set(false);
                                            edit_name.set(course_name.clone());
                                            edit_description.set(course_desc.clone());
                                        }
                                    },
                                    "Cancel"
                                }
                            }
                        }
                    } else {
                        h1 { class: "text-2xl font-bold mb-2", "{c.name()}" }
                        if let Some(desc) = c.description() {
                            p { class: "text-base-content/70 mb-4", "{desc}" }
                        }
                        button {
                            class: "btn btn-ghost btn-sm",
                            onclick: move |_| edit_mode.set(true),
                            "✏️ Edit Course"
                        }
                    }

                    if let Some((is_success, ref msg)) = *edit_status.read() {
                        div { class: if is_success { "alert alert-success mt-4" } else { "alert alert-error mt-4" },
                            "{msg}"
                        }
                    }

                    div { class: "mt-4 bg-base-200 rounded-lg p-4 space-y-3",
                        div { class: "flex items-center justify-between",
                            span { class: "text-sm font-semibold", "Tags" }
                            TagInput { on_create: on_create_tag }
                        }

                        if !course_tags.read().is_empty() {
                            div { class: "flex flex-wrap gap-2",
                                for tag in course_tags.read().iter() {
                                    {
                                        let tag_id = tag.id().clone();
                                        let backend_clone = state.backend.clone();
                                        let course_id_clone = course_id_parsed.clone();
                                        let mut course_tags_clone = course_tags;
                                        let mut tag_status_clone = tag_status;
                                        rsx! {
                                            TagBadge {
                                                tag: tag.clone(),
                                                removable: true,
                                                on_remove: move |_| {
                                                    if let Ok(ref cid) = course_id_clone {
                                                        if let Some(ref ctx) = backend_clone {
                                                            if let Err(e) = ctx.tag_repo.remove_from_course(cid, &tag_id) {
                                                                tag_status_clone
                                                                    .set(Some((false, format!("Failed to remove tag: {}", e))));
                                                                return;
                                                            }
                                                            if let Ok(updated) = ctx.tag_repo.find_by_course(cid) {
                                                                course_tags_clone.set(updated);
                                                            }
                                                            tag_status_clone.set(Some((true, "Tag removed.".to_string())));
                                                        }
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "text-sm text-base-content/60", "No tags yet" }
                        }

                        div { class: "flex gap-2 items-center",
                            select {
                                class: "select select-bordered select-sm w-full",
                                value: "{selected_tag_id}",
                                oninput: move |e| selected_tag_id.set(e.value()),
                                option { value: "", "Select a tag to add" }
                                for tag in all_tags.read().iter() {
                                    if !course_tags.read().iter().any(|t| t.id() == tag.id()) {
                                        option { value: "{tag.id().as_uuid()}", "{tag.name()}" }
                                    }
                                }
                            }
                            button {
                                class: "btn btn-sm btn-outline",
                                onclick: on_attach_tag,
                                "Add Tag"
                            }
                        }

                        if let Some((is_success, ref msg)) = *tag_status.read() {
                            div { class: if is_success { "text-xs text-success" } else { "text-xs text-error" },
                                "{msg}"
                            }
                        }
                    }
                }
            } else {
                h1 { class: "text-2xl font-bold mb-2", "Course: {course_id}" }
            }

            // Progress bar
            div { class: "w-full max-w-md bg-base-300 rounded-full h-3 mb-6",
                div {
                    class: "bg-primary h-3 rounded-full transition-all",
                    style: "width: {progress}%",
                }
            }

            if let Some(last_video_id) = state.last_video_by_course.read().get(&course_id).cloned() {
                div { class: "mb-6",
                    Link {
                        to: Route::VideoPlayer {
                            course_id: course_id.clone(),
                            video_id: last_video_id,
                        },
                        class: "btn btn-primary btn-sm",
                        "▶ Resume last video"
                    }
                }
            }

            // Modules accordion
            div { class: "space-y-4",

                if modules.read().is_empty() {
                    div { class: "text-center py-8 bg-base-200 rounded-lg",
                        p { class: "text-base-content/60", "No modules found" }
                    }
                } else {
                    for module in modules.read().iter() {
                        ModuleAccordion {
                            course_id: course_id.clone(),
                            module: module.clone(),
                            all_modules: modules.read().clone(),
                            boundary_edit_mode: *boundary_edit_mode.read(),
                        }
                    }
                }
            }
        }

        // Delete confirmation modal
        if *show_delete_modal.read() {
            div {
                class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                onclick: move |_| show_delete_modal.set(false),

                div {
                    class: "bg-base-100 rounded-2xl p-6 max-w-md mx-4 shadow-2xl",
                    onclick: |e| e.stop_propagation(),

                    h3 { class: "text-xl font-bold mb-4", "Delete Course?" }
                    p { class: "text-base-content/70 mb-6",
                        "This will permanently delete this course, all its modules, videos, and any associated quizzes. This action cannot be undone."
                    }

                    div { class: "flex justify-end gap-3",
                        button {
                            class: "btn btn-ghost",
                            onclick: move |_| show_delete_modal.set(false),
                            "Cancel"
                        }
                        button {
                            class: "btn btn-error",
                            disabled: *is_deleting.read(),
                            onclick: on_delete_confirm,
                            if *is_deleting.read() {
                                span { class: "loading loading-spinner loading-sm" }
                            } else {
                                "Delete"
                            }
                        }
                    }
                }
            }
        }

        // Session planning modal
        if *show_session_modal.read() {
            div {
                class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                onclick: move |_| show_session_modal.set(false),

                div {
                    class: "bg-base-100 rounded-2xl p-6 max-w-2xl mx-4 shadow-2xl max-h-[80vh] overflow-y-auto",
                    onclick: |e| e.stop_propagation(),

                    h3 { class: "text-xl font-bold mb-4", "📅 Plan Your Study Sessions" }

                    // Cognitive limit slider
                    div { class: "mb-6",
                        label { class: "block text-sm font-medium mb-2",
                            "Daily study time: {cognitive_limit} minutes"
                        }
                        input {
                            r#type: "range",
                            class: "range range-primary w-full",
                            min: "15",
                            max: "120",
                            step: "5",
                            value: "{cognitive_limit}",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u32>() {
                                    cognitive_limit.set(val);
                                }
                            },
                        }
                        div { class: "flex justify-between text-xs text-base-content/50 mt-1",
                            span { "15 min" }
                            span { "45 min" }
                            span { "120 min" }
                        }
                    }

                    button {
                        class: "btn btn-primary w-full mb-4",
                        onclick: on_plan_sessions,
                        "Generate Study Plan"
                    }

                    // Session results
                    if !session_plans.read().is_empty() {
                        div { class: "space-y-3",
                            p { class: "text-sm text-base-content/70 mb-3",
                                "Estimated {session_plans.read().len()} days to complete:"
                            }
                            div { class: "flex flex-wrap items-center gap-2",
                                span { class: "text-xs text-base-content/60", "Jump to day:" }
                                {
                                    let mut active_plan_day_reset = active_plan_day;
                                    rsx! {
                                        button {
                                            class: if active_plan_day.read().is_none() { "btn btn-xs btn-primary" } else { "btn btn-xs btn-outline" },
                                            onclick: move |_| active_plan_day_reset.set(None),
                                            "All"
                                        }
                                    }
                                }
                                for plan in session_plans.read().iter() {
                                    {
                                        let day = plan.day;
                                        rsx! {
                                            button {
                                                class: if Some(day) == *active_plan_day.read() { "btn btn-xs btn-primary" } else { "btn btn-xs btn-outline" },
                                                onclick: move |_| {
                                                    let mut active_plan_day_for_jump = active_plan_day;
                                                    active_plan_day_for_jump.set(Some(day));
                                                },
                                                "Day {day}"
                                            }
                                        }
                                    }
                                }
                            }
                            for plan in session_plans
                                .read()
                                .iter()
                                .filter(|p| {
                                    let active_day = *active_plan_day.read();
                                    active_day.map(|day| p.day == day).unwrap_or(true)
                                })
                            {
                                div { class: "bg-base-200 rounded-xl p-4 space-y-2",
                                    div { class: "flex justify-between items-center",
                                        span { class: "font-bold", "Day {plan.day}" }
                                        span { class: "text-sm text-base-content/60",
                                            "{plan.total_duration_secs / 60} min"
                                        }
                                    }
                                    div { class: "text-sm text-base-content/70",
                                        "{plan.video_indices.len()} video(s)"
                                    }
                                    div { class: "divider my-2" }
                                    ul { class: "space-y-2",
                                        for idx in plan.video_indices.iter() {
                                            if let Some(video) = ordered_videos.get(*idx) {
                                                li { class: "flex justify-between text-sm",
                                                    span { class: "truncate", "{video.title()}" }
                                                    span { class: "text-base-content/60",
                                                        "{format_duration(video.duration_secs())}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "mt-6 flex justify-end",
                        button {
                            class: "btn btn-ghost",
                            onclick: move |_| show_session_modal.set(false),
                            "Close"
                        }
                    }
                }
            }
        }
    }
}

/// Module accordion with lazy-loaded videos and boundary editing controls.
#[component]
fn ModuleAccordion(
    course_id: String,
    module: Module,
    all_modules: Vec<Module>,
    boundary_edit_mode: bool,
) -> Element {
    let state = use_context::<AppState>();
    let backend_for_effect = state.backend.clone();
    let module_id = module.id().clone();
    let module_id_for_effect = module_id.clone();

    // Load videos for this module
    let mut videos = use_signal(Vec::new);

    use_effect(move || {
        if let Some(ref ctx) = backend_for_effect {
            if let Ok(loaded) = ctx.video_repo.find_by_module(&module_id_for_effect) {
                videos.set(loaded);
            }
        }
    });

    let mut is_editing_title = use_signal(|| false);
    let mut edit_title = use_signal(|| module.title().to_string());
    let edit_status = use_signal(|| None::<(bool, String)>);
    let move_status = use_signal(|| None::<(bool, String)>);

    let backend_for_title = state.backend.clone();
    let module_id_for_title = module_id.clone();
    let mut edit_status_for_title = edit_status;
    let on_save_title = move |_| {
        let title = edit_title.read().trim().to_string();
        if title.is_empty() {
            edit_status_for_title.set(Some((false, "Module title cannot be empty.".to_string())));
            return;
        }
        if let Some(ref ctx) = backend_for_title {
            let use_case = ServiceFactory::update_module_title(ctx);
            let input = UpdateModuleTitleInput { module_id: module_id_for_title.clone(), title };
            match use_case.execute(input) {
                Ok(_) => {
                    edit_status_for_title.set(Some((true, "Module updated.".to_string())));
                    is_editing_title.set(false);
                },
                Err(e) => {
                    edit_status_for_title
                        .set(Some((false, format!("Failed to update module: {e}"))));
                },
            }
        }
    };

    let move_targets: Signal<HashMap<String, String>> = use_signal(HashMap::new);

    // Clone module for use in multiple closures
    let module_for_cancel = module.clone();
    let module_for_loop = module.clone();

    rsx! {
        div { class: "collapse collapse-arrow bg-base-200",
            input { r#type: "checkbox" }
            div { class: "collapse-title font-medium flex items-center justify-between gap-2",

                if *is_editing_title.read() {
                    div { class: "flex-1 flex items-center gap-2",
                        input {
                            class: "input input-bordered input-sm w-full",
                            value: "{edit_title}",
                            oninput: move |e| edit_title.set(e.value()),
                        }
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: on_save_title,
                            "Save"
                        }
                        button {
                            class: "btn btn-ghost btn-sm",
                            onclick: move |_| {
                                is_editing_title.set(false);
                                edit_title.set(module_for_cancel.title().to_string());
                            },
                            "Cancel"
                        }
                    }
                } else {
                    div { class: "flex-1",
                        "{module.title()}"
                        span { class: "text-sm text-base-content/60 ml-2",
                            "({videos.read().len()} videos)"
                        }
                    }
                    if boundary_edit_mode {
                        button {
                            class: "btn btn-ghost btn-sm",
                            onclick: move |_| is_editing_title.set(true),
                            "✏️ Rename"
                        }
                    }
                }
            }
            div { class: "collapse-content",
                if let Some((is_success, ref msg)) = *edit_status.read() {
                    div { class: if is_success { "text-xs text-success mb-2" } else { "text-xs text-error mb-2" },
                        "{msg}"
                    }
                }
                if let Some((is_success, ref msg)) = *move_status.read() {
                    if is_success {
                        div { class: "text-xs text-success mb-2", "{msg}" }
                    } else {
                        ErrorAlert { message: msg.clone(), on_dismiss: None }
                    }
                }

                if videos.read().is_empty() {
                    p { class: "text-base-content/60 py-2", "No videos in this module" }
                } else {
                    div { class: "space-y-2",
                        {
                            let current_videos = videos.read().clone();
                            current_videos
                                .iter()
                                .map(|video| {
                                    let vid = video.id().clone();
                                    let vid_key = vid.as_uuid().to_string();
                                    let vid_key_for_oninput = vid_key.clone();
                                    let vid_key_for_onclick = vid_key.clone();
                                    let vid_for_onclick = vid.clone();
                                    let backend_for_move = state.backend.clone();
                                    let mut move_status_for_move = move_status;
                                    let mut move_targets_for_select = move_targets;
                                    let move_targets_for_click = move_targets;
                                    let module_id_for_filter = module_for_loop.id().clone();
                                    rsx! {
                                        div { class: "flex items-center gap-3",
                                            VideoItem {
                                                course_id: course_id.clone(),
                                                video_id: vid_key.clone(),
                                                title: video.title().to_string(),
                                                duration_secs: video.duration_secs(),
                                                is_completed: video.is_completed(),
                                            }
                                            if boundary_edit_mode {
                                                div { class: "flex items-center gap-2",
                                                    select {
                                                        class: "select select-bordered select-sm",
                                                        value: "{move_targets.read().get(&vid_key).cloned().unwrap_or_default()}",
                                                        oninput: move |e| {
                                                            let mut map = move_targets_for_select.write();
                                                            map.insert(vid_key_for_oninput.clone(), e.value());
                                                        },
                                                        option { value: "", "Move to..." }
                                                        for target in all_modules.iter() {
                                                            if target.id() != &module_id_for_filter {
                                                                option { value: "{target.id().as_uuid()}", "{target.title()}" }
                                                            }
                                                        }
                                                    }
                                                    button {
                                                        class: "btn btn-outline btn-sm",
                                                        onclick: move |_| {
                                                            if let Some(value) = move_targets_for_click
                                                                .read()
                                                                .get(&vid_key_for_onclick)
                                                                .cloned()
                                                            {
                                                                if let Ok(target_id) = ModuleId::from_str(&value) {
                                                                    if let Some(ref ctx) = backend_for_move {
                                                                        let use_case = ServiceFactory::move_video_to_module(ctx);
                                                                        let input = MoveVideoInput {
                                                                            video_id: vid_for_onclick.clone(),
                                                                            target_module_id: target_id,
                                                                            sort_order: 0,
                                                                        };
                                                                        match use_case.execute(input) {
                                                                            Ok(_) => {
                                                                                move_status_for_move
                                                                                    .set(Some((true, "Video moved successfully.".to_string())));
                                                                            }
                                                                            Err(e) => {
                                                                                log::error!("Failed to move video: {}", e);
                                                                                move_status_for_move
                                                                                    .set(Some((false, format!("Failed to move video: {}", e))));
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        "Move"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                })
                                .collect::<Vec<_>>()
                                .into_iter()
                        }
                    }
                }
            }
        }
    }
}

fn format_duration(secs: u32) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    if mins >= 60 {
        let hours = mins / 60;
        let mins = mins % 60;
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}
