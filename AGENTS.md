# Course Pilot: AI Engineering Guide

**Course Pilot** takes unstructured video courses (YouTube, local files) and turns them into structured, AI-augmented learning experiences. It generates logical module groupings, schedules study sessions based on cognitive load, and provides tools like AI exams, video summarization, and a companion chat for Q&A -- minus the noise of ads and recommendation algorithms.

Built as a Rust desktop app using GTK4/libadwaita with strict Hexagonal (Ports & Adapters) architecture and Domain-Driven Design. The code is in Rust edition 2024 with Diesel for persistence, GStreamer for video playback, and Gemini for AI features.

## Core Architecture (DDD Hexagonal)

Business logic stays separate from infrastructure:

- **`src/domain/`** - Zero external dependencies.
  - `entities/`: Course, Module, Video, Note, Exam, Tag, UserPreferences, SearchResult.
  - `ports/`: Traits for Repository, LLM, Transcript, Presence, SecretStore, YouTube, LocalMedia.
  - `value_objects/`: Newtype IDs (CourseId, VideoId, ModuleId, ExamId, TagId), VideoSource enum, session plans.
  - `services/`: BoundaryDetector (groups videos into modules by title analysis), SessionPlanner (cognitive load scheduling), Sanitizer, SubtitleCleaner.
- **`src/application/`** - Orchestration.
  - `context.rs`: AppContext (DI container) + ServiceFactory + AppConfig.
  - `use_cases/`: IngestPlaylist, IngestLocal, SummarizeVideo, TakeExam, AskCompanion, PlanSession, Notes, Dashboard, etc.
- **`src/infrastructure/`** - Adapters implementing domain ports.
  - `persistence/`: Diesel SQLite repositories + FTS5 search.
  - `llm/`: Gemini (upgraded to Gemini 3.1 Flash Lite default model) via genai-rs (CompanionAI + ExaminerAI + SummarizerAI).
  - `discord/`: Rich Presence via discord-rich-presence crate.
  - `video/`: GStreamer player with custom controls (fullscreen toggle, double-click gestures, and F/F11 hotkeys), position tracking, and YouTube embed handler.
  - `keystore/`: OS keyring via keyring crate.
  - `transcript/`: YouTube subtitles via yt-transcript-rs.
  - `youtube/`: Playlist fetching via rusty-ytdl.
  - `local_media/`: Scans local filesystem for MP4/MKV/WEBM with subtitle matching.
  - `tokio_bridge.rs`: Global tokio runtime so GTK callbacks can run async tasks.
- **`src/ui/`** - GTK4/libadwaita widgets.
  - Pages: dashboard (Resume Study Dashboard with gradient hero banner, glassmorphic stat cards, completion levelbars, and interactive progress cards), course_list, course_view, video_player (with scroll-down sub-panels), quiz_list, quiz_view (Interactive Quiz System with explanation drawers and option highlighting), settings, onboarding.
  - `layout.rs`: MainLayout with sidebar nav, gtk::Stack page switcher, right panel, and floating popup notes.
  - `right_panel.rs`: Notes tab + AI Chat tab.
  - `state.rs`: AppState (Rc<RefCell<AppState>>).
  - `toast.rs`: Non-blocking toast notifications.
  - `dialogs/`: Import playlist dialog, import local media dialog.

## DI & Use Cases

**ServiceFactory** wires everything. Every use case takes an `&AppContext`, pulls the adapters it needs, and returns a ready-to-execute use case. Generics carry the concrete adapter types for zero-cost abstraction.

```rust
let ctx = state.backend.as_ref()?;
let uc = ServiceFactory::plan_session(ctx);
let plan = uc.execute(input)?;
```

AppContext holds repositories and adapters behind `Arc`, so cloning is cheap. LLM adapters are `Option<Arc<...>>` -- they only exist if a Gemini API key is configured. The key is stored in the OS keyring, never in code.

## UI Development (GTK4/libadwaita)

We're not using Dioxus. This is straight GTK4 with libadwaita widgets.

### State

`SharedState = Rc<RefCell<AppState>>`. It's cheap to clone (just bumps the Rc refcount). Every widget gets a clone passed in at construction. Read with `.borrow()`, write with `.borrow_mut()`. Keep borrow scopes tight to avoid runtime panics.

### Page Pattern

Each page is a struct with three public methods:
- `new(state, stack)` - builds the widget tree, wires signals, calls refresh.
- `widget()` - returns the root widget (`>`).
- `refresh()` - reads state and updates widget properties (text, visibility, etc).

Pages are stored in `Rc<PageType>` and added to the navigation stack by name. The MainLayout watches `visible-child-name` changes and calls `refresh()` or `stop()` as pages appear and disappear.

### Navigation

String constants in `src/ui/navigation.rs` (PAGE_DASHBOARD, PAGE_COURSE_LIST, etc.). A `gtk::Stack` switches between them. Sidebar toggle buttons sync with the stack. Escape goes back one level (course list -> course view -> video player -> etc).

### Async from GTK

