use crate::ui::components::{
    modal::Modal, tabs::Tabs, toast, youtube_import_form::YouTubeImportForm,
};
use dioxus::prelude::*;

/// Import source types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImportSource {
    LocalFolder,
    YouTube,
    OtherResources,
}

impl ImportSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportSource::LocalFolder => "Local Course",
            ImportSource::YouTube => "YouTube",
            ImportSource::OtherResources => "Other Resources",
        }
    }
}

/// Import settings for customizing the import process
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportSettings {
    pub sort_by_title: bool,
    pub filter_duplicates: bool,
    pub include_metadata: bool,
    pub auto_structure: bool,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            sort_by_title: true,
            filter_duplicates: true,
            include_metadata: true,
            auto_structure: true,
        }
    }
}

/// Preview data for import content
#[derive(Debug, Clone, PartialEq)]
pub struct ImportPreview {
    pub title: String,
    pub video_count: usize,
    pub total_duration: Option<std::time::Duration>,
    pub videos: Vec<ImportVideoPreview>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportVideoPreview {
    pub title: String,
    pub duration: Option<std::time::Duration>,
    pub index: usize,
}

/// Props for the ImportModal component
#[derive(Props, PartialEq, Clone)]
pub struct ImportModalProps {
    /// Whether the modal is open
    pub open: bool,
    /// Callback when modal is closed
    pub on_close: EventHandler<()>,
    /// Callback when import is initiated
    pub on_import: EventHandler<(ImportSource, String, ImportSettings)>,
    /// Optional preview data to display
    #[props(optional)]
    pub preview: Option<ImportPreview>,
    /// Whether preview is loading
    #[props(default = false)]
    pub preview_loading: bool,
    /// Optional refresh callback for dashboard
    #[props(optional)]
    pub on_course_imported: Option<EventHandler<()>>,
}

/// Import source selection modal with tabs for YouTube and Local Folder
#[component]
pub fn ImportModal(props: ImportModalProps) -> Element {
    let mut selected_tab = use_signal(|| 0usize);
    let mut youtube_url = use_signal(String::new);
    let mut local_path = use_signal(String::new);
    let mut import_settings = use_signal(ImportSettings::default);
    let mut is_validating = use_signal(|| false);
    let mut folder_validation = use_signal(|| None::<crate::ui::backend_adapter::FolderValidation>);

    let backend = crate::ui::backend_adapter::use_backend_adapter();

    // Tab labels and sources
    let tab_labels = vec![
        "Local Course".to_string(),
        "YouTube".to_string(),
        "Other Resources".to_string(),
    ];
    let sources = [
        ImportSource::LocalFolder,
        ImportSource::YouTube,
        ImportSource::OtherResources,
    ];

    // Get current source
    let current_source = sources[selected_tab()];

    // Validation state
    let is_valid = match current_source {
        ImportSource::LocalFolder => {
            if let Some(validation) = folder_validation() {
                validation.is_valid
            } else {
                false
            }
        }
        ImportSource::YouTube => {
            !youtube_url().trim().is_empty() && youtube_url().contains("youtube.com")
        }
        ImportSource::OtherResources => false, // Always disabled for now
    };

    // Handle import action
    let handle_import = {
        let on_import = props.on_import;
        let on_close = props.on_close;
        let youtube_url = youtube_url;
        let local_path = local_path;
        let import_settings = import_settings;
        let backend = backend.clone();
        let folder_validation = folder_validation;

        move |_| {
            let source = sources[selected_tab()];
            match source {
                ImportSource::LocalFolder => {
                    let path = local_path().trim().to_string();
                    if !path.is_empty() {
                        if let Some(validation) = folder_validation() {
                            if validation.is_valid {
                                // Start local folder import
                                let backend = backend.clone();
                                let on_close = on_close;
                                let path = path.clone();
                                spawn(async move {
                                    toast::toast::info("Starting local folder import...");
                                    match backend
                                        .import_from_local_folder(
                                            std::path::Path::new(&path),
                                            None, // Let it auto-generate title from folder name
                                        )
                                        .await
                                    {
                                        Ok(course) => {
                                            toast::toast::success(format!(
                                                "Course '{}' imported successfully!",
                                                course.name
                                            ));
                                            on_close.call(());

                                            // Trigger dashboard refresh if callback provided
                                            if let Some(refresh_callback) =
                                                props.on_course_imported.as_ref()
                                            {
                                                refresh_callback.call(());
                                            }
                                        }
                                        Err(e) => {
                                            toast::toast::error(format!("Import failed: {e}"));
                                        }
                                    }
                                });
                            } else {
                                toast::toast::error(
                                    "Please select a valid folder with video files",
                                );
                            }
                        } else {
                            toast::toast::error("Please validate the folder first");
                        }
                    } else {
                        toast::toast::error("Please select a folder");
                    }
                }
                ImportSource::YouTube => {
                    let input = youtube_url().trim().to_string();
                    if !input.is_empty() {
                        on_import.call((source, input, import_settings()));
                    } else {
                        toast::toast::error("Please provide a valid YouTube URL");
                    }
                }
                ImportSource::OtherResources => {
                    toast::toast::info("Other resources import coming soon!");
                }
            }
        }
    };

    // Handle URL validation for YouTube
    let _handle_url_validation = {
        let mut youtube_url = youtube_url;
        let mut is_validating = is_validating;

        move |url: String| {
            youtube_url.set(url.clone());

            if !url.trim().is_empty() && url.contains("youtube.com") {
                is_validating.set(true);
                // In a real implementation, this would trigger preview loading
                spawn(async move {
                    // Simulate validation delay
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    is_validating.set(false);
                });
            }
        }
    };

    // Validate folder when path changes
    use_effect({
        let backend = backend.clone();
        let local_path = local_path;
        let mut folder_validation = folder_validation;
        let mut is_validating = is_validating;

        move || {
            let path = local_path();
            if !path.trim().is_empty() && current_source == ImportSource::LocalFolder {
                is_validating.set(true);
                let backend = backend.clone();
                let path = path.clone();
                spawn(async move {
                    match backend.validate_folder(std::path::Path::new(&path)).await {
                        Ok(validation) => {
                            folder_validation.set(Some(validation));
                        }
                        Err(e) => {
                            folder_validation.set(Some(
                                crate::ui::backend_adapter::FolderValidation {
                                    is_valid: false,
                                    video_count: 0,
                                    supported_files: Vec::new(),
                                    unsupported_files: Vec::new(),
                                    total_size: 0,
                                    error_message: Some(format!("Validation error: {e}")),
                                },
                            ));
                        }
                    }
                    is_validating.set(false);
                });
            } else {
                folder_validation.set(None);
            }
        }
    });

    // Reset form when modal closes
    use_effect(move || {
        if !props.open {
            youtube_url.set(String::new());
            local_path.set(String::new());
            selected_tab.set(0);
            is_validating.set(false);
            folder_validation.set(None);
        }
    });

    rsx! {
        Modal {
            variant: crate::ui::components::modal::form_modal(rsx! {
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| props.on_close.call(()),
                    "Cancel"
                }
                // Only show import button for non-YouTube tabs (YouTube has its own button)
                if current_source != ImportSource::YouTube {
                    button {
                        class: "btn btn-primary",
                        disabled: !is_valid || is_validating() || props.preview_loading,
                        onclick: handle_import,
                        if props.preview_loading {
                            span { class: "loading loading-spinner loading-sm mr-2" }
                        }
                        "Import Course"
                    }
                }
            }),
            open: props.open,
            on_close: props.on_close,
            title: "Import Course Content".to_string(),

            div { class: "space-y-4",
                // Source selection tabs
                Tabs {
                    tabs: tab_labels,
                    selected: selected_tab(),
                    on_select: move |idx| selected_tab.set(idx),
                    class: Some("tabs-boxed".to_string()),
                }

                // Tab content
                div { class: "min-h-[200px]",
                    match current_source {
                        ImportSource::LocalFolder => rsx! {
                            LocalFolderImportForm {
                                path: local_path(),
                                on_path_change: move |path| local_path.set(path),
                                preview: props.preview.clone(),
                                preview_loading: props.preview_loading,
                                folder_validation: folder_validation(),
                                is_validating: is_validating(),
                            }
                        },
                        ImportSource::YouTube => rsx! {
                            YouTubeImportFormWrapper {
                                on_import_complete: move |_course| {
                                    props.on_close.call(());
                                    if let Some(refresh_callback) = props.on_course_imported.as_ref() {
                                        refresh_callback.call(());
                                    }
                                },
                            }
                        },
                        ImportSource::OtherResources => rsx! {
                            OtherResourcesForm {}
                        },
                    }
                }

                // Import settings
                ImportSettingsPanel {
                    settings: import_settings(),
                    on_settings_change: move |settings| import_settings.set(settings),
                }
            }
        }
    }
}

