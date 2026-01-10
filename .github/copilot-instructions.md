# Course Pilot: AI Agent Instructions

## 🏗️ Architecture

**Local-First Learning Sanctuary** - Transforms YouTube playlists into structured study plans.

### DDD Hexagonal Architecture
```
src/
├── domain/          # Entities, Value Objects, Ports, Services
├── application/     # Use Cases, AppContext (DI Container)
├── infrastructure/  # Adapters (SQLite, YouTube, FastEmbed, Gemini, Keyring)
└── schema.rs        # Diesel-generated
```

### Key Patterns
- **Domain Ports** define interfaces; **Infrastructure Adapters** implement them
- **AppContext** wires all dependencies via `AppConfig.from_env()` or `AppConfigBuilder`
- **ServiceFactory** creates use cases with injected dependencies

## 🛠️ Tech Stack

| Layer | Technology |
|-------|------------|
| UI | Dioxus 0.7 Desktop|
| Database | Diesel + SQLite + r2d2 pool |
| YouTube | google-youtube3 v7 |
| ML Embeddings | fastembed (optional) |
| LLM | genai-rs (Gemini API) |
| Secrets | keyring (OS keychain) |
| Config | dotenvy (.env files) |

## 🔧 Configuration

All config via `.env` (see `.env.example`):

```env
DATABASE_URL=course_pilot.db
YOUTUBE_API_KEY=required
GEMINI_API_KEY=optional
ENABLE_ML_BOUNDARY_DETECTION=false  # Default: import playlists as-is
```

For GUI, use `AppConfigBuilder`:
```rust
let config = AppConfig::builder()
    .youtube_api_key("...")
    .enable_ml_boundary_detection(true)
    .build();
```

## 🔄 Workflows

```bash
# Development
cargo check && cargo test

# Run with logging
RUST_LOG=info cargo run
```

## 🧩 Key Components

| Component | Purpose |
|-----------|---------|
| `AppContext` | DI container, holds all adapters |
| `ServiceFactory` | Creates use cases with dependencies |
| `IngestPlaylistUseCase` | YouTube → structured course |
| `PlanSessionUseCase` | Daily study scheduling |
| `AskCompanionUseCase` | Contextual AI Q&A |
| `TakeExamUseCase` | MCQ generation & grading |

## ⚡ Principles

- **Privacy First**: All data local, BYOK for cloud APIs
- **Completion > Consumption**: Focus on learning retention
- **Graceful Degradation**: Works without ML/LLM (basic import mode)
