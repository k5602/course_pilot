//! Application Theme
//
//! This module defines the visual theme for the application, including colors,
//! typography, and component styles.

use dioxus::prelude::*;

/// AppTheme injects a comprehensive design token system as CSS custom properties.
const THEME_LIGHT: &str = r#":root {
  /* Neutral Grays (WCAG AA compliant) */
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
  --color-gray-950: #030712;

  /* Semantic Colors */
  --color-success-50: #ecfdf5;
  --color-success-100: #d1fae5;
  --color-success-200: #a7f3d0;
  --color-success-300: #6ee7b7;
  --color-success-400: #34d399;
  --color-success-500: #10b981;
  --color-success-600: #059669;
  --color-success-700: #047857;
  --color-success-800: #065f46;
  --color-success-900: #064e3b;

  --color-warning-50: #fffbeb;
  --color-warning-100: #fef3c7;
  --color-warning-200: #fde68a;
  --color-warning-300: #fcd34d;
  --color-warning-400: #fbbf24;
  --color-warning-500: #f59e42;
  --color-warning-600: #d97706;
  --color-warning-700: #b45309;
  --color-warning-800: #92400e;
  --color-warning-900: #78350f;

  --color-danger-50: #fef2f2;
  --color-danger-100: #fee2e2;
  --color-danger-200: #fecaca;
  --color-danger-300: #fca5a5;
  --color-danger-400: #f87171;
  --color-danger-500: #ef4444;
  --color-danger-600: #dc2626;
  --color-danger-700: #b91c1c;
  --color-danger-800: #991b1b;
  --color-danger-900: #7f1d1d;

  --color-info-50: #eff6ff;
  --color-info-100: #dbeafe;
  --color-info-200: #bfdbfe;
  --color-info-300: #93c5fd;
  --color-info-400: #60a5fa;
  --color-info-500: #3b82f6;
  --color-info-600: #2563eb;
  --color-info-700: #1d4ed8;
  --color-info-800: #1e40af;
  --color-info-900: #1e3a8a;

  --color-primary-50: #eff6ff;
  --color-primary-100: #dbeafe;
  --color-primary-200: #bfdbfe;
  --color-primary-300: #93c5fd;
  --color-primary-400: #60a5fa;
  --color-primary-500: #3b82f6;
  --color-primary-600: #2563eb;
  --color-primary-700: #1d4ed8;
  --color-primary-800: #1e40af;
  --color-primary-900: #1e3a8a;

  --color-white: #fff;
  --color-black: #000;

  /* Typography Scale (modular) */
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

  /* Spacing Scale (4px base) */
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
  --focus-ring-color: var(--color-primary-500);
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
const THEME_DARK: &str = r#"@media (prefers-color-scheme: dark) {
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
    --color-gray-950: #fff;

    --color-success-50: #052e16;
    --color-success-100: #14532d;
    --color-success-200: #166534;
    --color-success-300: #15803d;
    --color-success-400: #22c55e;
    --color-success-500: #4ade80;
    --color-success-600: #86efac;
    --color-success-700: #bbf7d0;
    --color-success-800: #dcfce7;
    --color-success-900: #f0fdf4;

    --color-warning-50: #78350f;
    --color-warning-100: #92400e;
    --color-warning-200: #b45309;
    --color-warning-300: #d97706;
    --color-warning-400: #f59e42;
    --color-warning-500: #fbbf24;
    --color-warning-600: #fde68a;
    --color-warning-700: #fef3c7;
    --color-warning-800: #fffbeb;
    --color-warning-900: #fff9db;

    --color-danger-50: #7f1d1d;
    --color-danger-100: #991b1b;
    --color-danger-200: #b91c1c;
    --color-danger-300: #dc2626;
    --color-danger-400: #ef4444;
    --color-danger-500: #f87171;
    --color-danger-600: #fca5a5;
    --color-danger-700: #fecaca;
    --color-danger-800: #fee2e2;
    --color-danger-900: #fef2f2;

    --color-info-50: #1e3a8a;
    --color-info-100: #1e40af;
    --color-info-200: #1d4ed8;
    --color-info-300: #2563eb;
    --color-info-400: #3b82f6;
    --color-info-500: #60a5fa;
    --color-info-600: #93c5fd;
    --color-info-700: #bfdbfe;
    --color-info-800: #dbeafe;
    --color-info-900: #eff6ff;

    --color-primary-50: #1e3a8a;
    --color-primary-100: #1e40af;
    --color-primary-200: #1d4ed8;
    --color-primary-300: #2563eb;
    --color-primary-400: #3b82f6;
    --color-primary-500: #60a5fa;
    --color-primary-600: #93c5fd;
    --color-primary-700: #bfdbfe;
    --color-primary-800: #dbeafe;
    --color-primary-900: #eff6ff;

    --color-white: #18181b;
    --color-black: #fff;
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
