# Course Pilot 🎓

[![Rust CI](https://github.com/k5602/course_pilot/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/k5602/course_pilot/actions/workflows/rust.yml) [![Clippy](https://img.shields.io/github/actions/workflow/status/k5602/course_pilot/rust.yml?branch=main&label=clippy)](https://github.com/k5602/course_pilot/actions/workflows/rust.yml)

> Transform YouTube playlists into structured, intelligent study plans

A modern Rust desktop application that automatically analyzes video-based courses, creates logical learning structures, and generates personalized study schedules.

![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)
![Dioxus](https://img.shields.io/badge/dioxus-0.7+-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Development Status](https://img.shields.io/badge/status-active%20development-brightgreen.svg)

## ✨ Features

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

- **Dark Theme** - Eye-friendly DaisyUI dark theme
- **Settings Persistence** - Preferences saved to database
- **Course Management** - Delete courses with confirmation dialog
- **Loading States** - Skeleton loaders and spinners throughout

## 🚀 Quick Start

### Prerequisites

- Rust 1.80+
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started): `cargo install dioxus-cli`

### 1. Clone & Configure

```bash
git clone https://github.com/k5602/course_pilot.git
cd course_pilot
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=course_pilot.db
YOUTUBE_API_KEY=your_youtube_api_key_here
GEMINI_API_KEY=your_gemini_api_key_here  # optional
```

### 2. Setup Database

```bash
diesel migration run
```

### 3. Run

```bash
dx serve
# or with logging:
RUST_LOG=info dx serve
```

### 4. Build Release

```bash
dx build --release
```

## ⚙️ Configuration

| Variable          | Required | Default           | Description                               |
| ----------------- | -------- | ----------------- | ----------------------------------------- |
| `DATABASE_URL`    | No       | `course_pilot.db` | SQLite database path                      |
| `YOUTUBE_API_KEY` | No       | -                 | Optional YouTube Data API v3 key (unused) |
| `GEMINI_API_KEY`  | No       | -                 | Gemini API key for AI features            |

## 🔑 API Keys

### YouTube API Key (Optional)

Course Pilot uses API-free playlist import by default. A YouTube API key is not required for normal use.

### Gemini API Key (Optional)

1. Visit [Google AI Studio](https://aistudio.google.com/app/apikey)
2. Create an API key

> API keys can also be configured in Settings page within the app.

## 🛠️ Development

```bash
# Run development server
dx serve

# Run tests
cargo test --lib

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt

# Rebuild database
rm course_pilot.db && diesel migration run
```

## 📁 Project Structure

```
src/
├── application/      # Use cases and DI container
├── domain/           # Entities, ports, value objects
├── infrastructure/   # SQLite, YouTube, LLM adapters
└── ui/
    ├── pages/        # Dashboard, CourseView, VideoPlayer, etc.
    ├── custom/       # Reusable components
    └── hooks.rs      # Data loading hooks
```

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.
