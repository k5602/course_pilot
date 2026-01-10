//! Course Pilot - Local-First Learning Sanctuary
//!
//! Transforms YouTube playlists into structured study plans.

use course_pilot::application::{AppConfig, AppContext};

#[tokio::main]
async fn main() {
    // Load environment from .env file if present
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    log::info!("Starting Course Pilot...");

    // Load configuration from environment
    let config = AppConfig::from_env();

    match AppContext::new(config) {
        Ok(ctx) => {
            log::info!("Course Pilot initialized successfully!");
            log::info!("  Database: {}", ctx.config.database_url);
            log::info!(
                "  YouTube: {}",
                if ctx.has_youtube() { "✓" } else { "✗ (set YOUTUBE_API_KEY)" }
            );
            log::info!("  Gemini: {}", if ctx.has_llm() { "✓" } else { "✗ (set GEMINI_API_KEY)" });
            log::info!(
                "  ML Boundary Detection: {}",
                if ctx.ml_enabled() {
                    if ctx.has_embedder() { "✓ enabled" } else { "⚠ enabled but model failed" }
                } else {
                    "disabled "
                }
            );

            // TODO: Launch Dioxus UI here
            // dioxus::launch(App);

            log::info!("Backend ready. UI integration pending.");
        },
        Err(e) => {
            log::error!("Failed to initialize: {}", e);
            std::process::exit(1);
        },
    }
}
