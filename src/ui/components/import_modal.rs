use crate::ui::components::{
    modal::Modal, tabs::Tabs, toast,
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

/// YouTube playlist preview data
#[derive(Debug, Clone, PartialEq)]
pub struct YouTubePlaylistPreview {
    pub title: String,
    pub video_count: usize,
    pub total_duration: std::time::Duration,
    pub videos: Vec<YouTubeVideoPreview>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YouTubeVideoPreview {
    pub title: String,
    pub duration: std::time::Duration,
    pub index: usize,
}

/// YouTube import form with URL validation and playlist preview
#[component]
fn YouTubeImportForm(
    /// Callback when import is completed successfully
    on_import_complete: EventHandler<crate::types::Course>,
    /// Callback when import fails
    #[props(optional)]
    on_import_error: Option<EventHandler<String>>,
) -> Element {
    let backend = crate::ui::backend_adapter::use_backend_adapter();

    // Load settings and initialize API key from storage
    let settings = use_resource(|| async { crate::storage::AppSettings::load().unwrap_or_default() });

    // Form state
    let url = use_signal(String::new);
    let api_key = use_signal(|| {
        settings
            .read()
            .as_ref()
            .and_then(|s| s.get_youtube_api_key().map(|k| k.to_string()))
            .unwrap_or_default()
    });

    // Validation and preview state
    let validation_error = use_signal(|| Option::<String>::None);
    let is_validating = use_signal(|| false);
    let preview = use_signal(|| Option::<YouTubePlaylistPreview>::None);
    let is_loading_preview = use_signal(|| false);

    // Import progress state
    let import_job = use_signal(|| Option::<crate::types::ImportJob>::None);

    // Handle URL input changes with validation
    let mut handle_url_change = {
        let mut url = url;
        let mut validation_error = validation_error;
        let mut is_validating = is_validating;
        let mut preview = preview;
        let mut is_loading_preview = is_loading_preview;
        let api_key = api_key;

        move |new_url: String| {
            url.set(new_url.clone());
            validation_error.set(None);
            preview.set(None);

            if new_url.trim().is_empty() {
                return;
            }

            // Basic URL format validation
            if !new_url.contains("youtube.com")
                || (!new_url.contains("playlist") && !new_url.contains("list="))
            {
                validation_error.set(Some(
                    "Please enter a valid YouTube playlist URL".to_string(),
                ));
                return;
            }

            // If we have an API key, validate the playlist and load preview
            if !api_key().trim().is_empty() {
                is_validating.set(true);
                is_loading_preview.set(true);

                let api_key_val = api_key();
                let mut validation_error = validation_error;
                let mut preview = preview;
                let mut is_validating = is_validating;
                let mut is_loading_preview = is_loading_preview;

                spawn(async move {
                    // First validate the playlist exists
                    match crate::ingest::youtube::validate_playlist_url(&new_url, &api_key_val).await {
                        Ok(true) => {
                            // Load preview data
                            match crate::ingest::youtube::import_from_youtube(&new_url, &api_key_val).await {
                                Ok((sections, metadata)) => {
                                    let total_duration =
                                        sections.iter().map(|s| s.duration).sum::<std::time::Duration>();

                                    let videos = sections
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, section)| YouTubeVideoPreview {
                                            title: section.title,
                                            duration: section.duration,
                                            index: idx,
                                        })
                                        .collect::<Vec<_>>();

                                    let preview_data = YouTubePlaylistPreview {
                                        title: metadata.title,
                                        video_count: metadata.video_count,
                                        total_duration,
                                        videos,
                                    };

                                    preview.set(Some(preview_data));
                                }
                                Err(crate::ImportError::Network(msg)) => {
                                    validation_error.set(Some(format!("Network error: {msg}")));
                                }
                                Err(crate::ImportError::InvalidUrl(msg)) => {
                                    validation_error.set(Some(msg));
                                }
                                Err(crate::ImportError::NoContent) => {
                                    validation_error.set(Some(
                                        "Playlist is empty or contains no accessible videos"
                                            .to_string(),
                                    ));
                                }
                                Err(e) => {
                                    validation_error
                                        .set(Some(format!("Failed to load playlist: {e}")));
                                }
                            }
                        }
                        Ok(false) => {
                            validation_error.set(Some("Playlist not found or not accessible. Please check the URL and ensure the playlist is public or unlisted.".to_string()));
                        }
                        Err(crate::ImportError::Network(msg)) => {
                            validation_error.set(Some(format!("Network error: {msg}")));
                        }
                        Err(crate::ImportError::InvalidUrl(msg)) => {
                            validation_error.set(Some(msg));
                        }
                        Err(e) => {
                            validation_error.set(Some(format!("Validation error: {e}")));
                        }
                    }

                    is_validating.set(false);
                    is_loading_preview.set(false);
                });
            }
        }
    };

    // Handle API key changes
    let mut handle_api_key_change = {
        let mut api_key = api_key;
        let url = url;
        let mut handle_url_change = handle_url_change;

        move |new_api_key: String| {
            api_key.set(new_api_key.clone());

            // Save API key to settings asynchronously
            spawn(async move {
                if let Ok(mut settings) = crate::storage::AppSettings::load() {
                    let api_key_to_save = if new_api_key.trim().is_empty() {
                        None
                    } else {
                        Some(new_api_key.trim().to_string())
                    };

                    if let Err(e) = settings.set_youtube_api_key(api_key_to_save) {
                        log::error!("Failed to save API key: {e}");
                        toast::toast::error("Failed to save API key settings");
                    } else {
                        log::info!("YouTube API key saved to settings");
                    }
                }
            });

            // Re-validate URL if we have one
            if !url().trim().is_empty() {
                handle_url_change(url());
            }
        }
    };

    // Handle import
    let handle_import = {
        let backend = backend.clone();
        let import_job = import_job;
        let on_import_complete = on_import_complete;
        let on_import_error = on_import_error;
        let url = url;
        let api_key = api_key;
        let preview = preview;

        move |_| {
            let url_val = url().trim().to_string();
            let api_key_val = api_key().trim().to_string();

            if url_val.is_empty() || api_key_val.is_empty() {
                toast::toast::error("Please provide both URL and API key");
                return;
            }

            let _course_name = if let Some(preview_data) = preview() {
                preview_data.title
            } else {
                extract_course_name_from_url(&url_val)
            };

            let backend = backend.clone();
            let mut import_job = import_job;
            let on_import_complete = on_import_complete;
            let on_import_error = on_import_error;

            spawn(async move {
                // Create initial import job
                let job = crate::types::ImportJob::new("Starting import from YouTube playlist".to_string());
                import_job.set(Some(job.clone()));

                // Progress callback
                let mut progress_callback = {
                    let mut import_job = import_job;
                    move |percentage: f32, message: String| {
                        if let Some(mut job) = import_job() {
                            job.update_progress(percentage, message);
                            import_job.set(Some(job));
                        }
                    }
                };

                // Perform the import
                progress_callback(10.0, "Fetching playlist data...".to_string());

                match crate::ingest::youtube::import_from_youtube(&url_val, &api_key_val).await {
                    Ok((sections, metadata)) => {
                        progress_callback(40.0, "Processing video data...".to_string());

                        // Convert to course
                        let raw_titles: Vec<String> =
                            sections.iter().map(|s| s.title.clone()).collect();
                        let course_name = metadata.title;
                        let mut course = crate::types::Course::new(course_name, raw_titles);

                        // Structure the course using NLP
                        progress_callback(70.0, "Analyzing course structure...".to_string());
                        match crate::nlp::structure_course(course.raw_titles.clone()) {
                            Ok(course_structure) => {
                                course.structure = Some(course_structure);
                                progress_callback(90.0, "Saving course...".to_string());

                                // Save to database
                                match backend.create_course(course.clone()).await {
                                    Ok(_) => {
                                        progress_callback(
                                            100.0,
                                            "Import completed successfully!".to_string(),
                                        );
                                        toast::toast::success("Course imported successfully!");
                                        on_import_complete.call(course);
                                    }
                                    Err(e) => {
                                        let error_msg = format!("Failed to save course: {e}");
                                        if let Some(mut job) = import_job() {
                                            job.mark_failed(error_msg.clone());
                                            import_job.set(Some(job));
                                        }
                                        toast::toast::error("Failed to save course");
                                        if let Some(on_error) = on_import_error {
                                            on_error.call(error_msg);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to structure course: {e}");
                                if let Some(mut job) = import_job() {
                                    job.mark_failed(error_msg.clone());
                                    import_job.set(Some(job));
                                }
                                toast::toast::error("Failed to structure course");
                                if let Some(on_error) = on_import_error {
                                    on_error.call(error_msg);
                                }
                            }
                        }
                    }
                    Err(crate::ImportError::Network(msg)) => {
                        let error_msg = format!("Network error: {msg}");
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error(
                            "Network error occurred. Please check your connection and try again.",
                        );
                        if let Some(on_error) = on_import_error {
                            on_error.call(error_msg);
                        }
                    }
                    Err(crate::ImportError::InvalidUrl(msg)) => {
                        let error_msg = format!("Invalid URL: {msg}");
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error(
                            "Invalid playlist URL. Please check the URL and try again.",
                        );
                        if let Some(on_error) = on_import_error {
                            on_error.call(error_msg);
                        }
                    }
                    Err(crate::ImportError::NoContent) => {
                        let error_msg = "No accessible content found in playlist".to_string();
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error("Playlist is empty or contains no accessible videos.");
                        if let Some(on_error) = on_import_error {
                            on_error.call(error_msg);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Import failed: {e}");
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error(format!("Import failed: {e}"));
                        if let Some(on_error) = on_import_error {
                            on_error.call(error_msg);
                        }
                    }
                }
            });
        }
    };

    // Check if form is valid for import
    let is_valid_for_import = !url().trim().is_empty()
        && !api_key().trim().is_empty()
        && validation_error().is_none()
        && preview().is_some();

    let is_importing = import_job().is_some_and(|job| {
        matches!(
            job.status,
            crate::types::ImportStatus::Starting | crate::types::ImportStatus::InProgress
        )
    });

    rsx! {
        div { class: "space-y-6",
            // API Key input
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "YouTube Data API Key" }
                    span { class: "label-text-alt",
                        a {
                            href: "https://developers.google.com/youtube/v3/getting-started",
                            target: "_blank",
                            class: "link link-primary text-xs",
                            "Get API Key"
                        }
                    }
                }
                input {
                    r#type: "password",
                    placeholder: "Enter your YouTube Data API v3 key",
                    class: "input input-bordered w-full",
                    value: api_key(),
                    oninput: move |evt| handle_api_key_change(evt.value()),
                    disabled: is_importing,
                }
                label { class: "label",
                    span { class: "label-text-alt text-base-content/70",
                        "Required for accessing YouTube playlist data. Your key is stored locally and never shared."
                    }
                }
            }

            // URL input
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium", "YouTube Playlist URL" }
                }
                div { class: "relative",
                    input {
                        r#type: "url",
                        placeholder: "https://www.youtube.com/playlist?list=...",
                        class: format!("input input-bordered w-full pr-10 {}",
                            if validation_error().is_some() { "input-error" } else { "" }
                        ),
                        value: url(),
                        oninput: move |evt| handle_url_change(evt.value()),
                        disabled: is_importing,
                    }
                    if is_validating() {
                        div { class: "absolute right-3 top-1/2 transform -translate-y-1/2",
                            span { class: "loading loading-spinner loading-sm" }
                        }
                    }
                }
                label { class: "label",
                    span { class: "label-text-alt text-base-content/70",
                        "Enter a YouTube playlist URL to import videos as course content"
                    }
                }
            }

            // Validation feedback
            if let Some(error) = validation_error() {
                div { class: "alert alert-error",
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
                    span { "{error}" }
                }
            }

            // Preview section
            if let Some(preview_data) = preview() {
                YouTubePlaylistPreviewPanel { preview: preview_data }
            } else if is_loading_preview() {
                div { class: "card bg-base-200",
                    div { class: "card-body",
                        div { class: "flex items-center gap-3",
                            span { class: "loading loading-spinner loading-md" }
                            span { "Loading playlist preview..." }
                        }
                    }
                }
            }

            // Import progress
            if let Some(job) = import_job() {
                YouTubeImportProgressPanel { job }
            }

            // Import button
            div { class: "flex justify-end",
                button {
                    class: "btn btn-primary",
                    disabled: !is_valid_for_import || is_importing,
                    onclick: handle_import,
                    if is_importing {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                        "Importing..."
                    } else {
                        "Import Course"
                    }
                }
            }
        }
    }
}

