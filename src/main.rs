use dioxus::prelude::*;
use dioxus_desktop::tao::dpi::LogicalSize;
use dioxus_desktop::{Config, WindowBuilder};

mod ui;
use ui::app_root::AppRoot;

fn main() {
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Course Pilot")
                .with_inner_size(LogicalSize::new(1280, 800))
                .with_min_inner_size(LogicalSize::new(1024, 768)),
        )
        .with_custom_index(
            r##"
            <!DOCTYPE html>
            <!-- Set a default theme to prevent flashing on startup -->
            <!-- This will be immediately updated by the theme loaded from storage in AppRoot -->
            <html class="h-full" data-theme="corporate">
            <head>
              <title>Course Pilot</title>
              <meta name="viewport" content="width=device-width, initial-scale=1.0" />
              <meta name="theme-color" content="#ffffff" />
              <link rel="stylesheet" href="assets/tailwind.out.css">
              <link data-hot-reload data-dioxus-hot-reload>
            </head>
            <body class="min-h-screen bg-base-100">
              <div id="main"></div>
            </body>
            </html>
        "##
            .to_string(),
        );

    LaunchBuilder::desktop().with_cfg(config).launch(AppRoot);
}
