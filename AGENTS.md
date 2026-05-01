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
  - `llm/`: Gemini via genai-rs (CompanionAI + ExaminerAI + SummarizerAI).
  - `discord/`: Rich Presence via discord-rich-presence crate.
  - `video/`: GStreamer player, position tracking, YouTube embed handler.
  - `keystore/`: OS keyring via keyring crate.
  - `transcript/`: YouTube subtitles via yt-transcript-rs.
  - `youtube/`: Playlist fetching via rusty-ytdl.
  - `local_media/`: Scans local filesystem for MP4/MKV/WEBM with subtitle matching.
  - `tokio_bridge.rs`: Global tokio runtime so GTK callbacks can run async tasks.
- **`src/ui/`** - GTK4/libadwaita widgets.
  - Pages: dashboard, course_list, course_view, video_player, quiz_list, quiz_view, settings, onboarding.
  - `layout.rs`: MainLayout with sidebar nav, gtk::Stack page switcher, right panel.
  - `right_panel.rs`: Notes tab + AI Chat tab.
  - `state.rs`: AppState (Rc<RefCell<AppState>>).
  - `toast.rs`: Non-blocking toast notifications.
  - `shortcuts.rs`: Escape key navigation (back buttons).
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

## Persistence & Search

Diesel with SQLite. Migration files in `migrations/`. Schema auto-generated by diesel_cli into `src/schema.rs`. Repository implementations map between diesel rows and domain entities.

Search uses SQLite FTS5. The search index is updated transactionally inside use cases alongside the entity writes -- no separate sync step.

Available repositories: Course, Module, Video, Exam, Note, Tag, UserPreferences, Search.

## Infrastructure Notes

- **Discord Rich Presence**: Updates activity based on the current page and user state.
- **Video Player**: GStreamer pipeline with `playbin`, rendering frames into a `gtk::Picture` via `appsink` + `MemoryTexture`. Supports play, pause, seek, volume, playback rate, and external subtitle URIs.

## Domain Services Worth Knowing

- **BoundaryDetector**: Groups videos into modules by scanning titles for numbered patterns (`1.1`, `Module 2`, `Chapter 3.1`, `Week 4`). Falls back to batch-size grouping (default 5) when title signals are weak.
- **SessionPlanner**: Divides video lists into daily sessions based on a configurable cognitive limit (minutes per day). Respects module boundaries so you don't split a module across two days unless necessary.
- **Sanitizer**: Strips noise from YouTube auto-generated titles (parenthetical notes, bracketed prefixes, common filler).

## Error Handling

Use `thiserror` for application and infrastructure errors. Domain errors use `RepositoryError`. UI code should catch errors from use cases and surface them through `Toast::show_error()` -- never silently ignore a `Result`.
