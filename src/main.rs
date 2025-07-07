use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use dioxus_desktop::tao::dpi::LogicalSize;

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
        .with_custom_index(r#"
            <!DOCTYPE html>
            <html>
            <head>
              <title>Course Pilot</title>
              <meta name="viewport" content="width=device-width, initial-scale=1.0" />
              <style id="tailwind-css">
                /* CSS will be injected here by the build script */
              </style>
              <script>
                // Load CSS dynamically
                document.addEventListener('DOMContentLoaded', function() {
                  // Try to load from /assets first, then fall back to root
                  const cssPaths = [
                    '/assets/tailwind.out.css',
                    '/tailwind.out.css',
                    'tailwind.out.css',
                    './assets/tailwind.out.css',
                    './tailwind.out.css'
                  ];
                  
                  function tryLoadCSS(path, index) {
                    if (index >= cssPaths.length) {
                      console.error('Failed to load Tailwind CSS: All paths exhausted');
                      return;
                    }
                    
                    const link = document.createElement('link');
                    link.rel = 'stylesheet';
                    link.href = cssPaths[index];
                    link.onerror = () => tryLoadCSS(cssPaths, index + 1);
                    link.onload = () => console.log('Successfully loaded CSS from:', cssPaths[index]);
                    document.head.appendChild(link);
                  }
                  
                  tryLoadCSS(cssPaths, 0);
                });
              </script>
              <link data-hot-reload data-dioxus-hot-reload>
            </head>
            <body class="min-h-screen bg-base-100">
              <div id="main"></div>
            </body>
            </html>
        "#.to_string());

    LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(AppRoot);
}
