//! Unified Theme System for Course Pilot
//!
//!
//! Features:
//! - Unified semantic color tokens
//! - Manual theme switching (overrides OS preference)
//! - Consistent design system across all components
//! - Component-specific theme tokens
//! - Comprehensive accessibility support

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

/// Base design tokens - comprehensive color palette
const BASE_DESIGN_TOKENS: &str = r#"
:root {
  /* === BASE COLOR PALETTE === */

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

  /* Blue Scale (Primary) */
  --blue-50: #eff6ff;
  --blue-100: #dbeafe;
  --blue-200: #bfdbfe;
  --blue-300: #93c5fd;
  --blue-400: #60a5fa;
  --blue-500: #3b82f6;
  --blue-600: #2563eb;
  --blue-700: #1d4ed8;
  --blue-800: #1e40af;
  --blue-900: #1e3a8a;

  /* Green Scale (Success) */
  --green-50: #f0fdf4;
  --green-100: #dcfce7;
  --green-200: #bbf7d0;
  --green-300: #86efac;
  --green-400: #4ade80;
  --green-500: #22c55e;
  --green-600: #16a34a;
  --green-700: #15803d;
  --green-800: #166534;
  --green-900: #14532d;

  /* Yellow Scale (Warning) */
  --yellow-50: #fefce8;
  --yellow-100: #fef3c7;
  --yellow-200: #fde047;
  --yellow-300: #facc15;
  --yellow-400: #eab308;
  --yellow-500: #ca8a04;
  --yellow-600: #a16207;
  --yellow-700: #854d0e;
  --yellow-800: #713f12;
  --yellow-900: #422006;

  /* Red Scale (Danger) */
  --red-50: #fef2f2;
  --red-100: #fee2e2;
  --red-200: #fecaca;
  --red-300: #fca5a5;
  --red-400: #f87171;
  --red-500: #ef4444;
  --red-600: #dc2626;
  --red-700: #b91c1c;
  --red-800: #991b1b;
  --red-900: #7f1d1d;

  /* === TYPOGRAPHY SCALE === */
  --text-xs: 0.75rem;      /* 12px */
  --text-sm: 0.875rem;     /* 14px */
  --text-base: 1rem;       /* 16px */
  --text-lg: 1.125rem;     /* 18px */
  --text-xl: 1.25rem;      /* 20px */
  --text-2xl: 1.5rem;      /* 24px */
  --text-3xl: 1.875rem;    /* 30px */
  --text-4xl: 2.25rem;     /* 36px */

  --font-light: 300;
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
  --font-extrabold: 800;

  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;

  /* === SPACING SCALE === */
  --space-0: 0;
  --space-1: 0.25rem;      /* 4px */
  --space-2: 0.5rem;       /* 8px */
  --space-3: 0.75rem;      /* 12px */
  --space-4: 1rem;         /* 16px */
  --space-5: 1.25rem;      /* 20px */
  --space-6: 1.5rem;       /* 24px */
  --space-8: 2rem;         /* 32px */
  --space-10: 2.5rem;      /* 40px */
  --space-12: 3rem;        /* 48px */
  --space-16: 4rem;        /* 64px */
  --space-20: 5rem;        /* 80px */
  --space-24: 6rem;        /* 96px */

  /* === RADIUS SCALE === */
  --radius-none: 0;
  --radius-sm: 0.25rem;    /* 4px */
  --radius-md: 0.5rem;     /* 8px */
  --radius-lg: 1rem;       /* 16px */
  --radius-xl: 1.5rem;     /* 24px */
  --radius-full: 9999px;

  /* === SHADOW SCALE === */
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1);
  --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  --shadow-2xl: 0 25px 50px -12px rgba(0, 0, 0, 0.25);

  /* === FOCUS RING === */
  --focus-ring-width: 2px;
  --focus-ring-offset: 2px;
}
"#;

