use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaMoon, FaSun};

/// Supported DaisyUI themes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn toggle(&self) -> AppTheme {
        match self {
            AppTheme::Lofi => AppTheme::Night,
            AppTheme::Night => AppTheme::Lofi,
        }
    }
}

/// Context for theme management
#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: Signal<AppTheme>,
}

/// Get the theme context
pub fn use_theme_context() -> ThemeContext {
    use_context::<ThemeContext>()
}

/// Call this at the root of your app to provide theme context
pub fn provide_theme_context() {
    use_context_provider(|| ThemeContext {
        theme: Signal::new(AppTheme::Lofi),
    });
}

/// Theme toggle button (example usage)
#[component]
pub fn ThemeToggleButton() -> Element {
    let mut theme_ctx = use_theme_context();
    let is_lofi = *theme_ctx.theme.read() == AppTheme::Lofi;

    rsx! {
        button {
            class: "btn btn-ghost btn-sm flex items-center gap-2",
            onclick: move |_| {
                theme_ctx.theme.with_mut(|theme| {
            *theme = theme.toggle();
            log::info!("🎨 Theme toggled to: {}", theme.as_str());
        });
            },

            if is_lofi {
                Icon { icon: FaSun, class: "w-5 h-5" }
                "Light Theme"
            } else {
                Icon { icon: FaMoon, class: "w-5 h-5" }
                "Dark Theme"
            }
        }
    }
}
