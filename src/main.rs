use course_pilot::ui::app_root::AppRoot;
use dioxus::prelude::*;
use dioxus_desktop::tao::dpi::LogicalSize;
use dioxus_desktop::{Config, WindowBuilder};

const TAILWIND_CSS: &str = include_str!("../assets/tailwind.out.css");

fn main() {
    let config = Config::new().with_window(
        WindowBuilder::new()
            .with_title("Course Pilot")
            .with_inner_size(LogicalSize::new(1280, 800))
            .with_min_inner_size(LogicalSize::new(1024, 768)),
    );

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(AppRoot);
}