/// Playlist preview panel component
#[component]
fn YouTubePlaylistPreviewPanel(preview: YouTubePlaylistPreview) -> Element {
    let duration_text = {
        let hours = preview.total_duration.as_secs() / 3600;
        let minutes = (preview.total_duration.as_secs() % 3600) / 60;
        if hours > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{minutes}m")
        }
    };

    rsx! {
        div { class: "card bg-base-200 border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    svg {
                        class: "w-5 h-5 text-red-500",
                        fill: "currentColor",
                        view_box: "0 0 24 24",
                        path { d: "M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" }
                    }
                    "Playlist Preview"
                }

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
                                let duration_str = {
                                    let minutes = video.duration.as_secs() / 60;
                                    let seconds = video.duration.as_secs() % 60;
                                    format!("{minutes}:{seconds:02}")
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

/// Import progress panel component
#[component]
fn YouTubeImportProgressPanel(job: crate::types::ImportJob) -> Element {
    let status_color = match job.status {
        crate::types::ImportStatus::Starting => "text-info",
        crate::types::ImportStatus::InProgress => "text-primary",
        crate::types::ImportStatus::Completed => "text-success",
        crate::types::ImportStatus::Failed => "text-error",
    };

    let status_text = match job.status {
        crate::types::ImportStatus::Starting => "Starting...",
        crate::types::ImportStatus::InProgress => "In Progress",
        crate::types::ImportStatus::Completed => "Completed",
        crate::types::ImportStatus::Failed => "Failed",
    };

    rsx! {
        div { class: "card bg-base-200 border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    if matches!(job.status, crate::types::ImportStatus::Starting | crate::types::ImportStatus::InProgress) {
                        span { class: "loading loading-spinner loading-sm" }
                    }
                    "Import Progress"
                }

                div { class: "space-y-3",
                    // Status and message
                    div { class: "flex justify-between items-center",
                        span { class: "font-medium {status_color}", "{status_text}" }
                        span { class: "text-sm text-base-content/70", "{job.progress_percentage:.1}%" }
                    }

                    // Progress bar
                    div { class: "w-full bg-base-300 rounded-full h-2",
                        div {
                            class: format!("h-2 rounded-full transition-all duration-300 {}",
                                match job.status {
                                    crate::types::ImportStatus::Completed => "bg-success",
                                    crate::types::ImportStatus::Failed => "bg-error",
                                    _ => "bg-primary",
                                }
                            ),
                            style: "width: {job.progress_percentage}%",
                        }
                    }

                    // Current message
                    div { class: "text-sm text-base-content/80",
                        "{job.message}"
                    }

                    // Error details for failed imports
                    if matches!(job.status, crate::types::ImportStatus::Failed) {
                        div { class: "alert alert-error",
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
                                div { class: "font-medium", "Import Failed" }
                                div { class: "text-sm", "Please check your API key and playlist URL, then try again." }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Extract a course name from a YouTube playlist URL
fn extract_course_name_from_url(url: &str) -> String {
    if let Some(playlist_id) = crate::ingest::youtube::extract_playlist_id(url) {
        format!(
            "YouTube Playlist {}",
            &playlist_id[..8.min(playlist_id.len())]
        )
    } else {
        "YouTube Course".to_string()
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