/// YouTube import form wrapper component
#[component]
fn YouTubeImportFormWrapper(on_import_complete: EventHandler<crate::types::Course>) -> Element {
    rsx! {
        YouTubeImportForm {
            on_import_complete: on_import_complete,
        }
    }
}

/// Local folder import form component
#[component]
fn LocalFolderImportForm(
    path: String,
    on_path_change: EventHandler<String>,
    preview: Option<ImportPreview>,
    preview_loading: bool,
    folder_validation: Option<crate::ui::backend_adapter::FolderValidation>,
    is_validating: bool,
) -> Element {
    let backend = crate::ui::backend_adapter::use_backend_adapter();
    rsx! {
        div { class: "space-y-4",
            // Path input
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "Local Folder Path" }
                }
                div { class: "flex gap-2",
                    input {
                        r#type: "text",
                        placeholder: "/path/to/video/folder",
                        class: "input input-bordered flex-1",
                        value: path,
                        oninput: move |evt| on_path_change.call(evt.value()),
                    }
                    button {
                        class: "btn btn-outline",
                        onclick: {
                            let backend = backend.clone();
                            let on_path_change = on_path_change;
                            move |_| {
                                let backend = backend.clone();
                                let on_path_change = on_path_change;
                                spawn(async move {
                                    match backend.browse_folder().await {
                                        Ok(Some(folder_path)) => {
                                            if let Some(path_str) = folder_path.to_str() {
                                                on_path_change.call(path_str.to_string());
                                                toast::toast::success("Folder selected successfully!");
                                            } else {
                                                toast::toast::error("Invalid folder path selected");
                                            }
                                        },
                                        Ok(None) => {
                                            // User cancelled the dialog - no action needed
                                        },
                                        Err(e) => {
                                            toast::toast::error(format!("Failed to open folder dialog: {e}"));
                                        }
                                    }
                                });
                            }
                        },
                        "Browse"
                    }
                }
                label { class: "label",
                    span { class: "label-text-alt text-base-content/70",
                        "Select a folder containing video files to import as course content"
                    }
                }
            }

            // Supported formats info
            div { class: "alert alert-info",
                svg {
                    class: "stroke-current shrink-0 h-6 w-6",
                    fill: "none",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    }
                }
                div {
                    div { class: "font-medium", "Supported video formats:" }
                    div { class: "text-sm opacity-80", "MP4, AVI, MKV, MOV, WMV, FLV, WebM, M4V, MPG, MPEG" }
                }
            }

            // Preview section
            if let Some(preview_data) = preview {
                ImportPreviewPanel { preview: preview_data }
            } else if let Some(validation) = folder_validation {
                FolderValidationPanel { validation: validation }
            } else if is_validating {
                div { class: "card bg-base-200",
                    div { class: "card-body",
                        div { class: "flex items-center gap-3",
                            span { class: "loading loading-spinner loading-md" }
                            span { "Validating folder..." }
                        }
                    }
                }
            }
        }
    }
}

