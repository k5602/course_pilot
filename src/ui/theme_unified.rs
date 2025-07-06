//! Unified Theme System for Course Pilot
//!
//! This module provides a comprehensive, consistent theme system that replaces
//! the fragmented theme implementations. It includes:
//! - Unified CSS variable naming
//! - Consistent design tokens
//! - Proper light/dark theme support
//! - Component-specific theme tokens
//! - Accessibility-compliant color contrast

use dioxus::prelude::*;

/// Theme mode enumeration
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Light
    }
}

/// Core design tokens - the foundation of the design system
const DESIGN_TOKENS: &str = r#"
:root {
  /* === COLOR PALETTE === */

  /* Neutral Scale */
  --neutral-50: #fafafa;
  --neutral-100: #f4f4f5;
  --neutral-200: #e4e4e7;
  --neutral-300: #d4d4d8;
  --neutral-400: #a1a1aa;
  --neutral-500: #71717a;
  --neutral-600: #52525b;
  --neutral-700: #3f3f46;
  --neutral-800: #27272a;
  --neutral-900: #18181b;
  --neutral-950: #09090b;

  /* Primary Scale (Blue) */
  --primary-50: #eff6ff;
  --primary-100: #dbeafe;
  --primary-200: #bfdbfe;
  --primary-300: #93c5fd;
  --primary-400: #60a5fa;
  --primary-500: #3b82f6;
  --primary-600: #2563eb;
  --primary-700: #1d4ed8;
  --primary-800: #1e40af;
  --primary-900: #1e3a8a;

  /* Success Scale (Green) */
  --success-50: #f0fdf4;
  --success-100: #dcfce7;
  --success-200: #bbf7d0;
  --success-300: #86efac;
  --success-400: #4ade80;
  --success-500: #22c55e;
  --success-600: #16a34a;
  --success-700: #15803d;
  --success-800: #166534;
  --success-900: #14532d;

  /* Warning Scale (Yellow) */
  --warning-50: #fefce8;
  --warning-100: #fef3c7;
  --warning-200: #fde047;
  --warning-300: #facc15;
  --warning-400: #eab308;
  --warning-500: #ca8a04;
  --warning-600: #a16207;
  --warning-700: #854d0e;
  --warning-800: #713f12;
  --warning-900: #422006;

  /* Error Scale (Red) */
  --error-50: #fef2f2;
  --error-100: #fee2e2;
  --error-200: #fecaca;
  --error-300: #fca5a5;
  --error-400: #f87171;
  --error-500: #ef4444;
  --error-600: #dc2626;
  --error-700: #b91c1c;
  --error-800: #991b1b;
  --error-900: #7f1d1d;

  /* === TYPOGRAPHY SCALE === */
  --font-size-xs: 0.75rem;      /* 12px */
  --font-size-sm: 0.875rem;     /* 14px */
  --font-size-base: 1rem;       /* 16px */
  --font-size-lg: 1.125rem;     /* 18px */
  --font-size-xl: 1.25rem;      /* 20px */
  --font-size-2xl: 1.5rem;      /* 24px */
  --font-size-3xl: 1.875rem;    /* 30px */
  --font-size-4xl: 2.25rem;     /* 36px */

  --font-weight-light: 300;
  --font-weight-normal: 400;
  --font-weight-medium: 500;
  --font-weight-semibold: 600;
  --font-weight-bold: 700;

  --line-height-tight: 1.25;
  --line-height-normal: 1.5;
  --line-height-relaxed: 1.75;

  /* === SPACING SCALE === */
  --spacing-0: 0;
  --spacing-1: 0.25rem;      /* 4px */
  --spacing-2: 0.5rem;       /* 8px */
  --spacing-3: 0.75rem;      /* 12px */
  --spacing-4: 1rem;         /* 16px */
  --spacing-5: 1.25rem;      /* 20px */
  --spacing-6: 1.5rem;       /* 24px */
  --spacing-8: 2rem;         /* 32px */
  --spacing-10: 2.5rem;      /* 40px */
  --spacing-12: 3rem;        /* 48px */
  --spacing-16: 4rem;        /* 64px */
  --spacing-20: 5rem;        /* 80px */
  --spacing-24: 6rem;        /* 96px */

  /* === BORDER RADIUS === */
  --radius-none: 0;
  --radius-sm: 0.25rem;      /* 4px */
  --radius-md: 0.5rem;       /* 8px */
  --radius-lg: 1rem;         /* 16px */
  --radius-xl: 1.5rem;       /* 24px */
  --radius-full: 9999px;

  /* === SHADOWS === */
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1);
  --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);

  /* === TRANSITIONS === */
  --transition-fast: 0.15s ease;
  --transition-normal: 0.2s ease;
  --transition-slow: 0.3s ease;

  /* === BREAKPOINTS === */
  --breakpoint-sm: 640px;
  --breakpoint-md: 768px;
  --breakpoint-lg: 1024px;
  --breakpoint-xl: 1280px;
  --breakpoint-2xl: 1536px;
}
"#;