GTK owns the main thread. Spawn async work with `crate::infrastructure::tokio_bridge::spawn()`. Get results back to the UI by sending through a `std::sync::mpsc::channel` and polling it in a `glib::idle_add_local` callback.

```rust
let (tx, rx) = std::sync::mpsc::channel::<String>();
crate::infrastructure::tokio_bridge::spawn(async move {
    let result = do_something_async().await;
    let _ = tx.send(result);
});
glib::idle_add_local(move || match rx.try_recv() {
    Ok(response) => { update_ui(response); glib::ControlFlow::Break },
    Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
    Err(TryRecvError::Disconnected) => glib::ControlFlow::Break,
});
```

### Toast Notifications

`Toast::show("message")` and `Toast::show_error("error")`. They appear as overlay labels at the bottom of the window and auto-dismiss after 3 seconds. Initialized once from `MainLayout::new()`.

### Right Panel

Two tabs: Notes (save/load markdown for the current video) and AI Chat (ask questions about the current video). Loads data when the page refreshes. Chat history is keyed by video ID.

### Resume Study Dashboard

The primary hub featuring a gradient hero banner, glassmorphic stat cards, overall completion levelbars, and interactive progress cards for enrolled courses supporting direct study navigation.

### Scroll-Down Video Sub-panels

Positioned below the video player. Includes a fullscreen video toggle button, double-click gestures, and `F`/`F11` keyboard shortcuts. Features a tabbed layout presenting:
- **Associated Quizzes** - Connects the current video to generated and saved multiple-choice assessments.
- **Video Summary** - Replaces the raw transcript displaying clean, high-fidelity summaries.
- **Video Transcript Reader** - Accessible for reference, cleanly separated from AI prompts.

### Interactive Quiz System

A major redesign presenting university-grade MCQs with plausible distractor fallacies, deep conceptual testing, and thorough refutations. Equipped with:
- Selected option highlighting and correct/incorrect status badges.
- A sliding drawer revealing detailed explanations and rationale.

### Dynamic Floating Popup Notes Window

A globally available notes window triggered via the `Ctrl+N` hotkey. Operates in two states:
- **Type Mode**: Full-featured text editor with "Insert Video Reference" shortcut.
- **Preview Mode**: Rich markdown rendering compiled with Pango, including LaTeX support for inline and block mathematical equations, complete with robust Pango safety escaping.

### AI Chat UI Enhancements

- Message bubbles with User right-aligned and Assistant left-aligned.
- Submission bound to the `Enter` key in the text entry.
- Instant user-message rendering upon sending.

## Persistence & Search

Diesel with SQLite. Migration files in `migrations/`. Schema auto-generated by diesel_cli into `src/schema.rs`. Repository implementations map between diesel rows and domain entities.

Search uses SQLite FTS5. The search index is updated transactionally inside use cases alongside the entity writes -- no separate sync step.

Available repositories: Course, Module, Video, Exam, Note, Tag, UserPreferences, Search.

## Infrastructure Notes

- **Discord Rich Presence**: Updates activity based on the current page and user state.
- **Video Player**: GStreamer pipeline with `playbin`, rendering frames into a `gtk::Picture` via `appsink` + `MemoryTexture`. Supports play, pause, seek, volume, playback rate, and external subtitle URIs. Upgraded with fullscreen toggle, double-click gestures, and `F`/`F11` hotkey actions.
- **AI Context Optimization**: The pipeline no longer passes raw, noisy transcripts to the AI chat or quiz generator. It uses dense, high-fidelity AI-extracted summaries to optimize context performance and cost efficiency.
- **Upgraded Default Model**: Default LLM updated to **Gemini 3.1 Flash Lite** for fast and highly efficient processing.
- **Dynamic H:MM:SS Duration Format**: All durations are dynamically rendered in the clean `H:MM:SS` format across all page layouts.

## Domain Services Worth Knowing

- **BoundaryDetector**: Groups videos into modules by scanning titles for numbered patterns (`1.1`, `Module 2`, `Chapter 3.1`, `Week 4`). Falls back to batch-size grouping (default 5) when title signals are weak.
- **SessionPlanner**: Divides video lists into daily sessions based on a configurable cognitive limit (minutes per day). Respects module boundaries so you don't split a module across two days unless necessary.
- **Sanitizer**: Strips noise from YouTube auto-generated titles (parenthetical notes, bracketed prefixes, common filler).

## Error Handling

Use `thiserror` for application and infrastructure errors. Domain errors use `RepositoryError`. UI code should catch errors from use cases and surface them through `Toast::show_error()` -- never silently ignore a `Result`. Use `if let Err(e) = ... { Toast::show_error(...); }` for user-initiated mutations.

## Repository Batch Methods

For operations that need transactional atomicity (e.g., ingestion), use batch repository methods instead of direct diesel queries:

```rust
course_repo.save_batch(&courses)?;
module_repo.save_batch(&modules)?;
video_repo.save_batch(&videos)?;
search_repo.index_batch(&entries)?;
```

Each method runs in its own diesel transaction. The application layer should never import diesel directly -- always go through port traits.