/// Import settings configuration panel
#[component]
fn ImportSettingsPanel(
    settings: ImportSettings,
    on_settings_change: EventHandler<ImportSettings>,
) -> Element {
    rsx! {
        div { class: "collapse collapse-arrow bg-base-200",
            input { r#type: "checkbox" }
            div { class: "collapse-title font-medium", "Import Settings" }
            div { class: "collapse-content space-y-3",
                div { class: "form-control",
                    label { class: "label cursor-pointer",
                        span { class: "label-text", "Sort videos by title" }
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-primary",
                            checked: settings.sort_by_title,
                            onchange: move |evt| {
                                let mut new_settings = settings;
                                new_settings.sort_by_title = evt.checked();
                                on_settings_change.call(new_settings);
                            },
                        }
                    }
                }

                div { class: "form-control",
                    label { class: "label cursor-pointer",
                        span { class: "label-text", "Filter duplicate videos" }
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-primary",
                            checked: settings.filter_duplicates,
                            onchange: move |evt| {
                                let mut new_settings = settings;
                                new_settings.filter_duplicates = evt.checked();
                                on_settings_change.call(new_settings);
                            },
                        }
                    }
                }

                div { class: "form-control",
                    label { class: "label cursor-pointer",
                        span { class: "label-text", "Include video metadata" }
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-primary",
                            checked: settings.include_metadata,
                            onchange: move |evt| {
                                let mut new_settings = settings;
                                new_settings.include_metadata = evt.checked();
                                on_settings_change.call(new_settings);
                            },
                        }
                    }
                }

                div { class: "form-control",
                    label { class: "label cursor-pointer",
                        span { class: "label-text", "Auto-structure course content" }
                        input {
                            r#type: "checkbox",
                            class: "checkbox checkbox-primary",
                            checked: settings.auto_structure,
                            onchange: move |evt| {
                                let mut new_settings = settings;
                                new_settings.auto_structure = evt.checked();
                                on_settings_change.call(new_settings);
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Preview panel showing import content details
#[component]
fn ImportPreviewPanel(preview: ImportPreview) -> Element {
    let duration_text = if let Some(duration) = preview.total_duration {
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        if hours > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{minutes}m")
        }
    } else {
        "Unknown".to_string()
    };

    rsx! {
        div { class: "card bg-base-200 border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg", "Import Preview" }

                // Summary stats
                div { class: "stats stats-horizontal shadow-sm bg-base-100 w-full",
                    div { class: "stat",
                        div { class: "stat-title", "Course Title" }
                        div { class: "stat-value text-base", "{preview.title}" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Videos" }
                        div { class: "stat-value text-primary", "{preview.video_count}" }
                    }
                    div { class: "stat",
                        div { class: "stat-title", "Duration" }
                        div { class: "stat-value text-secondary", "{duration_text}" }
                    }
                }

                // Video list preview (first 5 videos)
                if !preview.videos.is_empty() {
                    div { class: "mt-4",
                        h4 { class: "font-medium mb-2", "Video Preview:" }
                        div { class: "space-y-1 max-h-32 overflow-y-auto",
                            {preview.videos.iter().take(5).enumerate().map(|(idx, video)| {
                                let duration_str = if let Some(duration) = video.duration {
                                    let minutes = duration.as_secs() / 60;
                                    let seconds = duration.as_secs() % 60;
                                    format!("{minutes}:{seconds:02}")
                                } else {
                                    "Unknown".to_string()
                                };

                                rsx! {
                                    div {
                                        key: "{idx}",
                                        class: "flex justify-between items-center text-sm p-2 bg-base-100 rounded",
                                        span { class: "truncate flex-1 mr-2", "{video.title}" }
                                        span { class: "text-base-content/70 text-xs", "{duration_str}" }
                                    }
                                }
                            })}

                            if preview.videos.len() > 5 {
                                div { class: "text-center text-sm text-base-content/70 py-2",
                                    "... and {preview.videos.len() - 5} more videos"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Folder validation panel showing scan results
#[component]
fn FolderValidationPanel(validation: crate::ui::backend_adapter::FolderValidation) -> Element {
    let total_size_mb = (validation.total_size as f64) / (1024.0 * 1024.0);
    let size_text = if total_size_mb > 1024.0 {
        format!("{:.1} GB", total_size_mb / 1024.0)
    } else {
        format!("{total_size_mb:.1} MB")
    };

    rsx! {
        div { class: "card bg-base-200 border border-base-300",
            div { class: "card-body",
                if validation.is_valid {
                    h3 { class: "card-title text-lg text-success",
                        svg {
                            class: "w-5 h-5",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" }
                        }
                        "Folder Validation - Success"
                    }
                } else {
                    h3 { class: "card-title text-lg text-error",
                        svg {
                            class: "w-5 h-5",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                        }
                        "Folder Validation - Error"
                    }
                }

                if validation.is_valid {
                    // Success stats
                    div { class: "stats stats-horizontal shadow-sm bg-base-100 w-full mt-4",
                        div { class: "stat",
                            div { class: "stat-title", "Video Files" }
                            div { class: "stat-value text-primary", "{validation.video_count}" }
                        }
                        div { class: "stat",
                            div { class: "stat-title", "Total Size" }
                            div { class: "stat-value text-secondary", "{size_text}" }
                        }
                        if !validation.unsupported_files.is_empty() {
                            div { class: "stat",
                                div { class: "stat-title", "Unsupported" }
                                div { class: "stat-value text-warning", "{validation.unsupported_files.len()}" }
                            }
                        }
                    }

                    // File list preview (first 5 files)
                    if !validation.supported_files.is_empty() {
                        div { class: "mt-4",
                            h4 { class: "font-medium mb-2", "Video Files Found:" }
                            div { class: "space-y-1 max-h-32 overflow-y-auto",
                                {validation.supported_files.iter().take(5).enumerate().map(|(idx, file_path)| {
                                    let file_name = file_path.file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or("Unknown");

                                    rsx! {
                                        div {
                                            key: "{idx}",
                                            class: "flex justify-between items-center text-sm p-2 bg-base-100 rounded",
                                            span { class: "truncate flex-1 mr-2", "{file_name}" }
                                            span { class: "text-base-content/70 text-xs",
                                                {file_path.extension()
                                                    .and_then(|ext| ext.to_str())
                                                    .unwrap_or("")
                                                    .to_uppercase()
                                                }
                                            }
                                        }
                                    }
                                })}

                                if validation.supported_files.len() > 5 {
                                    div { class: "text-center text-sm text-base-content/70 py-2",
                                        "... and {validation.supported_files.len() - 5} more video files"
                                    }
                                }
                            }
                        }
                    }

                    // Unsupported files warning
                    if !validation.unsupported_files.is_empty() {
                        div { class: "alert alert-warning mt-4",
                            svg {
                                class: "stroke-current shrink-0 h-6 w-6",
                                fill: "none",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
                                }
                            }
                            div {
                                div { class: "font-medium", "{validation.unsupported_files.len()} unsupported files found" }
                                div { class: "text-sm opacity-80", "These files will be skipped during import" }
                            }
                        }
                    }
                } else {
                    // Error message
                    div { class: "alert alert-error mt-4",
                        svg {
                            class: "stroke-current shrink-0 h-6 w-6",
                            fill: "none",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                            }
                        }
                        div {
                            div { class: "font-medium", "Validation Failed" }
                            div { class: "text-sm opacity-80",
                                {validation.error_message.unwrap_or_else(|| "Unknown error occurred".to_string())}
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Other resources form component (to be implemented)
#[component]
fn OtherResourcesForm() -> Element {
    rsx! {
        div { class: "space-y-4",
            // Coming soon message
            div { class: "alert alert-info",
                svg {
                    class: "stroke-current shrink-0 h-6 w-6",
                    fill: "none",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    }
                }
                div {
                    div { class: "font-medium", "Coming Soon!" }
                    div { class: "text-sm opacity-80", "Support for additional course sources will be added in future updates." }
                }
            }

            // Placeholder content
            div { class: "card bg-base-200",
                div { class: "card-body text-center",
                    h3 { class: "card-title justify-center mb-4", "Additional Import Sources" }

                    div { class: "space-y-3 text-base-content/70",
                        div { class: "flex items-center gap-3 p-3 bg-base-100 rounded",
                            div { class: "w-8 h-8 bg-primary/20 rounded-full flex items-center justify-center",
                                svg {
                                    class: "w-4 h-4 text-primary",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path { d: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" }
                                }
                            }
                            div { class: "text-left",
                                div { class: "font-medium text-base-content", "Online Course Platforms" }
                                div { class: "text-sm", "Udemy, Coursera, edX, Khan Academy" }
                            }
                        }

                        div { class: "flex items-center gap-3 p-3 bg-base-100 rounded",
                            div { class: "w-8 h-8 bg-secondary/20 rounded-full flex items-center justify-center",
                                svg {
                                    class: "w-4 h-4 text-secondary",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path { d: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" }
                                }
                            }
                            div { class: "text-left",
                                div { class: "font-medium text-base-content", "Video Streaming Services" }
                                div { class: "text-sm", "Vimeo, Twitch, custom video URLs" }
                            }
                        }

                        div { class: "flex items-center gap-3 p-3 bg-base-100 rounded",
                            div { class: "w-8 h-8 bg-accent/20 rounded-full flex items-center justify-center",
                                svg {
                                    class: "w-4 h-4 text-accent",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path { d: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" }
                                }
                            }
                            div { class: "text-left",
                                div { class: "font-medium text-base-content", "Document & Text Sources" }
                                div { class: "text-sm", "PDFs, web articles, documentation sites" }
                            }
                        }
                    }

                    div { class: "mt-6 text-sm text-base-content/60",
                        "These import sources are planned for future releases. Stay tuned for updates!"
                    }
                }
            }
        }
    }
}
