//! Content clustering algorithms for intelligent video grouping
//!
//! This module provides content-aware clustering algorithms that analyze video titles
//! to group related content together while respecting duration constraints.

#[cfg(feature = "advanced_nlp")]
pub mod content_similarity;
pub mod difficulty_analyzer;
#[cfg(feature = "advanced_nlp")]
pub mod duration_balancer;
#[cfg(feature = "advanced_nlp")]
pub mod hierarchical;
#[cfg(feature = "advanced_nlp")]
pub mod hybrid;
#[cfg(feature = "advanced_nlp")]
pub mod kmeans;
#[cfg(feature = "advanced_nlp")]
pub mod lda;
#[cfg(feature = "advanced_nlp")]
pub mod metadata_generator;
#[cfg(feature = "advanced_nlp")]
pub mod preference_learning;
#[cfg(feature = "advanced_nlp")]
pub mod topic_extractor;

#[cfg(not(feature = "advanced_nlp"))]
mod stubs {
    use std::collections::HashMap;

    #[derive(Debug, Clone, Default)]
    pub struct FeatureVector(pub HashMap<String, f32>);

    pub type SimilarityMatrix = Vec<Vec<f32>>;

    #[derive(Debug, Clone, Default)]
    pub struct TfIdfAnalyzer;

    #[derive(Debug, Clone, Default)]
    pub struct ContentAnalysis;

    // Strategy/Ensemble stubs
    #[derive(Debug, Clone)]
    pub enum StrategySelection {
        Sequential,
        DurationBalanced,
        ContentBased,
        Hybrid,
        Fallback,
    }

    #[derive(Debug, Clone)]
    pub enum EnsembleMethod {
        Weighted,
        MajorityVote,
        Stacking,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ContentCharacteristics;

    #[derive(Debug, Clone, Default)]
    pub struct EnsembleResults;

    // Preference learning and feedback stubs
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct ClusteringPreferences;

    #[derive(Clone, Debug)]
    pub enum FeedbackType {
        ImplicitAcceptance,
        Rejection,
        ManualAdjustment,
        ParameterChange,
    }

    #[derive(Clone, Debug, Default)]
    pub struct ManualAdjustment {
        pub reason: String,
    }

    #[derive(Clone, Debug)]
    pub enum ABTestVariant {
        A,
        B,
    }

    #[derive(Clone, Debug)]
    pub enum AdjustmentType {
        Increase,
        Decrease,
        Toggle,
    }

    #[derive(Clone, Debug)]
    pub struct ClusteringFeedback {
        pub id: uuid::Uuid,
        pub course_id: uuid::Uuid,
        pub clustering_parameters: ClusteringPreferences,
        pub feedback_type: FeedbackType,
        pub rating: f32,
        pub comments: Option<String>,
        pub manual_adjustments: Vec<ManualAdjustment>,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Clone, Debug, Default)]
    pub struct ABTestConfig {
        pub id: uuid::Uuid,
        pub name: String,
        pub description: Option<String>,
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        pub current_sample_size: i64,
        pub target_sample_size: i64,
        pub is_active: bool,
    }

    #[derive(Clone, Debug)]
    pub struct ABTestAnalysis;

