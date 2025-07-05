//! Main Application Component
//
//! This is the root Dioxus component that manages application state, routing,
//! and provides the overall application structure.

use crate::types::AppState;
use crate::ui::AppTheme;
use crate::ui::Layout;
use dioxus::prelude::*;
use dioxus_toast::{ToastFrame, ToastManager};

/// Root application component
#[component]
pub fn App() -> Element {
    let app_state = use_signal(|| {
        let mut state = AppState::default();
        #[cfg(debug_assertions)]
        {
            state.courses = load_demo_data();
        }
        state
    });

    // Toast manager signal
    let toast = use_signal(|| ToastManager::default());
    use_context_provider(|| app_state);
    use_context_provider(|| toast);

    rsx! {
        AppTheme {},
        ToastFrame { manager: toast.clone() }
        Layout {}
    }
}

/// Initialize the application with any required setup
pub fn initialize_app() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    {
        env_logger::init();
        log::info!("Course Pilot application starting in debug mode");
    }

    Ok(())
}

/// Demo data for development and testing
#[cfg(debug_assertions)]
pub fn load_demo_data() -> Vec<crate::types::Course> {
    use crate::types::{Course, CourseStructure, Module, Section, StructureMetadata};
    use chrono::Utc;
    use std::time::Duration;
    use uuid::Uuid;

    vec![
        Course {
            id: Uuid::new_v4(),
            name: "Rust Programming Fundamentals".to_string(),
            created_at: Utc::now() - chrono::Duration::days(7),
            raw_titles: vec![
                "Introduction to Rust".to_string(),
                "Setting up the Development Environment".to_string(),
                "Variables and Data Types".to_string(),
                "Control Flow and Functions".to_string(),
                "Ownership and Borrowing".to_string(),
                "Structs and Enums".to_string(),
                "Error Handling".to_string(),
                "Collections and Iterators".to_string(),
                "Building a CLI Application".to_string(),
                "Testing and Documentation".to_string(),
            ],
            structure: Some(CourseStructure {
                modules: vec![
                    Module {
                        title: "Getting Started".to_string(),
                        sections: vec![
                            Section {
                                title: "Introduction to Rust".to_string(),
                                video_index: 0,
                                estimated_duration: Some(Duration::from_secs(15 * 60)),
                            },
                            Section {
                                title: "Setting up the Development Environment".to_string(),
                                video_index: 1,
                                estimated_duration: Some(Duration::from_secs(20 * 60)),
                            },
                        ],
                    },
                    Module {
                        title: "Core Concepts".to_string(),
                        sections: vec![
                            Section {
                                title: "Variables and Data Types".to_string(),
                                video_index: 2,
                                estimated_duration: Some(Duration::from_secs(25 * 60)),
                            },
                            Section {
                                title: "Control Flow and Functions".to_string(),
                                video_index: 3,
                                estimated_duration: Some(Duration::from_secs(30 * 60)),
                            },
                            Section {
                                title: "Ownership and Borrowing".to_string(),
                                video_index: 4,
                                estimated_duration: Some(Duration::from_secs(35 * 60)),
                            },
                        ],
                    },
                ],
                metadata: StructureMetadata {
                    total_videos: 10,
                    estimated_duration_hours: Some(4.5),
                    difficulty_level: Some("Intermediate".to_string()),
                },
            }),
        },
        Course {
            id: Uuid::new_v4(),
            name: "Web Development with Dioxus".to_string(),
            created_at: Utc::now() - chrono::Duration::days(3),
            raw_titles: vec![
                "Introduction to Dioxus".to_string(),
                "Setting up a Dioxus Project".to_string(),
                "Components and Props".to_string(),
                "State Management".to_string(),
                "Event Handling".to_string(),
                "Routing and Navigation".to_string(),
                "Styling with CSS".to_string(),
                "Building for Desktop".to_string(),
                "Building for Web".to_string(),
                "Deployment Strategies".to_string(),
            ],
            structure: None,
        },
    ]
}
