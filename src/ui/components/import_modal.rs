use crate::ui::components::{modal::Badge, modal::Modal, toast};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_brands_icons::FaYoutube;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCheck, FaCircleExclamation, FaCircleInfo, FaClock, FaFolder, FaGlobe, FaPlay, FaSpinner,
    FaVideo,
};

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
                    class: "btn btn-ghost btn-sm",
                    onclick: move |_| props.on_close.call(()),
                    "Cancel"
                }
                // Only show import button for non-YouTube tabs (YouTube has its own button)
                if current_source != ImportSource::YouTube {
                    button {
                        class: "btn btn-primary btn-sm",
                        disabled: !is_valid || is_validating() || props.preview_loading,
                        onclick: handle_import,
                        if props.preview_loading {
                            Icon { icon: FaSpinner, class: "w-4 h-4 mr-2 animate-spin" }
                        }
                        "Import Course"
                    }
                }
            }),
            open: props.open,
            on_close: props.on_close,
            title: "Import Course Content".to_string(),
            size: Some("lg".to_string()),

            div { class: "space-y-6",
                // Enhanced header with description
                div { class: "text-center pb-4 border-b border-base-300",
                    h2 { class: "text-xl font-semibold text-base-content mb-2", "Import Course Content" }
                    p { class: "text-sm text-base-content/70",
                        "Choose your content source and import videos to create a structured learning experience"
                    }
                }

                // Enhanced source selection tabs with icons
                div { class: "w-full",
                    div { class: "tabs tabs-boxed tabs-lg w-full justify-center bg-base-200 p-1",
                        {tab_labels.iter().enumerate().map(|(idx, label)| {
                            let is_selected = selected_tab() == idx;
                            let color_class = match idx {
                                0 => if is_selected { "text-primary" } else { "text-base-content/70" },
                                1 => if is_selected { "text-red-500" } else { "text-base-content/70" },
                                _ => if is_selected { "text-secondary" } else { "text-base-content/70" },
                            };

                            rsx! {
                                button {
                                    key: "{idx}",
                                    class: format!("tab tab-lg flex-1 gap-2 {}",
                                        if is_selected { "tab-active" } else { "" }
                                    ),
                                    onclick: move |_| selected_tab.set(idx),
                                    {match idx {
                                        0 => rsx! { Icon { icon: FaFolder, class: format!("w-4 h-4 {}", color_class) } },
                                        1 => rsx! { Icon { icon: FaYoutube, class: format!("w-4 h-4 {}", color_class) } },
                                        _ => rsx! { Icon { icon: FaGlobe, class: format!("w-4 h-4 {}", color_class) } },
                                    }}
                                    span { class: color_class, "{label}" }
                                }
                            }
                        })}
                    }
                }

                // Enhanced tab content with better spacing
                div { class: "min-h-[300px] bg-base-50 rounded-lg p-6",
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

                // Enhanced import settings with better styling
                if current_source != ImportSource::OtherResources {
                    ImportSettingsPanel {
                        settings: import_settings(),
                        on_settings_change: move |settings| import_settings.set(settings),
                    }
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
    let settings =
        use_resource(|| async { crate::storage::AppSettings::load().unwrap_or_default() });

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
                    match crate::ingest::youtube::validate_playlist_url(&new_url, &api_key_val)
                        .await
                    {
                        Ok(true) => {
                            // Load preview data
                            match crate::ingest::youtube::import_from_youtube(
                                &new_url,
                                &api_key_val,
                            )
                            .await
                            {
                                Ok((sections, metadata)) => {
                                    let total_duration = sections
                                        .iter()
                                        .map(|s| s.duration)
                                        .sum::<std::time::Duration>();

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
                let job = crate::types::ImportJob::new(
                    "Starting import from YouTube playlist".to_string(),
                );
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
            // Enhanced header section
            div { class: "flex items-center gap-3 mb-6",
                div { class: "w-10 h-10 bg-red-100 rounded-full flex items-center justify-center",
                    Icon { icon: FaYoutube, class: "w-5 h-5 text-red-500" }
                }
                div {
                    h3 { class: "text-lg font-semibold text-base-content", "YouTube Playlist Import" }
                    p { class: "text-sm text-base-content/70", "Import videos from a YouTube playlist" }
                }
            }

            // API Key input with enhanced styling
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium flex items-center gap-2",
                        Icon { icon: FaCircleInfo, class: "w-4 h-4 text-info" }
                        "YouTube Data API Key"
                    }
                    span { class: "label-text-alt",
                        a {
                            href: "https://developers.google.com/youtube/v3/getting-started",
                            target: "_blank",
                            class: "link link-primary text-xs hover:link-hover",
                            "Get API Key"
                        }
                    }
                }
                div { class: "relative",
                    input {
                        r#type: "password",
                        placeholder: "Enter your YouTube Data API v3 key",
                        class: format!("input input-bordered w-full pr-10 {}",
                            if api_key().trim().is_empty() { "input-warning" } else { "input-success" }
                        ),
                        value: api_key(),
                        oninput: move |evt| handle_api_key_change(evt.value()),
                        disabled: is_importing,
                    }
                    if !api_key().trim().is_empty() {
                        div { class: "absolute right-3 top-1/2 transform -translate-y-1/2",
                            Icon { icon: FaCheck, class: "w-4 h-4 text-success" }
                        }
                    }
                }
                label { class: "label",
                    span { class: "label-text-alt text-base-content/70 flex items-center gap-1",
                        Icon { icon: FaCircleInfo, class: "w-3 h-3" }
                        "Required for accessing YouTube playlist data. Your key is stored locally and never shared."
                    }
                }
            }

            // URL input with enhanced styling
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium flex items-center gap-2",
                        Icon { icon: FaYoutube, class: "w-4 h-4 text-red-500" }
                        "YouTube Playlist URL"
                    }
                }
                div { class: "relative",
                    input {
                        r#type: "url",
                        placeholder: "https://www.youtube.com/playlist?list=...",
                        class: format!("input input-bordered w-full pr-10 {}",
                            if validation_error().is_some() {
                                "input-error"
                            } else if !url().trim().is_empty() && validation_error().is_none() {
                                "input-success"
                            } else {
                                ""
                            }
                        ),
                        value: url(),
                        oninput: move |evt| handle_url_change(evt.value()),
                        disabled: is_importing,
                    }
                    if is_validating() {
                        div { class: "absolute right-3 top-1/2 transform -translate-y-1/2",
                            Icon { icon: FaSpinner, class: "w-4 h-4 animate-spin text-primary" }
                        }
                    } else if !url().trim().is_empty() && validation_error().is_none() {
                        div { class: "absolute right-3 top-1/2 transform -translate-y-1/2",
                            Icon { icon: FaCheck, class: "w-4 h-4 text-success" }
                        }
                    }
                }
                label { class: "label",
                    span { class: "label-text-alt text-base-content/70",
                        "Enter a YouTube playlist URL to import videos as course content"
                    }
                }
            }

            // Enhanced validation feedback
            if let Some(error) = validation_error() {
                div { class: "alert alert-error shadow-sm",
                    Icon { icon: FaCircleExclamation, class: "w-5 h-5" }
                    div {
                        div { class: "font-medium", "Validation Error" }
                        div { class: "text-sm opacity-90", "{error}" }
                    }
                }
            }

            // Enhanced preview section
            if let Some(preview_data) = preview() {
                YouTubePlaylistPreviewPanel { preview: preview_data }
            } else if is_loading_preview() {
                div { class: "card bg-gradient-to-r from-base-200 to-base-300 shadow-sm",
                    div { class: "card-body",
                        div { class: "flex items-center gap-3",
                            Icon { icon: FaSpinner, class: "w-5 h-5 animate-spin text-primary" }
                            span { class: "text-base-content", "Loading playlist preview..." }
                        }
                        div { class: "mt-2",
                            progress { class: "progress progress-primary w-full" }
                        }
                    }
                }
            }

            // Enhanced import progress
            if let Some(job) = import_job() {
                YouTubeImportProgressPanel { job }
            }

            // Enhanced import button
            div { class: "flex justify-end pt-4 border-t border-base-300",
                button {
                    class: format!("btn btn-primary btn-lg gap-2 {}",
                        if is_importing { "loading" } else { "" }
                    ),
                    disabled: !is_valid_for_import || is_importing,
                    onclick: handle_import,
                    if is_importing {
                        Icon { icon: FaSpinner, class: "w-4 h-4 animate-spin" }
                        "Importing..."
                    } else {
                        Icon { icon: FaPlay, class: "w-4 h-4" }
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
        div { class: "card bg-gradient-to-br from-red-50 to-red-100 border border-red-200 shadow-lg",
            div { class: "card-body",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "card-title text-lg flex items-center gap-2",
                        Icon { icon: FaYoutube, class: "w-5 h-5 text-red-500" }
                        "Playlist Preview"
                        Badge {
                            label: "Ready".to_string(),
                            color: Some("success".to_string()),
                            class: Some("badge-sm".to_string())
                        }
                    }
                    div { class: "text-xs text-base-content/60",
                        "Found {preview.video_count} videos"
                    }
                }

                // Enhanced summary stats
                div { class: "stats stats-horizontal shadow-md bg-white/80 backdrop-blur-sm w-full mb-4",
                    div { class: "stat",
                        div { class: "stat-figure text-red-500",
                            Icon { icon: FaYoutube, class: "w-8 h-8" }
                        }
                        div { class: "stat-title text-xs", "Course Title" }
                        div { class: "stat-value text-sm text-base-content truncate", "{preview.title}" }
                    }
                    div { class: "stat",
                        div { class: "stat-figure text-primary",
                            Icon { icon: FaVideo, class: "w-6 h-6" }
                        }
                        div { class: "stat-title text-xs", "Videos" }
                        div { class: "stat-value text-primary", "{preview.video_count}" }
                    }
                    div { class: "stat",
                        div { class: "stat-figure text-secondary",
                            Icon { icon: FaClock, class: "w-6 h-6" }
                        }
                        div { class: "stat-title text-xs", "Duration" }
                        div { class: "stat-value text-secondary text-sm", "{duration_text}" }
                    }
                }

                // Enhanced video list preview
                if !preview.videos.is_empty() {
                    div { class: "mt-4",
                        div { class: "flex items-center justify-between mb-3",
                            h4 { class: "font-medium text-base-content flex items-center gap-2",
                                Icon { icon: FaPlay, class: "w-4 h-4 text-primary" }
                                "Video Preview"
                            }
                            Badge {
                                label: format!("{} videos", preview.videos.len()),
                                color: Some("info".to_string()),
                                class: Some("badge-sm".to_string())
                            }
                        }
                        div { class: "space-y-2 max-h-40 overflow-y-auto bg-white/50 rounded-lg p-3",
                            {preview.videos.iter().take(5).enumerate().map(|(idx, video)| {
                                let duration_str = {
                                    let minutes = video.duration.as_secs() / 60;
                                    let seconds = video.duration.as_secs() % 60;
                                    format!("{minutes}:{seconds:02}")
                                };

                                rsx! {
                                    div {
                                        key: "{idx}",
                                        class: "flex justify-between items-center text-sm p-3 bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow",
                                        div { class: "flex items-center gap-3 flex-1 min-w-0",
                                            div { class: "w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0",
                                                span { class: "text-xs font-medium text-primary", "{idx + 1}" }
                                            }
                                            span { class: "truncate text-base-content", "{video.title}" }
                                        }
                                        div { class: "flex items-center gap-2 flex-shrink-0",
                                            Icon { icon: FaClock, class: "w-3 h-3 text-base-content/50" }
                                            span { class: "text-xs text-base-content/70 font-mono", "{duration_str}" }
                                        }
                                    }
                                }
                            })}

                            if preview.videos.len() > 5 {
                                div { class: "text-center py-3 border-t border-base-300",
                                    Badge {
                                        label: format!("+ {} more videos", preview.videos.len() - 5),
                                        color: Some("ghost".to_string()),
                                        class: Some("badge-sm".to_string())
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
        div { class: format!("card shadow-lg border {}",
            match job.status {
                crate::types::ImportStatus::Completed => "bg-gradient-to-br from-success/10 to-success/5 border-success/20",
                crate::types::ImportStatus::Failed => "bg-gradient-to-br from-error/10 to-error/5 border-error/20",
                _ => "bg-gradient-to-br from-primary/10 to-primary/5 border-primary/20",
            }
        ),
            div { class: "card-body",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "card-title text-lg flex items-center gap-2",
                        if matches!(job.status, crate::types::ImportStatus::Starting | crate::types::ImportStatus::InProgress) {
                            Icon { icon: FaSpinner, class: "w-5 h-5 animate-spin text-primary" }
                        } else if matches!(job.status, crate::types::ImportStatus::Completed) {
                            Icon { icon: FaCheck, class: "w-5 h-5 text-success" }
                        } else {
                            Icon { icon: FaCircleExclamation, class: "w-5 h-5 text-error" }
                        }
                        "Import Progress"
                        Badge {
                            label: status_text.to_string(),
                            color: Some(match job.status {
                                crate::types::ImportStatus::Completed => "success",
                                crate::types::ImportStatus::Failed => "error",
                                _ => "primary",
                            }.to_string()),
                            class: Some("badge-sm".to_string())
                        }
                    }
                    div { class: "text-sm font-mono {status_color}",
                        "{job.progress_percentage:.1}%"
                    }
                }

                div { class: "space-y-4",
                    // Enhanced progress bar
                    div { class: "w-full",
                        div { class: "flex justify-between items-center mb-2",
                            span { class: "text-sm font-medium {status_color}", "{status_text}" }
                            span { class: "text-xs text-base-content/60", "Step {job.progress_percentage:.0}/100" }
                        }
                        div { class: "w-full bg-base-300 rounded-full h-3 shadow-inner",
                            div {
                                class: format!("h-3 rounded-full transition-all duration-500 ease-out {}",
                                    match job.status {
                                        crate::types::ImportStatus::Completed => "bg-gradient-to-r from-success to-success/80",
                                        crate::types::ImportStatus::Failed => "bg-gradient-to-r from-error to-error/80",
                                        _ => "bg-gradient-to-r from-primary to-primary/80",
                                    }
                                ),
                                style: "width: {job.progress_percentage}%",
                            }
                        }
                    }

                    // Enhanced current message
                    div { class: "bg-white/50 rounded-lg p-3 border border-base-300",
                        div { class: "flex items-center gap-2",
                            Icon { icon: FaCircleInfo, class: "w-4 h-4 text-info flex-shrink-0" }
                            span { class: "text-sm text-base-content", "{job.message}" }
                        }
                    }

                    // Enhanced error details for failed imports
                    if matches!(job.status, crate::types::ImportStatus::Failed) {
                        div { class: "alert alert-error shadow-sm",
                            Icon { icon: FaCircleExclamation, class: "w-5 h-5" }
                            div {
                                div { class: "font-medium", "Import Failed" }
                                div { class: "text-sm opacity-90", "Please check your API key and playlist URL, then try again." }
                            }
                        }
                    }

                    // Success message for completed imports
                    if matches!(job.status, crate::types::ImportStatus::Completed) {
                        div { class: "alert alert-success shadow-sm",
                            Icon { icon: FaCheck, class: "w-5 h-5" }
                            div {
                                div { class: "font-medium", "Import Completed Successfully!" }
                                div { class: "text-sm opacity-90", "Your course has been imported and is ready to use." }
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
        div { class: "space-y-6",
            // Enhanced header section
            div { class: "flex items-center gap-3 mb-6",
                div { class: "w-10 h-10 bg-primary/10 rounded-full flex items-center justify-center",
                    Icon { icon: FaFolder, class: "w-5 h-5 text-primary" }
                }
                div {
                    h3 { class: "text-lg font-semibold text-base-content", "Local Folder Import" }
                    p { class: "text-sm text-base-content/70", "Import videos from a local folder on your computer" }
                }
            }

            // Enhanced path input
            div { class: "form-control",
                label { class: "label",
                    span { class: "label-text font-medium flex items-center gap-2",
                        Icon { icon: FaFolder, class: "w-4 h-4 text-primary" }
                        "Local Folder Path"
                    }
                }
                div { class: "flex gap-3",
                    div { class: "relative flex-1",
                        input {
                            r#type: "text",
                            placeholder: "/path/to/video/folder",
                            class: format!("input input-bordered w-full pr-10 {}",
                                if path.trim().is_empty() {
                                    ""
                                } else if folder_validation.is_some() && folder_validation.as_ref().unwrap().is_valid {
                                    "input-success"
                                } else if folder_validation.is_some() {
                                    "input-error"
                                } else {
                                    "input-warning"
                                }
                            ),
                            value: path,
                            oninput: move |evt| on_path_change.call(evt.value()),
                        }
                        if !path.trim().is_empty() {
                            div { class: "absolute right-3 top-1/2 transform -translate-y-1/2",
                                if is_validating {
                                    Icon { icon: FaSpinner, class: "w-4 h-4 animate-spin text-primary" }
                                } else if folder_validation.is_some() && folder_validation.as_ref().unwrap().is_valid {
                                    Icon { icon: FaCheck, class: "w-4 h-4 text-success" }
                                } else if folder_validation.is_some() {
                                    Icon { icon: FaCircleExclamation, class: "w-4 h-4 text-error" }
                                }
                            }
                        }
                    }
                    button {
                        class: "btn btn-outline btn-primary gap-2",
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
                        Icon { icon: FaFolder, class: "w-4 h-4" }
                        "Browse"
                    }
                }
                label { class: "label",
                    span { class: "label-text-alt text-base-content/70",
                        "Select a folder containing video files to import as course content"
                    }
                }
            }

            // Enhanced supported formats info
            div { class: "alert alert-info shadow-sm",
                Icon { icon: FaCircleInfo, class: "w-5 h-5" }
                div {
                    div { class: "font-medium", "Supported video formats:" }
                    div { class: "text-sm opacity-90 mt-1",
                        div { class: "flex flex-wrap gap-1",
                            {["MP4", "AVI", "MKV", "MOV", "WMV", "FLV", "WebM", "M4V", "MPG", "MPEG"].iter().map(|format| {
                                rsx! {
                                    Badge {
                                        key: "{format}",
                                        label: format.to_string(),
                                        color: Some("info".to_string()),
                                        class: Some("badge-xs".to_string())
                                    }
                                }
                            })}
                        }
                    }
                }
            }

            // Enhanced preview section
            if let Some(preview_data) = preview {
                ImportPreviewPanel { preview: preview_data }
            } else if let Some(ref validation) = folder_validation {
                FolderValidationPanel { validation: validation.clone() }
            } else if is_validating {
                div { class: "card bg-gradient-to-r from-base-200 to-base-300 shadow-sm",
                    div { class: "card-body",
                        div { class: "flex items-center gap-3",
                            Icon { icon: FaSpinner, class: "w-5 h-5 animate-spin text-primary" }
                            span { class: "text-base-content", "Validating folder..." }
                        }
                        div { class: "mt-2",
                            progress { class: "progress progress-primary w-full" }
                        }
                    }
                }
            }
        }
    }
}

/// Enhanced import settings configuration panel
#[component]
fn ImportSettingsPanel(
    settings: ImportSettings,
    on_settings_change: EventHandler<ImportSettings>,
) -> Element {
    rsx! {
        div { class: "collapse collapse-arrow bg-gradient-to-r from-base-200 to-base-300 shadow-sm border border-base-300",
            input { r#type: "checkbox" }
            div { class: "collapse-title font-medium text-base-content flex items-center gap-2",
                Icon { icon: FaCircleInfo, class: "w-4 h-4 text-info" }
                "Import Settings"
                Badge {
                    label: "Optional".to_string(),
                    color: Some("ghost".to_string()),
                    class: Some("badge-sm".to_string())
                }
            }
            div { class: "collapse-content space-y-4 bg-white/50 rounded-lg p-4",
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    // Sort by title setting
                    div { class: "form-control bg-white rounded-lg p-3 shadow-sm",
                        label { class: "label cursor-pointer",
                            div { class: "flex flex-col items-start",
                                span { class: "label-text font-medium", "Sort videos by title" }
                                span { class: "label-text-alt text-xs", "Alphabetically organize imported videos" }
                            }
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

                    // Filter duplicates setting
                    div { class: "form-control bg-white rounded-lg p-3 shadow-sm",
                        label { class: "label cursor-pointer",
                            div { class: "flex flex-col items-start",
                                span { class: "label-text font-medium", "Filter duplicate videos" }
                                span { class: "label-text-alt text-xs", "Remove videos with identical titles" }
                            }
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

                    // Include metadata setting
                    div { class: "form-control bg-white rounded-lg p-3 shadow-sm",
                        label { class: "label cursor-pointer",
                            div { class: "flex flex-col items-start",
                                span { class: "label-text font-medium", "Include video metadata" }
                                span { class: "label-text-alt text-xs", "Import additional video information" }
                            }
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

                    // Auto-structure setting
                    div { class: "form-control bg-white rounded-lg p-3 shadow-sm",
                        label { class: "label cursor-pointer",
                            div { class: "flex flex-col items-start",
                                span { class: "label-text font-medium", "Auto-structure course content" }
                                span { class: "label-text-alt text-xs", "Automatically organize into modules" }
                            }
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

                // Settings summary
                div { class: "mt-4 p-3 bg-info/10 rounded-lg border border-info/20",
                    div { class: "flex items-center gap-2 mb-2",
                        Icon { icon: FaCircleInfo, class: "w-4 h-4 text-info" }
                        span { class: "text-sm font-medium text-info", "Current Settings" }
                    }
                    div { class: "flex flex-wrap gap-1",
                        if settings.sort_by_title {
                            Badge { label: "Sort by title".to_string(), color: Some("success".to_string()), class: Some("badge-xs".to_string()) }
                        }
                        if settings.filter_duplicates {
                            Badge { label: "Filter duplicates".to_string(), color: Some("success".to_string()), class: Some("badge-xs".to_string()) }
                        }
                        if settings.include_metadata {
                            Badge { label: "Include metadata".to_string(), color: Some("success".to_string()), class: Some("badge-xs".to_string()) }
                        }
                        if settings.auto_structure {
                            Badge { label: "Auto-structure".to_string(), color: Some("success".to_string()), class: Some("badge-xs".to_string()) }
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
        div { class: format!("card shadow-lg border {}",
            if validation.is_valid {
                "bg-gradient-to-br from-success/10 to-success/5 border-success/20"
            } else {
                "bg-gradient-to-br from-error/10 to-error/5 border-error/20"
            }
        ),
            div { class: "card-body",
                div { class: "flex items-center justify-between mb-4",
                    if validation.is_valid {
                        h3 { class: "card-title text-lg text-success flex items-center gap-2",
                            Icon { icon: FaCheck, class: "w-5 h-5" }
                            "Folder Validation - Success"
                            Badge {
                                label: "Valid".to_string(),
                                color: Some("success".to_string()),
                                class: Some("badge-sm".to_string())
                            }
                        }
                    } else {
                        h3 { class: "card-title text-lg text-error flex items-center gap-2",
                            Icon { icon: FaCircleExclamation, class: "w-5 h-5" }
                            "Folder Validation - Error"
                            Badge {
                                label: "Invalid".to_string(),
                                color: Some("error".to_string()),
                                class: Some("badge-sm".to_string())
                            }
                        }
                    }
                }

                if validation.is_valid {
                    // Enhanced success stats
                    div { class: "stats stats-horizontal shadow-md bg-white/80 backdrop-blur-sm w-full mt-4",
                        div { class: "stat",
                            div { class: "stat-figure text-primary",
                                Icon { icon: FaVideo, class: "w-6 h-6" }
                            }
                            div { class: "stat-title text-xs", "Video Files" }
                            div { class: "stat-value text-primary", "{validation.video_count}" }
                        }
                        div { class: "stat",
                            div { class: "stat-figure text-secondary",
                                Icon { icon: FaFolder, class: "w-6 h-6" }
                            }
                            div { class: "stat-title text-xs", "Total Size" }
                            div { class: "stat-value text-secondary text-sm", "{size_text}" }
                        }
                        if !validation.unsupported_files.is_empty() {
                            div { class: "stat",
                                div { class: "stat-figure text-warning",
                                    Icon { icon: FaCircleExclamation, class: "w-6 h-6" }
                                }
                                div { class: "stat-title text-xs", "Unsupported" }
                                div { class: "stat-value text-warning text-sm", "{validation.unsupported_files.len()}" }
                            }
                        }
                    }

                    // Enhanced file list preview
                    if !validation.supported_files.is_empty() {
                        div { class: "mt-4",
                            div { class: "flex items-center justify-between mb-3",
                                h4 { class: "font-medium text-base-content flex items-center gap-2",
                                    Icon { icon: FaVideo, class: "w-4 h-4 text-primary" }
                                    "Video Files Found"
                                }
                                Badge {
                                    label: format!("{} files", validation.supported_files.len()),
                                    color: Some("success".to_string()),
                                    class: Some("badge-sm".to_string())
                                }
                            }
                            div { class: "space-y-2 max-h-40 overflow-y-auto bg-white/50 rounded-lg p-3",
                                {validation.supported_files.iter().take(5).enumerate().map(|(idx, file_path)| {
                                    let file_name = file_path.file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or("Unknown");
                                    let extension = file_path.extension()
                                        .and_then(|ext| ext.to_str())
                                        .unwrap_or("")
                                        .to_uppercase();

                                    rsx! {
                                        div {
                                            key: "{idx}",
                                            class: "flex justify-between items-center text-sm p-3 bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow",
                                            div { class: "flex items-center gap-3 flex-1 min-w-0",
                                                div { class: "w-6 h-6 bg-success/10 rounded-full flex items-center justify-center flex-shrink-0",
                                                    Icon { icon: FaVideo, class: "w-3 h-3 text-success" }
                                                }
                                                span { class: "truncate text-base-content", "{file_name}" }
                                            }
                                            Badge {
                                                label: extension,
                                                color: Some("primary".to_string()),
                                                class: Some("badge-xs".to_string())
                                            }
                                        }
                                    }
                                })}

                                if validation.supported_files.len() > 5 {
                                    div { class: "text-center py-3 border-t border-base-300",
                                        Badge {
                                            label: format!("+ {} more video files", validation.supported_files.len() - 5),
                                            color: Some("ghost".to_string()),
                                            class: Some("badge-sm".to_string())
                                        }
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

/// Enhanced other resources form component
#[component]
fn OtherResourcesForm() -> Element {
    rsx! {
        div { class: "space-y-6",
            // Enhanced header section
            div { class: "flex items-center gap-3 mb-6",
                div { class: "w-10 h-10 bg-secondary/10 rounded-full flex items-center justify-center",
                    Icon { icon: FaGlobe, class: "w-5 h-5 text-secondary" }
                }
                div {
                    h3 { class: "text-lg font-semibold text-base-content", "Other Resources" }
                    p { class: "text-sm text-base-content/70", "Additional import sources coming soon" }
                }
            }

            // Enhanced coming soon message
            div { class: "alert alert-info shadow-sm",
                Icon { icon: FaCircleInfo, class: "w-5 h-5" }
                div {
                    div { class: "font-medium", "Coming Soon!" }
                    div { class: "text-sm opacity-90", "Support for additional course sources will be added in future updates." }
                }
            }

            // Enhanced placeholder content
            div { class: "card bg-gradient-to-br from-base-200 to-base-300 shadow-lg",
                div { class: "card-body text-center",
                    h3 { class: "card-title justify-center mb-6 text-xl", "Additional Import Sources" }

                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6",
                        // Online Course Platforms
                        div { class: "card bg-white shadow-md hover:shadow-lg transition-shadow",
                            div { class: "card-body p-4",
                                div { class: "w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-3",
                                    Icon { icon: FaPlay, class: "w-6 h-6 text-primary" }
                                }
                                h4 { class: "font-semibold text-base-content mb-2", "Online Course Platforms" }
                                p { class: "text-sm text-base-content/70 mb-3", "Import from popular learning platforms" }
                                div { class: "flex flex-wrap gap-1 justify-center",
                                    Badge { label: "Udemy".to_string(), color: Some("primary".to_string()), class: Some("badge-xs".to_string()) }
                                    Badge { label: "Coursera".to_string(), color: Some("primary".to_string()), class: Some("badge-xs".to_string()) }
                                    Badge { label: "edX".to_string(), color: Some("primary".to_string()), class: Some("badge-xs".to_string()) }
                                    Badge { label: "Khan Academy".to_string(), color: Some("primary".to_string()), class: Some("badge-xs".to_string()) }
                                }
                            }
                        }

                        // Video Streaming Services
                        div { class: "card bg-white shadow-md hover:shadow-lg transition-shadow",
                            div { class: "card-body p-4",
                                div { class: "w-12 h-12 bg-secondary/10 rounded-full flex items-center justify-center mx-auto mb-3",
                                    Icon { icon: FaVideo, class: "w-6 h-6 text-secondary" }
                                }
                                h4 { class: "font-semibold text-base-content mb-2", "Video Streaming Services" }
                                p { class: "text-sm text-base-content/70 mb-3", "Import from video platforms" }
                                div { class: "flex flex-wrap gap-1 justify-center",
                                    Badge { label: "Vimeo".to_string(), color: Some("secondary".to_string()), class: Some("badge-xs".to_string()) }
                                    Badge { label: "Twitch".to_string(), color: Some("secondary".to_string()), class: Some("badge-xs".to_string()) }
                                    Badge { label: "Custom URLs".to_string(), color: Some("secondary".to_string()), class: Some("badge-xs".to_string()) }
                                }
                            }
                        }

                        // Document & Text Sources
                        div { class: "card bg-white shadow-md hover:shadow-lg transition-shadow",
                            div { class: "card-body p-4",
                                div { class: "w-12 h-12 bg-accent/10 rounded-full flex items-center justify-center mx-auto mb-3",
                                    Icon { icon: FaCircleInfo, class: "w-6 h-6 text-accent" }
                                }
                                h4 { class: "font-semibold text-base-content mb-2", "Document & Text Sources" }
                                p { class: "text-sm text-base-content/70 mb-3", "Import from documents and articles" }
                                div { class: "flex flex-wrap gap-1 justify-center",
                                    Badge { label: "PDFs".to_string(), color: Some("accent".to_string()), class: Some("badge-xs".to_string()) }
                                    Badge { label: "Web Articles".to_string(), color: Some("accent".to_string()), class: Some("badge-xs".to_string()) }
                                    Badge { label: "Documentation".to_string(), color: Some("accent".to_string()), class: Some("badge-xs".to_string()) }
                                }
                            }
                        }
                    }

                    // Call to action
                    div { class: "bg-info/10 rounded-lg p-4 border border-info/20",
                        div { class: "flex items-center justify-center gap-2 mb-2",
                            Icon { icon: FaCircleInfo, class: "w-4 h-4 text-info" }
                            span { class: "text-sm font-medium text-info", "Stay Updated" }
                        }
                        p { class: "text-sm text-base-content/70",
                            "These import sources are planned for future releases. Follow our updates to be notified when they become available!"
                        }
                    }
                }
            }
        }
    }
}
