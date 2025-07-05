//! Application Theme
//
//! This module defines the visual theme for the application, including colors,
//! typography, and component styles.

use dioxus::prelude::*;

/// AppTheme injects a comprehensive design token system as CSS custom properties.
const THEME_LIGHT: &str = r#"
:root {
  /* Neutral Grays */
  --color-gray-50: #f9fafb;
  --color-gray-100: #f3f4f6;
  --color-gray-200: #e5e7eb;
  --color-gray-300: #d1d5db;
  --color-gray-400: #9ca3af;
  --color-gray-500: #6b7280;
  --color-gray-600: #4b5563;
  --color-gray-700: #374151;
  --color-gray-800: #1f2937;
  --color-gray-900: #111827;
  --color-white: #fff;
  --color-black: #000;

  /* Dashboard palette mapping */
  --color-primary-100: #dbeafe;
  --color-primary-600: #2563eb;
  --color-primary-700: #1d4ed8;
  --color-success-100: #d1fae5;
  --color-success-600: #059669;
  --color-warning-100: #fef3c7;
  --color-warning-600: #d97706;
  --color-danger-100: #fee2e2;
  --color-danger-600: #dc2626;

  /* Typography Scale */
  --text-xs: 0.75rem;
  --text-sm: 0.875rem;
  --text-base: 1rem;
  --text-lg: 1.125rem;
  --text-xl: 1.25rem;
  --text-2xl: 1.5rem;
  --text-3xl: 1.875rem;
  --text-4xl: 2.25rem;

  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;

  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;

  /* Spacing Scale */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;
  --space-5: 1.25rem;
  --space-6: 1.5rem;
  --space-8: 2rem;
  --space-10: 2.5rem;
  --space-12: 3rem;
  --space-16: 4rem;

  /* Component Tokens */
  --focus-ring-color: var(--color-primary-600);
  --focus-ring-width: 2px;
  --focus-ring-offset: 2px;

  --shadow-sm: 0 1px 2px 0 rgba(0,0,0,0.05);
  --shadow-md: 0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1);

  --radius-sm: 0.25rem;
  --radius-md: 0.5rem;
  --radius-lg: 1rem;
}
"#;
const THEME_DARK: &str = r#"
@media (prefers-color-scheme: dark) {
  :root {
    --color-gray-50: #18181b;
    --color-gray-100: #27272a;
    --color-gray-200: #3f3f46;
    --color-gray-300: #52525b;
    --color-gray-400: #71717a;
    --color-gray-500: #a1a1aa;
    --color-gray-600: #d4d4d8;
    --color-gray-700: #e4e4e7;
    --color-gray-800: #f4f4f5;
    --color-gray-900: #fafafa;
    --color-white: #18181b;
    --color-black: #fff;

    /* Dashboard palette mapping */
    --color-primary-100: #bfdbfe;
    --color-primary-600: #60a5fa;
    --color-primary-700: #3b82f6;
    --color-success-100: #bbf7d0;
    --color-success-600: #22c55e;
    --color-warning-100: #fde68a;
    --color-warning-600: #fbbf24;
    --color-danger-100: #fecaca;
    --color-danger-600: #ef4444;

    /* Typography Scale */
    --text-xs: 0.75rem;
    --text-sm: 0.875rem;
    --text-base: 1rem;
    --text-lg: 1.125rem;
    --text-xl: 1.25rem;
    --text-2xl: 1.5rem;
    --text-3xl: 1.875rem;
    --text-4xl: 2.25rem;

    --font-normal: 400;
    --font-medium: 500;
    --font-semibold: 600;
    --font-bold: 700;

    --leading-tight: 1.25;
    --leading-normal: 1.5;
    --leading-relaxed: 1.75;

    /* Spacing Scale */
    --space-1: 0.25rem;
    --space-2: 0.5rem;
    --space-3: 0.75rem;
    --space-4: 1rem;
    --space-5: 1.25rem;
    --space-6: 1.5rem;
    --space-8: 2rem;
    --space-10: 2.5rem;
    --space-12: 3rem;
    --space-16: 4rem;

    /* Component Tokens */
    --focus-ring-color: var(--color-primary-600);
    --focus-ring-width: 2px;
    --focus-ring-offset: 2px;

    --shadow-sm: 0 1px 2px 0 rgba(0,0,0,0.10);
    --shadow-md: 0 4px 6px -1px rgba(0,0,0,0.18), 0 2px 4px -2px rgba(0,0,0,0.14);
    --shadow-lg: 0 10px 15px -3px rgba(0,0,0,0.18), 0 4px 6px -4px rgba(0,0,0,0.14);

    --radius-sm: 0.25rem;
    --radius-md: 0.5rem;
    --radius-lg: 1rem;
  }
}
"#;

#[component]
pub fn AppTheme(children: Element) -> Element {
    rsx! {
        style { dangerous_inner_html: THEME_LIGHT }
        style { dangerous_inner_html: THEME_DARK }
        {children}
    }
}
