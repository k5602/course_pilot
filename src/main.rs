//! Course Pilot - Local-First Learning Sanctuary
//!
//! Transforms YouTube playlists into structured study plans.

use dioxus::prelude::*;

use course_pilot::ui::App;

fn main() {
    // Load environment from .env file if present
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    log::info!("Starting Course Pilot Desktop...");

    // Launch Dioxus desktop app
    dioxus::launch(App);
}
