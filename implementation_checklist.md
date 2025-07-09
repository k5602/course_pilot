# Course Pilot Implementation Checklist (Current Status)

A scalable, modular, and future-proof roadmap for Dioxus Desktop UI & Backend Integration


## Dioxus UI Crates & Dependencies

### Core UI

- dioxus-daisyui v0.8.0+: For all core UI components, theming, and layout patterns
- daisyui v5.0.46+: For Tailwind CSS v4 plugin system and component styles
- @tailwindcss/cli v4.1.11+: For processing Tailwind CSS with plugin support

### Styling & Theming

- Tailwind CSS v4+: For utility-first responsive design
- Autoprefixer: For cross-browser compatibility

### Icons & Interactions

- dioxus-free-icons: For Material Design iconography
- dioxus-motion: For animations and transitions
- dioxus-toast: For user notifications and feedback

### State Management

- dioxus-sdk: For application state and hooks
- context7-mcp: understnding Documentation

### Build & Tooling

- Node.js v22.16.0+: Required for Tailwind CSS v4
- npm v10.9.2+: For dependency management


## Phase 1: Global Foundation & Theming

- [x] Scaffold Three-Panel Layout
  - Use dioxus-daisyui for layout primitives and panel structure.
  - Sidebar (glassmorphism, icon-only, expands on hover, collapses on mobile) with dioxus-free-icons for navigation.
  - Main Content (dashboard, plan view, settings)
  - Contextual Panel (slide-in for notes/player, tabs for switching) using DaisyUI Tabs/Accordion.
  - dioxus-tailwindcss for responsive utility classes and spacing.
  - Scalability: Responsive, modular layout ready for additional panels or platform-specific tweaks.

- [x] Implement Global Theming
  - DaisyUI lofi (light) and night (dark) themes via dioxus-daisyui.
  - Theme toggle in App shell, propagated via context/signal.
  - All DaisyUI components inherit theme instantly.
  - Theme preference is persisted in a desktop-native config file (`theme_config.toml`), not browser localStorage.
  - No JS interop is used for theme switching in desktop mode; all logic is Rust-native and future-proofed for web.
  - Scalability: Context-driven theme system, supporting future custom themes or branding.

- [x] Establish AppState Signal/Context
  - Use dioxus-sdk for global, reactive state for all UI.
  - Scalability: Centralized state enables easy extension for new features and cross-component communication.


## Phase 2: Core Component Structure

- [ ] Create Modular File Structure
  - ui/layout.rs: App shell, theming, sidebar, main, contextual panel
  - ui/components/: DaisyUI-based reusable components (Card, Button, Progress, Modal, Toast, Dropdown, Tabs, Accordion, etc.) using dioxus-daisyui.
  - ui/dashboard.rs: Dashboard grid and CourseCard
  - ui/plan_view.rs: Plan checklist and progress
  - ui/notes_panel.rs: Notes editor (per-course/video), search, tagging, export
  - ui/theme_unified.rs: Theme context and switching logic
  - ui/navigation.rs: Routing and sidebar logic
  - ui/hooks.rs: Custom hooks for backend actions/state
  - dioxus-free-icons for all iconography.
  - dioxus-tailwindcss for utility styling and responsive design.
  - Scalability: Modular structure supports rapid feature addition and platform-specific overrides.

- [ ] Integrate dioxus-motion, dioxus-toast, and DaisyUI Feedback
  - Animate presence/layout for all major components and list items with dioxus-motion.
  - Toast notifications for feedback with dioxus-toast.
  - Use DaisyUI Modal/Dropdown for context menus and confirmations.
  - Scalability: Animation and feedback systems are reusable across new features.


## Phase 3: Backend Integration & State Management

- [ ] Connect UI to Backend via Async Actions/Hooks
  - All backend CRUD/search/export exposed as async actions/hooks using dioxus-sdk for state and effect management.
  - UI never touches DB directly—always via backend API.
  - Scalability: Backend API is decoupled from UI, enabling future migration to web/mobile or cloud sync.

- [ ] Elegant Error & Loading Handling
  - All mutations trigger toast notifications with dioxus-toast.
  - Error/loading states surfaced with DaisyUI skeletons, toasts, or inline feedback.
  - Scalability: Centralized error/loading handling simplifies maintenance and future enhancements.

- [ ] Prepare for Async DB (tokio)
  - Structure allows easy migration to async DB ops for responsiveness.
  - Scalability: Ready for high-concurrency or cloud-backed scenarios.


## Phase 4: Feature Mapping & UI Flows

- [ ] Courses
  - Dashboard grid (CourseCard), add/edit/delete, progress bar, export
  - Use dioxus-daisyui for cards, progress, and actions; dioxus-free-icons for visual cues.
  - Scalability: Course model supports metadata, tags, and future analytics.

- [ ] Planner
  - PlanView with checklist, progress, session controls
  - Use DaisyUI Accordion/Collapse for modules, checkboxes for progress.
  - Scalability: Planner logic is modular, supporting new scheduling strategies.

- [ ] Notes
  - Contextual panel with per-course and per-video notes, tagging, search, markdown editor, export
  - Use DaisyUI Tabs/Modal for editor, dioxus-motion for panel transitions, dioxus-toast for feedback.
  - Scalability: Notes backend supports tagging, advanced search, and future features (attachments, analytics).

