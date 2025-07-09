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
                .with_min_inner_size(LogicalSize::new(1024, 768))
        )
        .with_custom_index(r##"
            <!DOCTYPE html>
            <html data-theme="lofi" class="h-full">
            <head>
              <title>Course Pilot</title>
              <meta name="viewport" content="width=device-width, initial-scale=1.0" />
              <meta name="theme-color" content="#ffffff" />
              <link rel="stylesheet" href="assets/tailwind.out.css">
              <link data-hot-reload data-dioxus-hot-reload>
              <script>
                // Apply theme before the page loads to prevent flash of wrong theme
                (function() {
                  const theme = localStorage.getItem('theme');
                  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

                  if (theme) {
                    document.documentElement.setAttribute('data-theme', theme === '"' + 'Night' + '"' ? 'night' : 'lofi');
                  } else if (prefersDark) {
                    document.documentElement.setAttribute('data-theme', 'night');
                  }
                })();
              </script>
            </head>
            <body class="min-h-screen bg-base-100">
              <div id="main"></div>
            </body>
            </html>
        "##.to_string());

    LaunchBuilder::desktop().with_cfg(config).launch(AppRoot);
}
