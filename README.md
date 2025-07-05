# Course Pilot MVP

A Rust-based desktop application for intelligent course planning and study scheduling from YouTube playlists or local video folders.

## Overview

Course Pilot helps learners organize video-based courses into structured study plans. It automatically analyzes video titles to create logical course hierarchies and generates personalized study schedules based on user preferences.

## Features

### MVP Implementation

- **Course Import**: Import from YouTube playlists or local video folders
- **Intelligent Structuring**: NLP-based analysis to organize videos into modules and sections
- **Study Planning**: Generate personalized study schedules with customizable settings
- **Progress Tracking**: Track learning progress through structured courses
- **Desktop UI**: Native desktop interface built with Dioxus 0.7+

### Current Status

✅ **Completed**:
- Modular backend architecture with comprehensive error handling
- SQLite-based data persistence with JSON serialization
- Rule-based NLP processor for course structuring
- Intelligent scheduling algorithms with multiple distribution strategies
- Desktop UI components with Dioxus framework
- Native system file picker for folder selection
- Integration tests for core functionality

🚧 **In Progress**:
- UI polish and user experience improvements
- Advanced NLP integration (when Rust support improves)

## Architecture

### Backend Modules

- **`ingest/`**: Data import from YouTube and local folders
- **`nlp/`**: Course structure analysis and content organization
- **`planner/`**: Study schedule generation and optimization
- **`storage/`**: SQLite database operations with course/plan persistence
- **`types/`**: Core data structures and shared types

### UI Components

- **`ui/`**: Dioxus-based desktop interface
  - Course Dashboard: Main course overview and management
  - Add Course Dialog: Import workflow for new courses
  - Plan View: Detailed course structure and study plans

## Custom UI Components & Migration Path

We maintain three custom UI components to ensure consistency and prepare for future migration to official dioxus-primitives once they reach production readiness.

- **Button**  
  Props:
  - `id`: Optional identifier for accessibility/testing  
  - `class`: Optional CSS classes in addition to default `"button"`  
  - `variant`: `"primary" | "secondary" | "danger"` (default `"primary"`)  
  - `onpress`: Click event handler  

- **Input**  
  Props:
  - `id`: Optional identifier for accessibility/testing  
  - `class`: Optional CSS container classes plus `"input-container"`  
  - `name`: Name attribute for forms  
  - `required`: Mark field as required  
  - `autocomplete`: Browser autocomplete hint  
  - `value`: Current input value  
  - `onchange`: Change event handler  
  - `onblur`: Blur event handler  
  - `error`: Optional validation error message  

- **Card**  
  Props:
  - `id`: Optional identifier for accessibility/testing  
  - `class`: Optional CSS classes in addition to default `"card"`  
  - `variant`: `"elevated" | "outlined"` (default `"elevated"`)  
  - `header`, `body`, `footer`: Subcomponent slots  

