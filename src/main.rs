//! Course Pilot - Local-First Learning Sanctuary
//!
//! Transforms YouTube playlists into structured study plans.

use course_pilot::ui::App;
use dioxus_desktop::{Config, WindowBuilder};

fn main() {
    // Install rustls crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Load environment from .env file if present
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    log::info!("Starting Course Pilot Desktop...");

    // Launch Dioxus desktop app.
    let csp = r#"<meta http-equiv="Content-Security-Policy" content="default-src 'self' dioxus: data: blob:; script-src 'self' 'unsafe-inline' 'unsafe-eval' dioxus: https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline' dioxus: data: https://fonts.googleapis.com https://cdn.jsdelivr.net; font-src 'self' data: dioxus: https://fonts.gstatic.com https://cdn.jsdelivr.net; img-src 'self' data: dioxus: https://i.ytimg.com; frame-src https://www.youtube.com https://www.youtube-nocookie.com http://127.0.0.1:*; connect-src 'self' dioxus: http://127.0.0.1:* ws://127.0.0.1:* https://www.youtube.com https://www.youtube-nocookie.com; media-src https://www.youtube.com https://www.youtube-nocookie.com;">"#;
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(WindowBuilder::new().with_title("Course Pilot"))
                .with_navigation_handler(|url| {
                    url.starts_with("http://127.0.0.1:")
                        || url.starts_with("http://localhost:")
                        || url.starts_with("https://www.youtube.com/")
                        || url.starts_with("https://youtube.com/")
                        || url.starts_with("https://m.youtube.com/")
                        || url.starts_with("https://www.youtube-nocookie.com/")
                })
                .with_custom_head(csp.to_string()),
        )
        .launch(App);
}