- [ ] Ingest
  - Course import flows (YouTube, local), feedback via dioxus-toast
  - Scalability: Ingest system is modular, ready for new sources (Udemy, Coursera, etc.).


## Phase 5: Visual Polish & UX Enhancements

- [ ] Motion & Visual Effects
  - Animate all major transitions (presence, layout, hover/focus) with dioxus-motion.
  - Glassmorphism for sidebar and modals using DaisyUI and TailwindCSS utilities.
  - Glow for primary actions and active elements using DaisyUI accent colors.
  - Scalability: Visual system is theme-driven and easily extensible.

- [ ] Command Palette
  - Keyboard-driven modal for power users (Ctrl+K) using DaisyUI Modal and dioxus-free-icons for action icons.
  - Scalability: Command system can be extended with new actions as features grow.

- [ ] Data-Rich, Minimal UI
  - Use DaisyUI Dropdown/context menus and elegant visualizations (progress rings, etc.)
  - Scalability: UI shows complexity only when needed, keeping the experience clean as features expand.


## Phase 6: Testing, Accessibility, and Documentation

- [ ] Comprehensive Test Coverage
  - All backend and UI flows covered by unit and integration tests.
  - Use DaisyUI and Dioxus component test utilities where possible.
  - Scalability: Test suite grows with the codebase, ensuring reliability.

- [ ] Accessibility & Responsiveness
  - All UI components are keyboard-accessible and screen-reader friendly.
  - Responsive design for desktop, web, and mobile using dioxus-tailwindcss and DaisyUI.
  - Scalability: Accessibility is built-in, not bolted on.

- [ ] Documentation
  - Inline docs for all public APIs and UI flows.
  - Migration/upgrade notes for contributors.
  - Scalability: Good docs lower onboarding friction and support open-source/community growth.


## Backend Scalability Principles

- Modular, Pure Rust Core:
  - All business logic is platform-agnostic, enabling reuse across desktop, web, and mobile.

- Extensible Data Model:
  - Notes, courses, and planner are designed for easy extension (tags, attachments, analytics, etc.).

- Migration-Ready:
  - Schema migrations and upgrade paths are documented and tested.

- Async-Ready:
  - Backend can be migrated to async for high-concurrency or cloud scenarios.

- API-Driven:
  - All UI/backend interaction is via clear, documented APIs, supporting future REST/gRPC/websocket layers.


## Current Status & Blockers (as of last cargo check)

### Complete & Functional

- Tailwind CSS v4 + DaisyUI v5 integration complete with proper build pipeline
- Theme system with lofi (light) and night (dark) themes
- Responsive three-panel layout (sidebar, main, contextual panel)
- Sidebar navigation with Dioxus Free Icons and DaisyUI components
- Dashboard, PlanView, and NotesPanel UI scaffolding
- AppState management with SQLite persistence
- Basic error handling and loading states
- Markdown rendering for notes
- DaisyUI component patterns integrated
- Build system configured for development and production

### Current Focus Areas

- UI Polish: Refining component styling and theming
- Component Library: Building out reusable UI components with DaisyUI
- State Management: Implementing proper state management patterns
- Testing: Adding unit and integration tests

### In Progress

- Component Development: Building out core UI components with DaisyUI
- State Management: Implementing proper state management with Dioxus hooks
- Theming System: Finalizing the theming implementation
- Documentation: Updating documentation for the new setup


## Summary Table

| Area                        | Status           |
|-----------------------------|------------------|
| Build System                | Complete         |
| - Tailwind CSS v4           | Complete         |
| - DaisyUI v5 Integration    | Complete         |
| - Build Pipeline            | Complete         |
| Theming                     | In Progress      |
| - Light/Dark Themes         | Complete         |
| - Theme Switching           | Complete         |
| - Component Theming         | In Progress      |
| UI Components               | In Development   |
| - Core Components          | In Progress      |
| - Layout System            | Complete         |
| - Navigation               | Complete         |
| Layout (3-panel)            | Complete         |
| Sidebar Navigation          | Complete         |
| Dashboard                   | Functional (AppState/DB) |
| PlanView                    | Functional (AppState/DB) |
| NotesPanel                  | Functional (AppState/DB, markdown) |
| AppState/Context            | Complete         |
| Hooks                       | Errors in Dioxus usage |
| DaisyUI Advanced Patterns   | Complete         |
| Toast Feedback              | Complete, visual, and reactive |
| Loading/Error Handling      | Complete         |
| Accessibility/Responsive    | Needs audit       |
| Build/Run                   | Works            |


## Next Steps

- Systematically fix Dioxus API usage errors and module import issues
- Confirm all modules exist and are properly declared
- Use MCP/context7 to verify correct Dioxus hook/component syntax
- Only proceed with implementation after confirming the right approach for each error

---

## Recent Updates

- ✅ Toast notifications are now fully integrated, visually themed, and reactive to theme changes.
- ✅ The "Test Toasts" button has been removed from the sidebar navigation for a cleaner UI.
- ✅ Checklist and plan updated to reflect completed toast integration and sidebar cleanup.

You are at the “fix build errors and polish” stage of Phase 1. Once the build passes, you’ll have a fully functional, persistent, and production-ready foundation.
