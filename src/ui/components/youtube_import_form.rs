use dioxus::prelude::*;
use crate::ui::components::toast;
use crate::ingest::youtube::{import_from_youtube, validate_playlist_url, extract_playlist_id};
use crate::types::{Course, ImportJob, ImportStatus};
use crate::ui::backend_adapter::use_backend_adapter;
use crate::{ImportError, nlp};
use crate::storage::AppSettings;
use std::time::Duration;

/// YouTube playlist preview data
#[derive(Debug, Clone, PartialEq)]
pub struct YouTubePlaylistPreview {
    pub title: String,
    pub video_count: usize,
    pub total_duration: Duration,
    pub videos: Vec<YouTubeVideoPreview>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YouTubeVideoPreview {
    pub title: String,
    pub duration: Duration,
    pub index: usize,
}

/// YouTube import form with URL validation and playlist preview
#[component]
pub fn YouTubeImportForm(
    /// Callback when import is completed successfully
    on_import_complete: EventHandler<Course>,
    /// Callback when import fails
    #[props(optional)]
    on_import_error: Option<EventHandler<String>>,
) -> Element {
    let backend = use_backend_adapter();
    
    // Load settings and initialize API key from storage
    let settings = use_resource(|| async { AppSettings::load().unwrap_or_default() });
    
    // Form state
    let url = use_signal(|| String::new());
    let api_key = use_signal(|| {
        settings.read().as_ref().and_then(|s| s.get_youtube_api_key().map(|k| k.to_string())).unwrap_or_default()
    });
    
    // Validation and preview state
    let validation_error = use_signal(|| Option::<String>::None);
    let is_validating = use_signal(|| false);
    let preview = use_signal(|| Option::<YouTubePlaylistPreview>::None);
    let is_loading_preview = use_signal(|| false);
    
    // Import progress state
    let import_job = use_signal(|| Option::<ImportJob>::None);
    
    // Handle URL input changes with validation
    let mut handle_url_change = {
        let mut url = url.clone();
        let mut validation_error = validation_error.clone();
        let mut is_validating = is_validating.clone();
        let mut preview = preview.clone();
        let mut is_loading_preview = is_loading_preview.clone();
        let api_key = api_key.clone();
        
        move |new_url: String| {
            url.set(new_url.clone());
            validation_error.set(None);
            preview.set(None);
            
            if new_url.trim().is_empty() {
                return;
            }
            
            // Basic URL format validation
            if !new_url.contains("youtube.com") || (!new_url.contains("playlist") && !new_url.contains("list=")) {
                validation_error.set(Some("Please enter a valid YouTube playlist URL".to_string()));
                return;
            }
            
            // If we have an API key, validate the playlist and load preview
            if !api_key().trim().is_empty() {
                is_validating.set(true);
                is_loading_preview.set(true);
                
                let api_key_val = api_key();
                let mut validation_error = validation_error.clone();
                let mut preview = preview.clone();
                let mut is_validating = is_validating.clone();
                let mut is_loading_preview = is_loading_preview.clone();
                
                spawn(async move {
                    // First validate the playlist exists
                    match validate_playlist_url(&new_url, &api_key_val).await {
                        Ok(true) => {
                            // Load preview data
                            match import_from_youtube(&new_url, &api_key_val).await {
                                Ok((sections, metadata)) => {
                                    let total_duration = sections.iter()
                                        .map(|s| s.duration)
                                        .sum::<Duration>();
                                    
                                    let videos = sections.into_iter().enumerate().map(|(idx, section)| {
                                        YouTubeVideoPreview {
                                            title: section.title,
                                            duration: section.duration,
                                            index: idx,
                                        }
                                    }).collect::<Vec<_>>();
                                    
                                    let preview_data = YouTubePlaylistPreview {
                                        title: metadata.title,
                                        video_count: metadata.video_count,
                                        total_duration,
                                        videos,
                                    };
                                    
                                    preview.set(Some(preview_data));
                                }
                                Err(ImportError::Network(msg)) => {
                                    validation_error.set(Some(format!("Network error: {}", msg)));
                                }
                                Err(ImportError::InvalidUrl(msg)) => {
                                    validation_error.set(Some(msg));
                                }
                                Err(ImportError::NoContent) => {
                                    validation_error.set(Some("Playlist is empty or contains no accessible videos".to_string()));
                                }
                                Err(e) => {
                                    validation_error.set(Some(format!("Failed to load playlist: {}", e)));
                                }
                            }
                        }
                        Ok(false) => {
                            validation_error.set(Some("Playlist not found or not accessible. Please check the URL and ensure the playlist is public or unlisted.".to_string()));
                        }
                        Err(ImportError::Network(msg)) => {
                            validation_error.set(Some(format!("Network error: {}", msg)));
                        }
                        Err(ImportError::InvalidUrl(msg)) => {
                            validation_error.set(Some(msg));
                        }
                        Err(e) => {
                            validation_error.set(Some(format!("Validation error: {}", e)));
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
        let mut api_key = api_key.clone();
        let url = url.clone();
        let mut handle_url_change = handle_url_change.clone();
        
        move |new_api_key: String| {
            api_key.set(new_api_key.clone());
            
            // Save API key to settings asynchronously
            spawn(async move {
                if let Ok(mut settings) = AppSettings::load() {
                    let api_key_to_save = if new_api_key.trim().is_empty() {
                        None
                    } else {
                        Some(new_api_key.trim().to_string())
                    };
                    
                    if let Err(e) = settings.set_youtube_api_key(api_key_to_save) {
                        log::error!("Failed to save API key: {}", e);
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
        let import_job = import_job.clone();
        let on_import_complete = on_import_complete.clone();
        let on_import_error = on_import_error.clone();
        let url = url.clone();
        let api_key = api_key.clone();
        let preview = preview.clone();
        
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
            let mut import_job = import_job.clone();
            let on_import_complete = on_import_complete.clone();
            let on_import_error = on_import_error.clone();
            
            spawn(async move {
                // Create initial import job
                let job = ImportJob::new(format!("Starting import from YouTube playlist"));
                import_job.set(Some(job.clone()));
                
                // Progress callback
                let mut progress_callback = {
                    let mut import_job = import_job.clone();
                    move |percentage: f32, message: String| {
                        if let Some(mut job) = import_job() {
                            job.update_progress(percentage, message);
                            import_job.set(Some(job));
                        }
                    }
                };
                
                // Perform the import
                progress_callback(10.0, "Fetching playlist data...".to_string());
                
                match import_from_youtube(&url_val, &api_key_val).await {
                    Ok((sections, metadata)) => {
                        progress_callback(40.0, "Processing video data...".to_string());
                        
                        // Convert to course
                        let raw_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
                        let course_name = metadata.title;
                        let mut course = Course::new(course_name, raw_titles);
                        
                        // Structure the course using NLP
                        progress_callback(70.0, "Analyzing course structure...".to_string());
                        match nlp::structure_course(course.raw_titles.clone()) {
                            Ok(course_structure) => {
                                course.structure = Some(course_structure);
                                progress_callback(90.0, "Saving course...".to_string());
                                
                                // Save to database
                                match backend.create_course(course.clone()).await {
                                    Ok(_) => {
                                        progress_callback(100.0, "Import completed successfully!".to_string());
                                        toast::toast::success("Course imported successfully!");
                                        on_import_complete.call(course);
                                    }
                                    Err(e) => {
                                        let error_msg = format!("Failed to save course: {}", e);
                                        if let Some(mut job) = import_job() {
                                            job.mark_failed(error_msg.clone());
                                            import_job.set(Some(job));
                                        }
                                        toast::toast::error("Failed to save course");
                                        if let Some(on_error) = on_import_error.clone() {
                                            on_error.call(error_msg);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to structure course: {}", e);
                                if let Some(mut job) = import_job() {
                                    job.mark_failed(error_msg.clone());
                                    import_job.set(Some(job));
                                }
                                toast::toast::error("Failed to structure course");
                                if let Some(on_error) = on_import_error.clone() {
                                    on_error.call(error_msg);
                                }
                            }
                        }
                    }
                    Err(ImportError::Network(msg)) => {
                        let error_msg = format!("Network error: {}", msg);
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error("Network error occurred. Please check your connection and try again.");
                        if let Some(on_error) = on_import_error.clone() {
                            on_error.call(error_msg);
                        }
                    }
                    Err(ImportError::InvalidUrl(msg)) => {
                        let error_msg = format!("Invalid URL: {}", msg);
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error("Invalid playlist URL. Please check the URL and try again.");
                        if let Some(on_error) = on_import_error.clone() {
                            on_error.call(error_msg);
                        }
                    }
                    Err(ImportError::NoContent) => {
                        let error_msg = "No accessible content found in playlist".to_string();
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error("Playlist is empty or contains no accessible videos.");
                        if let Some(on_error) = on_import_error.clone() {
                            on_error.call(error_msg);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Import failed: {}", e);
                        if let Some(mut job) = import_job() {
                            job.mark_failed(error_msg.clone());
                            import_job.set(Some(job));
                        }
                        toast::toast::error(&format!("Import failed: {}", e));
                        if let Some(on_error) = on_import_error.clone() {
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
    
    let is_importing = import_job().map_or(false, |job| matches!(job.status, ImportStatus::Starting | ImportStatus::InProgress));

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
                PlaylistPreviewPanel { preview: preview_data }
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
                ImportProgressPanel { job }
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
fn PlaylistPreviewPanel(preview: YouTubePlaylistPreview) -> Element {
    let duration_text = {
        let hours = preview.total_duration.as_secs() / 3600;
        let minutes = (preview.total_duration.as_secs() % 3600) / 60;
        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
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
                                    format!("{}:{:02}", minutes, seconds)
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
fn ImportProgressPanel(job: ImportJob) -> Element {
    let status_color = match job.status {
        ImportStatus::Starting => "text-info",
        ImportStatus::InProgress => "text-primary",
        ImportStatus::Completed => "text-success",
        ImportStatus::Failed => "text-error",
    };
    
    let status_text = match job.status {
        ImportStatus::Starting => "Starting...",
        ImportStatus::InProgress => "In Progress",
        ImportStatus::Completed => "Completed",
        ImportStatus::Failed => "Failed",
    };

    rsx! {
        div { class: "card bg-base-200 border border-base-300",
            div { class: "card-body",
                h3 { class: "card-title text-lg flex items-center gap-2",
                    if matches!(job.status, ImportStatus::Starting | ImportStatus::InProgress) {
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
                                    ImportStatus::Completed => "bg-success",
                                    ImportStatus::Failed => "bg-error",
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
                    if matches!(job.status, ImportStatus::Failed) {
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
    if let Some(playlist_id) = extract_playlist_id(url) {
        format!("YouTube Playlist {}", &playlist_id[..8.min(playlist_id.len())])
    } else {
        "YouTube Course".to_string()
    }
}