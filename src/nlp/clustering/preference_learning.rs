//! User preference learning system for clustering parameters
//!
//! This module implements a system that tracks user preferences for clustering
//! parameters and automatically tunes them based on user feedback and behavior.

use crate::types::{ClusteringAlgorithm, ClusteringStrategy, DifficultyLevel};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// User preferences for clustering parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringPreferences {
    /// Preferred similarity threshold (0.0 - 1.0)
    pub similarity_threshold: f32,
    /// Preferred clustering algorithm
    pub preferred_algorithm: ClusteringAlgorithm,
    /// Preferred clustering strategy
    pub preferred_strategy: ClusteringStrategy,
    /// User's experience level for difficulty adaptation
    pub user_experience_level: DifficultyLevel,
    /// Maximum number of clusters to create
    pub max_clusters: usize,
    /// Minimum cluster size (number of videos)
    pub min_cluster_size: usize,
    /// Whether to enable duration balancing
    pub enable_duration_balancing: bool,
    /// Weight for content similarity vs duration balance (0.0 - 1.0)
    pub content_vs_duration_weight: f32,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
    /// Number of times these preferences have been used
    pub usage_count: u32,
    /// User satisfaction score (0.0 - 1.0) based on feedback
    pub satisfaction_score: f32,
}

impl Default for ClusteringPreferences {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.6,
            preferred_algorithm: ClusteringAlgorithm::Hybrid,
            preferred_strategy: ClusteringStrategy::Hybrid,
            user_experience_level: DifficultyLevel::Intermediate,
            max_clusters: 8,
            min_cluster_size: 2,
            enable_duration_balancing: true,
            content_vs_duration_weight: 0.7,
            last_updated: Utc::now(),
            usage_count: 0,
            satisfaction_score: 0.5,
        }
    }
}

/// User feedback on clustering results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringFeedback {
    pub id: Uuid,
    pub course_id: Uuid,
    pub clustering_parameters: ClusteringPreferences,
    pub feedback_type: FeedbackType,
    pub rating: f32, // 0.0 - 1.0
    pub comments: Option<String>,
    pub manual_adjustments: Vec<ManualAdjustment>,
    pub created_at: DateTime<Utc>,
}

/// Types of feedback users can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeedbackType {
    /// User explicitly rated the clustering result
    ExplicitRating,
    /// User made manual adjustments to the clustering
    ManualAdjustment,
    /// User regenerated clustering with different parameters
    ParameterChange,
    /// User accepted clustering without changes
    ImplicitAcceptance,
    /// User rejected clustering and used fallback
    Rejection,
}

/// Manual adjustment made by user to clustering results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManualAdjustment {
    pub adjustment_type: AdjustmentType,
    pub from_module: usize,
    pub to_module: usize,
    pub video_indices: Vec<usize>,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Types of manual adjustments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdjustmentType {
    /// Move videos between modules
    MoveVideos,
    /// Split a module into multiple modules
    SplitModule,
    /// Merge multiple modules into one
    MergeModules,
    /// Rename a module
    RenameModule,
    /// Reorder modules
    ReorderModules,
}

/// A/B test configuration for clustering algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ABTestConfig {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub algorithm_a: ClusteringAlgorithm,
    pub algorithm_b: ClusteringAlgorithm,
    pub parameters_a: ClusteringPreferences,
    pub parameters_b: ClusteringPreferences,
    pub target_sample_size: usize,
    pub current_sample_size: usize,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// A/B test result for a single user interaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ABTestResult {
    pub test_id: Uuid,
    pub course_id: Uuid,
    pub variant: ABTestVariant,
    pub parameters_used: ClusteringPreferences,
    pub user_satisfaction: f32,
    pub processing_time_ms: u64,
    pub quality_score: f32,
    pub user_made_adjustments: bool,
    pub adjustment_count: usize,
    pub timestamp: DateTime<Utc>,
}

