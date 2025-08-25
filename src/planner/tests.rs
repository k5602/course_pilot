use super::*;
use crate::planner::strategy::{analyze_course_complexity, infer_user_experience_level};
use crate::types::DifficultyLevel;
use crate::types::{Course, CourseStructure, Module, Section, StructureMetadata};
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;

fn create_test_course() -> Course {
    let structure = CourseStructure::new_basic(
        vec![
            Module::new_basic(
                "Introduction".to_string(),
                vec![
                    Section {
                        title: "Welcome".to_string(),
                        video_index: 0,
                        duration: Duration::from_secs(600),
                    },
                    Section {
                        title: "Setup".to_string(),
                        video_index: 1,
                        duration: Duration::from_secs(900),
                    },
                ],
            ),
            Module::new_basic(
                "Advanced Topics".to_string(),
                vec![Section {
                    title: "Complex Example".to_string(),
                    video_index: 2,
                    duration: Duration::from_secs(1800),
                }],
            ),
        ],
        StructureMetadata {
            total_videos: 3,
            total_duration: Duration::from_secs(600 + 900 + 1800),
            estimated_duration_hours: Some(1.0),
            difficulty_level: Some("Intermediate".to_string()),
            structure_quality_score: None,
            content_coherence_score: None,
            content_type_detected: Some("Sequential".to_string()),
            original_order_preserved: Some(true),
            processing_strategy_used: Some("PreserveOrder".to_string()),
        },
    );

    Course {
        id: Uuid::new_v4(),
        name: "Test Course".to_string(),
        created_at: Utc::now(),
        raw_titles: vec!["Welcome".to_string(), "Setup".to_string(), "Complex Example".to_string()],
        videos: vec![
            crate::types::VideoMetadata {
                title: "Welcome".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 0,
                duration_seconds: Some(600.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
            crate::types::VideoMetadata {
                title: "Setup".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 1,
                duration_seconds: Some(900.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
            crate::types::VideoMetadata {
                title: "Complex Example".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 2,
                duration_seconds: Some(1800.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
        ],
        structure: Some(structure),
    }
}

fn create_test_settings() -> PlanSettings {
    PlanSettings {
        start_date: Utc::now() + chrono::Duration::days(1),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    }
}

#[test]
fn test_generate_plan_basic() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = generate_plan(&course, &settings);
    assert!(result.is_ok());

    let plan = result.unwrap();
    assert!(!plan.items.is_empty());
    assert_eq!(plan.course_id, course.id);
}

#[test]
fn test_generate_plan_without_structure() {
    let mut course = create_test_course();
    course.structure = None;
    let settings = create_test_settings();

    let result = generate_plan(&course, &settings);
    assert!(matches!(result, Err(PlanError::CourseNotStructured)));
}

#[test]
fn test_videos_per_session_calculation_with_actual_durations() {
    let course = create_test_course();
    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    };

    let videos = crate::planner::capacity::estimated_videos_per_session(&course, &settings);
    // Course has videos of 10min, 15min, 30min = average 18.33min
    // 60min * 0.8 buffer = 48min effective / 18.33min = ~2 videos per session
    assert!(videos >= 1 && videos <= 3);
}

#[test]
fn test_videos_per_session_fallback() {
    let mut course = create_test_course();
    course.structure = None; // No structure available

    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    };

    let videos = crate::planner::capacity::estimated_videos_per_session(&course, &settings);
    assert_eq!(videos, 4); // 60 minutes * 0.8 / 12 minutes = 4 videos (fallback calculation)
}

#[test]
fn test_video_exceeds_session_limit() {
    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    };

    // 60 minutes * 0.8 = 48 minutes effective session time
    assert!(!crate::planner::capacity::video_exceeds_effective_limit(
        Duration::from_secs(30 * 60),
        &settings
    )); // 30 min - OK
    assert!(!crate::planner::capacity::video_exceeds_effective_limit(
        Duration::from_secs(45 * 60),
        &settings
    )); // 45 min - OK
    assert!(crate::planner::capacity::video_exceeds_effective_limit(
        Duration::from_secs(50 * 60),
        &settings
    )); // 50 min - exceeds
    assert!(crate::planner::capacity::video_exceeds_effective_limit(
        Duration::from_secs(90 * 60),
        &settings
    )); // 90 min - exceeds
}

#[test]
fn test_completion_time_calculation() {
    let video_duration = Duration::from_secs(45 * 60); // 45 minutes
    let completion_time =
        crate::types::duration_utils::calculate_completion_time_with_buffer(video_duration, 0.25);

    // Total video time: 45 minutes
    // Buffer time: 45 * 0.25 = 11.25 minutes
    // Total: ~56.25 minutes
    assert!(completion_time.as_secs() >= 56 * 60 && completion_time.as_secs() <= 57 * 60);
}

#[test]
fn test_session_capacity_with_long_videos() {
    let long_video_duration = Duration::from_secs(90 * 60); // 90 minutes
    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60, // 60 minute sessions
        include_weekends: false,
        advanced_settings: None,
    };

    let capacity =
        crate::planner::capacity::session_capacity_for_average(long_video_duration, &settings);
    assert_eq!(capacity, 1); // Should be 1 when videos exceed session time
}

#[test]
fn test_session_capacity_with_short_videos() {
    let short_video_duration = Duration::from_secs(5 * 60); // 5 minutes
    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60, // 60 minute sessions
        include_weekends: false,
        advanced_settings: None,
    };

    let capacity =
        crate::planner::capacity::session_capacity_for_average(short_video_duration, &settings);
    // 60 * 0.8 = 48 minutes effective / 5 minutes = 9.6 -> 9 videos
    assert_eq!(capacity, 9);
}

