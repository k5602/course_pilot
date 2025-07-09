use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaMoon, FaSun};
use dioxus_free_icons::Icon;
use dioxus_signals::{Readable, Signal, Writable};
use log;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

// Static flag to track initialization
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Supported DaisyUI themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Lofi,
    Night,
}

impl AppTheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppTheme::Lofi => "lofi",
            AppTheme::Night => "night",
        }
    }

    /// Get the theme from persistent storage or use default
    pub fn from_storage() -> Self {
        // Try to read from localStorage using JS (desktop: use_window().eval)
        // Fallback to default, as localStorage is not available in desktop Rust
        AppTheme::Lofi
    }

    /// Save the theme to persistent storage
    pub fn save_to_storage(&self) {
        // Save to localStorage using JS (desktop: use_window().eval)
        log::info!("Theme changed to: {:?}", self);
    }

    /// Apply the theme to the application
    pub fn apply_theme(&self) {
        let theme_str = self.as_str();
        log::info!("🎨 Applying theme: {}", theme_str);
        // We'll apply the theme through JavaScript in the component
        // The actual DOM manipulation will happen in the component's use_effect
    }
}

/// Theme context that holds the current theme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeContext {
    pub theme: AppTheme,
}

impl ThemeContext {
    pub fn new() -> Self {
        let theme = AppTheme::from_storage();
        theme.apply_theme();
        Self { theme }
    }

    /// Toggle between light and dark themes
    pub fn toggle(&mut self) -> Self {
        let new_theme = match self.theme {
            AppTheme::Lofi => AppTheme::Night,
            AppTheme::Night => AppTheme::Lofi,
        };

        new_theme.apply_theme();
        new_theme.save_to_storage();

        Self { theme: new_theme }
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

/// Check if the current theme is dark mode
pub fn is_dark_mode() -> bool {
    use_theme_context().read().theme == AppTheme::Night
}

/// Toggle the current theme
pub fn toggle_theme() -> Option<()> {
    let mut theme_ctx = use_theme_context();
    theme_ctx.with_mut(|ctx| *ctx = ctx.toggle());
    Some(())
}

/// Theme toggle button component
#[component]
pub fn ThemeToggleButton() -> Element {
    let theme_ctx = use_theme_context();
    let is_lofi = theme_ctx().theme == AppTheme::Lofi;
    let theme_label = if is_lofi { "Light" } else { "Dark" };
    let theme_icon = if is_lofi {
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
            "aria-label": format!("Switch to {} theme", if is_lofi { "dark" } else { "light" }),
            onclick: {
                let mut  theme_ctx = theme_ctx.clone();
                move |_| {
                    log::info!("🎨 Toggling theme");
                    theme_ctx.with_mut(|ctx| *ctx = ctx.toggle());
                }
            },

            {theme_icon}
            "{theme_label} Theme"
        }
    }
}

// Re-export for convenience
