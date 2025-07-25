//! Integration tests for the preference learning system

use course_pilot::nlp::PreferenceService;
use course_pilot::nlp::clustering::{
    AdjustmentType, ClusteringFeedback, ClusteringPreferences, FeedbackType, ManualAdjustment,
    PreferenceLearningEngine,
};
use course_pilot::storage::{AppSettings, PreferenceStorage};
use course_pilot::types::{ClusteringAlgorithm, ClusteringStrategy, Course, DifficultyLevel};
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn test_preference_learning_basic_functionality() {
    // Test basic preference learning engine functionality
    let mut engine = PreferenceLearningEngine::new();

    // Initial preferences should be defaults
    let initial_prefs = engine.get_preferences();
    assert_eq!(initial_prefs.similarity_threshold, 0.6);
    assert_eq!(
        initial_prefs.preferred_algorithm,
        ClusteringAlgorithm::Hybrid
    );

    // Create positive feedback
    let positive_feedback = ClusteringFeedback {
        id: Uuid::new_v4(),
        course_id: Uuid::new_v4(),
        clustering_parameters: ClusteringPreferences {
            similarity_threshold: 0.8,
            ..ClusteringPreferences::default()
        },
        feedback_type: FeedbackType::ExplicitRating,
        rating: 0.9, // High rating
        comments: None,
        manual_adjustments: Vec::new(),
        created_at: chrono::Utc::now(),
    };

    // Update preferences with positive feedback
    engine
        .update_preferences_from_feedback(positive_feedback)
        .unwrap();

    // Preferences should move towards the highly-rated parameters
    let updated_prefs = engine.get_preferences();
    assert!(updated_prefs.similarity_threshold > initial_prefs.similarity_threshold);
    assert_eq!(updated_prefs.usage_count, 1);
}

#[test]
fn test_preference_storage_integration() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = PreferenceStorage::new(db_path);

    // Initialize storage
    storage.initialize().unwrap();

    // Create and save preferences
    let mut preferences = ClusteringPreferences::default();
    preferences.similarity_threshold = 0.75;
    preferences.max_clusters = 12;

    storage.save_preferences(&preferences).unwrap();

    // Load preferences back
    let loaded_prefs = storage.load_preferences().unwrap().unwrap();
    assert_eq!(loaded_prefs.similarity_threshold, 0.75);
    assert_eq!(loaded_prefs.max_clusters, 12);
}

#[test]
fn test_preference_service_integration() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let settings = AppSettings::default();

    let service = PreferenceService::new(db_path, settings).unwrap();

    // Test getting preferences
    let preferences = service.get_preferences().unwrap();
    assert_eq!(preferences.similarity_threshold, 0.6);

    // Test course complexity estimation
    let beginner_course = Course::new(
        "Beginner Course".to_string(),
        vec![
            "Introduction to Programming".to_string(),
            "Getting Started Basics".to_string(),
        ],
    );

    let recommended_params = service
        .get_recommended_parameters(&beginner_course)
        .unwrap();
    // For small courses, max_clusters should be limited
    assert!(recommended_params.max_clusters <= 3);
}

#[test]
fn test_manual_adjustment_learning() {
    let mut engine = PreferenceLearningEngine::new();
    let initial_threshold = engine.get_preferences().similarity_threshold;

    // Create feedback with many manual adjustments (indicates poor clustering)
    let adjustment_feedback = ClusteringFeedback {
        id: Uuid::new_v4(),
        course_id: Uuid::new_v4(),
        clustering_parameters: ClusteringPreferences::default(),
        feedback_type: FeedbackType::ManualAdjustment,
        rating: 0.4, // Low rating due to many adjustments needed
        comments: None,
        manual_adjustments: vec![
            ManualAdjustment {
                adjustment_type: AdjustmentType::SplitModule,
                from_module: 0,
                to_module: 1,
                video_indices: vec![1, 2, 3],
                reason: Some("Videos didn't belong together".to_string()),
                timestamp: chrono::Utc::now(),
            },
            ManualAdjustment {
                adjustment_type: AdjustmentType::SplitModule,
                from_module: 1,
                to_module: 2,
                video_indices: vec![4, 5],
                reason: Some("More splitting needed".to_string()),
                timestamp: chrono::Utc::now(),
            },
        ],
        created_at: chrono::Utc::now(),
    };

    engine
        .update_preferences_from_feedback(adjustment_feedback)
        .unwrap();

    // With many split adjustments, the system should prefer smaller clusters
    let updated_prefs = engine.get_preferences();
    assert!(
        updated_prefs.max_clusters > engine.get_preferences().max_clusters
            || updated_prefs.similarity_threshold < initial_threshold
    );
}

