# Course Pilot: AI Engineering Guide

Follow these project-specific patterns and architectural constraints to maintain consistency and quality.

## 🏗️ Core Architecture (DDD Hexagonal)
We use a Strict Hexagonal Architecture to separate concerns:

- **src/domain/**: Core business logic. No external dependencies.
  - `entities/`: Rich domain models (Course, Module, Video, Note, Exam).
  - `ports/`: Trait definitions for infrastructure adapters.
  - `value_objects/`: Immutable data containers (IDs implement `FromStr` trait).
  - `services/`: Domain services (SessionPlanner, TitleSanitizer).
- **src/application/**: Use cases and orchestration.
  - `context.rs`: The DI container (`AppContext`). Holds all repositories in `Arc`.
  - `use_cases/`: Task-specific logic (IngestPlaylist, PlanSession, AskCompanion, TakeExam).
- **src/infrastructure/**: External implementations (adapters).
  - `persistence/`: SQLite repositories using Diesel (Course, Module, Video, Note, Exam).
  - `youtube/`, `llm/`, `ml/`: Third-party API wrappers.
- **src/ui/**: Dioxus 0.7 Desktop interface.

## 🧱 Dependency Injection (DI) Pattern
- **NEVER** instantiate repositories or adapters directly in UI components or use cases.
- Use `AppContext` to access shared infrastructure via `Arc`.
- Use `ServiceFactory` to build use cases with their required dependencies.

```rust
// In UI components:
let state = use_context::<AppState>();
let ctx = state.backend.as_ref()?;
let use_case = ServiceFactory::plan_session(ctx);
```

## ⚛️ UI Development (Dioxus 0.7)
- **State Management**: Use `AppState` via `use_context`. Access backend via `state.backend`.
- **Data Fetching**: Use custom hooks in `src/ui/hooks.rs` (e.g., `use_load_courses`, `use_load_videos_by_course`).
- **Async Actions**: Define in `src/ui/actions.rs` (e.g., `import_playlist`, `start_exam`, `ask_companion`).
- **Custom Components**: Add reusable UI in `src/ui/custom/` (Spinner, CardSkeleton, ErrorAlert, etc.).
- **Styling**: Use Tailwind CSS + DaisyUI classes. Theme in `assets/dx-components-theme.css`.

### Loading State Components
Use components from `src/ui/custom/loading.rs`:
- `Spinner` - Centered loading with optional message
- `CardSkeleton`, `PageSkeleton`, `VideoItemSkeleton` - Skeleton loaders
- `ErrorAlert`, `SuccessAlert` - Consistent feedback messaging

## 💾 Persistence Pattern
- Map Diesel models in `infrastructure/persistence/models.rs` to Domain Entities in repositories.
- Repository traits must reside in `domain/ports/repository.rs`.
- Database migrations are in `migrations/`. Run `diesel migration run` after schema changes.

### Database Tables
- `courses` - Course metadata (name, description, playlist_id)
- `modules` - Grouped videos with sort_order
- `videos` - Video data with youtube_id, duration_secs, description, is_completed
- `notes` - Per-video user notes
- `exams` - Generated quizzes with questions and scores
- `user_preferences` - ML toggle, cognitive_limit_minutes

## 🔄 Development Workflow
- **Run Dev Server**: `dx serve`
- **Build Release**: `dx build --release`
- **Run with Logs**: `RUST_LOG=info dx serve`
- **Linting**: `cargo clippy --all-targets -- -D warnings`
- **Formatting**: `cargo fmt`
- **Tests**: `cargo test --lib`
- **DB Rebuild**: `rm course_pilot.db && diesel migration run`

## 🧩 Key Integration Points
- **YouTube**: via `google-youtube3`. Requires `YOUTUBE_API_KEY` in `.env`.
- **LLM**: Gemini API via `genai-rs`. Optional, used for AI chat and exams.
- **ML**: `fastembed` for local semantic clustering of videos into modules.
- **Secrets**: Uses `keyring` for OS-native secure storage of API keys.
- **User Preferences**: Stored in `user_preferences` table (ML toggle, cognitive limit).

## ⚡ Performance & Quality
- Favor `Arc` sharing for heavy adapters (Repositories, YouTube client).
- Use `Signal` for all reactive UI state.
- Handle `Option` and `Result` explicitly in Use Cases for "Graceful Degradation".
- All IDs implement `FromStr` trait - use `std::str::FromStr` import when parsing.