/// Light theme semantic tokens
const LIGHT_THEME: &str = r#"
:root[data-theme="light"] {
  /* === SEMANTIC COLORS === */

  /* Background Colors */
  --bg-primary: var(--neutral-50);
  --bg-secondary: var(--neutral-100);
  --bg-tertiary: var(--neutral-200);
  --bg-elevated: #ffffff;
  --bg-overlay: rgba(0, 0, 0, 0.5);

  /* Text Colors */
  --text-primary: var(--neutral-900);
  --text-secondary: var(--neutral-600);
  --text-tertiary: var(--neutral-500);
  --text-inverse: var(--neutral-50);
  --text-disabled: var(--neutral-400);

  /* Interactive Colors */
  --color-primary: var(--primary-600);
  --color-primary-hover: var(--primary-700);
  --color-primary-active: var(--primary-800);
  --color-primary-light: var(--primary-100);
  --color-primary-text: var(--neutral-50);

  --color-secondary: var(--neutral-600);
  --color-secondary-hover: var(--neutral-700);
  --color-secondary-active: var(--neutral-800);
  --color-secondary-light: var(--neutral-100);
  --color-secondary-text: var(--neutral-50);

  /* Status Colors */
  --color-success: var(--success-600);
  --color-success-hover: var(--success-700);
  --color-success-light: var(--success-100);
  --color-success-text: var(--success-800);

  --color-warning: var(--warning-600);
  --color-warning-hover: var(--warning-700);
  --color-warning-light: var(--warning-100);
  --color-warning-text: var(--warning-800);

  --color-error: var(--error-600);
  --color-error-hover: var(--error-700);
  --color-error-light: var(--error-100);
  --color-error-text: var(--error-800);

  /* Border Colors */
  --border-primary: var(--neutral-200);
  --border-secondary: var(--neutral-300);
  --border-focus: var(--primary-500);
  --border-error: var(--error-500);

  /* === COMPONENT SPECIFIC === */

  /* Button Colors */
  --btn-primary-bg: var(--color-primary);
  --btn-primary-bg-hover: var(--color-primary-hover);
  --btn-primary-text: var(--color-primary-text);
  --btn-primary-border: var(--color-primary);

  --btn-secondary-bg: var(--bg-elevated);
  --btn-secondary-bg-hover: var(--bg-secondary);
  --btn-secondary-text: var(--color-secondary);
  --btn-secondary-border: var(--border-primary);

  --btn-outline-bg: transparent;
  --btn-outline-bg-hover: var(--color-primary-light);
  --btn-outline-text: var(--color-primary);
  --btn-outline-border: var(--color-primary);

  --btn-ghost-bg: transparent;
  --btn-ghost-bg-hover: var(--bg-secondary);
  --btn-ghost-text: var(--color-secondary);
  --btn-ghost-border: transparent;

  --btn-destructive-bg: var(--color-error);
  --btn-destructive-bg-hover: var(--color-error-hover);
  --btn-destructive-text: var(--neutral-50);
  --btn-destructive-border: var(--color-error);

  /* Card Colors */
  --card-bg: var(--bg-elevated);
  --card-border: var(--border-primary);
  --card-shadow: var(--shadow-md);

  /* Navigation Colors */
  --nav-bg: var(--neutral-900);
  --nav-text: var(--neutral-50);
  --nav-text-secondary: var(--neutral-400);
  --nav-item-hover: var(--neutral-800);
  --nav-item-active: var(--color-primary);

  /* App Bar Colors */
  --appbar-bg: var(--bg-elevated);
  --appbar-text: var(--text-primary);
  --appbar-border: var(--border-primary);

  /* Input Colors */
  --input-bg: var(--bg-elevated);
  --input-border: var(--border-primary);
  --input-border-focus: var(--border-focus);
  --input-text: var(--text-primary);
  --input-placeholder: var(--text-tertiary);

  /* Focus Ring */
  --focus-ring: 0 0 0 2px var(--color-primary-light), 0 0 0 4px var(--color-primary);
}
"#;

