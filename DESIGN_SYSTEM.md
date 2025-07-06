# Course Pilot Design System

A comprehensive design system built for the Course Pilot application using Dioxus and unified theme tokens.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Theme System](#theme-system)
- [Design Tokens](#design-tokens)
- [Components](#components)
- [Layout System](#layout-system)
- [Accessibility](#accessibility)
- [Best Practices](#best-practices)
- [Migration Guide](#migration-guide)

## Overview

The Course Pilot Design System provides a unified, accessible, and maintainable approach to building user interfaces. It features:

- **Unified Theme System**: Consistent design tokens across light and dark themes
- **Component Library**: 20+ accessible, reusable components
- **Layout System**: Responsive grid and container components
- **TypeScript-like Safety**: Strong typing with Rust enums
- **Accessibility First**: WCAG 2.1 AA compliant components
- **Mobile-First**: Responsive design that works on all devices

## Installation

### Using the Unified Theme System

```rust
use crate::ui::{ThemeProvider, Layout};
use crate::ui::components::prelude::*;

// Wrap your app with ThemeProvider
rsx! {
    ThemeProvider {
        Layout {}
    }
}
```

### Importing Components

```rust
// Import individual components
use crate::ui::{Button, Card, Input};

// Or use the prelude for common components
use crate::ui::prelude::*;
```

## Theme System

### Theme Modes

The design system supports automatic light/dark mode switching:

```rust
use crate::ui::{ThemeToggle, use_theme, ThemeMode};

#[component]
fn MyComponent() -> Element {
    let theme = use_theme();
    let is_dark = matches!(*theme.read(), ThemeMode::Dark);
    
    rsx! {
        div {
            class: if is_dark { "dark-specific-class" } else { "light-specific-class" },
            ThemeToggle {}
            "Current theme: {if is_dark { 'Dark' } else { 'Light' }}"
        }
    }
}
```

### CSS Variables

All components use CSS variables that automatically adapt to the current theme:

```css
/* Core colors */
--bg-primary: #fafafa;           /* Light theme background */
--text-primary: #18181b;         /* Light theme text */
--color-primary: #2563eb;        /* Primary brand color */

/* Component-specific tokens */
--btn-primary-bg: var(--color-primary);
--card-bg: var(--bg-elevated);
--input-border: var(--border-primary);
```

## Design Tokens

### Color Palette

#### Neutral Colors
```css
--neutral-50: #fafafa   /* Lightest */
--neutral-100: #f4f4f5
--neutral-200: #e4e4e7
--neutral-300: #d4d4d8
--neutral-400: #a1a1aa
--neutral-500: #71717a  /* Mid-point */
--neutral-600: #52525b
--neutral-700: #3f3f46
--neutral-800: #27272a
--neutral-900: #18181b
--neutral-950: #09090b  /* Darkest */
```

#### Semantic Colors
```css
/* Primary (Blue) */
--primary-500: #3b82f6
--primary-600: #2563eb  /* Main primary */
--primary-700: #1d4ed8

/* Success (Green) */
--success-500: #22c55e
--success-600: #16a34a  /* Main success */
--success-700: #15803d

/* Warning (Yellow) */
--warning-500: #eab308
--warning-600: #ca8a04  /* Main warning */
--warning-700: #a16207

/* Error (Red) */
--error-500: #ef4444
--error-600: #dc2626    /* Main error */
--error-700: #b91c1c
```

### Typography Scale

```css
--font-size-xs: 0.75rem     /* 12px */
--font-size-sm: 0.875rem    /* 14px */
--font-size-base: 1rem      /* 16px */
--font-size-lg: 1.125rem    /* 18px */
--font-size-xl: 1.25rem     /* 20px */
--font-size-2xl: 1.5rem     /* 24px */
--font-size-3xl: 1.875rem   /* 30px */
--font-size-4xl: 2.25rem    /* 36px */
```

### Spacing Scale

```css
--spacing-1: 0.25rem    /* 4px */
--spacing-2: 0.5rem     /* 8px */
--spacing-3: 0.75rem    /* 12px */
--spacing-4: 1rem       /* 16px */
--spacing-5: 1.25rem    /* 20px */
--spacing-6: 1.5rem     /* 24px */
--spacing-8: 2rem       /* 32px */
--spacing-10: 2.5rem    /* 40px */
--spacing-12: 3rem      /* 48px */
--spacing-16: 4rem      /* 64px */
```

### Border Radius

```css
--radius-sm: 0.25rem    /* 4px */
--radius-md: 0.5rem     /* 8px */
--radius-lg: 1rem       /* 16px */
--radius-xl: 1.5rem     /* 24px */
--radius-full: 9999px   /* Full radius */
```

## Components

### Button Component

The Button component supports multiple variants, sizes, and states:

```rust
// Basic usage
Button {
    "Click me"
}

// With variant and size
Button {
    variant: ButtonVariant::Primary,
    size: ButtonSize::Large,
    "Large Primary Button"
}

// With loading state
Button {
    variant: ButtonVariant::Secondary,
    loading: true,
    disabled: loading_state(),
    onclick: handle_submit,
    "Submit Form"
}

// Icon button
IconButton {
    icon: rsx! { span { "🔍" } },
    aria_label: "Search",
    onclick: handle_search
}

// Button group
ButtonGroup {
    connected: true,
    Button { variant: ButtonVariant::Outline, "Bold" }
    Button { variant: ButtonVariant::Outline, "Italic" }
    Button { variant: ButtonVariant::Outline, "Underline" }
}
```

#### Button Variants
- `Primary`: Main call-to-action
- `Secondary`: Secondary actions
- `Outline`: Subtle actions
- `Ghost`: Minimal actions
- `Destructive`: Dangerous actions

#### Button Sizes
- `Small`: Compact interfaces
- `Medium`: Standard size (default)
- `Large`: Prominent actions

### Card Component

Flexible container component with multiple composition options:

```rust
// Simple card
SimpleCard {
    title: "Card Title".to_string(),
    p { "Card content goes here." }
}

// Complex card with all sections
Card {
    variant: CardVariant::Elevated,
    
    CardHeader {
        title: Some("User Profile".to_string()),
        subtitle: Some("Software Engineer".to_string()),
        avatar: rsx! {
            div { class: "avatar", "JD" }
        },
        action: rsx! {
            IconButton {
                icon: rsx! { span { "⋮" } },
                aria_label: "More options"
            }
        }
    }
    
    CardMedia {
        src: "https://example.com/image.jpg",
        alt: "Profile image"
    }
    
    CardContent {
        p { "User bio and additional information." }
    }
    
    CardActions {
        align: "end",
        Button { variant: ButtonVariant::Outline, "Message" }
        Button { variant: ButtonVariant::Primary, "Connect" }
    }
}

// Media card
MediaCard {
    title: "Beautiful Landscape".to_string(),
    image_src: "https://example.com/landscape.jpg",
    image_alt: "Mountain landscape",
    actions: rsx! {
        Button { "Share" }
        Button { variant: ButtonVariant::Primary, "Download" }
    },
    p { "A stunning mountain landscape." }
}
```

#### Card Variants
- `Elevated`: With shadow (default)
- `Outlined`: With border
- `Filled`: With background color

### Input Component

Comprehensive form input with validation and accessibility:

```rust
// Basic text input
Input {
    label: Some("Full Name".to_string()),
    placeholder: Some("Enter your name".to_string()),
    value: Some(name_value()),
    oninput: move |evt| name_value.set(evt.value()),
    required: true
}

// Input with validation state
Input {
    label: Some("Email Address".to_string()),
    input_type: InputType::Email,
    state: if email_valid() { InputState::Success } else { InputState::Error },
    error_message: if !email_valid() { 
        Some("Please enter a valid email".to_string()) 
    } else { None },
    left_icon: rsx! { span { "✉️" } }
}

// Specialized inputs
SearchInput {
    value: Some(search_value()),
    oninput: move |evt| search_value.set(evt.value()),
    full_width: Some(true)
}

PasswordInput {
    label: Some("Password".to_string()),
    required: Some(true),
    full_width: Some(true)
}

NumberInput {
    label: Some("Age".to_string()),
    min: Some("18".to_string()),
    max: Some("120".to_string()),
    step: Some("1".to_string())
}

// TextArea
TextArea {
    label: Some("Description".to_string()),
    rows: 4,
    placeholder: Some("Enter description...".to_string()),
    helper_text: Some("Maximum 500 characters".to_string()),
    maxlength: Some(500)
}
```

#### Input Variants
- `Filled`: Filled background
- `Outlined`: Outlined border (default)
- `Standard`: Underlined

#### Input States
- `Normal`: Default state
- `Success`: Valid input
- `Warning`: Needs attention
- `Error`: Invalid input

## Layout System

### Application Layout

The main application layout provides responsive navigation:

```rust
// Main app structure
rsx! {
    ThemeProvider {
        Layout {}
    }
}
```

Features:
- Collapsible sidebar navigation
- Mobile-responsive design
- Theme toggle in app bar
- Proper focus management
- Keyboard navigation support

### Grid System

Use CSS Grid with design system utilities:

```rust
div {
    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: var(--spacing-4);",
    
    Card { /* Card 1 */ }
    Card { /* Card 2 */ }
    Card { /* Card 3 */ }
}
```

### Responsive Design

The system uses mobile-first responsive design:

```css
/* Mobile first (default) */
.component {
    padding: var(--spacing-3);
}

/* Tablet and up */
@media (min-width: 768px) {
    .component {
        padding: var(--spacing-4);
    }
}

/* Desktop and up */
@media (min-width: 1024px) {
    .component {
        padding: var(--spacing-6);
    }
}
```

## Accessibility

### Focus Management

All interactive components support proper focus management:

```css
/* Focus styles are applied automatically */
.component:focus-visible {
    box-shadow: var(--focus-ring);
    outline: none;
}
```

### ARIA Support

Components include proper ARIA attributes:

```rust
Button {
    aria_label: "Close dialog",
    aria_describedby: "help-text",
    "×"
}

Input {
    label: Some("Email".to_string()),
    aria_describedby: "email-help",
    helper_text: Some("We'll never share your email".to_string())
}
```

### Keyboard Navigation

All components support keyboard navigation:

- `Tab` / `Shift+Tab`: Navigate between elements
- `Enter` / `Space`: Activate buttons and interactive elements
- `Escape`: Close dialogs and menus
- Arrow keys: Navigate within component groups

### Screen Reader Support

Components include semantic HTML and proper labeling:

```rust
// Semantic structure
Card {
    role: Some("article".to_string()),
    aria_label: "Course information",
    
    CardHeader {
        // Automatically uses h3 for title
        title: Some("Course Title".to_string())
    }
}

// Hidden labels for screen readers
span { class: "sr-only", "Loading..." }
```

## Best Practices

### Component Usage

1. **Use semantic variants**: Choose component variants based on their semantic meaning
2. **Consistent sizing**: Use the same size scale across related components
3. **Proper nesting**: Follow component composition patterns
4. **State management**: Handle loading and error states appropriately

### Theme Integration

1. **Use CSS variables**: Always use theme variables instead of hardcoded values
2. **Test both themes**: Ensure components work in both light and dark modes
3. **Maintain contrast**: Ensure sufficient color contrast for accessibility

### Performance

1. **Minimize inline styles**: Use CSS classes whenever possible
2. **Optimize re-renders**: Use appropriate signal patterns
3. **Lazy loading**: Load components only when needed

### Code Organization

```rust
// Good: Organized imports
use crate::ui::prelude::*;

// Good: Clear component structure
#[component]
fn UserCard(user: User) -> Element {
    rsx! {
        Card {
            variant: CardVariant::Elevated,
            
            CardHeader {
                title: Some(user.name.clone()),
                subtitle: Some(user.role.clone())
            }
            
            CardContent {
                p { "{user.bio}" }
            }
            
            CardActions {
                Button { "View Profile" }
                Button { 
                    variant: ButtonVariant::Primary,
                    "Connect" 
                }
            }
        }
    }
}
```

## Migration Guide

### From Legacy Theme System

1. **Replace old theme imports**:
   ```rust
   // Old
   use crate::ui::theme::AppTheme;
   
   // New
   use crate::ui::ThemeProvider;
   ```

2. **Update CSS variables**:
   ```css
   /* Old */
   background: var(--card-bg-light);
   
   /* New */
   background: var(--card-bg);
   ```

3. **Use new component APIs**:
   ```rust
   // Old
   Button { button_type: "primary", "Click me" }
   
   // New
   Button { variant: ButtonVariant::Primary, "Click me" }
   ```

### Breaking Changes

- CSS variable names have been standardized
- Component prop names follow consistent patterns
- Theme provider is now required at app root
- Some legacy components have been removed

### Upgrade Steps

1. Install the new theme system
2. Replace old theme provider
3. Update component imports
4. Replace deprecated props
5. Test in both light and dark themes
6. Update custom CSS to use new variables

## Contributing

### Adding New Components

1. Create component in `src/ui/components/`
2. Follow existing patterns and naming conventions
3. Include comprehensive prop types
4. Add accessibility attributes
5. Create component styles using theme variables
6. Include demo and tests
7. Update module exports

### Modifying Existing Components

1. Maintain backward compatibility when possible
2. Update documentation
3. Test across all variants and states
4. Verify accessibility compliance
5. Update tests

### Design Token Changes

1. Update tokens in `theme_unified.rs`
2. Test impact across all components
3. Update documentation
4. Consider migration path for consumers

For questions or contributions, please refer to the project's contributing guidelines.