#[test]
fn test_ab_test_functionality() {
    let mut engine = PreferenceLearningEngine::new();

    // Create A/B test
    let test_id = engine.create_ab_test(
        "TF-IDF vs K-Means".to_string(),
        "Compare TF-IDF and K-Means algorithms".to_string(),
        ClusteringAlgorithm::TfIdf,
        ClusteringAlgorithm::KMeans,
        10, // Small sample size for testing
    );

    // Verify test was created
    let active_tests = engine.get_active_ab_tests();
    assert_eq!(active_tests.len(), 1);
    assert_eq!(active_tests[0].id, test_id);

    // Get test parameters for a course
    let course_id = Uuid::new_v4();
    let (variant, params) = engine.get_ab_test_parameters(test_id, course_id).unwrap();

    // Verify we get consistent variant for same course
    let (variant2, _) = engine.get_ab_test_parameters(test_id, course_id).unwrap();
    assert_eq!(variant, variant2);

    // Record test result
    let result = course_pilot::nlp::clustering::ABTestResult {
        test_id,
        course_id,
        variant,
        parameters_used: params,
        user_satisfaction: 0.8,
        processing_time_ms: 1500,
        quality_score: 0.75,
        user_made_adjustments: false,
        adjustment_count: 0,
        timestamp: chrono::Utc::now(),
    };

    engine.record_ab_test_result(result).unwrap();

    // Test should still be active (need more samples)
    let active_tests = engine.get_active_ab_tests();
    assert_eq!(active_tests.len(), 1);
    assert!(active_tests[0].is_active);
}

#[test]
fn test_parameter_recommendations() {
    let engine = PreferenceLearningEngine::new();

    // Test recommendations for different course sizes
    let small_params = engine.get_recommended_parameters(5, DifficultyLevel::Beginner);
    let large_params = engine.get_recommended_parameters(100, DifficultyLevel::Expert);

    // Small courses should have fewer clusters
    assert!(small_params.max_clusters <= 3);
    // Large courses should have more clusters
    assert!(large_params.max_clusters > 5);

    // Beginner courses should prioritize content grouping
    assert_eq!(
        small_params.preferred_strategy,
        ClusteringStrategy::ContentBased
    );
    assert!(small_params.content_vs_duration_weight > 0.7);

    // Expert courses should use hybrid approach
    assert_eq!(large_params.preferred_strategy, ClusteringStrategy::Hybrid);
}

#[test]
fn test_feedback_type_handling() {
    let mut engine = PreferenceLearningEngine::new();
    let initial_algorithm = engine.get_preferences().preferred_algorithm.clone();

    // Test rejection feedback - should try different algorithm
    let rejection_feedback = ClusteringFeedback {
        id: Uuid::new_v4(),
        course_id: Uuid::new_v4(),
        clustering_parameters: ClusteringPreferences {
            preferred_algorithm: initial_algorithm.clone(),
            ..ClusteringPreferences::default()
        },
        feedback_type: FeedbackType::Rejection,
        rating: 0.1, // Very low rating
        comments: Some("Clustering was terrible".to_string()),
        manual_adjustments: Vec::new(),
        created_at: chrono::Utc::now(),
    };

    engine
        .update_preferences_from_feedback(rejection_feedback)
        .unwrap();

    // Algorithm should change after rejection
    let updated_prefs = engine.get_preferences();
    assert_ne!(updated_prefs.preferred_algorithm, initial_algorithm);
}
