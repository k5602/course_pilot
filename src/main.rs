use course_pilot::app::{App, initialize_app};
use dioxus::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the application
    initialize_app()?;

    // Launch the Dioxus desktop application
    LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Course Pilot")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                    .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0)),
            ),
        )
        .launch(App);

    Ok(())
}
