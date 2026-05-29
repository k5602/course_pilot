# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.2.0] - 2026-05-29

### Added

- Error toasts for all user-initiated mutations (module create, note save, quiz results, credential storage).
- Video completion checkbox now persists state to the database.
- Stream resolution timeout/recovery with user-facing error messages.
- Batch repository methods (`save_batch`, `index_batch`) for atomic ingestion.
- Unit tests for domain entities (analytics, exam, video, note) and value objects (video_source, exam_difficulty).
- `DiscoveryFailed` error variant for GStreamer unavailability.
- CSS rules for `.right-panel` and `.notes-text-view` classes.
- macOS keyring support via `apple-native` feature.

- **Redesigned Interactive Quiz System & University-Grade MCQ Prompts**: Includes university-grade MCQs featuring plausible distractor fallacies, deep conceptual testing, and thorough refutations. Equipped with user interface additions such as selected option highlighting, correct/incorrect status badges, and expandable explanation drawers.
- **Dynamic Floating Popup Notes Window**: A globally accessible notes popup triggered via `Ctrl+N` featuring a dual-mode setup: Type Mode (text editor) and Preview Mode (supporting Pango-compiled rich markdown, LaTeX rendering for inline and block mathematical equations, and built-in Pango safety escaping). Includes an "Insert Video Reference" shortcut.
- **Upgraded Default Model**: Upgraded the default AI backend model to Gemini 3.1 Flash Lite.
- **Dynamic Duration Format**: Implemented consistent H:MM:SS formatting for video durations across the user interface.
- **Resume Study Dashboard**: A redesigned dashboard featuring a gradient hero banner, glassmorphic stat cards, overall course completion levelbars, and interactive progress cards for enrolled courses with direct resume-study navigation.
- **Scroll-Down Video Sub-panels**: Positioned below the video player, adding a fullscreen video toggle button, double-click gestures, and `F`/`F11` shortcuts. Features sections for Associated Quizzes, Video Summary, and Video Transcript reader.
- **AI Chat UI Enhancements**: Refactored the chat interface into clear speech bubbles, aligning User messages to the right and Assistant responses to the left. Supports immediate rendering of sent messages and sending on pressing the `Enter` key.
### Changed

- Refactored ingest use cases to remove direct diesel imports from the application layer (hexagonal architecture compliance).
- Replaced all `expect()` panics in `connection.rs` with `Result` propagation.
- Replaced `catch_unwind` anti-pattern in settings with direct `Result` handling.
- Replaced GTK widget `unwrap()` chains with safe `Option` patterns in course/quiz list factories.
- Replaced fragile string-comparison play/pause state with `Cell<bool>`.
- Tokio runtime initialization now degrades gracefully instead of panicking.
- GStreamer `set_state()` errors are now logged instead of silently discarded.
- Chat panel now displays per-video history instead of mixing all videos.
- Centralized default LLM model into a single constant.
- Rewrote `ARCHITECTURE.md` to accurately describe the GTK4/GStreamer/Diesel stack.
- Quiz question rendering deduplicated into a single function.
- Pass threshold uses `Exam::PASS_THRESHOLD` constant instead of hardcoded `0.7`.
- **AI Chat & Quiz Context Optimization**: Refactored the LLM context pipeline to no longer pass noisy raw transcripts to the AI chat or quiz generator. The pipeline now utilizes high-fidelity AI-extracted Video Summaries to deliver dense, high-performance, and cost-effective context.
- **UI Video Summary Display**: Configured the dashboard sub-panel below the video player to display the clean, high-fidelity Video Summary instead of raw transcripts.

### Fixed

- Chat history bug: navigating between videos would show messages from all videos mixed together.
- Video completion checkbox had no signal handler - checking it did nothing.
- Silent data loss on module create, note save, video reorder, quiz result persistence, and credential storage.
- Stream resolution failure left UI stuck "Loading..." with no recovery.
- Duplicate `keyring` dependency declarations in `Cargo.toml`.
- Stale `.gitignore` entries from removed Dioxus/Python/Node.js tooling.
- CSS dark mode shadows invisible against dark backgrounds.
- Duplicate `expander` CSS selector blocks.

### Removed

- Dead code: `NoopPresenceProvider`, `FallbackTitleGenerator`, `module_title_for()`.
- Redundant `chat_history` field from `AppState` (replaced by `chat_history_by_video`).
- Dead `shortcuts.rs` module.
- Stale Dioxus/Python/Node.js `.gitignore` entries.

## [0.1.4] - 2026-05-12

### Added

- Migrated entire UI from Dioxus to GTK4/libadwaita with responsive NavigationView.
- Quiz system: MCQ generation, quiz list, quiz view with pass/fail scoring.
- Video quality selector with preferred quality persistence in settings.
- Custom LLM model configuration via builder and `LLM_MODEL` env var.
- Transcript context for AI companion and MCQ generation.
- Session planner respects `week_study_days` schedule.
- HTTP relay servers for local video and YouTube embed proxy.
- Async yt-dlp execution with direct stream URL resolution.
- Right panel width preference.

### Changed

- Adopted NavigationView and ListView for navigation and list rendering.
- Persisted courses, modules, and videos in a single database transaction.
- Precomputed normalized names and tokens for faster subtitle matching.
- Streamlined CI and release workflows.

### Fixed

- Single-stream yt-dlp formats for reliable video playback.
- Proper file URL encoding for local video paths.
- Sanitizer infinite loops and edge-case handling.
- Multi-byte UTF-8 character handling in transcript chunking.

### Removed

- Dioxus frontend framework and all Dioxus-specific files.


## [0.1.3] - 2026-01-27

### Added

- Title-aware module grouping for intelligent content organization during ingestion.
- `UpdatePresenceUseCase` to decouple external presence synchronization logic.
- Standardized `LoadResult<T>` pattern for UI data fetching hooks.
- Database support for local media by making `videos.youtube_id` nullable.

### Changed

- Refactored Discord Presence synchronization with improved activity mapping and configurable intervals.
- Hardened LLM prompts for Companion, Examiner, and Summarizer agents to ensure grounding and strict output formats.
- Overhauled `SubtitleCleaner` with advanced VTT header detection, speaker label stripping, and whitespace normalization.
- Optimized UI performance by transitioning to keyed effects and reducing redundant hook executions.
- Enhanced persistence layer with better row-to-entity mappers and atomic `ON CONFLICT` updates.
- Optimized Cargo profiles (Thin LTO, symbol stripping) to significantly reduce binary size and link-time resources.

### Fixed

- Resolved CI disk space exhaustion on Linux runners.
- Fixed Diesel schema generation by excluding FTS5 internal tables.

## [0.1.2] - 2026-01-20

### Added

- CDN loading and retry logic for the markdown renderer.
- Module splitting improvements and quiz retake support.
- LLM integration updates (#20).

### Changed

- Release workflow improvements and artifact handling.
- Dioxus CLI caching and release bundle simplifications.


## [0.1.0] - 2026-01-17

### Added

- Initial Release
- Dashboard tabbed layout with analytics overview, courses, and tags.
- Settings tabbed layout with integrations, preferences, and about sections.
- App analytics aggregation use case and UI hook.
- User preferences persistence and repository.
- Linux launcher script for dependency checks and guided installs.
- Linux developer setup script for distro-specific dependencies.
- `.deb` packaging via `dx bundle` in the release workflow.
- Release artifacts with checksums and improved release notes.
- `CHANGELOG.md` for release tracking.

### Changed

- README updated with Linux dependency instructions and distribution details.

### Fixed

- Improved DB-to-entity conversions and course `created_at` handling.
