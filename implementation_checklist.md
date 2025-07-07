# Course Pilot Implementation Checklist (Current Status)

_A scalable, modular, and future-proof roadmap for Dioxus Desktop UI & Backend Integration_

---

## **Dioxus UI Crates Used**

- **dioxus-daisyui:** For all core UI components, theming, layout, and advanced UI patterns (cards, buttons, progress bars, modals, dropdowns, tabs, accordions, etc.).
- **dioxus-tailwindcss:** For utility-first, responsive styling and rapid iteration.
- **dioxus-free-icons:** For Material Design iconography in navigation, actions, and context menus.
- **dioxus-motion:** For purposeful, non-jarring presence/layout/hover animations.
- **dioxus-toast:** For sleek, non-intrusive notifications and feedback.
- **dioxus-sdk:** For state management, hooks, and platform utilities.

_All UI components and flows should be built using the Dioxus ecosystem: DaisyUI for structure and theming, TailwindCSS for utility styling, Free Icons for visual cues, Motion for animation, Toast for feedback, and SDK for state/platform integration._
## **must use**
- **context7-mcp** must use context7 for all Dioxus hooks and component lifetimes and Crates  and all syntaxes that you aren't sure about.

---

## Phase 1: Global Foundation & Theming

- [ ] **Scaffold Three-Panel Layout**
  - Use **dioxus-daisyui** for layout primitives and panel structure.
  - Sidebar (glassmorphism, icon-only, expands on hover, collapses on mobile) with **dioxus-free-icons** for navigation.
  - Main Content (dashboard, plan view, settings)
  - Contextual Panel (slide-in for notes/player, tabs for switching) using DaisyUI Tabs/Accordion.
  - **dioxus-tailwindcss** for responsive utility classes and spacing.
  - **Scalability:** Responsive, modular layout ready for additional panels or platform-specific tweaks.

- [ ] **Implement Global Theming**
  - DaisyUI lofi (light) and night (dark) themes via **dioxus-daisyui**.
  - Theme toggle in App shell, propagated via context/signal.
  - All DaisyUI components inherit theme instantly.
  - **Scalability:** Context-driven theme system, supporting future custom themes or branding.

- [ ] **Establish AppState Signal/Context**
  - Use **dioxus-sdk** for global, reactive state for all UI.
  - **Scalability:** Centralized state enables easy extension for new features and cross-component communication.

---

## Phase 2: Core Component Structure

- [ ] **Create Modular File Structure**
  - `ui/layout.rs`: App shell, theming, sidebar, main, contextual panel
  - `ui/components/`: DaisyUI-based reusable components (Card, Button, Progress, Modal, Toast, Dropdown, Tabs, Accordion, etc.) using **dioxus-daisyui**.
  - `ui/dashboard.rs`: Dashboard grid and CourseCard
  - `ui/plan_view.rs`: Plan checklist and progress
  - `ui/notes_panel.rs`: Notes editor (per-course/video), search, tagging, export
  - `ui/theme_unified.rs`: Theme context and switching logic
  - `ui/navigation.rs`: Routing and sidebar logic
  - `ui/hooks.rs`: Custom hooks for backend actions/state
  - **dioxus-free-icons** for all iconography.
  - **dioxus-tailwindcss** for utility styling and responsive design.
  - **Scalability:** Modular structure supports rapid feature addition and platform-specific overrides.

- [ ] **Integrate dioxus-motion, dioxus-toast, and DaisyUI Feedback**
  - Animate presence/layout for all major components and list items with **dioxus-motion**.
  - Toast notifications for feedback with **dioxus-toast**.
  - Use DaisyUI Modal/Dropdown for context menus and confirmations.
  - **Scalability:** Animation and feedback systems are reusable across new features.

---

## Phase 3: Backend Integration & State Management

- [ ] **Connect UI to Backend via Async Actions/Hooks**
  - All backend CRUD/search/export exposed as async actions/hooks using **dioxus-sdk** for state and effect management.
  - UI never touches DB directly—always via backend API.
  - **Scalability:** Backend API is decoupled from UI, enabling future migration to web/mobile or cloud sync.

- [ ] **Elegant Error & Loading Handling**
  - All mutations trigger toast notifications with **dioxus-toast**.
  - Error/loading states surfaced with DaisyUI skeletons, toasts, or inline feedback.
  - **Scalability:** Centralized error/loading handling simplifies maintenance and future enhancements.

- [ ] **Prepare for Async DB (tokio)**
  - Structure allows easy migration to async DB ops for responsiveness.
  - **Scalability:** Ready for high-concurrency or cloud-backed scenarios.

---

## Phase 4: Feature Mapping & UI Flows

- [ ] **Courses**
  - Dashboard grid (CourseCard), add/edit/delete, progress bar, export
  - Use **dioxus-daisyui** for cards, progress, and actions; **dioxus-free-icons** for visual cues.
  - **Scalability:** Course model supports metadata, tags, and future analytics.

- [ ] **Planner**
  - PlanView with checklist, progress, session controls
  - Use DaisyUI Accordion/Collapse for modules, checkboxes for progress.
  - **Scalability:** Planner logic is modular, supporting new scheduling strategies.

- [ ] **Notes**
  - Contextual panel with per-course and per-video notes, tagging, search, markdown editor, export
  - Use DaisyUI Tabs/Modal for editor, **dioxus-motion** for panel transitions, **dioxus-toast** for feedback.
  - **Scalability:** Notes backend supports tagging, advanced search, and future features (attachments, analytics).

