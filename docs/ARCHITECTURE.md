# Technical Architecture: Course Pilot

## 1. Local-First Persistence (Diesel + SQLite)

The application follows a "Sanctuary" model where user data, notes, and progress are stored entirely on the local machine.

- **Engine**: Diesel ORM with SQLite.
- **Strategy**: Strong relational integrity to support the "Knowledge Graph" aspect of learning.
- **Key Entities**:
  - `Courses`: High-level metadata (Title, YouTube Playlist ID).
  - `Modules`: Logical segments created by the ML Clustering engine.
  - `Videos`: Source references, sort order, and binary completion status.
  - `Exams`: Logs of AI-generated MCQs and user results.

## 2. Intelligence Stack

The app utilizes a hybrid intelligence model to balance privacy, speed, and reasoning capability.

### A. Local ML Engine (The Structurer)

- **Library**: `fastembed-rs` (or equivalent Candle-based embeddings).
- **Role**: Performs semantic vectorization of video titles and descriptions locally.
- **Task**: Uses clustering algorithms (e.g., K-Means) to transform a flat list of videos into a structured, modular curriculum.

### B. Cloud LLM (The Companion/Examiner)

- **Engine**: Gemini (BYOK - Bring Your Own Key).
- **Philosophy**: Course Pilot is a client, not a service. Users provide their own API keys to maintain ownership of their usage and costs.
- **Role**: High-level reasoning and content validation.
- **Tasks**:
  - **Contextual Q&A**: Answering questions based on the current video's metadata.
  - **MCQ Generation**: Generating structured JSON exams for the "Examiner" mode.

## 3. The Ingestion Pipeline

1. **Fetch**: Extract raw metadata from YouTube playlists.
2. **Normalize**: Strip algorithmic noise (emojis, clickbait keywords, tags).
3. **Embed**: Generate local semantic vectors for every video.
4. **Cluster**: Group videos into modules based on semantic proximity.
5. **Persist**: Store the structured course in the local SQLite database.

## 4. Interaction Model

- **UI Framework**: Dioxus (Desktop).
- **Video Playback**: YouTube iFrame wrapper with custom CSS injection to strip distractions (related videos, sidebar, comments).
- **State Management**: Dioxus Signals for real-time progress tracking.

## 5. Security & Privacy

- **BYOK (Bring Your Own Key)**: No central server handles your intelligence requests. Communication is direct from the local app to the provider (Google Gemini).
- **API Keys**: Stored securely on the user's machine (e.g., via OS keyring or local encrypted storage).
- **Zero Telemetry**: No tracking, no analytics, and no cloud-syncing of course data. Your learning sanctuary is yours alone.