/// A/B test variant identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ABTestVariant {
    VariantA,
    VariantB,
}

/// Preference learning engine that adapts clustering parameters
pub struct PreferenceLearningEngine {
    preferences: ClusteringPreferences,
    feedback_history: Vec<ClusteringFeedback>,
    ab_tests: Vec<ABTestConfig>,
    ab_results: Vec<ABTestResult>,
}

impl PreferenceLearningEngine {
    /// Create a new preference learning engine with default preferences
    pub fn new() -> Self {
        Self {
            preferences: ClusteringPreferences::default(),
            feedback_history: Vec::new(),
            ab_tests: Vec::new(),
            ab_results: Vec::new(),
        }
    }

    /// Create engine with existing preferences
    pub fn with_preferences(preferences: ClusteringPreferences) -> Self {
        Self {
            preferences,
            feedback_history: Vec::new(),
            ab_tests: Vec::new(),
            ab_results: Vec::new(),
        }
    }

    /// Get current clustering preferences
    pub fn get_preferences(&self) -> &ClusteringPreferences {
        &self.preferences
    }

    /// Update preferences based on user feedback
    pub fn update_preferences_from_feedback(&mut self, feedback: ClusteringFeedback) -> Result<()> {
        self.feedback_history.push(feedback.clone());

        // Adjust preferences based on feedback type and rating
        match feedback.feedback_type {
            FeedbackType::ExplicitRating => {
                self.adjust_preferences_from_rating(
                    feedback.rating,
                    &feedback.clustering_parameters,
                )?;
            }
            FeedbackType::ManualAdjustment => {
                self.adjust_preferences_from_adjustments(&feedback.manual_adjustments)?;
            }
            FeedbackType::ParameterChange => {
                self.learn_from_parameter_change(&feedback.clustering_parameters)?;
            }
            FeedbackType::ImplicitAcceptance => {
                self.reinforce_current_preferences()?;
            }
            FeedbackType::Rejection => {
                self.adjust_preferences_from_rejection(&feedback.clustering_parameters)?;
            }
        }

        // Update satisfaction score and usage count
        self.preferences.satisfaction_score = self.calculate_average_satisfaction();
        self.preferences.usage_count += 1;
        self.preferences.last_updated = Utc::now();

        Ok(())
    }

    /// Adjust preferences based on explicit user rating
    fn adjust_preferences_from_rating(
        &mut self,
        rating: f32,
        used_params: &ClusteringPreferences,
    ) -> Result<()> {
        let learning_rate = 0.1; // How much to adjust based on feedback

        if rating > 0.7 {
            // Good rating - move preferences closer to used parameters
            self.preferences.similarity_threshold = self.preferences.similarity_threshold
                + learning_rate
                    * (used_params.similarity_threshold - self.preferences.similarity_threshold);

            self.preferences.content_vs_duration_weight =
                self.preferences.content_vs_duration_weight
                    + learning_rate
                        * (used_params.content_vs_duration_weight
                            - self.preferences.content_vs_duration_weight);

            // If the algorithm/strategy worked well, increase preference for it
            if rating > 0.8 {
                self.preferences.preferred_algorithm = used_params.preferred_algorithm.clone();
                self.preferences.preferred_strategy = used_params.preferred_strategy.clone();
            }
        } else if rating < 0.4 {
            // Poor rating - move preferences away from used parameters
            let adjustment = learning_rate
                * (used_params.similarity_threshold - self.preferences.similarity_threshold);
            self.preferences.similarity_threshold =
                (self.preferences.similarity_threshold - adjustment).clamp(0.3, 0.9);

            let weight_adjustment = learning_rate
                * (used_params.content_vs_duration_weight
                    - self.preferences.content_vs_duration_weight);
            self.preferences.content_vs_duration_weight =
                (self.preferences.content_vs_duration_weight - weight_adjustment).clamp(0.1, 0.9);
        }

        Ok(())
    }