Migration Path:
- We will track the development of [dioxus-primitives](https://github.com/DioxusLabs/dioxus) for stable releases.
- Once primitives are production-ready, these custom components will be replaced with the official primitives.
- TODO: Replace `ui/components/*` implementations with `dioxus-primitives` types and APIs.

## Getting Started

### Prerequisites

- Rust 1.70+ with 2024 edition support
- SQLite development libraries
- System dependencies for Dioxus desktop (e.g., WebKit2GTK on Linux, WebView2 on Windows, WebKit on macOS)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd course_pilot

# Build the project
cargo build --release

# Run the application
cargo run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run only integration tests
cargo test --test integration_test

# Run with verbose output
cargo test -- --nocapture
```

## Usage

### Importing Courses

1. **YouTube Playlists**: Paste playlist URL to automatically extract video metadata
2. **Local Folders**: Use native file picker to browse and select folder containing video files

### Course Structuring

The NLP processor automatically:
- Identifies module boundaries using title patterns
- Groups related content into logical sections
- Estimates difficulty levels and duration
- Creates hierarchical course structure

### Study Planning

Generate personalized study schedules with:
- Customizable sessions per week (1-14)
- Flexible session lengths (15-180 minutes)
- Weekend inclusion options
- Automatic progress tracking

## Dioxus 0.7 Features Leveraged

- Subsecond hot-patching for rapid UI iteration
- WASM code splitting for optimized binary sizes
- Enhanced signal management and reactive hooks
- Improved desktop integration APIs

## Dependencies

### Core Framework
- **Dioxus 0.7+**: Desktop UI framework with hot-patching, WASM splitting, and improved signals  
- **dioxus-router 0.7**: Declarative routing for multi-view applications  
- **dioxus-toast 0.7**: Notification system for user feedback  
- **dioxus-sdk 0.7**: Utilities and helpers for Dioxus apps  
- **dioxus-desktop 0.7**: Desktop runtime for cross-platform support  

### Data Processing
- **ytextract 0.10**: YouTube metadata extraction  
- **rfd 0.14**: Native file dialogs for folder selection  
- **regex 1.0**: Pattern matching for NLP processing  
- **chrono 0.4**: Date/time handling for scheduling  

### Storage
- **rusqlite 0.36**: SQLite database with bundled SQLite  
- **serde/serde_json 1.0**: Struct serialization and JSON handling  
- **uuid 1.0**: Unique identifiers for courses and plans  

### Development
- **anyhow/thiserror 1.0**: Error handling  
- **tempfile 3.8**: Testing utilities  
- **env_logger 0.10**: Development logging  

## Development

### Project Structure

```
src/
├── lib.rs              # Library root and error types
├── main.rs             # Desktop application entry point
├── types.rs            # Core data structures
├── ingest/             # Data import modules
│   ├── mod.rs
│   ├── youtube.rs      # YouTube playlist processing
│   └── local_folder.rs # Local video folder scanning
├── nlp/                # Natural language processing
│   ├── mod.rs
│   └── processor.rs    # Course structure analysis
├── planner/            # Study planning algorithms
│   ├── mod.rs
│   └── scheduler.rs    # Schedule generation
├── storage/            # Data persistence
│   ├── mod.rs
│   └── database.rs     # SQLite operations
└── ui/                 # User interface components
    ├── mod.rs
    ├── course_dashboard.rs
    ├── add_course_dialog.rs
    └── plan_view.rs
```

### Adding New Features

1. **Backend Logic**: Implement in appropriate module with error handling  
2. **Storage**: Add database operations if persistence needed  
3. **UI Components**: Create Dioxus components with state management  
4. **Tests**: Add unit tests for logic and integration tests for workflows  

### Testing Strategy

- **Unit Tests**: Individual function testing within each module  
- **Integration Tests**: End-to-end workflow testing in `tests/`  
- **UI Testing**: Manual testing of desktop interface  

## Contributing

### Code Style

- Follow Rust idioms and best practices  
- Use `cargo fmt` for consistent formatting  
- Run `cargo clippy` for linting  
- Maintain comprehensive error handling with `thiserror`  

### Pull Request Process

1. Create feature branch from main  
2. Implement changes with tests  
3. Ensure all tests pass (`cargo test`)  
4. Update documentation as needed  
5. Submit pull request with description  

## Roadmap

### Near Term
- [ ] Improved error messaging and user feedback
- [ ] Course export/import functionality
- [ ] Basic analytics and progress visualization
- [ ] Drag-and-drop folder support

### Medium Term
- [ ] Advanced NLP with machine learning models
- [ ] Multi-language support for course content
- [ ] Cloud synchronization and backup
- [ ] Plugin system for custom importers

### Long Term
- [ ] Web application version
- [ ] Mobile companion app
- [ ] AI-powered study recommendations
- [ ] Collaborative learning features

## License

[License information to be added]

## Acknowledgments

- Dioxus team for the excellent desktop framework
- YouTube-related crates for metadata extraction
- SQLite team for reliable embedded database
- Rust community for comprehensive ecosystem

---

**Note**: This is an MVP implementation focused on core functionality. The codebase provides a solid foundation for future enhancements and production deployment.  
Keep an eye on the [dioxus-primitives](https://github.com/DioxusLabs/dioxus) repository for upcoming official primitives to replace our custom components.