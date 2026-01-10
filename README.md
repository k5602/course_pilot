# Course Pilot 🎓

[![Rust CI](https://github.com/k5602/course_pilot/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/k5602/course_pilot/actions/workflows/rust.yml) [![Clippy](https://img.shields.io/github/actions/workflow/status/k5602/course_pilot/rust.yml?branch=main&label=clippy)](https://github.com/k5602/course_pilot/actions/workflows/rust.yml)

> Transform YouTube playlists and video folders into structured, intelligent study plans

A modern Rust desktop application that automatically analyzes video-based courses, creates logical learning structures, and generates personalized study schedules. Built with performance, accessibility, and user experience at its core.

![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)
![Dioxus](https://img.shields.io/badge/dioxus-0.6+-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Development Status](https://img.shields.io/badge/status-active%20development-brightgreen.svg)

## 🌟 What Makes Course Pilot Special

Course Pilot bridges the gap between scattered video content and structured learning. Whether you're a student tackling online courses, a professional learning new skills, or an educator organizing content, Course Pilot transforms chaotic video collections into organized, trackable learning experiences.

### Key Problems Solved

- **Content Chaos**: No more losing track of where you left off in long video series
- **Poor Structure**: Automatically organizes videos into logical modules and sections
- **No Progress Tracking**: Visual progress indicators and completion tracking
- **Time Management**: Intelligent scheduling based on your availability
- **Note Scattered**: Centralized note-taking tied to specific videos and topics

## 🎨 Design Philosophy

### **User-Centered Design**

- **Keyboard Navigation**: Complete keyboard accessibility
- **Dark Mode**: Eye-friendly themes for all lighting conditions
- **Responsive Layout**: Adapts to various screen sizes and resolutions
- **Intuitive UI**: Clean, minimalistic interface focused on usability

### **Performance by Design**

- **Rust's Zero-Cost Abstractions**: Maximum performance, minimal overhead
- **Efficient Rendering**: Virtual DOM with smart diffing
- **Lazy Loading**: Components and data loaded on demand
- **Optimized Algorithms**: Fast video analysis and scheduling

## Quick Start

### 1. Configure

Copy the example config and add your API keys:

```bash
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=course_pilot.db
YOUTUBE_API_KEY=your_youtube_api_key_here
GEMINI_API_KEY=your_gemini_api_key_here  # optional
ENABLE_ML_BOUNDARY_DETECTION=false       # optional
```

### 2. Run

```bash
cargo run
# or with logging:
RUST_LOG=info cargo run
```

## Configuration Options

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | No | `course_pilot.db` | SQLite database path |
| `YOUTUBE_API_KEY` | Yes | - | YouTube Data API v3 key |
| `GEMINI_API_KEY` | No | - | Gemini API key for AI features |
| `ENABLE_ML_BOUNDARY_DETECTION` | No | `false` | Use ML to detect module boundaries |

## API Keys

### YouTube API Key (Required)

1. Go to [Google Cloud Console](https://console.cloud.google.com)
2. Create a project or select existing
3. Enable **YouTube Data API v3**
4. Create an API key under Credentials

### Gemini API Key (Optional)

1. Visit [Google AI Studio](https://aistudio.google.com/app/apikey)
2. Create an API key

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