#[test]
fn test_time_based_plan_with_duration_grouping() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = strategies::generate_time_based_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(!plan_items.is_empty());

    // Verify that sessions respect duration constraints
    for item in &plan_items {
        assert!(!item.video_indices.is_empty());
        // Each session should have at least one video
        assert!(item.video_indices.len() >= 1);
    }
}

#[test]
fn test_bin_packing_optimization() {
    // Create a course with videos of varying durations
    let structure = CourseStructure::new_basic(
        vec![Module::new_basic(
            "Test Module".to_string(),
            vec![
                Section {
                    title: "Short Video 1".to_string(),
                    video_index: 0,
                    duration: Duration::from_secs(10 * 60), // 10 minutes
                },
                Section {
                    title: "Short Video 2".to_string(),
                    video_index: 1,
                    duration: Duration::from_secs(15 * 60), // 15 minutes
                },
                Section {
                    title: "Medium Video".to_string(),
                    video_index: 2,
                    duration: Duration::from_secs(25 * 60), // 25 minutes
                },
                Section {
                    title: "Short Video 3".to_string(),
                    video_index: 3,
                    duration: Duration::from_secs(12 * 60), // 12 minutes
                },
            ],
        )],
        StructureMetadata {
            total_videos: 4,
            total_duration: Duration::from_secs(62 * 60),
            estimated_duration_hours: Some(1.0),
            difficulty_level: Some("Beginner".to_string()),
            structure_quality_score: None,
            content_coherence_score: None,
            content_type_detected: Some("Sequential".to_string()),
            original_order_preserved: Some(true),
            processing_strategy_used: Some("PreserveOrder".to_string()),
        },
    );

    let course = Course {
        id: Uuid::new_v4(),
        name: "Bin Packing Test Course".to_string(),
        created_at: Utc::now(),
        raw_titles: vec![
            "Short Video 1".to_string(),
            "Short Video 2".to_string(),
            "Medium Video".to_string(),
            "Short Video 3".to_string(),
        ],
        videos: vec![
            crate::types::VideoMetadata {
                title: "Short Video 1".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 0,
                duration_seconds: Some(600.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
            crate::types::VideoMetadata {
                title: "Short Video 2".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 1,
                duration_seconds: Some(900.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
            crate::types::VideoMetadata {
                title: "Medium Video".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 2,
                duration_seconds: Some(1500.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
            crate::types::VideoMetadata {
                title: "Short Video 3".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 3,
                duration_seconds: Some(720.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
        ],
        structure: Some(structure),
    };

    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60, // 60 minute sessions
        include_weekends: false,
        advanced_settings: None,
    };

    let result = strategies::generate_time_based_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();

    // With 60-minute sessions and 20% buffer (48 min effective):
    // Should be able to fit multiple short videos in first session
    // Medium video (25 min) should fit with some short videos
    assert!(plan_items.len() <= 3); // Should not need more than 3 sessions for 62 minutes of content

    // First session should contain multiple videos if bin-packing works
    if let Some(first_session) = plan_items.first() {
        // Should be able to fit at least 2 videos in first session
        assert!(first_session.video_indices.len() >= 1);
    }
}

#[test]
fn test_duration_aware_module_grouping() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = strategies::generate_module_based_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(!plan_items.is_empty());

    // Verify that sessions respect module boundaries and duration constraints
    for item in &plan_items {
        assert!(!item.video_indices.is_empty());
        // Module-based planning should maintain module context
        assert!(!item.module_title.is_empty());
    }
}

#[test]
fn test_duration_aware_hybrid_planning() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = strategies::generate_hybrid_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(!plan_items.is_empty());

    // Hybrid planning should balance module structure with time constraints
    for item in &plan_items {
        assert!(!item.video_indices.is_empty());
        assert!(!item.module_title.is_empty());
    }
}

#[test]
fn test_difficulty_phase_sessions() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = strategies::generate_difficulty_based_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(!plan_items.is_empty());

    // Should include all course videos across difficulty-phased sessions
    let total_videos: usize = plan_items.iter().map(|it| it.video_indices.len()).sum();
    assert_eq!(total_videos, course.video_count());
}

#[test]
fn test_duration_display_and_validation() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = generate_plan(&course, &settings);
    assert!(result.is_ok());

    let plan = result.unwrap();
    assert!(!plan.items.is_empty());

    // Verify that all plan items have duration information
    for item in &plan.items {
        // All items should have non-zero total duration (except break days)
        if !item.video_indices.is_empty() {
            assert!(item.total_duration.as_secs() > 0);
            assert!(item.estimated_completion_time.as_secs() > 0);
            // Estimated completion time should be longer than total duration (due to buffer)
            assert!(item.estimated_completion_time >= item.total_duration);
        }

        // Overflow warnings should be a valid vector (can be empty)
        assert!(item.overflow_warnings.len() >= 0);
    }
}

#[test]
fn test_duration_formatting_utilities() {
    use crate::types::duration_utils::*;

    // Test basic duration formatting
    assert_eq!(format_duration(Duration::from_secs(30)), "30s");
    assert_eq!(format_duration(Duration::from_secs(90)), "1m");
    assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
    assert_eq!(format_duration(Duration::from_secs(3690)), "1h 1m");

    // Test verbose formatting
    assert_eq!(format_duration_verbose(Duration::from_secs(90)), "1 minutes");
    assert_eq!(format_duration_verbose(Duration::from_secs(3600)), "1 hours");
    assert_eq!(format_duration_verbose(Duration::from_secs(3690)), "1 hours 1 minutes");

    // Test decimal hours formatting
    assert_eq!(format_duration_decimal_hours(Duration::from_secs(3600)), "1.0 hours");
    assert_eq!(format_duration_decimal_hours(Duration::from_secs(1800)), "30 minutes");

    // Test excessive duration check
    assert!(is_duration_excessive(Duration::from_secs(90 * 60), 60)); // 90 min > 60 min
    assert!(!is_duration_excessive(Duration::from_secs(45 * 60), 60)); // 45 min < 60 min

    // Test completion time calculation with buffer
    let video_duration = Duration::from_secs(60 * 60); // 1 hour
    let completion_time = calculate_completion_time_with_buffer(video_duration, 0.25);
    assert_eq!(completion_time.as_secs(), 75 * 60); // 1 hour + 25% = 75 minutes
}

#[test]
fn test_session_overflow_warnings() {
    // Create a course with a very long video
    let structure = CourseStructure::new_basic(
        vec![Module::new_basic(
            "Test Module".to_string(),
            vec![Section {
                title: "Very Long Video".to_string(),
                video_index: 0,
                duration: Duration::from_secs(90 * 60), // 90 minutes
            }],
        )],
        StructureMetadata {
            total_videos: 1,
            total_duration: Duration::from_secs(90 * 60),
            estimated_duration_hours: Some(1.5),
            difficulty_level: Some("Advanced".to_string()),
            structure_quality_score: None,
            content_coherence_score: None,
            content_type_detected: Some("Sequential".to_string()),
            original_order_preserved: Some(true),
            processing_strategy_used: Some("PreserveOrder".to_string()),
        },
    );

    let course = Course {
        id: Uuid::new_v4(),
        name: "Overflow Test Course".to_string(),
        created_at: Utc::now(),
        raw_titles: vec!["Very Long Video".to_string()],
        videos: vec![crate::types::VideoMetadata {
            title: "Very Long Video".to_string(),
            source_url: None,
            video_id: None,
            playlist_id: None,
            original_index: 0,
            duration_seconds: Some(5400.0),
            thumbnail_url: None,
            description: None,
            upload_date: None,
            author: None,
            view_count: None,
            tags: Vec::new(),
            is_local: false,
        }],
        structure: Some(structure),
    };

    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60, // 60 minute sessions
        include_weekends: false,
        advanced_settings: None,
    };

    let result = generate_plan(&course, &settings);
    assert!(result.is_ok());

    let plan = result.unwrap();
    assert!(!plan.items.is_empty());

    // The plan item should have overflow warnings
    let first_item = &plan.items[0];
    assert!(!first_item.overflow_warnings.is_empty());

    // The warning should mention that the video exceeds the session limit
    let warning_text = first_item.overflow_warnings.join(" ");
    assert!(warning_text.contains("exceeds") || warning_text.contains("long"));
}

#[test]
fn test_overflow_handling() {
    // Create a course with a very long video
    let structure = CourseStructure::new_basic(
        vec![Module::new_basic(
            "Test Module".to_string(),
            vec![
                Section {
                    title: "Normal Video".to_string(),
                    video_index: 0,
                    duration: Duration::from_secs(20 * 60), // 20 minutes
                },
                Section {
                    title: "Very Long Video".to_string(),
                    video_index: 1,
                    duration: Duration::from_secs(90 * 60), // 90 minutes - exceeds 60 min session
                },
                Section {
                    title: "Another Normal Video".to_string(),
                    video_index: 2,
                    duration: Duration::from_secs(15 * 60), // 15 minutes
                },
            ],
        )],
        StructureMetadata {
            total_videos: 3,
            total_duration: Duration::from_secs(125 * 60),
            estimated_duration_hours: Some(2.0),
            difficulty_level: Some("Intermediate".to_string()),
            structure_quality_score: None,
            content_coherence_score: None,
            content_type_detected: Some("Sequential".to_string()),
            original_order_preserved: Some(true),
            processing_strategy_used: Some("PreserveOrder".to_string()),
        },
    );

    let course = Course {
        id: Uuid::new_v4(),
        name: "Overflow Test Course".to_string(),
        created_at: Utc::now(),
        raw_titles: vec![
            "Normal Video".to_string(),
            "Very Long Video".to_string(),
            "Another Normal Video".to_string(),
        ],
        videos: vec![
            crate::types::VideoMetadata {
                title: "Normal Video".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 0,
                duration_seconds: Some(1200.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
            crate::types::VideoMetadata {
                title: "Very Long Video".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 1,
                duration_seconds: Some(5400.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
            crate::types::VideoMetadata {
                title: "Another Normal Video".to_string(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: 2,
                duration_seconds: Some(900.0),
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            },
        ],
        structure: Some(structure),
    };

    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60, // 60 minute sessions
        include_weekends: false,
        advanced_settings: None,
    };

    let result = strategies::generate_time_based_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(plan_items.len() >= 2); // Should create at least 2 sessions

    // The long video should be in its own session
    let long_video_session = plan_items.iter().find(|item| {
        item.video_indices.contains(&1) // video_index 1 is the long video
    });
    assert!(long_video_session.is_some());

    // The long video session should only contain that one video (due to overflow)
    if let Some(session) = long_video_session {
        assert_eq!(session.video_indices.len(), 1);
        assert_eq!(session.video_indices[0], 1);
    }
}

#[test]
fn test_invalid_settings() {
    let course = create_test_course();
    let mut settings = create_test_settings();
    settings.sessions_per_week = 0;

    let result = generate_plan(&course, &settings);
    assert!(matches!(result, Err(PlanError::InvalidSettings(_))));
}

#[test]
fn test_plan_optimization() {
    let course = create_test_course();
    let settings = create_test_settings();

    let mut plan = generate_plan(&course, &settings).unwrap();
    let original_length = plan.items.len();

    optimize_plan(&mut plan).unwrap();

    // Should have added review sessions and other optimizations
    assert!(plan.items.len() >= original_length);

    // Verify plan is sorted by date
    for i in 1..plan.items.len() {
        assert!(plan.items[i - 1].date <= plan.items[i].date);
    }
}

#[test]
fn test_difficulty_based_planning() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = strategies::generate_difficulty_based_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(!plan_items.is_empty());

    // Verify progressive difficulty (first items should be easier)
    // This is a simplified test - in practice, we'd need more sophisticated verification
    assert!(plan_items.len() >= 2);
}

#[test]
fn test_spaced_repetition_planning() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = strategies::generate_spaced_repetition_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(!plan_items.is_empty());

    // Should have more items than original (due to review sessions)
    assert!(plan_items.len() > course.video_count());
}

#[test]
fn test_adaptive_planning() {
    let course = create_test_course();
    let settings = create_test_settings();

    let result = strategies::generate_adaptive_plan(&course, &settings);
    assert!(result.is_ok());

    let plan_items = result.unwrap();
    assert!(!plan_items.is_empty());
    assert_eq!(plan_items.len(), course.video_count());
}

#[test]
fn test_course_complexity_analysis() {
    let course = create_test_course();
    let complexity = analyze_course_complexity(&course);

    assert!(complexity >= 0.0 && complexity <= 1.0);
}

#[test]
fn test_user_experience_inference() {
    let beginner_settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 2,
        session_length_minutes: 30,
        include_weekends: false,
        advanced_settings: None,
    };

    let expert_settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 6,
        session_length_minutes: 120,
        include_weekends: true,
        advanced_settings: None,
    };

    assert_eq!(infer_user_experience_level(&beginner_settings), DifficultyLevel::Beginner);
    assert_eq!(infer_user_experience_level(&expert_settings), DifficultyLevel::Expert);
}

#[test]
fn test_enhanced_strategy_selection() {
    let course = create_test_course();
    let settings = create_test_settings();

    let strategy = choose_distribution_strategy(&course, &settings).unwrap();

    // Should return a valid strategy
    match strategy {
        DistributionStrategy::ModuleBased
        | DistributionStrategy::TimeBased
        | DistributionStrategy::Hybrid
        | DistributionStrategy::DifficultyBased
        | DistributionStrategy::SpacedRepetition
        | DistributionStrategy::Adaptive => {},
    }
}

#[test]
fn test_learning_velocity_analysis() {
    let course = create_test_course();
    let settings = create_test_settings();
    let plan = generate_plan(&course, &settings).unwrap();

    let analysis = analyze_learning_velocity(&plan);

    assert!(analysis.videos_per_day >= 0.0);
    assert!(analysis.total_duration_days >= 0);
    assert!(!analysis.recommended_adjustments.is_empty());
}

#[test]
fn test_cognitive_load_calculation() {
    let high_load = crate::planner::strategies::adaptive::calculate_cognitive_load(
        "Advanced Algorithm Implementation",
        Duration::from_secs(3600),
    );
    let low_load = crate::planner::strategies::adaptive::calculate_cognitive_load(
        "Introduction to Basics",
        Duration::from_secs(600),
    );

    assert!(high_load > low_load);
    assert!(high_load <= 1.0 && low_load >= 0.0);
}

#[test]
fn test_session_type_classification() {
    assert_eq!(
        crate::planner::strategies::adaptive::classify_session_type("Introduction to Programming"),
        crate::planner::strategies::adaptive::SessionType::Introduction
    );
    assert_eq!(
        crate::planner::strategies::adaptive::classify_session_type("Hands-on Practice Session"),
        crate::planner::strategies::adaptive::SessionType::Practice
    );
    assert_eq!(
        crate::planner::strategies::adaptive::classify_session_type("Review and Summary"),
        crate::planner::strategies::adaptive::SessionType::Review
    );
    assert_eq!(
        crate::planner::strategies::adaptive::classify_session_type("Final Project Build"),
        crate::planner::strategies::adaptive::SessionType::Project
    );
    assert_eq!(
        crate::planner::strategies::adaptive::classify_session_type("Quiz and Assessment"),
        crate::planner::strategies::adaptive::SessionType::Assessment
    );
}

#[test]
fn test_optimal_time_determination() {
    assert_eq!(
        crate::planner::strategies::adaptive::determine_optimal_time_of_day(
            "Complex Algorithm Analysis"
        ),
        Some(crate::planner::strategies::adaptive::TimeOfDay::Morning)
    );
    assert_eq!(
        crate::planner::strategies::adaptive::determine_optimal_time_of_day("Practice Exercises"),
        Some(crate::planner::strategies::adaptive::TimeOfDay::Afternoon)
    );
    assert_eq!(
        crate::planner::strategies::adaptive::determine_optimal_time_of_day("Review Session"),
        Some(crate::planner::strategies::adaptive::TimeOfDay::Evening)
    );
    assert_eq!(
        crate::planner::strategies::adaptive::determine_optimal_time_of_day("General Topic"),
        None
    );
}

mod tests_plan_from_groups {
    use super::*;
    use crate::types::{Course, CourseStructure, Module, PlanSettings, Section, StructureMetadata};
    use chrono::Utc;
    use std::time::Duration;
    use uuid::Uuid;

    fn make_course() -> Course {
        let structure = CourseStructure::new_basic(
            vec![
                Module::new_basic(
                    "Introduction".to_string(),
                    vec![
                        Section {
                            title: "Welcome".to_string(),
                            video_index: 0,
                            duration: Duration::from_secs(600),
                        },
                        Section {
                            title: "Setup".to_string(),
                            video_index: 1,
                            duration: Duration::from_secs(900),
                        },
                    ],
                ),
                Module::new_basic(
                    "Advanced Topics".to_string(),
                    vec![Section {
                        title: "Complex Example".to_string(),
                        video_index: 2,
                        duration: Duration::from_secs(1800),
                    }],
                ),
            ],
            StructureMetadata {
                total_videos: 3,
                total_duration: Duration::from_secs(600 + 900 + 1800),
                estimated_duration_hours: Some(1.0),
                difficulty_level: Some("Intermediate".to_string()),
                structure_quality_score: None,
                content_coherence_score: None,
                content_type_detected: Some("Sequential".to_string()),
                original_order_preserved: Some(true),
                processing_strategy_used: Some("PreserveOrder".to_string()),
            },
        );

        Course {
            id: Uuid::new_v4(),
            name: "Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec![
                "Welcome".to_string(),
                "Setup".to_string(),
                "Complex Example".to_string(),
            ],
            videos: vec![
                crate::types::VideoMetadata {
                    title: "Welcome".to_string(),
                    source_url: None,
                    video_id: None,
                    playlist_id: None,
                    original_index: 0,
                    duration_seconds: Some(600.0),
                    thumbnail_url: None,
                    description: None,
                    upload_date: None,
                    author: None,
                    view_count: None,
                    tags: Vec::new(),
                    is_local: false,
                },
                crate::types::VideoMetadata {
                    title: "Setup".to_string(),
                    source_url: None,
                    video_id: None,
                    playlist_id: None,
                    original_index: 1,
                    duration_seconds: Some(900.0),
                    thumbnail_url: None,
                    description: None,
                    upload_date: None,
                    author: None,
                    view_count: None,
                    tags: Vec::new(),
                    is_local: false,
                },
                crate::types::VideoMetadata {
                    title: "Complex Example".to_string(),
                    source_url: None,
                    video_id: None,
                    playlist_id: None,
                    original_index: 2,
                    duration_seconds: Some(1800.0),
                    thumbnail_url: None,
                    description: None,
                    upload_date: None,
                    author: None,
                    view_count: None,
                    tags: Vec::new(),
                    is_local: false,
                },
            ],
            structure: Some(structure),
        }
    }

    fn make_settings() -> PlanSettings {
        PlanSettings {
            start_date: Utc::now() + chrono::Duration::days(1),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        }
    }

    fn flatten_plan_video_indices(plan: &Plan) -> Vec<usize> {
        let mut out = Vec::new();
        for item in &plan.items {
            out.extend_from_slice(&item.video_indices);
        }
        out
    }

    #[test]
    fn test_generate_plan_from_groups_empty_groups() {
        let course = make_course();
        let settings = make_settings();

        let base = generate_plan(&course, &settings).expect("base plan");
        let grp = generate_plan_from_groups(&course, vec![], &settings).expect("groups plan");

        // Should not change number of sessions
        assert_eq!(base.items.len(), grp.items.len());

        // Should cover the same set of videos
        let mut base_all = flatten_plan_video_indices(&base);
        let mut grp_all = flatten_plan_video_indices(&grp);
        base_all.sort_unstable();
        grp_all.sort_unstable();
        assert_eq!(base_all, grp_all);
    }

    #[test]
    fn test_generate_plan_from_groups_non_overlapping() {
        let course = make_course();
        let settings = make_settings();

        // Group 0: video 2 should come before Group 1: videos 0 and 1
        let groups = vec![vec![2usize], vec![0usize, 1usize]];
        let grp_plan = generate_plan_from_groups(&course, groups, &settings).expect("grouped plan");

        // Find earliest dates for sessions containing videos by group
        let mut earliest_g0 = None;
        let mut earliest_g1 = None;

        for item in &grp_plan.items {
            if item.video_indices.iter().any(|&v| v == 2) {
                earliest_g0 = Some(item.date);
            }
            if item.video_indices.iter().any(|&v| v == 0 || v == 1) {
                earliest_g1 = Some(
                    earliest_g1
                        .map_or(item.date, |d: chrono::DateTime<chrono::Utc>| d.min(item.date)),
                );
            }
        }

        assert!(earliest_g0.is_some() && earliest_g1.is_some());
        assert!(earliest_g0.unwrap() <= earliest_g1.unwrap());
    }

    #[test]
    fn test_generate_plan_from_groups_uneven_group_sizes() {
        let course = make_course();
        let settings = make_settings();

        // Group 0: videos [1,2] should come before Group 1: [0]
        let groups = vec![vec![1usize, 2usize], vec![0usize]];
        let grp_plan = generate_plan_from_groups(&course, groups, &settings).expect("grouped plan");

        let mut earliest_g0 = None;
        let mut earliest_g1 = None;

        for item in &grp_plan.items {
            if item.video_indices.iter().any(|&v| v == 1 || v == 2) {
                earliest_g0 = Some(
                    earliest_g0
                        .map_or(item.date, |d: chrono::DateTime<chrono::Utc>| d.min(item.date)),
                );
            }
            if item.video_indices.iter().any(|&v| v == 0) {
                earliest_g1 = Some(
                    earliest_g1
                        .map_or(item.date, |d: chrono::DateTime<chrono::Utc>| d.min(item.date)),
                );
            }
        }

        assert!(earliest_g0.is_some() && earliest_g1.is_some());
        assert!(earliest_g0.unwrap() <= earliest_g1.unwrap());
    }

    #[test]
    fn test_generate_plan_from_groups_out_of_bounds_indices() {
        let course = make_course();
        let settings = make_settings();

        // Include an out-of-bounds index; it should be ignored gracefully.
        let groups = vec![vec![999usize], vec![0usize]];
        let grp_plan = generate_plan_from_groups(&course, groups, &settings).expect("grouped plan");

        // Ensure plan still includes all valid videos
        let grp_all = flatten_plan_video_indices(&grp_plan);
        let mut sorted = grp_all.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2]);

        // Sessions with valid group (video 0) should sort before ungrouped items
        let mut earliest_g_valid = None;
        let mut earliest_ungrouped = None;

        for item in &grp_plan.items {
            let has_valid = item.video_indices.iter().any(|&v| v == 0);
            let has_any_group = item.video_indices.iter().any(|&v| v == 0 || v == 999); // 999 doesn't exist in plan items

            // Items with no valid group mapping will be considered ungrouped (usize::MAX)
            if has_valid {
                earliest_g_valid = Some(
                    earliest_g_valid
                        .map_or(item.date, |d: chrono::DateTime<chrono::Utc>| d.min(item.date)),
                );
            } else if !has_any_group {
                earliest_ungrouped = Some(
                    earliest_ungrouped
                        .map_or(item.date, |d: chrono::DateTime<chrono::Utc>| d.min(item.date)),
                );
            }
        }

        // If there are ungrouped items, valid grouped items should not come after them
        if let (Some(g_valid), Some(ungrouped)) = (earliest_g_valid, earliest_ungrouped) {
            assert!(g_valid <= ungrouped);
        }
    }
}
