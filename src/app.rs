//! Main Application Component
//
//! This is the root application module for backend logic only.

/// Initialize the application with any required setup
pub fn initialize_app() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    #[cfg(debug_assertions)]
    {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .format_timestamp_secs()
            .init();
        log::info!("Course Pilot application starting in debug mode with enhanced logging");
    }

    #[cfg(not(debug_assertions))]
    {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .format_timestamp_secs()
            .init();
        log::info!("Course Pilot application starting in release mode");
    }

    // Log system information
    log::info!("Application initialized successfully");
    log::debug!("Debug logging enabled");

    Ok(())
}

/// data for development and testing
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
            videos: vec![
                crate::types::VideoMetadata::new_local("Introduction to Rust".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Setting up the Development Environment".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Variables and Data Types".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Control Flow and Functions".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Ownership and Borrowing".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Structs and Enums".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Error Handling".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Collections and Iterators".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Building a CLI Application".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Testing and Documentation".to_string(), "".to_string()),
            ],
            structure: Some(CourseStructure {
                modules: vec![
                    Module {
                        title: "Getting Started".to_string(),
                        sections: vec![
                            Section {
                                title: "Introduction to Rust".to_string(),
                                video_index: 0,
                                duration: Duration::from_secs(15 * 60),
                            },
                            Section {
                                title: "Setting up the Development Environment".to_string(),
                                video_index: 1,
                                duration: Duration::from_secs(20 * 60),
                            },
                        ],
                        total_duration: Duration::from_secs(15 * 60 + 20 * 60),
                        similarity_score: None,
                        topic_keywords: Vec::new(),
                        difficulty_level: None,
                    },
                    Module {
                        title: "Core Concepts".to_string(),
                        sections: vec![
                            Section {
                                title: "Variables and Data Types".to_string(),
                                video_index: 2,
                                duration: Duration::from_secs(25 * 60),
                            },
                            Section {
                                title: "Control Flow and Functions".to_string(),
                                video_index: 3,
                                duration: Duration::from_secs(30 * 60),
                            },
                            Section {
                                title: "Ownership and Borrowing".to_string(),
                                video_index: 4,
                                duration: Duration::from_secs(35 * 60),
                            },
                        ],
                        total_duration: Duration::from_secs(25 * 60 + 30 * 60 + 35 * 60),
                        similarity_score: None,
                        topic_keywords: Vec::new(),
                        difficulty_level: None,
                    },
                ],
                metadata: StructureMetadata {
                    total_videos: 10,
                    total_duration: Duration::from_secs(
                        15 * 60 + 20 * 60 + 25 * 60 + 30 * 60 + 35 * 60,
                    ),
                    estimated_duration_hours: Some(4.5),
                    difficulty_level: Some("Intermediate".to_string()),
                    structure_quality_score: None,
                    content_coherence_score: None,
                },
                clustering_metadata: None,
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
            videos: vec![
                crate::types::VideoMetadata::new_local("Introduction to Dioxus".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Setting up a Dioxus Project".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Components and Props".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("State Management".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Event Handling".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Routing and Navigation".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Styling with CSS".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Building for Desktop".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Building for Web".to_string(), "".to_string()),
                crate::types::VideoMetadata::new_local("Deployment Strategies".to_string(), "".to_string()),
            ],
            structure: None,
        },
    ]
}
