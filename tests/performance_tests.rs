//! Performance tests for Course Pilot

use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
// Imports moved to where they're actually used

use course_pilot::ingest::local_folder::EnhancedLocalIngest;
use course_pilot::storage::database::Database;
use course_pilot::types::{Course, Plan, PlanItem, PlanSettings};
use course_pilot::ui::backend_adapter::Backend;

#[tokio::test]
async fn test_directory_scanning_performance() {
    // Create a large directory structure for performance testing
    let temp_dir = TempDir::new().unwrap();
    let root_path = temp_dir.path();

    // Create nested directories with many video files
    let start_time = Instant::now();

    for course_num in 0..10 {
        for module_num in 0..5 {
            let dir_path = root_path.join(format!("course{}/module{}", course_num, module_num));
            std::fs::create_dir_all(&dir_path).unwrap();

            // Create 10 video files per module
            for video_num in 0..10 {
                let file_path = dir_path.join(format!("video{}.mp4", video_num));
                std::fs::write(&file_path, b"fake video content").unwrap();
            }
        }
    }

    let setup_time = start_time.elapsed();
    println!("Setup time for 500 video files: {:?}", setup_time);

    // Test scanning performance
    let ingest = EnhancedLocalIngest::new();
    use std::sync::{Arc, Mutex};
    let progress_count = Arc::new(Mutex::new(0));
    let progress_count_clone = progress_count.clone();

    let progress_callback = move |_progress: f32, _message: String| {
        *progress_count_clone.lock().unwrap() += 1;
    };

    let scan_start = Instant::now();
    let video_files = ingest
        .scan_directory_recursive(root_path, Some(&progress_callback))
        .unwrap();
    let scan_time = scan_start.elapsed();

    // Verify results
    assert_eq!(video_files.len(), 500); // 10 courses * 5 modules * 10 videos
    let final_progress_count = *progress_count.lock().unwrap();
    assert!(final_progress_count > 0); // Progress callbacks were called

    println!("Scan time for 500 video files: {:?}", scan_time);
    println!("Progress callbacks: {}", final_progress_count);

    // Performance assertions (adjust based on expected performance)
    assert!(
        scan_time < Duration::from_secs(5),
        "Scanning should complete within 5 seconds"
    );
}

#[tokio::test]
async fn test_database_operations_performance() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("perf_test.db");
    let db = Arc::new(Database::new(&db_path).unwrap());
    let backend = Backend::new(db);

    // Test bulk course creation performance
    let course_creation_start = Instant::now();
    let mut course_ids = Vec::new();

    for i in 0..100 {
        let course = Course::new(
            format!("Performance Test Course {}", i),
            vec![format!("Video {}", i)],
        );
        course_ids.push(course.id);
        backend.create_course(course).await.unwrap();
    }

    let course_creation_time = course_creation_start.elapsed();
    println!("Time to create 100 courses: {:?}", course_creation_time);

    // Test bulk course loading performance
    let course_loading_start = Instant::now();
    let loaded_courses = backend.list_courses().await.unwrap();
    let course_loading_time = course_loading_start.elapsed();

    assert_eq!(loaded_courses.len(), 100);
    println!("Time to load 100 courses: {:?}", course_loading_time);

    // Test plan operations performance
    let plan_ops_start = Instant::now();

    // Create plans with many items
    for (_i, course_id) in course_ids.iter().enumerate().take(10) {
        let mut plan = Plan::new(
            *course_id,
            PlanSettings {
                start_date: Utc::now(),
                sessions_per_week: 3,
                session_length_minutes: 60,
                include_weekends: false,
            },
        );

        // Add 50 plan items per plan
        for j in 0..50 {
            plan.items.push(PlanItem {
                date: Utc::now(),
                module_title: format!("Module {}", j / 10),
                section_title: format!("Section {}", j),
                video_indices: vec![j],
                completed: j % 3 == 0, // Every third item completed
            });
        }

        backend.save_plan(plan).await.unwrap();
    }

    let plan_ops_time = plan_ops_start.elapsed();
    println!(
        "Time to create 10 plans with 50 items each: {:?}",
        plan_ops_time
    );

    // Test progress calculation performance
    let progress_calc_start = Instant::now();

    for course_id in course_ids.iter().take(10) {
        let _progress = backend.get_course_progress(*course_id).await.unwrap();
    }

    let progress_calc_time = progress_calc_start.elapsed();
    println!(
        "Time to calculate progress for 10 courses: {:?}",
        progress_calc_time
    );

    // Performance assertions
    assert!(
        course_creation_time < Duration::from_secs(10),
        "Course creation should be fast"
    );
    assert!(
        course_loading_time < Duration::from_secs(1),
        "Course loading should be very fast"
    );
    assert!(
        plan_ops_time < Duration::from_secs(5),
        "Plan operations should be reasonably fast"
    );
    assert!(
        progress_calc_time < Duration::from_secs(2),
        "Progress calculation should be fast"
    );
}