    /// Learn from manual adjustments made by the user
    fn adjust_preferences_from_adjustments(
        &mut self,
        adjustments: &[ManualAdjustment],
    ) -> Result<()> {
        let adjustment_count = adjustments.len() as f32;

        // If user made many adjustments, the clustering wasn't good
        if adjustment_count > 3.0 {
            // Reduce similarity threshold to create more granular clusters
            self.preferences.similarity_threshold =
                (self.preferences.similarity_threshold - 0.05).max(0.3);
        } else if adjustment_count == 1.0 {
            // Minor adjustment - slightly tune parameters
            self.preferences.similarity_threshold =
                (self.preferences.similarity_threshold + 0.02).min(0.9);
        }

        // Analyze types of adjustments to learn preferences
        let split_count = adjustments
            .iter()
            .filter(|a| matches!(a.adjustment_type, AdjustmentType::SplitModule))
            .count();
        let merge_count = adjustments
            .iter()
            .filter(|a| matches!(a.adjustment_type, AdjustmentType::MergeModules))
            .count();

        if split_count > merge_count {
            // User prefers smaller clusters
            self.preferences.max_clusters = (self.preferences.max_clusters + 1).min(15);
            self.preferences.min_cluster_size = (self.preferences.min_cluster_size - 1).max(1);
        } else if merge_count > split_count {
            // User prefers larger clusters
            self.preferences.max_clusters = (self.preferences.max_clusters - 1).max(3);
            self.preferences.min_cluster_size = (self.preferences.min_cluster_size + 1).min(10);
        }

        Ok(())
    }

    /// Learn from user changing parameters
    fn learn_from_parameter_change(&mut self, new_params: &ClusteringPreferences) -> Result<()> {
        // User explicitly changed parameters - adopt their preferences
        let learning_rate = 0.3; // Higher learning rate for explicit changes

        self.preferences.similarity_threshold = self.preferences.similarity_threshold
            + learning_rate
                * (new_params.similarity_threshold - self.preferences.similarity_threshold);

        self.preferences.content_vs_duration_weight = self.preferences.content_vs_duration_weight
            + learning_rate
                * (new_params.content_vs_duration_weight
                    - self.preferences.content_vs_duration_weight);

        self.preferences.max_clusters = new_params.max_clusters;
        self.preferences.min_cluster_size = new_params.min_cluster_size;
        self.preferences.enable_duration_balancing = new_params.enable_duration_balancing;

        Ok(())
    }

    /// Reinforce current preferences when user accepts results
    fn reinforce_current_preferences(&mut self) -> Result<()> {
        // Increase confidence in current settings by small amount
        self.preferences.satisfaction_score = (self.preferences.satisfaction_score + 0.05).min(1.0);
        Ok(())
    }

    /// Adjust preferences when user rejects clustering
    fn adjust_preferences_from_rejection(
        &mut self,
        rejected_params: &ClusteringPreferences,
    ) -> Result<()> {
        // Try different algorithm/strategy
        self.preferences.preferred_algorithm = match rejected_params.preferred_algorithm {
            ClusteringAlgorithm::TfIdf => ClusteringAlgorithm::KMeans,
            ClusteringAlgorithm::KMeans => ClusteringAlgorithm::Hierarchical,
            ClusteringAlgorithm::Hierarchical => ClusteringAlgorithm::Lda,
            ClusteringAlgorithm::Lda => ClusteringAlgorithm::Hybrid,
            ClusteringAlgorithm::Hybrid => ClusteringAlgorithm::TfIdf,
            ClusteringAlgorithm::Fallback => ClusteringAlgorithm::Hybrid,
        };

        // Adjust similarity threshold in opposite direction
        if rejected_params.similarity_threshold > 0.6 {
            self.preferences.similarity_threshold =
                (rejected_params.similarity_threshold - 0.1).max(0.3);
        } else {
            self.preferences.similarity_threshold =
                (rejected_params.similarity_threshold + 0.1).min(0.9);
        }

        Ok(())
    }