    #[derive(Clone, Debug)]
    pub struct ABTestResult {
        pub test_id: uuid::Uuid,
        pub course_id: uuid::Uuid,
        pub variant: ABTestVariant,
        pub parameters_used: ClusteringPreferences,
        pub user_satisfaction: f32,
        pub processing_time_ms: u64,
        pub quality_score: f32,
        pub user_made_adjustments: bool,
        pub adjustment_count: usize,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Default)]
    pub struct PreferenceLearningEngine;

    impl PreferenceLearningEngine {
        pub fn new(_settings: &crate::storage::AppSettings) -> Self {
            Self
        }

        pub fn with_preferences(_p: ClusteringPreferences) -> Self {
            Self
        }

        pub fn get_preferences(&self) -> ClusteringPreferences {
            ClusteringPreferences::default()
        }

        pub fn get_recommended_parameters(
            &self,
            _course: &crate::types::Course,
        ) -> ClusteringPreferences {
            ClusteringPreferences::default()
        }

        pub fn submit_feedback(&self, _feedback: &ClusteringFeedback) {}

        pub fn update_preferences_from_feedback(
            &mut self,
            _feedback: ClusteringFeedback,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        pub fn record_manual_adjustment(&self, _a: &ManualAdjustment) {}

        pub fn create_ab_test(
            &self,
            _course_id: uuid::Uuid,
        ) -> Option<(ABTestVariant, ClusteringPreferences)> {
            None
        }

        pub fn get_ab_test_parameters(
            &self,
            _course_id: uuid::Uuid,
        ) -> Option<(ABTestVariant, ClusteringPreferences)> {
            None
        }

        pub fn record_ab_test_result(&self, _r: &ABTestResult) {}

        pub fn analyze_ab_test(&self, _test_id: uuid::Uuid) -> Option<ABTestAnalysis> {
            None
        }

        pub fn optimize_parameters(&self, _course: &crate::types::Course) -> ClusteringPreferences {
            ClusteringPreferences::default()
        }

        pub fn get_feedback_history(&self) -> Vec<ClusteringFeedback> {
            vec![]
        }

        pub fn get_active_ab_tests(&self) -> Vec<ABTestConfig> {
            vec![]
        }

        pub fn update_settings(&self, _settings: &crate::storage::AppSettings) {}
    }
}

// Re-export main clustering types and functions
#[cfg(feature = "advanced_nlp")]
pub use content_similarity::{ContentAnalysis, FeatureVector, SimilarityMatrix, TfIdfAnalyzer};
pub use difficulty_analyzer::{
    DifficultyAnalyzer, DifficultyProgression, PacingRecommendation, ProgressionIssue,
    ProgressionValidation, SessionDifficultyAnalysis,
};
#[cfg(feature = "advanced_nlp")]
pub use duration_balancer::{BalancedCluster, DurationBalancer};
#[cfg(feature = "advanced_nlp")]
pub use hierarchical::{HierarchicalClusterer, LinkageMethod};
#[cfg(feature = "advanced_nlp")]
pub use hybrid::{
    ContentCharacteristics, EnsembleMethod, EnsembleResults, HybridClusterer, StrategySelection,
};
#[cfg(feature = "advanced_nlp")]
pub use kmeans::{Cluster, KMeansClusterer};
#[cfg(feature = "advanced_nlp")]
pub use lda::{DocumentTopics, LdaClusterer, LdaModel, Topic};
#[cfg(feature = "advanced_nlp")]
pub use preference_learning::{
    ABTestAnalysis, ABTestConfig, ABTestResult, ABTestVariant, AdjustmentType, ClusteringFeedback,
    ClusteringPreferences, FeedbackType, ManualAdjustment, PreferenceLearningEngine,
};
#[cfg(feature = "advanced_nlp")]
pub use topic_extractor::TopicExtractor;

#[cfg(not(feature = "advanced_nlp"))]
pub use stubs::{
    ABTestAnalysis, ABTestConfig, ABTestResult, ABTestVariant, AdjustmentType, ClusteringFeedback,
    ClusteringPreferences, ContentAnalysis, ContentCharacteristics, EnsembleMethod,
    EnsembleResults, FeatureVector, FeedbackType, ManualAdjustment, PreferenceLearningEngine,
    SimilarityMatrix, StrategySelection, TfIdfAnalyzer,
};

use crate::types::Section;
use anyhow::Result;
use std::time::Duration;

/// Clustering error types
#[derive(Debug, thiserror::Error)]
pub enum ClusteringError {
    #[error("Insufficient content for clustering: {0} videos (minimum 5 required)")]
    InsufficientContent(usize),

    #[error("Content analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Clustering algorithm failed to converge after {0} iterations")]
    ConvergenceFailed(usize),

    #[error("Duration data missing or invalid for {0} videos")]
    InvalidDurations(usize),

    #[error("Optimization timeout after {0}ms")]
    OptimizationTimeout(u64),
}

/// Video cluster representation
#[derive(Debug, Clone)]
pub struct VideoCluster {
    pub videos: Vec<usize>,
    pub centroid: FeatureVector,
    pub similarity_score: f32,
    pub topic_keywords: Vec<String>,
}

