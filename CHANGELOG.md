# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

### Fixed

- Subtitle matching test fixes in local media handling.

### Deprecated

- TBA

### Removed

- TBA

### Security

- TBA

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
