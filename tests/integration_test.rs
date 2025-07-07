//! Integration tests for Course Pilot backend functionality
//!
//! Tests for duration extraction, aggregation, and serialization.

//! This module tests the core backend components to ensure they work correctly
//! before integrating with the UI.

use chrono::Utc;
use course_pilot::ingest::import_from_local_folder;
use course_pilot::nlp::structure_course;
use course_pilot::planner::generate_plan;
use course_pilot::storage::{init_db, load_courses, save_course};
use course_pilot::types::{Course, PlanSettings};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_course_creation_and_storage() {
    // Create a temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Initialize database
    let conn = init_db(&db_path).expect("Failed to initialize database");

    // Create a mock course
    let titles = vec![
        "1. Introduction to Programming".to_string(),
        "2. Variables and Data Types".to_string(),
        "3. Control Flow".to_string(),
        "4. Functions".to_string(),
        "5. Object-Oriented Programming".to_string(),
    ];
    let course = Course::new("Test Course".to_string(), titles);

    // Save course to database
    save_course(&conn, &course).expect("Failed to save course");

    // Load courses from database
    let loaded_courses = load_courses(&conn).expect("Failed to load courses");

    // Verify course was saved and loaded correctly
    assert_eq!(loaded_courses.len(), 1);
    let loaded_course = &loaded_courses[0];
    assert_eq!(loaded_course.name, "Test Course");
    assert_eq!(loaded_course.raw_titles.len(), 5);
    assert_eq!(loaded_course.id, course.id);
}

#[test]
fn test_nlp_course_structuring() {
    // Create a mock course with structured titles
    let titles = vec![
        "Module 1: Introduction - Getting Started".to_string(),
        "Module 1: Introduction - Setting up Environment".to_string(),
        "Module 2: Basics - Variables and Types".to_string(),
        "Module 2: Basics - Control Structures".to_string(),
        "Module 3: Advanced - Functions and Scope".to_string(),
        "Module 3: Advanced - Object-Oriented Concepts".to_string(),
    ];

    // Structure the course
    let structure = structure_course(titles).expect("Failed to structure course");

    // Verify structure was created
    assert!(structure.modules.len() > 0);
    assert!(structure.modules.len() <= 6); // Should group into reasonable modules

    // Verify metadata was generated
    assert!(structure.metadata.total_videos > 0);
}

use course_pilot::types::{CourseStructure, Module, Section, StructureMetadata};
use std::time::Duration;

#[test]
fn test_module_and_course_duration_aggregation() {
    // Create sections with known durations
    let sections = vec![
        Section {
            title: "A".to_string(),
            video_index: 0,
            duration: Duration::from_secs(60),
        },
        Section {
            title: "B".to_string(),
            video_index: 1,
            duration: Duration::from_secs(120),
        },
        Section {
            title: "C".to_string(),
            video_index: 2,
            duration: Duration::from_secs(180),
        },
    ];
    let module = Module {
        title: "Test Module".to_string(),
        sections: sections.clone(),
        total_duration: sections.iter().map(|s| s.duration).sum(),
    };
    let metadata = StructureMetadata {
        total_videos: 0,
        total_duration: Duration::from_secs(0),
        estimated_duration_hours: None,
        difficulty_level: None,
    };
    let structure = CourseStructure {
        modules: vec![module],
        metadata,
    }
    .with_aggregated_metadata();

    // Check aggregation
    assert_eq!(structure.metadata.total_videos, 3);
    assert_eq!(
        structure.metadata.total_duration,
        Duration::from_secs(60 + 120 + 180)
    );
}

#[test]
fn test_duration_serialization_as_seconds() {
    use serde_json;
    let section = Section {
        title: "Test".to_string(),
        video_index: 0,
        duration: Duration::from_secs(90),
    };
    let json = serde_json::to_string(&section).unwrap();
    assert!(json.contains("\"duration\":90"));
    let deserialized: Section = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.duration, Duration::from_secs(90));
}

#[test]
fn test_plan_generation() {
    // Create a structured course
    let titles = vec![
        "Lesson 1: Introduction".to_string(),
        "Lesson 2: Basics".to_string(),
        "Lesson 3: Intermediate".to_string(),
        "Lesson 4: Advanced".to_string(),
    ];
    let mut course = Course::new("Test Course".to_string(), titles);

    // First structure the course
    let structure =
        structure_course(course.raw_titles.clone()).expect("Failed to structure course");
    course.structure = Some(structure);

    // Create plan settings
    let plan_settings = PlanSettings {
        sessions_per_week: 3,
        session_length_minutes: 60,
        start_date: Utc::now(),
        include_weekends: false,
    };

    // Generate study plan
    let plan = generate_plan(&course, &plan_settings).expect("Failed to generate plan");

    // Verify plan was created
    assert_eq!(plan.course_id, course.id);
    assert_eq!(plan.settings, plan_settings);
    assert!(plan.items.len() > 0);

    // Verify plan items have dates
    for item in &plan.items {
        assert!(item.date >= plan_settings.start_date);
    }
}

#[test]
fn test_local_folder_import_validation() {
    // Test with non-existent folder
    let non_existent_path = PathBuf::from("/non/existent/path");
    let result = import_from_local_folder(&non_existent_path);

    // Should handle invalid paths gracefully (return empty vec or error)
    assert!(result.is_ok() || result.is_err());
    if let Ok(titles) = result {
        assert_eq!(titles.len(), 0);
    }
}

#[test]
fn test_course_video_count() {
    let titles = vec!["Video 1".to_string(), "Video 2".to_string()];
    let course = Course::new("Test Course".to_string(), titles);
    assert_eq!(course.video_count(), 2);

    let empty_course = Course::new("Empty Course".to_string(), vec![]);
    assert_eq!(empty_course.video_count(), 0);
}

#[test]
fn test_course_structured_status() {
    let titles = vec!["Video 1".to_string()];
    let mut course = Course::new("Test Course".to_string(), titles);
    assert!(!course.is_structured());

    // Mock a basic structure
    course.structure = Some(course_pilot::types::CourseStructure {
        modules: vec![],
        metadata: course_pilot::types::StructureMetadata {
            total_videos: 1,
            total_duration: std::time::Duration::from_secs(36000),
            estimated_duration_hours: Some(10.0),
            difficulty_level: Some("Beginner".to_string()),
        },
    });

    assert!(course.is_structured());
}

#[test]
fn test_plan_settings_validation() {
    use course_pilot::planner::validate_plan_settings;

    // Valid settings
    assert!(validate_plan_settings(3, 60, Utc::now()).is_ok());

    // Invalid settings - too many sessions (over 14)
    assert!(validate_plan_settings(15, 60, Utc::now()).is_err());

    // Invalid settings - too short session (under 15 minutes)
    assert!(validate_plan_settings(3, 10, Utc::now()).is_err());
}

#[test]
fn test_course_creation() {
    let titles = vec!["Introduction".to_string(), "Advanced Topics".to_string()];
    let course = Course::new("Test Course".to_string(), titles.clone());

    assert_eq!(course.name, "Test Course");
    assert_eq!(course.raw_titles, titles);
    assert!(course.structure.is_none());
    assert!(!course.is_structured());
}