/// Optimized cluster with duration balancing
#[derive(Debug, Clone)]
pub struct OptimizedCluster {
    pub videos: Vec<VideoWithMetadata>,
    pub total_duration: Duration,
    pub average_similarity: f32,
    pub difficulty_level: crate::types::DifficultyLevel,
    pub suggested_title: String,
}

/// Video with clustering metadata
#[derive(Debug, Clone)]
pub struct VideoWithMetadata {
    pub index: usize,
    pub title: String,
    pub duration: Duration,
    pub feature_vector: FeatureVector,
    pub difficulty_score: f32,
    pub topic_tags: Vec<String>,
}

/// Clustering quality metrics
#[derive(Debug, Clone)]
pub struct ClusteringQuality {
    pub silhouette_score: f32,
    pub intra_cluster_similarity: f32,
    pub inter_cluster_separation: f32,
    pub duration_balance_score: f32,
}

// Re-export types from main types module to avoid duplication
pub use crate::types::{
    ClusteringConfidenceScores, ClusteringRationale, InputMetrics, ModuleConfidence,
    ModuleRationale, PerformanceMetrics,
};

/// Main content clusterer trait
pub trait ContentClusterer {
    fn analyze_content(&self, titles: &[String]) -> Result<ContentAnalysis, ClusteringError>;
    fn cluster_videos(
        &self,
        analysis: &ContentAnalysis,
        target_clusters: usize,
    ) -> Result<Vec<VideoCluster>, ClusteringError>;
    fn optimize_clusters(
        &self,
        clusters: Vec<VideoCluster>,
        durations: &[Duration],
    ) -> Result<Vec<OptimizedCluster>, ClusteringError>;
}

// Use ClusteringMetadata from types.rs to avoid duplication
pub use crate::types::ClusteringMetadata;

/// Convert sections to videos with metadata for clustering
pub fn sections_to_videos_with_metadata(sections: &[Section]) -> Vec<VideoWithMetadata> {
    sections_to_videos_with_metadata_for_user(sections, crate::types::DifficultyLevel::Intermediate)
}

/// Convert sections to videos with metadata for clustering with user experience level
pub fn sections_to_videos_with_metadata_for_user(
    sections: &[Section],
    user_level: crate::types::DifficultyLevel,
) -> Vec<VideoWithMetadata> {
    let analyzer = DifficultyAnalyzer::new(user_level);

    sections
        .iter()
        .map(|section| VideoWithMetadata {
            index: section.video_index,
            title: section.title.clone(),
            duration: section.duration,
            feature_vector: FeatureVector::default(), // Will be populated during analysis
            difficulty_score: analyzer.calculate_difficulty_score(section),
            topic_tags: Vec::new(), // Will be populated during analysis
        })
        .collect()
}

/// Estimate difficulty score based on title content (simple heuristic)
/// This is a lightweight alternative to DifficultyAnalyzer for quick estimates
pub fn estimate_difficulty_score(title: &str) -> f32 {
    let title_lower = title.to_lowercase();

    let beginner_keywords = [
        "introduction",
        "basics",
        "fundamentals",
        "getting started",
        "beginner",
        "overview",
        "what is",
        "how to",
    ];
    let advanced_keywords = [
        "advanced",
        "expert",
        "master",
        "deep dive",
        "optimization",
        "architecture",
        "complex",
        "sophisticated",
        "implementation",
    ];

    let mut score: f32 = 0.5; // Default intermediate score

    for keyword in &beginner_keywords {
        if title_lower.contains(keyword) {
            score -= 0.1;
        }
    }

    for keyword in &advanced_keywords {
        if title_lower.contains(keyword) {
            score += 0.1;
        }
    }

    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_score_estimation() {
        assert!(estimate_difficulty_score("Introduction to Programming") < 0.5);
        assert!(estimate_difficulty_score("Advanced Optimization Techniques") > 0.5);
        assert_eq!(estimate_difficulty_score("Regular Video Title"), 0.5);
    }

    #[test]
    fn test_clustering_metadata_default() {
        let metadata = ClusteringMetadata::default();
        assert_eq!(metadata.algorithm_used, crate::types::ClusteringAlgorithm::Fallback);
        assert_eq!(metadata.similarity_threshold, 0.6);
        assert_eq!(metadata.cluster_count, 0);
    }
}
