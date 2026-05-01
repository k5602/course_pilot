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

- **Playlist Import** - Import YouTube playlists with automatic metadata extraction
- **Smart Modules** - Automatic grouping of videos into logical modules
- **Progress Tracking** - Track video completion with visual progress bars
- **Video Navigation** - Previous/next video links for seamless learning flow

### Learning Tools

- **Session Planning** - Plan study sessions based on your cognitive limit (15-120 min/day)
- **AI Companion** - Ask questions about video content (requires Gemini API)
- **Quiz Generation** - Auto-generated MCQ quizzes from video context
- **Per-Video Notes** - Take and persist notes for each video

### User Experience

- **Dark Theme** - Eye-friendly dark theme (libadwaita)
- **Settings Persistence** - Preferences saved to database
- **Course Management** - Delete courses with confirmation dialog
- **Loading States** - Skeleton loaders and spinners throughout

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
