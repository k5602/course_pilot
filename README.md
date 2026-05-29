# Course Pilot

[![Rust CI](https://github.com/k5602/course_pilot/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/k5602/course_pilot/actions/workflows/rust.yml)

> Transform YouTube playlists into structured, intelligent study plans

A modern Rust desktop application that automatically analyzes video-based courses, creates logical learning structures, and generates personalized study schedules.

![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)
![GTK4](https://img.shields.io/badge/GTK4-0.9+-blue.svg)
![GStreamer](https://img.shields.io/badge/GStreamer-0.25+-lightblue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Development Status](https://img.shields.io/badge/status-active%20development-brightgreen.svg)

## Features

### Core Functionality

- **Playlist Import** - Import YouTube playlists with automatic metadata extraction and parsing.
- **Smart Modules** - Automatic ordering-preserved grouping of videos into logical modules based on title patterns.
- **Progress Tracking** - Track video completion with persisted visual progress bars and completions.
- **Video Navigation** - Previous/next video links for seamless, sequential learning flow.
- **Dynamic Duration Formats** - High-fidelity H:MM:SS formatted duration displays standard across all UI elements.

### Learning Tools

- **Resume Study Dashboard** - An immersive learning homepage with a gradient hero banner, glassmorphic stat cards, overall course completion levelbars, and interactive progress cards for quick resume-study navigation.
- **Redesigned Interactive Quiz System** - Deep university-grade MCQ assessments featuring plausible distractor fallacies and comprehensive refutations. Features interactive elements like selected option highlighting, correct/incorrect visual status badges, and slide-out explanation drawers.
- **AI Chat Companion** - Ask dense, context-aware questions during playback. Chat responses are optimized via Gemini 3.1 Flash Lite.
- **AI Chat & Quiz Context Optimization** - Optimized context pipeline that utilizes dense, high-fidelity AI-extracted summaries rather than noisy raw transcripts. This ensures high-performance, cost-effective, and rapid generation.
- **Dynamic Floating Popup Notes Window** - Globally accessible via a `Ctrl+N` hotkey. Features a dual-mode workflow: **Type Mode** (markdown editor) and **Preview Mode** (compiled Pango rich markdown & LaTeX rendering supporting inline and block mathematical equations). Includes an "Insert Video Reference" hotkey and robust Pango safety escaping.

### User Experience & Media Player

- **Scroll-Down Video Sub-panels** - Rich sections placed directly below the player comprising Associated Quizzes, high-fidelity Video Summaries, and a Video Transcript reader.
- **Advanced Media Controls** - Support for a fullscreen video toggle button, double-click gestures for fullscreen, and `F`/`F11` hotkey actions.
- **AI Chat UI Enhancements** - Fluid conversation flow displaying User right-aligned and Assistant left-aligned speech bubbles, immediate local rendering of sent messages, and input submission mapped to the `Enter` key.
- **Dark Theme** - Sleek, eye-friendly dark styling natively integrated via libadwaita.
- **Settings Persistence** - User preferences, including model configurations, are saved securely in the SQLite database and OS keyring.
- **Loading States** - Fluid UX using skeleton loaders and loading spinners throughout the application.

## Quick Start

### Prerequisites

- **Rust 1.88+**

#### Linux Dependencies

Building and running on Linux requires several system libraries (GTK4, libadwaita, GStreamer, SQLite3).

**Automatic Setup (Recommended):**

```bash
chmod +x scripts/setup-linux.sh
./scripts/setup-linux.sh
```

**Manual Installation:**
| Distribution | Command |
| :--- | :--- |
| **Ubuntu/Debian** | `sudo apt install libgtk-4-dev libadwaita-1-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgraphene-1.0-dev libsqlite3-dev` |
| **Fedora** | `sudo dnf install gtk4-devel libadwaita-devel gstreamer1-devel gstreamer1-plugins-base-devel graphene-devel sqlite-devel` |
| **Arch Linux** | `sudo pacman -S gtk4 libadwaita gstreamer gst-plugins-base graphene sqlite` |

### 1. Clone & Configure

```bash
git clone https://github.com/k5602/course_pilot.git
cd course_pilot
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=course_pilot.db
GEMINI_API_KEY=your_gemini_api_key_here  # optional
```

### 2. Setup Database

```bash
diesel migration run
```

### 3. Build & Run

```bash
cargo run --release
# or with logging:
RUST_LOG=info cargo run --release
```

## Distribution

### Linux

We provide a generic tarball (.tar.gz) with each release. Extract and run the binary directly:

```bash
tar -xzf course-pilot-linux-x64.tar.gz
./course_pilot
```

## Configuration

| Variable          | Required | Default           | Description                               |
| ----------------- | -------- | ----------------- | ----------------------------------------- |
| `DATABASE_URL`    | No       | `course_pilot.db` | SQLite database path                      |
| `YOUTUBE_API_KEY` | No       | -                 | Optional YouTube Data API v3 key (unused) |
| `GEMINI_API_KEY`  | No       | -                 | Gemini API key for AI features            |

## API Keys

### YouTube API Key (Optional)

Course Pilot uses API-free playlist import by default. A YouTube API key is not required for normal use.

### Gemini API Key (Optional)

1. Visit [Google AI Studio](https://aistudio.google.com/app/apikey)
2. Create an API key

> API keys can also be configured in Settings page within the app.

## Development

```bash
# Run
cargo run --release

# Run tests
cargo test --lib

# Lint
cargo clippy

# Format
cargo fmt

# Rebuild database
rm course_pilot.db && diesel migration run
```

## Project Structure

```
src/
├── application/      # Use cases and DI container
├── domain/           # Entities, ports, value objects
├── infrastructure/   # SQLite, YouTube, LLM adapters
└── ui/
    ├── app.rs        # Main application entry point
    ├── layout.rs     # Top-level layout
    ├── navigation.rs # Navigation state management
    ├── right_panel.rs# Right sidebar panel
    ├── shortcuts.rs  # Keyboard shortcuts
    ├── state.rs      # UI state management
    ├── toast.rs      # Toast notifications
    ├── css.rs        # Custom CSS styling
    ├── dialogs/      # Import dialogs, etc.
    └── pages/        # Dashboard, CourseView, VideoPlayer, etc.
```

## License

MIT License - see [LICENSE](LICENSE) for details.