    /// Calculate average satisfaction from feedback history
    fn calculate_average_satisfaction(&self) -> f32 {
        if self.feedback_history.is_empty() {
            return 0.5;
        }

        let total: f32 = self.feedback_history.iter().map(|f| f.rating).sum();
        total / self.feedback_history.len() as f32
    }

    /// Get recommended parameters for a specific course based on learning
    pub fn get_recommended_parameters(
        &self,
        course_video_count: usize,
        course_complexity: DifficultyLevel,
    ) -> ClusteringPreferences {
        let mut params = self.preferences.clone();

        // Adjust based on course size
        if course_video_count < 10 {
            params.max_clusters = (params.max_clusters).min(3);
            params.similarity_threshold = (params.similarity_threshold + 0.1).min(0.9);
        } else if course_video_count > 50 {
            params.max_clusters = (params.max_clusters + 2).min(15);
            params.similarity_threshold = (params.similarity_threshold - 0.05).max(0.3);
        }

        // Adjust based on course complexity
        match course_complexity {
            DifficultyLevel::Beginner => {
                params.preferred_strategy = ClusteringStrategy::ContentBased;
                params.content_vs_duration_weight = 0.8; // Prioritize content grouping
            }
            DifficultyLevel::Expert => {
                params.preferred_strategy = ClusteringStrategy::Hybrid;
                params.content_vs_duration_weight = 0.6; // Balance content and duration
            }
            _ => {} // Keep current preferences
        }

        params
    }

    /// Create a new A/B test configuration
    pub fn create_ab_test(
        &mut self,
        name: String,
        description: String,
        algorithm_a: ClusteringAlgorithm,
        algorithm_b: ClusteringAlgorithm,
        target_sample_size: usize,
    ) -> Uuid {
        let test_id = Uuid::new_v4();

        let test_config = ABTestConfig {
            id: test_id,
            name,
            description,
            algorithm_a,
            algorithm_b: algorithm_b.clone(),
            parameters_a: self.preferences.clone(),
            parameters_b: {
                let mut params_b = self.preferences.clone();
                params_b.preferred_algorithm = algorithm_b;
                // Vary some parameters for B variant
                params_b.similarity_threshold = (params_b.similarity_threshold + 0.1).min(0.9);
                params_b
            },
            target_sample_size,
            current_sample_size: 0,
            start_date: Utc::now(),
            end_date: None,
            is_active: true,
        };

        self.ab_tests.push(test_config);
        test_id
    }

    /// Get parameters for A/B test variant
    pub fn get_ab_test_parameters(
        &self,
        test_id: Uuid,
        course_id: Uuid,
    ) -> Option<(ABTestVariant, ClusteringPreferences)> {
        let test = self
            .ab_tests
            .iter()
            .find(|t| t.id == test_id && t.is_active)?;

        if test.current_sample_size >= test.target_sample_size {
            return None; // Test completed
        }

        // Simple hash-based assignment for consistent variant selection
        let hash = course_id.as_u128() % 2;
        if hash == 0 {
            Some((ABTestVariant::VariantA, test.parameters_a.clone()))
        } else {
            Some((ABTestVariant::VariantB, test.parameters_b.clone()))
        }
    }

    /// Record A/B test result
    pub fn record_ab_test_result(&mut self, result: ABTestResult) -> Result<()> {
        // Find the test and increment sample size
        if let Some(test) = self.ab_tests.iter_mut().find(|t| t.id == result.test_id) {
            test.current_sample_size += 1;

            // Check if test is complete
            if test.current_sample_size >= test.target_sample_size {
                test.is_active = false;
                test.end_date = Some(Utc::now());
            }
        }

        self.ab_results.push(result);
        Ok(())
    }