/// Light theme semantic tokens
const LIGHT_THEME_TOKENS: &str = r#"
:root[data-theme="light"] {
  /* === SEMANTIC COLOR TOKENS === */

  /* Core Application Colors */
  --bg: var(--neutral-50);
  --bg-secondary: var(--neutral-100);
  --bg-tertiary: var(--neutral-200);
  --fg: var(--neutral-900);
  --fg-secondary: var(--neutral-600);
  --fg-tertiary: var(--neutral-500);
  --fg-inverse: var(--neutral-50);

  /* Interactive Colors */
  --primary: var(--blue-600);
  --primary-hover: var(--blue-700);
  --primary-active: var(--blue-800);
  --primary-light: var(--blue-100);
  --primary-text: var(--neutral-50);

  /* Status Colors */
  --success: var(--green-600);
  --success-light: var(--green-100);
  --success-text: var(--green-800);

  --warning: var(--yellow-600);
  --warning-light: var(--yellow-100);
  --warning-text: var(--yellow-800);

  --danger: var(--red-600);
  --danger-light: var(--red-100);
  --danger-text: var(--red-800);

  /* Text Colors */
  --text-primary: var(--neutral-900);
  --text-secondary: var(--neutral-600);
  --text-tertiary: var(--neutral-500);
  --text-inverse: var(--neutral-50);
  --text-muted: var(--neutral-400);

  /* Border Colors */
  --border: var(--neutral-200);
  --border-hover: var(--neutral-300);
  --border-active: var(--primary);

  /* Card Colors */
  --card-bg: var(--neutral-50);
  --card-border: var(--neutral-200);
  --card-shadow: var(--shadow-md);

  /* === LAYOUT SPECIFIC TOKENS === */

  /* Sidebar */
  --sidebar-bg: var(--neutral-900);
  --sidebar-fg: var(--neutral-50);
  --sidebar-active: var(--primary);
  --sidebar-hover: var(--neutral-800);

  /* App Bar */
  --appbar-bg: var(--neutral-50);
  --appbar-fg: var(--neutral-900);
  --appbar-border: var(--neutral-200);

  /* === COMPONENT SPECIFIC TOKENS === */

  /* Dashboard */
  --dashboard-bg: var(--bg);
  --dashboard-card-bg: var(--card-bg);
  --dashboard-card-border: var(--card-border);
  --dashboard-text-primary: var(--text-primary);
  --dashboard-text-secondary: var(--text-secondary);

  /* Plan View */
  --plan-bg: var(--bg);
  --plan-card: var(--card-bg);
  --plan-card-border: var(--card-border);
  --plan-text-primary: var(--text-primary);
  --plan-text-secondary: var(--text-secondary);
  --plan-text-inverse: var(--text-inverse);
  --plan-primary: var(--primary);
  --plan-danger: var(--danger);
  --plan-danger-light: var(--danger-light);
  --plan-success: var(--success);
  --plan-success-light: var(--success-light);

  /* Focus Ring */
  --focus-ring-color: var(--primary);
}
"#;

