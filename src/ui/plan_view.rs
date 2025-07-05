use crate::nlp::structure_course;
use crate::planner::generate_plan;
use crate::types::{AppState, Course, Plan, PlanSettings, Route};
use crate::ui::Button;
use crate::ui::Input;
use crate::ui::components::alert_dialog::{AlertDialogContent, AlertDialogRoot};
use crate::ui::components::skeleton::SkeletonLoader;
use crate::ui::components::{Checkbox, Label, Progress};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use dioxus_toast::ToastInfo;
use dioxus_toast::ToastManager;
use freyr::prelude::*;
use uuid::Uuid;

/// Plan view component for displaying and managing course plans
#[component]
pub fn PlanView(course_id: Uuid) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut course = use_signal(|| Option::<Course>::None);
    let mut plan = use_signal(|| Option::<Plan>::None);
    let mut is_loading = use_signal(|| true);
    let mut is_structuring = use_signal(|| false);
    let mut is_planning = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);
    let mut show_plan_settings = use_signal(|| false);

    // ToastManager from context
    let mut toast: Signal<ToastManager> = use_context();

    let mut sessions_per_week = use_signal(|| 3u8);
    let mut session_length_minutes = use_signal(|| 60u32);
    let mut include_weekends = use_signal(|| false);
    let mut start_date = use_signal(|| {
        let tomorrow = Utc::now() + chrono::Duration::days(1);
        tomorrow.format("%Y-%m-%d").to_string()
    });

    use_effect(move || {
        let course_id = course_id;
        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            let found_course = {
                let state = app_state.read();
                state.courses.iter().find(|c| c.id == course_id).cloned()
            };

            match found_course {
                Some(c) => {
                    course.set(Some(c.clone()));
                    plan.set(None);
                    is_loading.set(false);
                }
                None => {
                    error_message.set(Some("Course not found".to_string()));
                    is_loading.set(false);
                }
            }
        });
    });

    // Skeleton loader config for plan view
    let skeletons = rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 1rem; margin-top: 2rem;",
            SkeletonLoader {
                width: "100%".to_string(),
                height: "2.5rem".to_string(),
                border_radius: "8px".to_string(),
            }
            SkeletonLoader {
                width: "80%".to_string(),
                height: "1.5rem".to_string(),
                border_radius: "8px".to_string(),
            }
            SkeletonLoader {
                width: "60%".to_string(),
                height: "1.5rem".to_string(),
                border_radius: "8px".to_string(),
            }
            SkeletonLoader {
                width: "100%".to_string(),
                height: "3.5rem".to_string(),
                border_radius: "8px".to_string(),
            }
        }
    };

    // Render skeletons if loading or structuring, else render normal UI
    if *is_loading.read() || *is_structuring.read() {
        return skeletons;
    }

    // --- Plan item editing state initialization (moved outside rsx!) ---
    let current_plan_items_len = plan.read().as_ref().map(|p| p.items.len()).unwrap_or(0);
    let mut editing_title_vec = use_signal(|| vec![false; current_plan_items_len]);
    let mut editing_date_vec = use_signal(|| vec![false; current_plan_items_len]);
    let mut temp_title_vec = use_signal(|| {
        plan.read()
            .as_ref()
            .map(|p| {
                p.items
                    .iter()
                    .map(|item| item.section_title.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });
    let mut temp_date_vec = use_signal(|| {
        plan.read()
            .as_ref()
            .map(|p| {
                p.items
                    .iter()
                    .map(|item| item.date.format("%Y-%m-%d").to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });
    let scale_vec = use_signal(|| {
        plan.read()
            .as_ref()
            .map(|p| {
                p.items
                    .iter()
                    .map(|item| if item.completed { 0.97 } else { 1.0 })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    // Helper to reinitialize vectors when plan changes
    fn reinit_plan_edit_vectors(
        plan: &Option<Plan>,
        editing_title_vec: &mut Signal<Vec<bool>>,
        editing_date_vec: &mut Signal<Vec<bool>>,
        temp_title_vec: &mut Signal<Vec<String>>,
        temp_date_vec: &mut Signal<Vec<String>>,
    ) {
        let len = plan.as_ref().map(|p| p.items.len()).unwrap_or(0);
        editing_title_vec.set(vec![false; len]);
        editing_date_vec.set(vec![false; len]);
        temp_title_vec.set(
            plan.as_ref()
                .map(|p| {
                    p.items
                        .iter()
                        .map(|item| item.section_title.clone())
                        .collect()
                })
                .unwrap_or_default(),
        );
        temp_date_vec.set(
            plan.as_ref()
                .map(|p| {
                    p.items
                        .iter()
                        .map(|item| item.date.format("%Y-%m-%d").to_string())
                        .collect()
                })
                .unwrap_or_default(),
        );
    }

    let go_back = {
        let mut app_state_back = app_state.clone();
        move |_| {
            app_state_back.write().current_route = Route::Dashboard;
        }
    };

    let structure_course_action = move |_| {
        if let Some(current_course) = course.read().clone() {
            spawn(async move {
                is_structuring.set(true);
                error_message.set(None);

                match structure_course(current_course.raw_titles.clone()) {
                    Ok(structure) => {
                        let mut updated_course = current_course;
                        updated_course.structure = Some(structure);

                        let course_index = app_state
                            .read()
                            .courses
                            .iter()
                            .position(|c| c.id == updated_course.id);

                        if let Some(index) = course_index {
                            app_state.write().courses[index] = updated_course.clone();
                        }

                        course.set(Some(updated_course));
                        success_message.set(Some("Course structured successfully!".to_string()));
                        // Reinitialize editing vectors after structure changes
                        reinit_plan_edit_vectors(
                            &plan.read(),
                            &mut editing_title_vec,
                            &mut editing_date_vec,
                            &mut temp_title_vec,
                            &mut temp_date_vec,
                        );
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to structure course: {}", e)));
                    }
                }

                is_structuring.set(false);
            });
        }
    };

    let generate_plan_action = move |_| {
        if let Some(current_course) = course.read().clone() {
            if current_course.structure.is_none() {
                error_message.set(Some("Please structure the course first".to_string()));
                return;
            }

            spawn(async move {
                is_planning.set(true);
                error_message.set(None);

                let start_date_str = start_date.read();
                let parsed_start_date = match start_date_str.parse::<chrono::NaiveDate>() {
                    Ok(date) => date.and_hms_opt(9, 0, 0).unwrap().and_utc(),
                    Err(_) => Utc::now() + chrono::Duration::days(1),
                };

                let settings = PlanSettings {
                    start_date: parsed_start_date,
                    sessions_per_week: *sessions_per_week.read(),
                    session_length_minutes: *session_length_minutes.read(),
                    include_weekends: *include_weekends.read(),
                };

                match generate_plan(&current_course, &settings) {
                    Ok(generated_plan) => {
                        plan.set(Some(generated_plan));
                        success_message.set(Some("Study plan generated successfully!".to_string()));
                        show_plan_settings.set(false);
                        // Reinitialize editing vectors after plan changes
                        reinit_plan_edit_vectors(
                            &plan.read(),
                            &mut editing_title_vec,
                            &mut editing_date_vec,
                            &mut temp_title_vec,
                            &mut temp_date_vec,
                        );
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to generate plan: {}", e)));
                    }
                }

                is_planning.set(false);
            });
        }
    };

    let toggle_plan_settings = move |_| {
        let current_value = *show_plan_settings.read();
        show_plan_settings.set(!current_value);
    };

    if *is_loading.read() {
        return rsx! { div { "Loading course..." } };
    }

    // Loading overlay for async actions
    if *is_structuring.read() || *is_planning.read() {
        return rsx! {
            div {
                class: "planview-loading-overlay",
                div { class: "planview-spinner" }
                div { class: "planview-loading-msg",
                    if *is_structuring.read() { "Analyzing course structure..." }
                    else { "Generating study plan..." }
                }
            }
        };
    }

    let current_course = match course.read().as_ref() {
        Some(c) => c.clone(),
        None => {
            return rsx! { div { "Course not found" } };
        }
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/checkbox/style.css"),
        }
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/progress/style.css"),
        }
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/label/style.css"),
        }
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/card/style.css"),
        }

        div {
            class: "plan-view",

            Button {
                onclick: go_back,
                "← Back to Dashboard"
            }

            FreyrCard {
                has_shadow: true,
                div {
                    h1 { "{current_course.name}" }
                    div { "📺 {current_course.video_count()} videos • Created {format_date(current_course.created_at)}" }
                }
                div {
                    if current_course.is_structured() {
                        span { "✓ STRUCTURED" }
                    } else {
                        span { "⏳ NEEDS STRUCTURE" }
                    }
                }
            }

            if let Some(error) = error_message.read().as_ref() {
                div { class: "error-message", "{error}" }
            }

            if let Some(success) = success_message.read().as_ref() {
                div { class: "success-message", "{success}" }
            }

            FreyrCard {
                has_shadow: true,
                div {
                    h2 { "Course Structure" }
                    if !current_course.is_structured() {
                        Button {
                            disabled: *is_structuring.read(),
                            onclick: structure_course_action,
                            if *is_structuring.read() {
                                "🔄 Analyzing..."
                            } else {
                                "🧠 Analyze Structure"
                            }
                        }
                    } else {
                        Button {
                            onclick: structure_course_action,
                            disabled: *is_structuring.read(),
                            "🔄 Re-analyze"
                        }
                    }
                }

                if *is_structuring.read() {
                    div { "Analyzing course structure..." }
                } else if let Some(structure) = &current_course.structure {
                    div {
                        div {
                            div {
                                div { "{structure.modules.len()}" }
                                div { "Modules" }
                            }
                            if let Some(duration) = structure.metadata.estimated_duration_hours {
                                div {
                                    div { "{duration:.1}h" }
                                    div { "Est. Duration" }
                                }
                            }
                            if let Some(difficulty) = &structure.metadata.difficulty_level {
                                div {
                                    div { "{difficulty}" }
                                    div { "Difficulty" }
                                }
                            }
                        }

                        div {
                            for (i, module) in structure.modules.iter().enumerate() {
                                div {
                                    key: "{i}",
                                    div {
                                        div { "📁 {module.title}" }
                                        div { "{module.sections.len()} sections" }
                                    }
                                    div {
                                        for (j, section) in module.sections.iter().enumerate() {
                                            div {
                                                key: "{j}",
                                                span { "{j + 1}." }
                                                span { "{section.title}" }
                                                if let Some(duration) = section.estimated_duration {
                                                    span { "{duration.as_secs() / 60}min" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div {
                        div { "🧠" }
                        h3 { "Course Structure Needed" }
                        p { "Analyze the course content to create a structured learning path with modules and sections." }
                    }
                }
            }

            // --- NESTED: Study Plan, Plan Settings, and Plan Rendering ---
            if current_course.is_structured() {
                FreyrCard {
                    has_shadow: true,
                    div {
                        h2 { "Study Plan" }
                        div {
                            Button {
                                onclick: toggle_plan_settings,
                                "⚙️ Settings"
                            }
                            Button {
                                disabled: *is_planning.read(),
                                onclick: generate_plan_action,
                                if *is_planning.read() {
                                    "📅 Generating..."
                                } else {
                                    "Generate Plan"
                                }
                            }
                        }
                    }
                }

                if *show_plan_settings.read() {
                    AlertDialogRoot {
                        open: true,
                        on_open_change: move |open| show_plan_settings.set(open),
                        AlertDialogContent {
                            div {
                                h3 { "Plan Settings" }
                                div {
                                    div {
                                        Label {
                                            class: "label",
                                            html_for: "sessions-per-week",
                                            "Sessions per week"
                                        }
                                        Input {
                                            id: "sessions-per-week",
                                            r#type: "number",
                                            min: "1",
                                            max: "14",
                                            value: "{sessions_per_week}",
                                            oninput: move |evt: FormEvent| {
                                                if let Ok(val) = evt.value().parse::<u8>() {
                                                    sessions_per_week.set(val);
                                                }
                                            }
                                        }
                                    }
                                    div {
                                        Label {
                                            class: "label",
                                            html_for: "session-length",
                                            "Session length (minutes)"
                                        }
                                        Input {
                                            id: "session-length",
                                            r#type: "number",
                                            min: "15",
                                            max: "180",
                                            value: "{session_length_minutes}",
                                            oninput: move |evt: FormEvent| {
                                                if let Ok(val) = evt.value().parse::<u32>() {
                                                    session_length_minutes.set(val);
                                                }
                                            }
                                        }
                                    }
                                    div {
                                        Label {
                                            class: "label",
                                            html_for: "start-date",
                                            "Start date"
                                        }
                                        Input {
                                            id: "start-date",
                                            r#type: "date",
                                            value: "{start_date}",
                                            oninput: move |evt: FormEvent| start_date.set(evt.value())
                                        }
                                    }
                                    div {
                                        Label {
                                            class: "label",
                                            html_for: "include-weekends",
                                            Checkbox {
                                                id: "include-weekends",
                                                class: "checkbox",
                                                name: "include-weekends",
                                                checked: *include_weekends.read(),
                                                onchange: move |evt: dioxus::events::FormEvent| {
                                                    let checked = evt.value().parse::<bool>().unwrap_or(false);
                                                    include_weekends.set(checked);
                                                },
                                            }
                                            "Include weekends"
                                        }
                                    }
                                }
                                div {
                                    Button { onclick: move |_| show_plan_settings.set(false), "Cancel" }
                                    Button { onclick: generate_plan_action, "Save & Generate" }
                                }
                            }
                        }
                    }
                }

                if *is_planning.read() {
                    div { "Generating plan..." }
                } else if let Some(current_plan) = plan.read().as_ref() {
                    div {
                        div {
                            div {
                                div { "{current_plan.total_sessions()}" }
                                div { "Total Sessions" }
                            }
                            div {
                                div { "{calculate_plan_duration_weeks(current_plan)}" }
                                div { "Weeks" }
                            }
                            div {
                                div { "{current_plan.completed_sessions()}" }
                                div { "Completed" }
                            }
                        }

                        div {
                            div { "Overall Progress" }
                            div { "{current_plan.progress_percentage():.1}%" }
                        }
                        Progress {
                            aria_label: "Overall Progress",
                            class: "progress",
                            value: current_plan.progress_percentage() as f64,
                        }

                        div {
                            for (i, item) in current_plan.items.iter().enumerate() {
                                // Animate plan item scale on completion
                                // Animate fade/scale for inline editing
                                div {
                                    key: "{i}",
                                    class: format!("plan-item {}", if item.completed { "completed" } else { "" }),
                                    style: format!(
                                        "transform: scale({}); transition: transform 0.18s cubic-bezier(0.4,0,0.2,1);",
                                        scale_vec.read().get(i).cloned().unwrap_or(1.0)
                                    ),
                                    div {
                                        // Completion toggle
                                        input {
                                            r#type: "checkbox",
                                            checked: item.completed,
                                            aria_label: "Mark as complete",
                                            tabindex: "0",
                                            onchange: {
                                                let mut plan = plan.clone();
                                                let mut toast = toast.clone();
                                                move |_| {
                                                    if let Some(ref mut p) = plan.write().as_mut() {
                                                        p.items[i].completed = !p.items[i].completed;
                                                        toast.write().popup(ToastInfo::success(
                                                            if p.items[i].completed { "Marked complete" } else { "Marked incomplete" },
                                                            "Plan Item"
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        if item.completed {
                                            span { "✅" }
                                        } else {
                                            span { "⭕" }
                                        }
                                    }
                                    div {
                                        // Inline title editing with fade/scale
                                        if editing_title_vec.read().get(i).cloned().unwrap_or(false) {
                                            input {
                                                value: temp_title_vec.read().get(i).cloned().unwrap_or_else(|| item.section_title.clone()),
                                                aria_label: "Edit section title",
                                                autofocus: true,
                                                style: "opacity: 1.0; transition: opacity 0.18s;",
                                                oninput: move |evt| {
                                                    let mut temp_title_vec = temp_title_vec.write();
                                                    temp_title_vec[i] = evt.value();
                                                },
                                                onkeydown: move |evt| {
                                                    use dioxus::events::Key;
                                                    if evt.key() == Key::Enter {
                                                        if let Some(ref mut p) = plan.write().as_mut() {
                                                            let new_title = temp_title_vec.read()[i].clone();
                                                            if new_title.trim().is_empty() {
                                                                toast.write().popup(ToastInfo::simple("Title cannot be empty"));
                                                            } else {
                                                                p.items[i].section_title = new_title;
                                                            }
                                                        }
                                                        let mut editing_title_vec = editing_title_vec.write();
                                                        editing_title_vec[i] = false;
                                                    } else if evt.key() == Key::Escape {
                                                        let mut editing_title_vec = editing_title_vec.write();
                                                        editing_title_vec[i] = false;
                                                    }
                                                },
                                                onblur: move |_| {
                                                    let mut editing_title_vec = editing_title_vec.write();
                                                    editing_title_vec[i] = false;
                                                },
                                            }
                                        } else {
                                            button {
                                                class: "plan-item-title-btn",
                                                aria_label: "Edit section title",
                                                tabindex: "0",
                                                style: "opacity: 1.0; transition: opacity 0.18s;",
                                                onclick: move |_| {
                                                    let mut editing_title_vec = editing_title_vec.write();
                                                    if i < editing_title_vec.len() {
                                                        editing_title_vec[i] = true;
                                                    }
                                                },
                                                {item.section_title.clone()}
                                            }
                                        }
                                        div { "📁 {item.module_title}" }
                                        // Inline date editing with fade/scale
                                        if editing_date_vec.read().get(i).cloned().unwrap_or(false) {
                                            input {
                                                r#type: "date",
                                                value: temp_date_vec.read().get(i).cloned().unwrap_or_else(|| item.date.format("%Y-%m-%d").to_string()),
                                                aria_label: "Edit session date",
                                                autofocus: true,
                                                style: "opacity: 1.0; transition: opacity 0.18s;",
                                                oninput: move |evt| {
                                                    let mut temp_date_vec = temp_date_vec.write();
                                                    temp_date_vec[i] = evt.value();
                                                },
                                                onkeydown: move |evt| {
                                                    use dioxus::events::Key;
                                                    if evt.key() == Key::Enter {
                                                        if let Some(ref mut p) = plan.write().as_mut() {
                                                            if let Ok(new_date) = chrono::NaiveDate::parse_from_str(&temp_date_vec.read()[i], "%Y-%m-%d") {
                                                                let dt = chrono::DateTime::<Utc>::from_utc(new_date.and_hms_opt(0,0,0).unwrap(), Utc);
                                                                p.items[i].date = dt;
                                                            } else {
                                                                toast.write().popup(ToastInfo::simple("Invalid date"));
                                                            }
                                                        }
                                                        let mut editing_date_vec = editing_date_vec.write();
                                                        editing_date_vec[i] = false;
                                                    } else if evt.key() == Key::Escape {
                                                        let mut editing_date_vec = editing_date_vec.write();
                                                        editing_date_vec[i] = false;
                                                    }
                                                },
                                                onblur: move |_| {
                                                    let mut editing_date_vec = editing_date_vec.write();
                                                    editing_date_vec[i] = false;
                                                },
                                            }
                                        } else {
                                            button {
                                                class: "plan-item-date-btn",
                                                aria_label: "Edit session date",
                                                tabindex: "0",
                                                style: "opacity: 1.0; transition: opacity 0.18s;",
                                                onclick: move |_| {
                                                    let mut editing_date_vec = editing_date_vec.write();
                                                    if i < editing_date_vec.len() {
                                                        editing_date_vec[i] = true;
                                                    }
                                                },
                                                "{format_date(item.date)}"
                                            }
                                        }
                                        div { "{item.video_indices.len()} videos" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

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

fn calculate_plan_duration_weeks(plan: &Plan) -> usize {
    if plan.items.is_empty() {
        return 0;
    }

    let start_date = plan.items.first().unwrap().date;
    let end_date = plan.items.last().unwrap().date;
    let duration = end_date.signed_duration_since(start_date);

    std::cmp::max(1, (duration.num_days() / 7) as usize)
}