- [ ] **Ingest**
  - Course import flows (YouTube, local), feedback via **dioxus-toast**
  - **Scalability:** Ingest system is modular, ready for new sources (Udemy, Coursera, etc.).

---

## Phase 5: Visual Polish & UX Enhancements

- [ ] **Motion & Visual Effects**
  - Animate all major transitions (presence, layout, hover/focus) with **dioxus-motion**.
  - Glassmorphism for sidebar and modals using DaisyUI and TailwindCSS utilities.
  - Glow for primary actions and active elements using DaisyUI accent colors.
  - **Scalability:** Visual system is theme-driven and easily extensible.

- [ ] **Command Palette**
  - Keyboard-driven modal for power users (Ctrl+K) using DaisyUI Modal and **dioxus-free-icons** for action icons.
  - **Scalability:** Command system can be extended with new actions as features grow.

- [ ] **Data-Rich, Minimal UI**
  - Use DaisyUI Dropdown/context menus and elegant visualizations (progress rings, etc.)
  - **Scalability:** UI shows complexity only when needed, keeping the experience clean as features expand.

---

## Phase 6: Testing, Accessibility, and Documentation

- [ ] **Comprehensive Test Coverage**
  - All backend and UI flows covered by unit and integration tests.
  - Use DaisyUI and Dioxus component test utilities where possible.
  - **Scalability:** Test suite grows with the codebase, ensuring reliability.

- [ ] **Accessibility & Responsiveness**
  - All UI components are keyboard-accessible and screen-reader friendly.
  - Responsive design for desktop, web, and mobile using **dioxus-tailwindcss** and DaisyUI.
  - **Scalability:** Accessibility is built-in, not bolted on.

- [ ] **Documentation**
  - Inline docs for all public APIs and UI flows.
  - Migration/upgrade notes for contributors.
  - **Scalability:** Good docs lower onboarding friction and support open-source/community growth.

---

## Backend Scalability Principles

- **Modular, Pure Rust Core:**
  - All business logic is platform-agnostic, enabling reuse across desktop, web, and mobile.

- **Extensible Data Model:**
  - Notes, courses, and planner are designed for easy extension (tags, attachments, analytics, etc.).

- **Migration-Ready:**
  - Schema migrations and upgrade paths are documented and tested.

- **Async-Ready:**
  - Backend can be migrated to async for high-concurrency or cloud scenarios.

- **API-Driven:**
  - All UI/backend interaction is via clear, documented APIs, supporting future REST/gRPC/websocket layers.

---

---

## Current Status & Blockers (as of last cargo check)

### ✅ Complete & Functional
- TailwindCSS/DaisyUI configured and used in all UI
- Theme context and switching (lofi/forest)
- Three-panel layout (sidebar, main, contextual panel)
- Sidebar navigation (Dioxus Free Icons, DaisyUI)
- Dashboard, PlanView, NotesPanel: UI scaffolded, backend CRUD logic present
- AppState loads from SQLite DB at startup, hooks update both DB and AppState
- Loading skeletons and error handling in all major panels
- Markdown rendering for notes (markdown-rs)
- DaisyUI advanced patterns (dropdowns, context menus) present
- Toast feedback (currently logs, not visual)

### ⚠️ Blockers / Failing Areas
- ❌ **Build Fails:** Dioxus API usage errors, module import errors, and component signature issues
- ❌ **Hooks:** Incorrect use of Dioxus hooks (`use_signal`, `use_context`, `use_memo`), component lifetimes, and tuple arguments
- ❌ **Module Imports:** Some modules/files missing or misdeclared
- ❌ **Icon Imports:** Some icon imports and DaisyUI component usage need correction
- ❌ **No Visual Toasts:** Toasts are logged, not shown in the UI
- ❌ **No Real Routing:** State-based routing only; no deep linking

### 🟡 In Progress / Needs Fix
- Systematically fix Dioxus API usage errors and module import issues
- Confirm all modules exist and are properly declared
- Use MCP/context7 to verify correct Dioxus hook/component syntax
- Only proceed with implementation after confirming the right approach for each error

---

## **Summary Table**

| Area                        | Status           |
|-----------------------------|------------------|
| Theming/Context             | ✅ Complete      |
| Layout (3-panel)            | ✅ Complete      |
| Sidebar Navigation          | ✅ Complete      |
| Dashboard                   | ✅ Functional (AppState/DB) |
| PlanView                    | ✅ Functional (AppState/DB) |
| NotesPanel                  | ✅ Functional (AppState/DB, markdown) |
| AppState/Context            | ✅ Complete      |
| Hooks                       | ⚠️ Errors in Dioxus usage |
| DaisyUI Advanced Patterns   | ✅ Complete      |
| Toast Feedback              | ⚠️ Log only, not visual |
| Loading/Error Handling      | ✅ Complete      |
| Accessibility/Responsive    | ⚠️ Needs audit   |
| Build/Run                   | ❌ Fails (see above) |

---

## **Next Steps**
- Systematically fix Dioxus API usage errors and module import issues
- Confirm all modules exist and are properly declared
- Use MCP/context7 to verify correct Dioxus hook/component syntax
- Only proceed with implementation after confirming the right approach for each error

_You are at the “fix build errors and polish” stage of Phase 1. Once the build passes, you’ll have a fully functional, persistent, and production-ready foundation._
