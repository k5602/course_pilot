//! Unit tests for backend adapter functionality

use chrono::Utc;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

use course_pilot::storage::database::Database;
use course_pilot::types::{Course, Plan, PlanItem, PlanSettings};
use course_pilot::ui::backend_adapter::Backend;

/// Setup test backend with temporary database
async fn setup_test_backend() -> (Backend, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Arc::new(Database::new(&db_path).unwrap());
    let backend = Backend::new(db);
    (backend, temp_dir)
}

#[tokio::test]
async fn test_create_and_load_course() {
    let (backend, _temp_dir) = setup_test_backend().await;

    let course = Course::new("Test Course".to_string(), vec!["Video 1".to_string()]);
    let course_id = course.id;

    // Test create
    backend.create_course(course).await.unwrap();

    // Test load
    let loaded_course = backend.get_course(course_id).await.unwrap();
    assert!(loaded_course.is_some());
    assert_eq!(loaded_course.unwrap().name, "Test Course");
}

#[tokio::test]
async fn test_update_course() {
    let (backend, _temp_dir) = setup_test_backend().await;

    let mut course = Course::new("Original Name".to_string(), vec!["Video 1".to_string()]);
    backend.create_course(course.clone()).await.unwrap();

    // Update course
    course.name = "Updated Name".to_string();
    backend.update_course(course.clone()).await.unwrap();

    // Verify update
    let loaded_course = backend.get_course(course.id).await.unwrap().unwrap();
    assert_eq!(loaded_course.name, "Updated Name");
}

#[tokio::test]
async fn test_delete_course() {
    let (backend, _temp_dir) = setup_test_backend().await;

    let course = Course::new("Test Course".to_string(), vec!["Video 1".to_string()]);
    let course_id = course.id;

    // Create and verify
    backend.create_course(course).await.unwrap();
    assert!(backend.get_course(course_id).await.unwrap().is_some());

    // Delete and verify
    backend.delete_course(course_id).await.unwrap();
    assert!(backend.get_course(course_id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_plan_item_completion_toggle() {
    let (backend, _temp_dir) = setup_test_backend().await;

    // Setup test data
    let course = Course::new("Test Course".to_string(), vec!["Video 1".to_string()]);
    backend.create_course(course.clone()).await.unwrap();

    let mut plan = Plan::new(
        course.id,
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        },
    );

    // Add a test plan item
    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Module 1".to_string(),
        section_title: "Section 1".to_string(),
        video_indices: vec![0],
        completed: false,
        total_duration: Duration::from_secs(600),
        estimated_completion_time: Duration::from_secs(720),
        overflow_warnings: Vec::new(),
    });

    backend.save_plan(plan.clone()).await.unwrap();

    // Test completion toggle
    backend
        .update_plan_item_completion(plan.id, 0, true)
        .await
        .unwrap();

    let updated_plan = backend
        .get_plan_by_course(course.id)
        .await
        .unwrap()
        .unwrap();
    assert!(updated_plan.items[0].completed);

    // Test toggle back
    backend
        .update_plan_item_completion(plan.id, 0, false)
        .await
        .unwrap();

    let updated_plan = backend
        .get_plan_by_course(course.id)
        .await
        .unwrap()
        .unwrap();
    assert!(!updated_plan.items[0].completed);
}

#[tokio::test]
async fn test_plan_progress_calculation() {
    let (backend, _temp_dir) = setup_test_backend().await;

    // Setup test data
    let course = Course::new(
        "Test Course".to_string(),
        vec!["Video 1".to_string(), "Video 2".to_string()],
    );
    backend.create_course(course.clone()).await.unwrap();

    let mut plan = Plan::new(
        course.id,
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        },
    );

    // Add test plan items
    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Module 1".to_string(),
        section_title: "Section 1".to_string(),
        video_indices: vec![0],
        completed: true, // Completed
        total_duration: Duration::from_secs(600),
        estimated_completion_time: Duration::from_secs(720),
        overflow_warnings: Vec::new(),
    });

    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Module 1".to_string(),
        section_title: "Section 2".to_string(),
        video_indices: vec![1],
        completed: false, // Not completed
        total_duration: Duration::from_secs(900),
        estimated_completion_time: Duration::from_secs(1080),
        overflow_warnings: Vec::new(),
    });

    backend.save_plan(plan.clone()).await.unwrap();

    // Test progress calculation
    let progress = backend.get_plan_progress(plan.id).await.unwrap();
    assert_eq!(progress.completed_count, 1);
    assert_eq!(progress.total_count, 2);
    assert_eq!(progress.percentage, 50.0);

    // Test course progress
    let course_progress = backend
        .get_course_progress(course.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(course_progress.percentage, 50.0);
}

#[tokio::test]
async fn test_plan_item_out_of_bounds() {
    let (backend, _temp_dir) = setup_test_backend().await;

    // Setup test data
    let course = Course::new("Test Course".to_string(), vec!["Video 1".to_string()]);
    backend.create_course(course.clone()).await.unwrap();

    let plan = Plan::new(
        course.id,
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        },
    );

    backend.save_plan(plan.clone()).await.unwrap();

    // Test out of bounds access
    let result = backend
        .update_plan_item_completion(plan.id, 999, true)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_nonexistent_plan() {
    let (backend, _temp_dir) = setup_test_backend().await;

    let fake_plan_id = Uuid::new_v4();

    // Test accessing nonexistent plan
    let result = backend.get_plan_progress(fake_plan_id).await;
    assert!(result.is_err());

    let result = backend
        .update_plan_item_completion(fake_plan_id, 0, true)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_courses() {
    let (backend, _temp_dir) = setup_test_backend().await;

    // Initially empty
    let courses = backend.list_courses().await.unwrap();
    assert!(courses.is_empty());

    // Add some courses
    let course1 = Course::new("Course 1".to_string(), vec!["Video 1".to_string()]);
    let course2 = Course::new("Course 2".to_string(), vec!["Video 2".to_string()]);

    backend.create_course(course1.clone()).await.unwrap();
    backend.create_course(course2.clone()).await.unwrap();

    // Verify list
    let courses = backend.list_courses().await.unwrap();
    assert_eq!(courses.len(), 2);

    let course_names: Vec<String> = courses.iter().map(|c| c.name.clone()).collect();
    assert!(course_names.contains(&"Course 1".to_string()));
    assert!(course_names.contains(&"Course 2".to_string()));
}
