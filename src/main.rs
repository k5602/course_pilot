use course_pilot::app::initialize_app;
use course_pilot::ui::app_root::AppRoot;
use dioxus_desktop::tao::dpi::LogicalSize;
use dioxus_desktop::wry::http::{Response, StatusCode};
use dioxus_desktop::{Config, WindowBuilder};
use log::{error, info, warn};
use std::borrow::Cow;

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
        .with_custom_protocol("local-video".to_string(), handle_local_video);

    // Set up panic handler for better error reporting
    std::panic::set_hook(Box::new(|panic_info| {
        error!("Application panic: {panic_info}");
        eprintln!("Course Pilot encountered a critical error and must close.");
        eprintln!("Error details: {panic_info}");
    }));

    info!("Launching Dioxus desktop application");

    dioxus::LaunchBuilder::new().with_cfg(config).launch(AppRoot);
}

/// local video file handler using mime_guess
fn handle_local_video(
    _webview_id: dioxus_desktop::wry::WebViewId,
    request: dioxus_desktop::wry::http::Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let uri = request.uri().to_string();

    // Extract path from URI (strip protocol prefix)
    let path = uri
        .strip_prefix("local-video://file/")
        .or_else(|| uri.strip_prefix("local-video://"))
        .unwrap_or("");

    // Percent-decode the path for spaces and special characters
    let decoded_path = percent_decode(path);

    info!("Serving local video: {}", decoded_path);

    // Read the file
    match std::fs::read(&decoded_path) {
        Ok(bytes) => {
            // Use mime_guess for content type, fallback to video/mp4
            let mime = mime_guess::from_path(&decoded_path)
                .first()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "video/mp4".to_string());

            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", mime)
                .header("accept-ranges", "bytes")
                .body(Cow::Owned(bytes))
                .unwrap()
        },
        Err(e) => {
            warn!("Failed to read video file '{}': {}", decoded_path, e);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "text/plain")
                .body(Cow::Borrowed(b"Video file not found" as &[u8]))
                .unwrap()
        },
    }
}

/// Simple percent-decoding for URL paths
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h1), Some(h2)) =
                ((bytes[i + 1] as char).to_digit(16), (bytes[i + 2] as char).to_digit(16))
            {
                decoded.push(((h1 << 4) | h2) as u8);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }

    String::from_utf8(decoded).unwrap_or_else(|_| input.to_string())
}