/// Dark theme semantic tokens
const DARK_THEME: &str = r#"
:root[data-theme="dark"] {
  /* === SEMANTIC COLORS === */

  /* Background Colors */
  --bg-primary: var(--neutral-950);
  --bg-secondary: var(--neutral-900);
  --bg-tertiary: var(--neutral-800);
  --bg-elevated: var(--neutral-900);
  --bg-overlay: rgba(0, 0, 0, 0.7);

  /* Text Colors */
  --text-primary: var(--neutral-50);
  --text-secondary: var(--neutral-400);
  --text-tertiary: var(--neutral-500);
  --text-inverse: var(--neutral-950);
  --text-disabled: var(--neutral-600);

  /* Interactive Colors */
  --color-primary: var(--primary-500);
  --color-primary-hover: var(--primary-400);
  --color-primary-active: var(--primary-300);
  --color-primary-light: var(--primary-900);
  --color-primary-text: var(--neutral-950);

  --color-secondary: var(--neutral-400);
  --color-secondary-hover: var(--neutral-300);
  --color-secondary-active: var(--neutral-200);
  --color-secondary-light: var(--neutral-800);
  --color-secondary-text: var(--neutral-950);

  /* Status Colors */
  --color-success: var(--success-500);
  --color-success-hover: var(--success-400);
  --color-success-light: var(--success-900);
  --color-success-text: var(--success-200);

  --color-warning: var(--warning-500);
  --color-warning-hover: var(--warning-400);
  --color-warning-light: var(--warning-900);
  --color-warning-text: var(--warning-200);

  --color-error: var(--error-500);
  --color-error-hover: var(--error-400);
  --color-error-light: var(--error-900);
  --color-error-text: var(--error-200);

  /* Border Colors */
  --border-primary: var(--neutral-800);
  --border-secondary: var(--neutral-700);
  --border-focus: var(--primary-400);
  --border-error: var(--error-400);

  /* === COMPONENT SPECIFIC === */

  /* Button Colors */
  --btn-primary-bg: var(--color-primary);
  --btn-primary-bg-hover: var(--color-primary-hover);
  --btn-primary-text: var(--color-primary-text);
  --btn-primary-border: var(--color-primary);

  --btn-secondary-bg: var(--bg-elevated);
  --btn-secondary-bg-hover: var(--bg-tertiary);
  --btn-secondary-text: var(--color-secondary);
  --btn-secondary-border: var(--border-primary);

  --btn-outline-bg: transparent;
  --btn-outline-bg-hover: var(--color-primary-light);
  --btn-outline-text: var(--color-primary);
  --btn-outline-border: var(--color-primary);

  --btn-ghost-bg: transparent;
  --btn-ghost-bg-hover: var(--bg-tertiary);
  --btn-ghost-text: var(--color-secondary);
  --btn-ghost-border: transparent;

  --btn-destructive-bg: var(--color-error);
  --btn-destructive-bg-hover: var(--color-error-hover);
  --btn-destructive-text: var(--neutral-50);
  --btn-destructive-border: var(--color-error);

  /* Card Colors */
  --card-bg: var(--bg-elevated);
  --card-border: var(--border-primary);
  --card-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3), 0 4px 6px -4px rgba(0, 0, 0, 0.3);

  /* Navigation Colors */
  --nav-bg: var(--neutral-900);
  --nav-text: var(--neutral-50);
  --nav-text-secondary: var(--neutral-400);
  --nav-item-hover: var(--neutral-800);
  --nav-item-active: var(--color-primary);

  /* App Bar Colors */
  --appbar-bg: var(--bg-elevated);
  --appbar-text: var(--text-primary);
  --appbar-border: var(--border-primary);

  /* Input Colors */
  --input-bg: var(--bg-elevated);
  --input-border: var(--border-primary);
  --input-border-focus: var(--border-focus);
  --input-text: var(--text-primary);
  --input-placeholder: var(--text-tertiary);

  /* Focus Ring */
  --focus-ring: 0 0 0 2px var(--color-primary-light), 0 0 0 4px var(--color-primary);
}
"#;

/// Global styles that apply to all themes
const GLOBAL_STYLES: &str = r#"
/* === GLOBAL STYLES === */

*,
*::before,
*::after {
  box-sizing: border-box;
}

