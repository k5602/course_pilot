use adw::prelude::*;

use course_pilot::ui::CoursePilotApp;

fn main() -> glib::ExitCode {
    dotenvy::dotenv().ok();
    env_logger::init();

    if rustls::crypto::ring::default_provider().install_default().is_err() {
        log::error!("Failed to install TLS provider");
        return glib::ExitCode::FAILURE;
    }

    log::info!("Starting Course Pilot");

    let app = adw::Application::builder().application_id("com.coursepilot.app").build();

    app.connect_activate(|app| {
        CoursePilotApp::activate(app);
    });

    app.run()
}
