# Course Pilot: AI Engineering Guide

**Course Pilot** is a cognitive-enhancement platform designed to transform unstructured Noisy video courses into personalized, AI-augmented educational experiences. It automates the creation of logical learning structures, generates study schedules based on cognitive load, and provides deep-learning tools like AI-generated exams, video summarization, and interactive companion chat wittout noise of adds and suggestion algorithms.

The application is built as a high-performance Rust desktop suite using Dioxus 0.7, adhering to strict Hexagonal (Ports & Adapters) architecture and Domain-Driven Design (DDD) principles to ensure modularity, testability, and long-term maintainability.

## 🏗️ Core Architecture (DDD Hexagonal)

We maintain a clear separation between business logic and technical implementation:

- **`src/domain/`**: The "Inner Circle". No external dependencies.
  - `entities/`: Domain models (Course, Module, Video, Note, Exam, Tag).
  - `ports/`: Trait definitions (Repository, LLM, Transcript, Presence).
  - `value_objects/`: Immutable types (CourseId, VideoId) using the "Newtype" pattern.
- **`src/application/`**: Orchestration layer.
  - `context.rs`: The DI container (`AppContext`).
  - `use_cases/`: Atomic business operations (e.g., `IngestPlaylistUseCase`, `SummarizeVideoUseCase`).
- **`src/infrastructure/`**: The "Adapters". Implementation of ports using external crates.
  - `persistence/`: Diesel-based SQLite repositories.
  - `llm/`: Gemini API via `genai-rs`.
  - `keystore/`: OS-native secure storage via `keyring`.
  - `discord/`: Rich Presence synchronization.
- **`src/ui/`**: Dioxus 0.7 Desktop interface.

## 🧱 Dependency Injection (DI) & Use Cases

- **ServiceFactory**: Always use `ServiceFactory` to instantiate use cases. It handles wiring dependencies from `AppContext`.
- **AppContext**: Shared via `Arc`. Holds initialized repositories and adapters.
- **Manual Wiring**: Infrastructure adapters are injected into Use Cases as `Arc<dyn Port>`.

```rust
// Pattern: Using a use case in an action
let ctx = state.backend.as_ref()?;
let use_case = ServiceFactory::plan_session(ctx);
use_case.execute(input).await;
```

## ⚛️ UI Development (Dioxus 0.7)

### State Management

- **`AppState`**: Global state accessed via `use_context::<AppState>()`.
- **Signals**: Use `Signal<T>` for all reactive data. Favor `read()` and `set()` / `write()`.

### Data Fetching (Unified Hook Pattern)

We use a standardized async loader pattern in `src/ui/hooks.rs`:

- **`use_async_loader`**: Handles background tasks using `tokio::task::spawn_blocking`.
- **`LoadResult<T>`**: Returns `{ data: Signal<T>, state: LoadState }`.
- **`LoadState`**: Provides `is_loading` and `error` signals for UI feedback.
- **Keyed Effects**: Hooks are "keyed" to refresh when the backend or IDs change.

```rust
// Hook Usage Example
let result = use_load_course(backend, &course_id);
if *result.state.is_loading.read() { return rsx! { Spinner {} }; }
```

### Async Actions

Mutations (POST/PUT/DELETE equivalents) reside in `src/ui/actions.rs`. They are `async` functions that interact with use cases and return `Result` or custom enum results.

## 💾 Persistence & Search

- **Diesel Mapping**: Database models (`infrastructure/persistence/models.rs`) map to Domain Entities via `From`/`Into` or manual mapping in repositories.
- **Search Indexing**: `SearchRepository` (FTS5) is updated transactionally alongside core entity changes in use cases.
- **Transactions**: Use the `db_pool` in `AppContext` for atomic operations.

## 🧩 Strategic Integration Patterns

- **Secret Management**: API keys (Gemini) must never be hardcoded. Use `AppContext::keystore` for secure OS-native storage.
- **Presence Sync**: Use the `use_presence_sync` hook to automatically update Discord status based on current `Route` and `AppState`.
- **Debouncing**: Use `use_debounced_value` for search inputs to prevent database/API thrashing.
- **Rich Presence**: The `Activity` enum in domain defines valid presence states, mapped from UI routes.

## ⚡ Performance & Quality

- **Blocking Tasks**: Heavy computations or DB queries in UI hooks MUST use `spawn_blocking`.
- **Error Handling**: Use `thiserror` for custom error types in application/infrastructure. Handle errors explicitly in UI actions to provide user feedback.
- **Type Safety**: Newtype IDs (e.g., `CourseId(Uuid)`) prevent accidental ID swapping across different entity types.
- **Functional Idioms**: Prefer `.map()`, `.and_then()`, and `.filter()` over manual `if let` nesting where readable.