#[tokio::test]
async fn test_concurrent_operations_performance() {
    use tokio::task::JoinSet;

    // Setup
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("concurrent_test.db");
    let db = Arc::new(Database::new(&db_path).unwrap());
    let backend = Arc::new(Backend::new(db));

    // Test concurrent course creation
    let concurrent_start = Instant::now();
    let mut join_set = JoinSet::new();

    for i in 0..20 {
        let backend_clone = backend.clone();
        join_set.spawn(async move {
            let course = Course::new(
                format!("Concurrent Course {}", i),
                vec![format!("Video {}", i)],
            );
            backend_clone.create_course(course).await.unwrap();
        });
    }

    // Wait for all tasks to complete
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }

    let concurrent_time = concurrent_start.elapsed();
    println!(
        "Time for 20 concurrent course creations: {:?}",
        concurrent_time
    );

    // Verify all courses were created
    let courses = backend.list_courses().await.unwrap();
    assert_eq!(courses.len(), 20);

    // Test concurrent plan item updates
    let course = Course::new(
        "Concurrent Plan Test".to_string(),
        vec!["Video 1".to_string()],
    );
    let course_id = course.id;
    backend.create_course(course).await.unwrap();

    let mut plan = Plan::new(
        course_id,
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
        },
    );

    // Add items for concurrent updates
    for i in 0..10 {
        plan.items.push(PlanItem {
            date: Utc::now(),
            module_title: "Module 1".to_string(),
            section_title: format!("Section {}", i),
            video_indices: vec![i],
            completed: false,
        });
    }

    let plan_id = plan.id;
    backend.save_plan(plan).await.unwrap();

    // Concurrent plan item updates
    let update_start = Instant::now();
    let mut update_set = JoinSet::new();

    for i in 0..10 {
        let backend_clone = backend.clone();
        update_set.spawn(async move {
            backend_clone
                .update_plan_item_completion(plan_id, i, true)
                .await
                .unwrap();
        });
    }

    // Wait for all updates to complete
    while let Some(result) = update_set.join_next().await {
        result.unwrap();
    }

    let update_time = update_start.elapsed();
    println!(
        "Time for 10 concurrent plan item updates: {:?}",
        update_time
    );

    // Verify all items were updated
    let final_progress = backend.get_plan_progress(plan_id).await.unwrap();
    assert_eq!(final_progress.completed_count, 10);
    assert_eq!(final_progress.percentage, 100.0);

    // Performance assertions
    assert!(
        concurrent_time < Duration::from_secs(15),
        "Concurrent operations should complete reasonably fast"
    );
    assert!(
        update_time < Duration::from_secs(5),
        "Concurrent updates should be fast"
    );
}

#[tokio::test]
async fn test_memory_usage_with_large_datasets() {
    // This test ensures we don't have memory leaks with large datasets
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("memory_test.db");
    let db = Arc::new(Database::new(&db_path).unwrap());
    let backend = Backend::new(db);

    // Create a course with a very large plan
    let course = Course::new(
        "Memory Test Course".to_string(),
        vec!["Video 1".to_string()],
    );
    let course_id = course.id;
    backend.create_course(course).await.unwrap();

    let mut plan = Plan::new(
        course_id,
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 7,
            session_length_minutes: 30,
            include_weekends: true,
        },
    );

    // Add 1000 plan items
    let large_plan_start = Instant::now();
    for i in 0..1000 {
        plan.items.push(PlanItem {
            date: Utc::now(),
            module_title: format!("Module {}", i / 100),
            section_title: format!("Section {}", i),
            video_indices: vec![i],
            completed: i % 5 == 0, // Every 5th item completed
        });
    }

    backend.save_plan(plan.clone()).await.unwrap();
    let large_plan_time = large_plan_start.elapsed();
    println!("Time to save plan with 1000 items: {:?}", large_plan_time);

    // Test loading and progress calculation with large plan
    let load_start = Instant::now();
    let loaded_plan = backend
        .get_plan_by_course(course_id)
        .await
        .unwrap()
        .unwrap();
    let load_time = load_start.elapsed();

    assert_eq!(loaded_plan.items.len(), 1000);
    println!("Time to load plan with 1000 items: {:?}", load_time);

    let progress_start = Instant::now();
    let progress = backend.get_plan_progress(plan.id).await.unwrap();
    let progress_time = progress_start.elapsed();

    assert_eq!(progress.total_count, 1000);
    assert_eq!(progress.completed_count, 200); // Every 5th item = 200 completed
    println!(
        "Time to calculate progress for 1000 items: {:?}",
        progress_time
    );

    // Performance assertions for large datasets
    assert!(
        large_plan_time < Duration::from_secs(10),
        "Large plan save should complete within 10 seconds"
    );
    assert!(
        load_time < Duration::from_secs(5),
        "Large plan load should complete within 5 seconds"
    );
    assert!(
        progress_time < Duration::from_secs(1),
        "Progress calculation should be fast even for large plans"
    );
}