html {
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif;
  line-height: var(--line-height-normal);
  font-size: var(--font-size-base);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  padding: 0;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  transition: background-color var(--transition-normal), color var(--transition-normal);
  min-height: 100vh;
}

/* === FOCUS MANAGEMENT === */
:focus {
  outline: none;
  box-shadow: var(--focus-ring);
}

:focus:not(:focus-visible) {
  box-shadow: none;
}

:focus-visible {
  box-shadow: var(--focus-ring);
}

/* === SELECTION === */
::selection {
  background-color: var(--color-primary-light);
  color: var(--text-primary);
}

/* === SCROLLBARS === */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: var(--bg-secondary);
}

::-webkit-scrollbar-thumb {
  background: var(--border-secondary);
  border-radius: var(--radius-full);
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* === REDUCED MOTION === */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}

/* === THEME TRANSITION === */
.theme-transition * {
  transition:
    background-color var(--transition-normal),
    color var(--transition-normal),
    border-color var(--transition-normal) !important;
}

/* === UTILITY CLASSES === */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.focus-visible-only {
  opacity: 0;
  pointer-events: none;
}

.focus-visible-only:focus-visible {
  opacity: 1;
  pointer-events: auto;
}
"#;

/// Hook for accessing theme mode signal
pub fn use_theme() -> Signal<ThemeMode> {
    use_context::<Signal<ThemeMode>>()
}

/// Theme provider component with unified theme system
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    let theme_mode = use_signal(|| ThemeMode::Light);
    use_context_provider(|| theme_mode);

    let current_mode = theme_mode.read().clone();
    let theme_data_attr = match current_mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    };

    // Apply theme to document root
    use_effect(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(html) = document.document_element() {
                    let _ = html.set_attribute("data-theme", theme_data_attr);
                }
            }
        }
    });

    rsx! {
        // Inject design tokens
        style { dangerous_inner_html: DESIGN_TOKENS }
        // Inject theme-specific styles
        style { dangerous_inner_html: LIGHT_THEME }
        style { dangerous_inner_html: DARK_THEME }
        // Inject global styles
        style { dangerous_inner_html: GLOBAL_STYLES }

        div {
            class: "theme-transition",
            {children}
        }
    }
}

/// Theme toggle button component
#[component]
pub fn ThemeToggle() -> Element {
    let mut theme_mode = use_theme();
    let current_mode = theme_mode.read().clone();
    let is_dark = matches!(current_mode, ThemeMode::Dark);

    let toggle_theme = move |_| {
        let new_mode = match *theme_mode.read() {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        theme_mode.set(new_mode);
    };

    rsx! {
        button {
            class: "theme-toggle",
            onclick: toggle_theme,
            aria_label: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
            title: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
            style: "
                background: var(--btn-ghost-bg);
                border: 1px solid var(--btn-ghost-border);
                color: var(--btn-ghost-text);
                cursor: pointer;
                font-size: 1.2rem;
                border-radius: var(--radius-md);
                width: 2.5rem;
                height: 2.5rem;
                display: flex;
                align-items: center;
                justify-content: center;
                transition: all var(--transition-fast);
            ",
            onmouseenter: |_| {},
            onmouseleave: |_| {},

            if is_dark { "☀️" } else { "🌙" }
        }

        style {{ "
            .theme-toggle:hover {{
                background: var(--btn-ghost-bg-hover);
                transform: scale(1.05);
            }}

            .theme-toggle:active {{
                transform: scale(0.95);
            }}
        " }}
    }
}

/// Utility function to get current theme mode
pub fn get_theme_mode(theme: &Signal<ThemeMode>) -> ThemeMode {
    *theme.read()
}

/// Utility function to check if current theme is dark
pub fn is_dark_theme(theme: &Signal<ThemeMode>) -> bool {
    matches!(*theme.read(), ThemeMode::Dark)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_default() {
        assert_eq!(ThemeMode::default(), ThemeMode::Light);
    }

    #[test]
    fn test_theme_mode_toggle() {
        let light = ThemeMode::Light;
        let dark = ThemeMode::Dark;

        assert_ne!(light, dark);
        assert_eq!(light, ThemeMode::Light);
        assert_eq!(dark, ThemeMode::Dark);
    }

    #[test]
    fn test_theme_constants_not_empty() {
        assert!(!DESIGN_TOKENS.is_empty());
        assert!(!LIGHT_THEME.is_empty());
        assert!(!DARK_THEME.is_empty());
        assert!(!GLOBAL_STYLES.is_empty());
    }
}
