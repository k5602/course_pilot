# Course Pilot: AI Engineering Guide

Follow these project-specific patterns and architectural constraints to maintain consistency and quality.

## 🏗️ Core Architecture (DDD Hexagonal)

We use a Strict Hexagonal Architecture to separate concerns:

- **src/domain/**: Core business logic. No external dependencies.
  - `entities/`: Rich domain models (Course, Module, Video, Note, Exam, Tag, AppAnalytics).
  - `ports/`: Trait definitions for infrastructure adapters (Repositories, Fetchers).
  - `value_objects/`: Immutable data containers (IDs implement `FromStr` trait).
  - `services/`: Domain services (SessionPlanner, TitleSanitizer).
- **src/application/**: Use cases and orchestration.
  - `context.rs`: The DI container (`AppContext`). Holds all repositories in `Arc`.
  - `use_cases/`: Task-specific logic (IngestPlaylist, PlanSession, AskCompanion, TakeExam).
- **src/infrastructure/**: External implementations (adapters).
  - `persistence/`: SQLite repositories using Diesel.
  - `youtube/`: API-free fetching via `rusty_ytdl`.
  - `transcript/`: YouTube transcript fetching via `yt-transcript-rs`.
  - `llm/`: Gemini API integration via `genai-rs`.
  - `keystore/`: Secure storage via `keyring`.
- **src/ui/**: Dioxus 0.7 Desktop interface.
- **src/components/**: Reusable, atomic UI components (Accordion, Button, Dialog, Input, Tabs).

## 🧱 Dependency Injection (DI) Pattern

- **NEVER** instantiate repositories or adapters directly in UI components or use cases.
- Use `AppContext` (wrapped in `Arc`) to access shared infrastructure.
- Use `ServiceFactory` to build use cases with their required dependencies.

```rust
// In UI components:
let state = use_context::<AppState>();
let ctx = state.backend.as_ref()?;
let use_case = ServiceFactory::plan_session(ctx);
```

## ⚛️ UI Development (Dioxus 0.7)

- **State Management**: Use `AppState` via `use_context`. Access backend via `state.backend`.
- **Reactive State**: Use `Signal` for all reactive UI state.
- **Data Fetching**: Use custom hooks in `src/ui/hooks.rs`. Many hooks return `(Signal<T>, LoadState)`.
- **Async Actions**: Define in `src/ui/actions.rs` for mutations (e.g., `import_playlist`, `start_exam`).
- **Custom Components**:
  - `src/components/`: Base UI primitives (Button, Input, etc.).
  - `src/ui/custom/`: App-specific logic components (CourseCard, VideoItem, YoutubePlayer, Sidebar).
- **Styling**: Tailwind CSS + DaisyUI classes. Theme configuration in `theme_config.toml` and `assets/dx-components-theme.css`.

### Loading & Error Handling

Use `LoadState` from `src/ui/hooks.rs` to track async operations:

- `is_loading: Signal<bool>`
- `error: Signal<Option<String>>`
- UI Feedback: Use components from `src/ui/custom/loading.rs` (`Spinner`, `CardSkeleton`, `ErrorAlert`).

## 💾 Persistence Pattern

- Map Diesel models in `infrastructure/persistence/models.rs` to Domain Entities in repositories.
- Repository traits must reside in `domain/ports/repository.rs`.
- Database migrations are in `migrations/`.

### Database Tables

- `courses` - Metadata and source URL.
- `modules` - Logical grouping of videos.
- `videos` - YouTube ID, duration, completion status, transcript, and summary.
- `notes` - Per-video Markdown notes.
- `exams` - Generated quizzes with question/answer JSON and scores.
- `tags` & `course_tags` - Taxonomy and association.
- `user_preferences` - `ml_boundary_enabled` and `cognitive_limit_minutes`.

## 🔄 Development Workflow

- **Run Dev Server**: `dx serve`
- **Build Release**: `dx build --release`
- **Linting**: `cargo clippy --all-targets -- -D warnings`
- **Formatting**: `cargo fmt`
- **DB Rebuild**: `rm course_pilot.db && diesel migration run`

## 🧩 Key Integration Points

- **YouTube Fetcher**: Uses `rusty_ytdl` (custom fork/branch). Can use `YOUTUBE_COOKIES` env var for restricted content.
- **Transcripts**: Fetched via `yt-transcript-rs`.
- **LLM**: Gemini API via `genai-rs`. Used for chat, summaries, and exam generation.
- **Secrets**: Uses `keyring` for OS-native secure storage of API keys.

## ⚡ Performance & Quality

- Use `Arc` for sharing heavy objects (AppContext, Repositories).
- IDs (CourseId, VideoId, etc.) are newtypes around UUID or String - always use `std::str::FromStr` to parse.
- Handle `Option` and `Result` explicitly; avoid `unwrap()` in UI or Use Cases and better use this errors.
- Favor functional patterns (Iterators) over imperative loops.