    /// Analyze A/B test results and update preferences
    pub fn analyze_ab_test_results(&mut self, test_id: Uuid) -> Result<ABTestAnalysis> {
        let results: Vec<&ABTestResult> = self
            .ab_results
            .iter()
            .filter(|r| r.test_id == test_id)
            .collect();

        if results.is_empty() {
            return Err(anyhow::anyhow!("No results found for test {}", test_id));
        }

        let variant_a_results: Vec<&ABTestResult> = results
            .iter()
            .filter(|r| matches!(r.variant, ABTestVariant::VariantA))
            .cloned()
            .collect();

        let variant_b_results: Vec<&ABTestResult> = results
            .iter()
            .filter(|r| matches!(r.variant, ABTestVariant::VariantB))
            .cloned()
            .collect();

        let analysis = ABTestAnalysis {
            test_id,
            variant_a_satisfaction: variant_a_results
                .iter()
                .map(|r| r.user_satisfaction)
                .sum::<f32>()
                / variant_a_results.len() as f32,
            variant_b_satisfaction: variant_b_results
                .iter()
                .map(|r| r.user_satisfaction)
                .sum::<f32>()
                / variant_b_results.len() as f32,
            variant_a_quality: variant_a_results
                .iter()
                .map(|r| r.quality_score)
                .sum::<f32>()
                / variant_a_results.len() as f32,
            variant_b_quality: variant_b_results
                .iter()
                .map(|r| r.quality_score)
                .sum::<f32>()
                / variant_b_results.len() as f32,
            variant_a_adjustments: variant_a_results
                .iter()
                .map(|r| r.adjustment_count as f32)
                .sum::<f32>()
                / variant_a_results.len() as f32,
            variant_b_adjustments: variant_b_results
                .iter()
                .map(|r| r.adjustment_count as f32)
                .sum::<f32>()
                / variant_b_results.len() as f32,
            sample_size_a: variant_a_results.len(),
            sample_size_b: variant_b_results.len(),
            statistical_significance: self
                .calculate_statistical_significance(&variant_a_results, &variant_b_results),
            winner: self.determine_winner(&variant_a_results, &variant_b_results),
        };

        // Update preferences based on winning variant
        if let Some(test) = self.ab_tests.iter().find(|t| t.id == test_id) {
            match analysis.winner {
                Some(ABTestVariant::VariantA) => {
                    self.preferences = test.parameters_a.clone();
                }
                Some(ABTestVariant::VariantB) => {
                    self.preferences = test.parameters_b.clone();
                }
                None => {} // No clear winner, keep current preferences
            }
        }

        Ok(analysis)
    }

    /// Calculate statistical significance of A/B test results
    fn calculate_statistical_significance(
        &self,
        variant_a: &[&ABTestResult],
        variant_b: &[&ABTestResult],
    ) -> f32 {
        // Simplified statistical significance calculation
        // In a real implementation, you'd use proper statistical tests
        let min_sample_size = 30;
        if variant_a.len() < min_sample_size || variant_b.len() < min_sample_size {
            return 0.0; // Not enough data
        }

        let mean_a =
            variant_a.iter().map(|r| r.user_satisfaction).sum::<f32>() / variant_a.len() as f32;
        let mean_b =
            variant_b.iter().map(|r| r.user_satisfaction).sum::<f32>() / variant_b.len() as f32;

        let diff = (mean_a - mean_b).abs();

        // Simple heuristic: larger differences with larger sample sizes = higher significance
        let combined_sample_size = variant_a.len() + variant_b.len();
        (diff * combined_sample_size as f32 / 100.0).min(1.0)
    }

