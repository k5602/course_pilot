use course_pilot::app::initialize_app;
use course_pilot::ui::app_root::AppRoot;
use course_pilot::video_player::protocol::handle_video_request;
use dioxus_desktop::tao::dpi::LogicalSize;
use dioxus_desktop::{Config, WindowBuilder};
use log::{error, info};

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
            info!("Video protocol request: {}", request.uri());
            handle_video_request(&request.uri().to_string(), request.headers())
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
