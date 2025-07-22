use dioxus::prelude::*;
use crate::ui::components::{modal::Modal, tabs::Tabs, toast, youtube_import_form::YouTubeImportForm};

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
}

/// Import source selection modal with tabs for YouTube and Local Folder
#[component]
pub fn ImportModal(props: ImportModalProps) -> Element {
    let mut selected_tab = use_signal(|| 0usize);
    let mut youtube_url = use_signal(|| String::new());
    let mut local_path = use_signal(|| String::new());
    let mut import_settings = use_signal(|| ImportSettings::default());
    let mut is_validating = use_signal(|| false);
    
    // Tab labels and sources
    let tab_labels = vec!["Local Course".to_string(), "YouTube".to_string(), "Other Resources".to_string()];
    let sources = vec![ImportSource::LocalFolder, ImportSource::YouTube, ImportSource::OtherResources];
    
    // Get current source
    let current_source = sources[selected_tab()];
    
    // Validation state
    let is_valid = match current_source {
        ImportSource::LocalFolder => !local_path().trim().is_empty(),
        ImportSource::YouTube => !youtube_url().trim().is_empty() && youtube_url().contains("youtube.com"),
        ImportSource::OtherResources => false, // Always disabled for now
    };
    
    // Handle import action
    let handle_import = {
        let on_import = props.on_import.clone();
        let youtube_url = youtube_url.clone();
        let local_path = local_path.clone();
        let import_settings = import_settings.clone();
        
        move |_| {
            let source = sources[selected_tab()];
            let input = match source {
                ImportSource::LocalFolder => local_path().trim().to_string(),
                ImportSource::YouTube => youtube_url().trim().to_string(),
                ImportSource::OtherResources => String::new(),
            };
            
            if !input.is_empty() {
                on_import.call((source, input, import_settings()));
            } else {
                toast::toast::error("Please provide a valid input");
            }
        }
    };
    
    // Handle URL validation for YouTube
    let _handle_url_validation = {
        let mut youtube_url = youtube_url.clone();
        let mut is_validating = is_validating.clone();
        
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
    
    // Reset form when modal closes
    use_effect(move || {
        if !props.open {
            youtube_url.set(String::new());
            local_path.set(String::new());
            selected_tab.set(0);
            is_validating.set(false);
        }
    });

    rsx! {
        Modal {
            open: props.open,
            on_close: props.on_close.clone(),
            title: "Import Course Content".to_string(),
            actions: rsx! {
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| props.on_close.call(()),
                    "Cancel"
                }
                button {
                    class: "btn btn-primary",
                    disabled: !is_valid || is_validating() || props.preview_loading,
                    onclick: handle_import,
                    if props.preview_loading {
                        span { class: "loading loading-spinner loading-sm mr-2" }
                    }
                    "Import Course"
                }
            },
            
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
                            }
                        },
                        ImportSource::YouTube => rsx! {
                            YouTubeImportFormWrapper {
                                on_import_complete: move |_course| {
                                    props.on_close.call(());
                                    toast::toast::success("Course imported successfully!");
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
fn YouTubeImportFormWrapper(
    on_import_complete: EventHandler<crate::types::Course>,
) -> Element {
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
) -> Element {
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
                        onclick: move |_| {
                            // In a desktop app, this would open a folder picker dialog
                            toast::toast::info("Folder picker would open here in desktop app");
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
            } else if preview_loading {
                div { class: "card bg-base-200",
                    div { class: "card-body",
                        div { class: "flex items-center gap-3",
                            span { class: "loading loading-spinner loading-md" }
                            span { "Scanning folder..." }
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
                                let mut new_settings = settings.clone();
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
                                let mut new_settings = settings.clone();
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
                                let mut new_settings = settings.clone();
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
                                let mut new_settings = settings.clone();
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
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
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
                                    format!("{}:{:02}", minutes, seconds)
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