    /// Determine the winning variant
    fn determine_winner(
        &self,
        variant_a: &[&ABTestResult],
        variant_b: &[&ABTestResult],
    ) -> Option<ABTestVariant> {
        if variant_a.is_empty() || variant_b.is_empty() {
            return None;
        }

        let satisfaction_a =
            variant_a.iter().map(|r| r.user_satisfaction).sum::<f32>() / variant_a.len() as f32;
        let satisfaction_b =
            variant_b.iter().map(|r| r.user_satisfaction).sum::<f32>() / variant_b.len() as f32;

        let quality_a =
            variant_a.iter().map(|r| r.quality_score).sum::<f32>() / variant_a.len() as f32;
        let quality_b =
            variant_b.iter().map(|r| r.quality_score).sum::<f32>() / variant_b.len() as f32;

        // Combined score: 70% satisfaction, 30% quality
        let score_a = satisfaction_a * 0.7 + quality_a * 0.3;
        let score_b = satisfaction_b * 0.7 + quality_b * 0.3;

        let diff = (score_a - score_b).abs();
        if diff < 0.05 {
            None // Too close to call
        } else if score_a > score_b {
            Some(ABTestVariant::VariantA)
        } else {
            Some(ABTestVariant::VariantB)
        }
    }

    /// Get active A/B tests
    pub fn get_active_ab_tests(&self) -> Vec<&ABTestConfig> {
        self.ab_tests.iter().filter(|t| t.is_active).collect()
    }

    /// Get feedback history
    pub fn get_feedback_history(&self) -> &[ClusteringFeedback] {
        &self.feedback_history
    }
}

/// Analysis results from an A/B test
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ABTestAnalysis {
    pub test_id: Uuid,
    pub variant_a_satisfaction: f32,
    pub variant_b_satisfaction: f32,
    pub variant_a_quality: f32,
    pub variant_b_quality: f32,
    pub variant_a_adjustments: f32,
    pub variant_b_adjustments: f32,
    pub sample_size_a: usize,
    pub sample_size_b: usize,
    pub statistical_significance: f32,
    pub winner: Option<ABTestVariant>,
}

impl Default for PreferenceLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_learning_creation() {
        let engine = PreferenceLearningEngine::new();
        assert_eq!(engine.get_preferences().similarity_threshold, 0.6);
        assert_eq!(
            engine.get_preferences().preferred_algorithm,
            ClusteringAlgorithm::Hybrid
        );
    }

    #[test]
    fn test_feedback_processing() {
        let mut engine = PreferenceLearningEngine::new();
        let initial_threshold = engine.get_preferences().similarity_threshold;

        let feedback = ClusteringFeedback {
            id: Uuid::new_v4(),
            course_id: Uuid::new_v4(),
            clustering_parameters: ClusteringPreferences {
                similarity_threshold: 0.8,
                ..ClusteringPreferences::default()
            },
            feedback_type: FeedbackType::ExplicitRating,
            rating: 0.9,
            comments: None,
            manual_adjustments: Vec::new(),
            created_at: Utc::now(),
        };

        engine.update_preferences_from_feedback(feedback).unwrap();

        // Threshold should move towards the highly-rated parameter
        assert!(engine.get_preferences().similarity_threshold > initial_threshold);
    }

    #[test]
    fn test_ab_test_creation() {
        let mut engine = PreferenceLearningEngine::new();
        let test_id = engine.create_ab_test(
            "TF-IDF vs K-Means".to_string(),
            "Compare TF-IDF and K-Means algorithms".to_string(),
            ClusteringAlgorithm::TfIdf,
            ClusteringAlgorithm::KMeans,
            100,
        );

        assert_eq!(engine.get_active_ab_tests().len(), 1);
        assert_eq!(engine.get_active_ab_tests()[0].id, test_id);
    }

    #[test]
    fn test_parameter_recommendation() {
        let engine = PreferenceLearningEngine::new();

        // Small course should get fewer clusters
        let small_course_params = engine.get_recommended_parameters(5, DifficultyLevel::Beginner);
        assert!(small_course_params.max_clusters <= 3);

        // Large course should get more clusters
        let large_course_params = engine.get_recommended_parameters(100, DifficultyLevel::Expert);
        assert!(large_course_params.max_clusters > 5);
    }
}
