# Course Pilot 🎓

> Transform YouTube playlists and video folders into structured, intelligent study plans

A modern Rust desktop application that automatically analyzes video-based courses, creates logical learning structures, and generates personalized study schedules. Built with performance, accessibility, and user experience at its core.

![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)
![Dioxus](https://img.shields.io/badge/dioxus-0.6+-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## 🌟 What Makes Course Pilot Special

Course Pilot bridges the gap between scattered video content and structured learning. Whether you're a student tackling online courses, a professional learning new skills, or an educator organizing content, Course Pilot transforms chaotic video collections into organized, trackable learning experiences.

### Key Problems Solved
- **Content Chaos**: No more losing track of where you left off in long video series
- **Poor Structure**: Automatically organizes videos into logical modules and sections
- **No Progress Tracking**: Visual progress indicators and completion tracking
- **Time Management**: Intelligent scheduling based on your availability
- **Note Scattered**: Centralized note-taking tied to specific videos and topics

## 🚀 Features

### ✅ Current Features (MVP Complete)

#### **Intelligent Course Import**
- **YouTube Playlists**: Paste any YouTube playlist URL for instant import
- **Local Video Folders**: Native file picker with drag-and-drop support
- **Metadata Extraction**: Automatic title, duration, and content analysis
- **Bulk Processing**: Handle courses with hundreds of videos efficiently

#### **Smart Course Structuring**
- **NLP-Powered Analysis**: Advanced pattern recognition in video titles
- **Automatic Module Detection**: Groups related content into logical sections
- **Difficulty Assessment**: Estimates complexity based on content patterns
- **Hierarchical Organization**: Creates clear learning progressions

#### **Personalized Study Planning**
- **Flexible Scheduling**: 1-14 sessions per week, 15-180 minutes each
- **Weekend Options**: Include or exclude weekend study sessions
- **Progress Adaptation**: Schedules adjust based on actual completion rates
- **Multiple Strategies**: Front-loaded, even distribution, or custom pacing

#### **Modern Desktop UI**
- **Unified Design System**: Consistent, accessible components across the app
- **Light/Dark Themes**: Automatic theme switching with system preferences
- **Responsive Layout**: Adapts beautifully to different screen sizes
- **Keyboard Navigation**: Full keyboard accessibility support
- **Touch-Friendly**: Works great on touch-enabled devices

#### **Robust Data Management**
- **SQLite Backend**: Reliable, embedded database with no setup required
- **JSON Serialization**: Future-proof data formats for easy migration
- **Backup & Restore**: Export/import your entire course library
- **Performance Optimized**: Handles large course collections efficiently

### 🎯 Next Priority Features (In Development)

#### **Interactive Course Management** ✅ COMPLETED
Full CRUD operations for course management with comprehensive hook system:
- **Course Dashboard**: Grid view with progress tracking and visual indicators
- **Course Creation**: Add new courses with validation and error handling via `use_courses()` hook
- **Course Editing**: Complete modal-based editing with form validation and real-time feedback
- **Course Deletion**: Confirmation dialogs with cascade deletion warnings and undo functionality
- **Progress Visualization**: Real-time progress bars and completion tracking via `use_course_progress()` hook
- **Navigation Integration**: Seamless routing between dashboard and course views with state management
- **Modal Management**: Proper modal state management with `use_modal_manager()` hook
- **Form Management**: Reactive form handling with `use_form_manager()` for validation and state
- **Error Handling**: Comprehensive error handling with user-friendly messages and recovery options
- **Auto-Refresh**: Course list automatically refreshes after create, update, and delete operations
- **UI Polish**: Improved ActionMenu component with appropriate ellipsis icon for better UX

#### **"Aha!" Notes Panel**
Capture insights while you learn:
- **Per-Video Notes**: Rich text editor for each video with auto-save
- **Timestamp Linking**: Notes tied to specific moments in videos
- **Search & Filter**: Find notes across all your courses instantly
- **Export Notes**: Generate study guides from your collected insights
- **Markdown Support**: Format notes with headers, lists, and emphasis

## 🔮 Future Enhancements: The Power-Up Suite

### **Knowledge Hub Exporter**
Turn your learning into lasting value:
- **Structured Export**: Generate beautiful Markdown or PDF study guides
- **Course Summaries**: Automatic compilation of all notes for completed courses
- **Custom Templates**: Choose from academic, professional, or personal formats
- **Share & Collaborate**: Export shareable study materials
- **Version Control**: Track how your understanding evolves over time

### **Focus Mode Timer**
Integrate proven productivity techniques:
- **Pomodoro Integration**: Built-in 25/5 minute work/break cycles
- **Custom Timer**: Set your own focus periods based on video length
- **Daily Goals**: Track study time against personal targets
- **Distraction Blocking**: Minimize other apps during focus sessions
- **Progress Rewards**: Gamification elements to maintain motivation

### **Smart Review & Recall System**
Leverage spaced repetition for long-term retention:
- **Spaced Repetition Scheduling**: "Review in 3 days?" prompts after module completion
- **Note Review Sessions**: Revisit your insights at optimal intervals
- **Knowledge Retention Tracking**: See which topics stick and which need reinforcement
- **Adaptive Scheduling**: Review frequency adjusts based on your retention patterns
- **Quiz Generation**: Auto-generated review questions from your notes

### **AI-Powered NLP v2**
Next-generation course structuring:
- **Advanced ML Models**: Replace rule-based system with GLiNER or similar models
- **Context Understanding**: Better comprehension of unconventional naming schemes
- **Content Analysis**: Analyze actual video content, not just titles
- **Auto-Tagging**: Intelligent topic and skill categorization
- **Prerequisite Detection**: Identify learning dependencies automatically

## 🏗 Architecture Deep Dive

### **Modern Component Architecture**
Built with a unified design system that scales:

```rust
// Type-safe, accessible components
Button {
    variant: ButtonVariant::Primary,
    size: ButtonSize::Large,
    loading: form_state.submitting,
    onclick: handle_submit,
    "Create Study Plan"
}

// Flexible card compositions
Card {
    variant: CardVariant::Elevated,

    CardHeader {
        title: "Advanced React Concepts",
        subtitle: "12 videos • 4.5 hours",
        action: progress_menu
    }

    CardContent {
        ProgressBar { completion: 60 }
        p { "Master hooks, context, and advanced patterns" }
    }

    CardActions {
        Button { variant: ButtonVariant::Outline, "View Notes" }
        Button { variant: ButtonVariant::Primary, "Continue Learning" }
    }
}
```

### **Backend Modules**

```
src/
├── lib.rs              # Core library with error handling
├── main.rs             # Desktop application entry
├── types.rs            # Shared data structures
├── state.rs            # Application state management
├── ingest/             # Course import system
│   ├── youtube.rs      # YouTube API integration
│   └── local_folder.rs # Local video scanning
├── nlp/                # Content analysis engine
│   └── processor.rs    # Smart course structuring
├── planner/            # Study scheduling algorithms
│   └── scheduler.rs    # Personalized timeline generation
├── storage/            # Data persistence layer
│   └── database.rs     # SQLite operations
└── ui/                 # Modern component library
    ├── theme_unified.rs # Design system
    ├── layout.rs       # Application shell
    ├── navigation.rs   # Routing system
    ├── hooks/          # Custom hooks for state management
    │   ├── use_courses.rs # Course management operations
    │   ├── use_modals.rs  # Modal state management
    │   └── use_navigation.rs # Navigation utilities
    └── components/     # Reusable UI components
        ├── button/     # Enhanced button component
        ├── card/       # Flexible card system
        ├── input/      # Form input components
        └── ...         # 20+ accessible components
```

### **Hooks System**

Course Pilot uses a comprehensive hooks system for state management and backend integration:

```rust
// Course management with full CRUD operations
let course_manager = use_course_manager();

// Create a new course
course_manager.create_course.call("Advanced React Patterns".to_string());

// Update existing course
course_manager.update_course.call((course_id, "Updated Course Name".to_string()));

// Delete course with confirmation
course_manager.delete_course.call(course_id);

// Navigate to course plan view
course_manager.navigate_to_course.call(course_id);

// Track course progress
let (progress, status, badge_color) = use_course_progress(course_id);
```

**Key Features:**
- **Reactive State**: Automatic UI updates when data changes
- **Error Handling**: Built-in toast notifications and error recovery
- **Optimistic Updates**: Immediate UI feedback with rollback on errors
- **Type Safety**: Full type safety with Rust's type system
- **Performance**: Efficient resource management with `use_resource`

### **Technology Stack**

#### **Core Framework**
- **Dioxus 0.6+**: Modern Rust UI framework with hot-reloading
- **dioxus-router**: Type-safe client-side routing
- **dioxus-desktop**: Cross-platform desktop runtime
- **SQLite**: Embedded database with JSON support

#### **Data Processing**
- **ytextract**: YouTube metadata extraction
- **regex**: Pattern matching for NLP analysis
- **chrono**: Date/time handling for scheduling
- **serde**: Serialization with future-proof formats

#### **UI & UX**
- **rfd**: Native file dialogs
- **dioxus-free-icons**: Material Design icon library
- **CSS Variables**: Theme-aware styling system
- **Responsive Grid**: Mobile-first layout system

#### **Development**
- **anyhow/thiserror**: Comprehensive error handling
- **tokio**: Async runtime for I/O operations
- **tempfile**: Testing utilities
- **tracing**: Structured logging

## 🛠 Development Setup

### **Prerequisites**
```bash
# Install Rust (1.70+ required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Platform-specific dependencies
# Ubuntu/Debian:
sudo apt install libwebkit2gtk-4.0-dev libgtk-3-dev libsqlite3-dev

# macOS:
# Xcode Command Line Tools (automatic)

# Windows:
# WebView2 (usually pre-installed on Windows 11)
```

### **Quick Start**
```bash
# Clone and build
git clone <repository-url>
cd course_pilot
cargo build --release

# Run the application
cargo run

# Run tests
cargo test

# Development with hot-reload
cargo run --features hot-reload
```

### **Development Workflow**
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run specific test suites
cargo test --test integration_test
cargo test storage::tests
cargo test ui::components::tests

# Generate documentation
cargo doc --open
```

## 📊 Performance & Scale

### **Benchmarks**
- **Startup Time**: < 2 seconds cold start
- **Course Import**: 1000+ videos in < 30 seconds
- **UI Responsiveness**: 60fps animations, < 16ms interaction response
- **Memory Usage**: < 50MB for typical course libraries
- **Database Size**: ~1KB per course, ~100 bytes per video

### **Scalability**
- **Courses**: Tested with 1000+ courses
- **Videos per Course**: Handles 500+ video playlists efficiently
- **Concurrent Operations**: Non-blocking import and analysis
- **Cross-Platform**: Windows, macOS, Linux support

## 🎨 Design Philosophy

### **User-Centered Design**
- **Accessibility First**: WCAG 2.1 AA compliance across all components
- **Mobile-Responsive**: Works beautifully on tablets and touch devices
- **Keyboard Navigation**: Complete keyboard accessibility
- **Screen Reader Support**: Semantic HTML and ARIA attributes

### **Performance by Design**
- **Rust's Zero-Cost Abstractions**: Maximum performance, minimal overhead
- **Efficient Rendering**: Virtual DOM with smart diffing
- **Lazy Loading**: Components and data loaded on demand
- **Memory Safety**: No garbage collection pauses or memory leaks

### **Maintainable Architecture**
- **Type Safety**: Compile-time error prevention
- **Modular Design**: Clear separation of concerns
- **Test Coverage**: Unit, integration, and UI tests
- **Documentation**: Comprehensive inline and external docs

## 🤝 Contributing

I welcome contributions! Here's how to get involved:

### **Priority Areas**
1. **Interactive Features**: Help implement progress tracking and note-taking
2. **UI/UX Polish**: Improve animations, transitions, and micro-interactions
3. **Platform Integration**: Better OS integration (notifications, shortcuts)
4. **Import Sources**: Add support for new platforms (Udemy, Coursera, etc.)
5. **Export Formats**: Additional study guide formats and templates

### **Contribution Process**
1. **Check Issues**: Look for "good first issue" or "help wanted" labels
2. **Fork & Branch**: Create a feature branch from main
3. **Develop**: Implement with tests and documentation
4. **Test**: Ensure all tests pass (`cargo test`)
5. **Submit PR**: Include description and link to related issues

### **Code Standards**
- **Rust Idioms**: Follow official Rust style guidelines
- **Error Handling**: Use `thiserror` for custom errors, `anyhow` for applications
- **Testing**: Unit tests for logic, integration tests for workflows
- **Documentation**: Doc comments for public APIs
- **Accessibility**: ARIA attributes and semantic HTML for UI components

## 📈 Roadmap

### **Q3 2025: Interactive Learning**
- [x] Per-video note-taking with rich text editor
- [ ] Interactive progress tracking with checkboxes and visual indicators
- [ ] UI Design with Dioxus-DaisyUI
- [ ] Basic export functionality for notes

### **Q4 2025: Productivity Features**
- [ ] Focus Mode timer with Pomodoro integration
- [ ] Knowledge Hub exporter (Markdown/PDF)
- [ ] Advanced search across notes and courses
- [ ] Keyboard shortcuts and power-user features

### **Q1 2026: Smart Learning**
- [ ] Spaced repetition system for note review
- [ ] Learning analytics and insights dashboard
- [ ] Goal setting and milestone tracking
- [ ] Collaborative features (share courses/notes)

### **Q2 2026: AI Integration**
- [ ] Advanced NLP with machine learning models
- [ ] Automatic quiz generation from notes With AI
- [ ] Intelligent content recommendations
- [ ] Voice notes and transcription

### **2026+: Platform Expansion**
- [ ] Web application version
- [ ] Mobile companion app
- [ ] Cloud synchronization
- [ ] API for third-party integrations

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Dioxus Team**: For creating an amazing Rust UI framework
- **Rust Community**: For the incredible ecosystem and tooling
- **Contributors**: Everyone who helps make Course Pilot better
- **Early Users**: Feedback that shapes the product direction

---

**Ready to transform your learning experience?** Download Course Pilot and turn your video chaos into structured success!

[📥 Download Latest Release](https://github.com/course_pilot/course-pilot/releases) | [📖 Documentation](https://docs.course-pilot.dev)

**Made With Insistence By Khaled**
