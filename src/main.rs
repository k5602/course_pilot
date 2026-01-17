//! Course Pilot - Local-First Learning Sanctuary
//!
//! Transforms YouTube playlists into structured study plans.

use course_pilot::ui::App;

fn main() {
    // Install rustls crypto provider (required for TLS)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Load environment from .env file if present
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    log::info!("Starting Course Pilot Desktop...");

    // Launch Dioxus desktop app
    dioxus::launch(App);
}
