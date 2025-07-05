//! Application Theme
//
//! This module defines the visual theme for the application, including colors,
//! typography, and component styles.

use dioxus::prelude::*;

pub const PRIMARY_COLOR: &str = "#6200EE";
pub const SECONDARY_COLOR: &str = "#03DAC6";

#[component]
pub fn AppTheme() -> Element {
    rsx! {
        style {
            // Global styles
            "* {{ box-sizing: border-box; margin: 0; padding: 0; }}"
            "body {{ font-family: sans-serif; background-color: #f5f5f5; color: #333; }}"
        }
    }
}