/// Dark theme semantic tokens
const DARK_THEME_TOKENS: &str = r#"
:root[data-theme="dark"] {
  /* === SEMANTIC COLOR TOKENS === */

  /* Core Application Colors */
  --bg: var(--neutral-950);
  --bg-secondary: var(--neutral-900);
  --bg-tertiary: var(--neutral-800);
  --fg: var(--neutral-50);
  --fg-secondary: var(--neutral-400);
  --fg-tertiary: var(--neutral-500);
  --fg-inverse: var(--neutral-950);

  /* Interactive Colors */
  --primary: var(--blue-500);
  --primary-hover: var(--blue-400);
  --primary-active: var(--blue-300);
  --primary-light: var(--blue-900);
  --primary-text: var(--neutral-950);

  /* Status Colors */
  --success: var(--green-500);
  --success-light: var(--green-900);
  --success-text: var(--green-200);

  --warning: var(--yellow-500);
  --warning-light: var(--yellow-900);
  --warning-text: var(--yellow-200);

  --danger: var(--red-500);
  --danger-light: var(--red-900);
  --danger-text: var(--red-200);

  /* Text Colors */
  --text-primary: var(--neutral-50);
  --text-secondary: var(--neutral-400);
  --text-tertiary: var(--neutral-500);
  --text-inverse: var(--neutral-950);
  --text-muted: var(--neutral-600);

  /* Border Colors */
  --border: var(--neutral-800);
  --border-hover: var(--neutral-700);
  --border-active: var(--primary);

  /* Card Colors */
  --card-bg: var(--neutral-900);
  --card-border: var(--neutral-800);
  --card-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3), 0 4px 6px -4px rgba(0, 0, 0, 0.3);

  /* === LAYOUT SPECIFIC TOKENS === */

  /* Sidebar */
  --sidebar-bg: var(--neutral-900);
  --sidebar-fg: var(--neutral-50);
  --sidebar-active: var(--primary);
  --sidebar-hover: var(--neutral-800);

  /* App Bar */
  --appbar-bg: var(--neutral-900);
  --appbar-fg: var(--neutral-50);
  --appbar-border: var(--neutral-800);

  /* === COMPONENT SPECIFIC TOKENS === */

  /* Dashboard */
  --dashboard-bg: var(--bg);
  --dashboard-card-bg: var(--card-bg);
  --dashboard-card-border: var(--card-border);
  --dashboard-text-primary: var(--text-primary);
  --dashboard-text-secondary: var(--text-secondary);

  /* Plan View */
  --plan-bg: var(--bg);
  --plan-card: var(--card-bg);
  --plan-card-border: var(--card-border);
  --plan-text-primary: var(--text-primary);
  --plan-text-secondary: var(--text-secondary);
  --plan-text-inverse: var(--text-inverse);
  --plan-primary: var(--primary);
  --plan-danger: var(--danger);
  --plan-danger-light: var(--danger-light);
  --plan-success: var(--success);
  --plan-success-light: var(--success-light);

  /* Focus Ring */
  --focus-ring-color: var(--primary);
}
"#;

/// Global theme application styles
const GLOBAL_THEME_STYLES: &str = r#"
/* === GLOBAL THEME APPLICATION === */

* {
  box-sizing: border-box;
}

html {
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif;
  line-height: var(--leading-normal);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  padding: 0;
  background: var(--bg);
  color: var(--fg);
  transition: background-color 0.2s ease, color 0.2s ease;
  min-height: 100vh;
}

/* === FOCUS STYLES === */
*:focus {
  outline: var(--focus-ring-width) solid var(--focus-ring-color);
  outline-offset: var(--focus-ring-offset);
}

*:focus:not(:focus-visible) {
  outline: none;
}

/* === SCROLLBAR THEMING === */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: var(--bg-secondary);
}

::-webkit-scrollbar-thumb {
  background: var(--border-hover);
  border-radius: var(--radius-full);
}

::-webkit-scrollbar-thumb:hover {
  background: var(--fg-tertiary);
}

/* === SELECTION THEMING === */
::selection {
  background: var(--primary-light);
  color: var(--text-primary);
}

/* === THEME TRANSITION === */
.theme-transition * {
  transition: background-color 0.2s ease, color 0.2s ease, border-color 0.2s ease !important;
}
"#;

/// Hook for accessing theme mode signal
pub fn use_theme() -> Signal<ThemeMode> {
    use_context::<Signal<ThemeMode>>()
}

/// Get theme CSS variables based on mode
pub fn get_theme_vars(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => LIGHT_THEME_VARS,
        ThemeMode::Dark => DARK_THEME_VARS,
    }
}

/// Light theme CSS variables
const LIGHT_THEME_VARS: &str = r#"
:root {
    /* Core Application Colors */
    --bg: #fafafa;
    --bg-secondary: #f4f4f5;
    --fg: #18181b;
    --fg-secondary: #52525b;
    --text-primary: #18181b;
    --text-secondary: #52525b;
    --text-inverse: #fafafa;

    /* Interactive Colors */
    --primary: #2563eb;
    --primary-hover: #1d4ed8;
    --success: #16a34a;
    --warning: #ca8a04;
    --danger: #dc2626;
    --danger-light: #fee2e2;

    /* Layout Colors */
    --sidebar-bg: #18181b;
    --sidebar-fg: #fafafa;
    --sidebar-active: #2563eb;
    --sidebar-hover: #27272a;
    --appbar-bg: #fafafa;
    --appbar-fg: #18181b;
    --appbar-border: #e4e4e7;

    /* Card Colors */
    --card-bg: #ffffff;
    --card-border: #e4e4e7;

    /* Plan View Colors */
    --plan-bg: #fafafa;
    --plan-card: #ffffff;
    --plan-text-primary: #18181b;
    --plan-text-secondary: #52525b;
    --plan-text-inverse: #fafafa;
    --plan-primary: #2563eb;
    --plan-danger: #dc2626;
    --plan-danger-light: #fee2e2;
    --plan-success: #16a34a;
    --plan-success-light: #dcfce7;

    /* Focus */
    --focus-ring-color: #2563eb;
}
"#;

