// Add Course Dialog UI Component
//
// This component provides the interface for adding new courses from YouTube playlists
// or local folders, with real-time validation and progress tracking.

use crate::ingest::{import_from_local_folder, import_from_youtube};
use crate::types::{AppState, Course, ImportJob, Route};
use crate::ui::components::progress::Progress;
use crate::ui::components::radio_group::{RadioGroup, RadioItem};
use crate::ui::components::skeleton::SkeletonLoader;
use crate::ui::components::{Button, Input};
use crate::ui::navigation::{async_navigate_to, navigate_to_dashboard};
use dioxus::prelude::*;
use dioxus_toast::ToastManager;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

/// Source type for course import
#[derive(Debug, Clone, PartialEq)]
enum ImportSource {
    YouTube,
    LocalFolder,
}

/// Add course dialog component
#[component]
pub fn AddCourseDialog() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut course_name = use_signal(String::new);
    let mut import_source = use_signal(|| ImportSource::YouTube);
    let mut import_source_string = use_signal(|| "youtube".to_string());
    let mut youtube_url = use_signal(String::new);
    let mut selected_folder = use_signal(|| Option::<PathBuf>::None);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);
    let mut is_importing = use_signal(|| false);

    // ToastManager from context
    let _toast: Signal<ToastManager> = use_context();

    // Validation state
    let is_valid = use_memo(move || {
        let name = course_name.read();
        let source = import_source.read();

        if name.trim().is_empty() {
            return false;
        }

        match *source {
            ImportSource::YouTube => {
                let url = youtube_url.read();
                !url.trim().is_empty() && (url.contains("youtube.com") || url.contains("youtu.be"))
            }
            ImportSource::LocalFolder => selected_folder.read().is_some(),
        }
    });

    // Skeleton loading state for import/validation
    let show_skeleton = is_importing.read();

    // Render skeleton loader if importing/validating, else render normal dialog UI
    if *show_skeleton {
        return rsx! {
            div {
                style: "padding: 2rem; display: flex; flex-direction: column; gap: 1.5rem;",
                SkeletonLoader {
                    width: "100%".to_string(),
                    height: "2.5rem".to_string(),
                }
                SkeletonLoader {
                    width: "80%".to_string(),
                    height: "1.5rem".to_string(),
                }
                SkeletonLoader {
                    width: "60%".to_string(),
                    height: "1.5rem".to_string(),
                }
            }
        };
    }

    // Handle folder selection with native file picker
    let select_folder = move |_| {
        spawn(async move {
            if let Some(folder) = AsyncFileDialog::new()
                .set_title("Select Video Folder")
                .pick_folder()
                .await
            {
                selected_folder.set(Some(folder.path().to_path_buf()));
            }
        });
    };

    // Handle course import
    let import_course = move |_| {
        if !*is_valid.read() {
            error_message.set(Some("Please fill in all required fields".to_string()));
            return;
        }

        let name = course_name.read().clone();
        let source = import_source.read().clone();
        let url = youtube_url.read().clone();
        let folder = selected_folder.read().clone();

        spawn(async move {
            is_importing.set(true);
            error_message.set(None);
            success_message.set(None);

            let mut import_job = ImportJob::new(format!("Starting import of '{}'", name));
            app_state.write().active_import = Some(import_job.clone());

            let import_result = match source {
                ImportSource::YouTube => {
                    import_job.update_progress(25.0, "Connecting to YouTube...".to_string());
                    app_state.write().active_import = Some(import_job.clone());
                    import_from_youtube(&url).await
                }
                ImportSource::LocalFolder => {
                    if let Some(folder_path) = folder {
                        import_job
                            .update_progress(50.0, "Scanning folder for videos...".to_string());
                        app_state.write().active_import = Some(import_job.clone());
                        import_from_local_folder(&folder_path)
                    } else {
                        Err(crate::ImportError::FileSystem(
                            "No folder selected".to_string(),
                        ))
                    }
                }
            };

            match import_result {
                Ok(titles) => {
                    if titles.is_empty() {
                        import_job.mark_failed("No valid content found".to_string());
                        error_message
                            .set(Some("No videos found in the selected source".to_string()));
                    } else {
                        import_job.update_progress(
                            100.0,
                            format!("Successfully imported {} videos", titles.len()),
                        );

                        let course = Course::new(name.clone(), titles);
                        {
                            let mut state = app_state.write();
                            state.courses.push(course.clone());
                            state.active_import = None;
                        }

                        success_message.set(Some(format!(
                            "Successfully imported '{}' with {} videos",
                            name,
                            course.video_count()
                        )));

                        course_name.set(String::new());
                        youtube_url.set(String::new());
                        selected_folder.set(None);

                        let mut app_state_nav = app_state.clone();
                        spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            if let Err(e) = async_navigate_to(app_state_nav, Route::Dashboard) {
                                log::error!("Failed to navigate to dashboard: {:?}", e);
                            }
                        });
                    }
                }
                Err(e) => {
                    import_job.mark_failed(format!("Import failed: {}", e));
                    error_message.set(Some(format!("Import failed: {}", e)));
                    app_state.write().active_import = None;
                }
            }

            if app_state.read().active_import.is_some() {
                app_state.write().active_import = Some(import_job);
            }

            is_importing.set(false);
        });
    };

    let cancel_import = {
        let mut app_state_cancel = app_state.clone();
        move |_| {
            if let Err(e) = navigate_to_dashboard(app_state_cancel) {
                log::error!("Failed to navigate to dashboard: {:?}", e);
            }
        }
    };

    // Only one rsx! block allowed. Use fragments and Vec/Option for conditional UI.

    // Source-specific input
    let source_input = match *import_source.read() {
        ImportSource::YouTube => Some(rsx!(Input {
            label: Some("YouTube Playlist URL *".to_string()),
            placeholder: Some("https://www.youtube.com/playlist?list=...".to_string()),
            value: Some(youtube_url.read().clone()),
            disabled: *is_importing.read(),
            oninput: move |evt: FormEvent| {
                youtube_url.set(evt.value());
                error_message.set(None);
            },
        })),
        ImportSource::LocalFolder => {
            let folder_display = if let Some(path) = selected_folder.read().as_ref() {
                format!("{}", path.display())
            } else {
                "No folder selected".to_string()
            };
            Some(rsx!(
                div { class: "add-course-folder-row",
                    label { "Video Folder *" },
                    Button {
                        onclick: select_folder,
                        disabled: *is_importing.read(),
                        "📁 Select Folder"
                    },
                    span { "{folder_display}" }
                }
            ))
        }
    };

    // Feedback
    let error_feedback = error_message
        .read()
        .as_ref()
        .map(|error| rsx!(div { class: "error-message", {error.clone()} }));
    let success_feedback = success_message
        .read()
        .as_ref()
        .map(|success| rsx!(div { class: "success-message", {success.clone()} }));

    // Progress
    let progress_feedback = if *is_importing.read() {
        if let Some(import_job) = app_state.read().active_import.as_ref() {
            Some(rsx!(
                div { class: "add-course-progress",
                    div { {import_job.message.clone()} }
                    Progress {
                        aria_label: "Import Progress",
                        class: "progress",
                        value: import_job.progress_percentage
                    }
                    div { {format!("{:.1}% complete", import_job.progress_percentage)} }
                }
            ))
        } else {
            None
        }
    } else {
        None
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/add_course_dialog/style.css")
        }
        div { class: "add-course-dialog-root",
            Button {
                onclick: cancel_import,
                disabled: *is_importing.read(),
                "← Back to Dashboard"
            },
            div { class: "add-course-title", "Add New Course" },
            div { class: "add-course-card",
                div { class: "add-course-form-row",
                    Input {
                        label: "Course Name *",
                        placeholder: "Enter a name for this course",
                        value: Some(course_name.read().clone()),
                        disabled: *is_importing.read(),
                        oninput: move |evt: FormEvent| {
                            course_name.set(evt.value());
                            error_message.set(None);
                        }
                    },
                    div {
                        span { class: "add-course-label", "Import Source *" },
                        div { class: "add-course-radio-row",
                            RadioGroup {
                                name: "import-source".to_string(),
                                class: "radio-group",
                                value: import_source_string(),
                                onchange: move |value: String| {
                                    import_source_string.set(value.clone());
                                    match value.as_str() {
                                        "youtube" => import_source.set(ImportSource::YouTube),
                                        "folder" => import_source.set(ImportSource::LocalFolder),
                                        _ => {}
                                    }
                                },
                                RadioItem {
                                    class: "radio-item",
                                    value: "youtube".to_string(),
                                    index: 0,
                                    "YouTube"
                                },
                                RadioItem {
                                    class: "radio-item",
                                    value: "folder".to_string(),
                                    index: 1usize,
                                    "Local Folder"
                                },
                            }
                        }
                    },
                    { source_input },
                    { error_feedback },
                    { success_feedback },
                    { progress_feedback },
                },
                div { class: "add-course-actions",
                    Button {
                        onclick: cancel_import,
                        disabled: *is_importing.read(),
                        "Cancel"
                    },
                    Button {
                        onclick: import_course,
                        disabled: !*is_valid.read() || *is_importing.read(),
                        if *is_importing.read() {
                            "Importing..."
                        } else {
                            "Import"
                        }
                    },
                },
            },
        },
    }
}
