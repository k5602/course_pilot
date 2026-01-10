# Course Pilot: AI Agent Instructions

## 🏗️ Architecture & Philosophy
Course Pilot is a **Local-First Learning Sanctuary** designed to transform YouTube playlists into structured study plans.
- **Distraction-Free**: No recommendations or comments. Just the video and your progress.
- **Local-First**: User data and progress are stored entirely on the local machine (Diesel/SQLite).
- **Hybrid AI**: 
  - **Local ML**: For semantic vectorization and clustering of video titles (`fastembed-rs`).
  - **Cloud LLM**: User-provided Gemini API keys (BYOK) for contextual Q&A and MCQ generation.
## 🛠️ Tech Stack & Conventions
- **Rust (1.80+, Edition 2024)**: Use modern Rust patterns.
- **Dioxus 0.7 (Desktop)**:
  - Use **Signals** (`use_signal`, `use_memo`) for reactive state.
  - Functional components only. Use `rsx!` for UI.
  - Desktop-specific features for video playback (YouTube iFrame integration).
- **Tailwind CSS 4.0 & DaisyUI 5.0**:
  - Styles are defined in `assets/tailwind.css` using the V4 CLI.
  - Use DaisyUI themes: `corporate` (Light) and `business` (Dark).
  - CSS is output to `assets/tailwind.out.css`.

## 🔄 Critical Workflows
- **CSS Development**: Run `npm run watch:css` to automatically rebuild styles when editing Rust files or CSS.
- **Build & Run**: Use `cargo run` for the desktop application.
- **Linting**: Respect `clippy.toml` and `rustfmt.toml`.

## 🧩 Project Patterns
- **Ingestion Pipeline**: Extraction -> Normalization -> Embedding -> Clustering -> Persistence.
- **Security**: Never hardcode API keys. Always use a BYOK model where the user provides their Gemini key.
- **UI Components**: Keep components small and focused.using Dioxus-components from the dx cli tool.

## 📁 Key Directories
- `src/`: Main application logic (currently being rebuilt).
- `docs/`: Comprehensive technical specs and philosophy (High source of truth).
- `assets/`: UI resources including Tailwind config and output.
- `theme_config.toml`: Current active DaisyUI theme.

When implementing features, always prioritize **completion over consumption** and **privacy over convenience**.
