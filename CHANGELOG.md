# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
