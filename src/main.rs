use course_pilot::app::initialize_app;
use course_pilot::ui::app_root::AppRoot;
use dioxus_desktop::tao::dpi::LogicalSize;
use dioxus_desktop::{Config, WindowBuilder};
use log::{error, info, warn};
use std::borrow::Cow;
use std::path::Path;

fn main() {
    // Initialize application with error handling
    if let Err(e) = initialize_app() {
        eprintln!("Failed to initialize application: {e}");
        std::process::exit(1);
    }

    info!("Starting Course Pilot desktop application");

    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Course Pilot")
                .with_inner_size(LogicalSize::new(1280, 800))
                .with_min_inner_size(LogicalSize::new(1024, 768)),
        )
        .with_custom_protocol("local-video".to_string(), |request| {
            info!("Custom protocol request: {}", request.uri());
            
            // Extract the file path from the URI
            // URI format: local-video://file/path/to/video.mp4
            let uri = request.uri().to_string();
            if let Some(path_part) = uri.strip_prefix("local-video://file/") {
                let file_path = Path::new(path_part);
                
                info!("Serving local video file: {}", file_path.display());
                
                // Check if file exists and is readable
                if !file_path.exists() {
                    warn!("Requested video file does not exist: {}", file_path.display());
                    return dioxus_desktop::wry::http::Response::builder()
                        .status(404)
                        .body(Cow::Borrowed(b"File not found" as &[u8]))
                        .unwrap();
                }
                
                // Read the file
                match std::fs::read(file_path) {
                    Ok(content) => {
                        // Determine MIME type based on file extension
                        let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
                            Some("mp4") => "video/mp4",
                            Some("webm") => "video/webm",
                            Some("ogg") => "video/ogg",
                            Some("mov") => "video/quicktime", 
                            Some("avi") => "video/x-msvideo",
                            Some("mkv") => "video/x-matroska",
                            Some("m4v") => "video/mp4",
                            Some("3gp") => "video/3gpp",
                            Some("flv") => "video/x-flv",
                            _ => "application/octet-stream",
                        };
                        
                        info!("Serving {} bytes of {} content for {}", content.len(), mime_type, file_path.display());
                        
                        dioxus_desktop::wry::http::Response::builder()
                            .status(200)
                            .header("Content-Type", mime_type)
                            .header("Content-Length", content.len().to_string())
                            .header("Accept-Ranges", "bytes")
                            .header("Cache-Control", "no-cache")
                            .body(Cow::Owned(content))
                            .unwrap()
                    }
                    Err(e) => {
                        error!("Failed to read video file {}: {}", file_path.display(), e);
                        dioxus_desktop::wry::http::Response::builder()
                            .status(500)
                            .body(Cow::Borrowed(b"Internal server error" as &[u8]))
                            .unwrap()
                    }
                }
            } else {
                warn!("Invalid custom protocol URI format: {}", uri);
                dioxus_desktop::wry::http::Response::builder()
                    .status(400)
                    .body(Cow::Borrowed(b"Invalid URI format" as &[u8]))
                    .unwrap()
            }
        });

    // Set up panic handler for better error reporting
    std::panic::set_hook(Box::new(|panic_info| {
        error!("Application panic: {panic_info}");
        eprintln!("Course Pilot encountered a critical error and must close.");
        eprintln!("Error details: {panic_info}");
        eprintln!("Please check the logs for more information.");
    }));

    info!("Launching Dioxus desktop application");

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(AppRoot);
}
