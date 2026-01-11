# Course Pilot: AI Engineering Guide

Follow these project-specific patterns and architectural constraints to maintain consistency and quality.

## 🏗️ Core Architecture (DDD Hexagonal)
We use a Strict Hexagonal Architecture to separate concerns:

- **src/domain/**: Core business logic. No external dependencies.
  - `entities/`: Rich domain models ([src/domain/entities/](src/domain/entities/)).
  - `ports/`: Trait definitions for infrastructure adapters ([src/domain/ports/](src/domain/ports/)).
  - `value_objects/`: Immutable data containers (IDs, YouTube URLs).
- **src/application/**: Use cases and orchestration.
  - `context.rs`: The DI container (`AppContext`). Holds all infrastructure adapters in `Arc`.
  - `use_cases/`: Task-specific logic (e.g., `IngestPlaylistUseCase`).
- **src/infrastructure/**: External implementations (adapters).
  - `persistence/`: SQLite repositories using Diesel.
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
let use_case = ServiceFactory::ingest_playlist(ctx)?;
```

## ⚛️ UI Development (Dioxus 0.7)
- **State Management**: Use `AppState` via `use_context`. Access backend functionality through `state.backend`.
- **Data Fetching**: Use custom hooks in [src/ui/hooks.rs](src/ui/hooks.rs) (e.g., `use_load_courses`). These hooks use `use_signal` and `use_effect`.
- **Async Actions**: Define complex or side-effect heavy operations in [src/ui/actions.rs](src/ui/actions.rs).
- **Styling**: strictly use Tailwind CSS classes. Use the theme variables from [assets/dx-components-theme.css](assets/dx-components-theme.css).

## 💾 Persistence Pattern
- Map Diesel models in `infrastructure/persistence/models.rs` to Domain Entities in the repository implementation.
- Repository traits must reside in `domain/ports/repository.rs`.
- Database migrations are in `migrations/`. Always run `diesel migration run` after schema changes.

## 🔄 Development Workflow
- **Run with Logs**: `RUST_LOG=info cargo run`
- **Linting**: `cargo clippy` (configured in [clippy.toml](clippy.toml)).
- **Formatting**: `cargo fmt` (configured in [rustfmt.toml](rustfmt.toml)).
- **DB Rebuild**: `rm course_pilot.db && diesel migration run`

## 🧩 Key Integration Points
- **YouTube**: via `google-youtube3`. Requires `YOUTUBE_API_KEY` in `.env`.
- **LLM**: Gemini API via `genai-rs`. Optional, used for AI chat and exams.
- **ML**: `fastembed` for local semantic clustering of videos into modules.
- **Secrets**: Uses `keyring` for OS-native secure storage of API keys.

## ⚡ Performance & Quality
- Favor `Arc` sharing for heavy adapters (Repositories, YouTube client).
- Use `Signal` for all reactive UI state.
- Handle `Option` and `Result` explicitly in Use Cases to ensure "Graceful Degradation" (e.g., app works without AI keys).