/// Dark theme CSS variables
const DARK_THEME_VARS: &str = r#"
:root {
    /* Core Application Colors */
    --bg: #09090b;
    --bg-secondary: #18181b;
    --fg: #fafafa;
    --fg-secondary: #a1a1aa;
    --text-primary: #fafafa;
    --text-secondary: #a1a1aa;
    --text-inverse: #09090b;

    /* Interactive Colors */
    --primary: #3b82f6;
    --primary-hover: #60a5fa;
    --success: #22c55e;
    --warning: #eab308;
    --danger: #ef4444;
    --danger-light: #7f1d1d;

    /* Layout Colors */
    --sidebar-bg: #18181b;
    --sidebar-fg: #fafafa;
    --sidebar-active: #3b82f6;
    --sidebar-hover: #27272a;
    --appbar-bg: #18181b;
    --appbar-fg: #fafafa;
    --appbar-border: #27272a;

    /* Card Colors */
    --card-bg: #18181b;
    --card-border: #27272a;

    /* Plan View Colors */
    --plan-bg: #09090b;
    --plan-card: #18181b;
    --plan-text-primary: #fafafa;
    --plan-text-secondary: #a1a1aa;
    --plan-text-inverse: #09090b;
    --plan-primary: #3b82f6;
    --plan-danger: #ef4444;
    --plan-danger-light: #7f1d1d;
    --plan-success: #22c55e;
    --plan-success-light: #14532d;

    /* Focus */
    --focus-ring-color: #3b82f6;
}
"#;

/// Theme provider component that injects unified theme system
#[component]
pub fn UnifiedThemeProvider(children: Element) -> Element {
    let theme_mode = use_signal(|| ThemeMode::Light);
    use_context_provider(|| theme_mode);

    let current_mode = theme_mode.read().clone();
    let theme_vars = get_theme_vars(current_mode);

    rsx! {
        // Inject base design tokens
        style { dangerous_inner_html: BASE_DESIGN_TOKENS }
        // Inject current theme variables
        style { dangerous_inner_html: theme_vars }
        // Inject global styles
        style { dangerous_inner_html: GLOBAL_THEME_STYLES }

        {children}
    }
}

/// Legacy AppTheme component for backwards compatibility
#[component]
pub fn AppTheme(children: Element) -> Element {
    rsx! {
        UnifiedThemeProvider {
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

    rsx! {
        button {
            class: "theme-toggle-btn",
            onclick: move |_| {
                let new_mode = match *theme_mode.read() {
                    ThemeMode::Light => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::Light,
                };
                theme_mode.set(new_mode);
            },
            aria_label: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
            style: "
                background: none;
                border: none;
                color: inherit;
                cursor: pointer;
                font-size: 1.3rem;
                border-radius: var(--radius-full);
                width: 2.2rem;
                height: 2.2rem;
                display: flex;
                align-items: center;
                justify-content: center;
                transition: background-color 0.15s ease;
            ",
            onmouseenter: move |_| {},
            onmouseleave: move |_| {},

            if is_dark { "☀️" } else { "🌙" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_default() {
        assert_eq!(ThemeMode::default(), ThemeMode::Light);
    }

    #[test]
    fn test_theme_vars() {
        let light_vars = get_theme_vars(ThemeMode::Light);
        assert!(light_vars.contains("--bg: #fafafa"));

        let dark_vars = get_theme_vars(ThemeMode::Dark);
        assert!(dark_vars.contains("--bg: #09090b"));
    }
}
