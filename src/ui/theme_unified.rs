use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaMoon, FaSun};
use dioxus_free_icons::Icon;
use dioxus_signals::{Readable, Signal, Writable};
use log;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const THEME_CONFIG_FILE: &str = "theme_config.toml";

/// Supported DaisyUI themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Corporate,
    Business,
}

impl AppTheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppTheme::Corporate => "corporate",
            AppTheme::Business => "business",
        }
    }

    /// Create a theme from a string slice
    pub fn from_str(s: &str) -> Self {
        match s {
            "business" => AppTheme::Business,
            _ => AppTheme::Corporate, // Default to Corporate
        }
    }

    /// Get the theme from persistent storage or use default
    pub fn from_storage() -> Self {
        let config_path = PathBuf::from(THEME_CONFIG_FILE);
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if contents.trim() == "business" {
                return AppTheme::Business;
            }
        }
        // Default to Corporate if the file doesn't exist or contains other values
        AppTheme::Corporate
    }

    /// Save the theme to persistent storage
    pub fn save_to_storage(&self) {
        let config_path = PathBuf::from(THEME_CONFIG_FILE);
        if let Err(e) = fs::write(&config_path, self.as_str()) {
            log::error!("Failed to save theme to config file: {}", e);
        } else {
            log::info!("Theme saved to storage: {:?}", self);
        }
    }
}

/// Theme context that holds the current theme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeContext {
    pub theme: AppTheme,
}

impl ThemeContext {
    pub fn new() -> Self {
        Self {
            theme: AppTheme::from_storage(),
        }
    }

    /// Toggle between light and dark themes and save the new state
    pub fn toggle(&mut self) {
        self.theme = match self.theme {
            AppTheme::Corporate => AppTheme::Business,
            AppTheme::Business => AppTheme::Corporate,
        };
        self.theme.save_to_storage();
    }
}

impl Default for ThemeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook to use the theme context
pub fn use_theme_context() -> Signal<ThemeContext> {
    use_context()
}

/// Theme toggle button component
#[component]
pub fn ThemeToggleButton(icon_only: bool) -> Element {
    let mut theme_ctx = use_theme_context();
    let is_corporate = theme_ctx.read().theme == AppTheme::Corporate;
    let theme_label = if is_corporate { "Light" } else { "Dark" };
    let theme_icon = if is_corporate {
        rsx!(Icon {
            icon: FaSun,
            class: "w-5 h-5"
        })
    } else {
        rsx!(Icon {
            icon: FaMoon,
            class: "w-5 h-5"
        })
    };

    rsx! {
        button {
            class: "btn btn-ghost btn-sm flex items-center gap-2",
            "aria-label": format!("Switch to {} theme", if is_corporate { "dark" } else { "light" }),
            onclick: move |_| {
                log::info!("🎨 Toggling theme via button click");
                theme_ctx.write().toggle();
            },

            {theme_icon}
            if !icon_only {
                span {
                    class: "sr-only",
                    "{theme_label} Theme"
                }
            }
        }
    }
}
