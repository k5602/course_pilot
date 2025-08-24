//! Preference learning service integration
//!
//! This module provides a service layer that integrates the preference learning
//! system with the existing clustering algorithms and NLP processor.

use crate::nlp::clustering::{
    ABTestConfig, ABTestResult, ABTestVariant, ClusteringFeedback, ClusteringPreferences,
    FeedbackType, ManualAdjustment, PreferenceLearningEngine,
};

use crate::storage::{AppSettings, PreferenceStorage};
use crate::types::{Course, DifficultyLevel};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Service for managing clustering preferences and learning
pub struct PreferenceService {
    engine: Arc<Mutex<PreferenceLearningEngine>>,
    storage: PreferenceStorage,
    settings: AppSettings,
}

impl PreferenceService {
    /// Create a new preference service
    pub fn new(db: crate::storage::Database, settings: AppSettings) -> Result<Self> {
        let storage = PreferenceStorage::new(db);
        storage.initialize()?;

        let engine = Arc::new(Mutex::new(storage.create_preference_engine()?));

        Ok(Self {
            engine,
            storage,
            settings,
        })
    }

    /// Get current clustering preferences
    pub fn get_preferences(&self) -> Result<ClusteringPreferences> {
        let engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(engine.get_preferences().clone())
    }

    /// Get recommended parameters for a specific course
    pub fn get_recommended_parameters(&self, course: &Course) -> Result<ClusteringPreferences> {
        let engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        // Estimate course complexity based on video count and titles
        let complexity = self.estimate_course_complexity(course);
        let params = engine.get_recommended_parameters(course.video_count(), complexity);

        Ok(params)
    }

    /// Submit user feedback and update preferences
    pub fn submit_feedback(&self, feedback: ClusteringFeedback) -> Result<()> {
        // Save feedback to storage
        self.storage.save_feedback(&feedback)?;

        // Update engine with feedback
        let mut engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        engine.update_preferences_from_feedback(feedback)?;

        // Save updated preferences
        self.storage.save_preferences(engine.get_preferences())?;

        Ok(())
    }

    /// Record manual adjustment and learn from it
    pub fn record_manual_adjustment(
        &self,
        course_id: Uuid,
        adjustment: ManualAdjustment,
    ) -> Result<()> {
        let current_preferences = self.get_preferences()?;

        let feedback = ClusteringFeedback {
            id: Uuid::new_v4(),
            course_id,
            clustering_parameters: current_preferences,
            feedback_type: FeedbackType::ManualAdjustment,
            rating: 0.3, // Manual adjustments indicate dissatisfaction
            comments: adjustment.reason.clone(),
            manual_adjustments: vec![adjustment],
            created_at: chrono::Utc::now(),
        };

        self.submit_feedback(feedback)
    }

    /// Create a new A/B test
    pub fn create_ab_test(
        &self,
        name: String,
        description: String,
        algorithm_a: crate::types::ClusteringAlgorithm,
        algorithm_b: crate::types::ClusteringAlgorithm,
        target_sample_size: usize,
    ) -> Result<Uuid> {
        if !self.settings.enable_ab_testing {
            return Err(anyhow::anyhow!("A/B testing is disabled in settings"));
        }

        let mut engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let test_id = engine.create_ab_test(
            name,
            description,
            algorithm_a,
            algorithm_b,
            target_sample_size,
        );

        // Save test configuration to storage
        if let Some(test_config) = engine
            .get_active_ab_tests()
            .iter()
            .find(|t| t.id == test_id)
        {
            self.storage.save_ab_test_config(test_config)?;
        }

        Ok(test_id)
    }

    /// Get parameters for A/B test (if user is part of active test)
    pub fn get_ab_test_parameters(
        &self,
        course_id: Uuid,
    ) -> Result<Option<(ABTestVariant, ClusteringPreferences)>> {
        if !self.settings.enable_ab_testing {
            return Ok(None);
        }

        let engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let active_tests = engine.get_active_ab_tests();

        // For simplicity, use the first active test
        if let Some(test) = active_tests.first() {
            return Ok(engine.get_ab_test_parameters(test.id, course_id));
        }

        Ok(None)
    }

    /// Record A/B test result
    pub fn record_ab_test_result(&self, result: ABTestResult) -> Result<()> {
        // Save result to storage
        self.storage.save_ab_test_result(&result)?;

        // Update engine
        let mut engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let _ = engine.record_ab_test_result(result.clone());

        // Update test sample size in storage
        self.storage.update_ab_test_sample_size(
            result.test_id,
            engine
                .get_active_ab_tests()
                .iter()
                .find(|t| t.id == result.test_id)
                .map(|t| t.current_sample_size)
                .unwrap_or(0),
        )?;

        Ok(())
    }

    /// Analyze A/B test results
    pub fn analyze_ab_test(&self, test_id: Uuid) -> Result<crate::nlp::clustering::ABTestAnalysis> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let analysis = engine.analyze_ab_test_results(test_id)?;

        // Mark test as completed if it has enough data
        if analysis.sample_size_a + analysis.sample_size_b >= 30 {
            self.storage.complete_ab_test(test_id)?;
        }

        // Save updated preferences
        self.storage.save_preferences(&engine.get_preferences())?;

        Ok(analysis)
    }

    /// Get clustering parameters with preference learning applied
    pub fn get_optimized_parameters(&self, course: &Course) -> Result<ClusteringPreferences> {
        // Check if user is part of an A/B test
        if let Some((variant, test_params)) = self.get_ab_test_parameters(course.id)? {
            log::info!(
                "Using A/B test parameters for course {}: {:?}",
                course.id,
                variant
            );
            return Ok(test_params);
        }

        // Use learned preferences
        self.get_recommended_parameters(course)
    }

    /// Record clustering success/failure for learning
    pub fn record_clustering_outcome(
        &self,
        course_id: Uuid,
        parameters_used: ClusteringPreferences,
        success: bool,
        quality_score: f32,
        processing_time_ms: u64,
        user_made_adjustments: bool,
        adjustment_count: usize,
    ) -> Result<()> {
        let rating = if success {
            if user_made_adjustments {
                0.6 // Success but needed adjustments
            } else {
                0.9 // Success without adjustments
            }
        } else {
            0.2 // Failure
        };

        let feedback = ClusteringFeedback {
            id: Uuid::new_v4(),
            course_id,
            clustering_parameters: parameters_used.clone(),
            feedback_type: if user_made_adjustments {
                FeedbackType::ManualAdjustment
            } else if success {
                FeedbackType::ImplicitAcceptance
            } else {
                FeedbackType::Rejection
            },
            rating,
            comments: None,
            manual_adjustments: Vec::new(),
            created_at: chrono::Utc::now(),
        };

        self.submit_feedback(feedback)?;

        // If this was part of an A/B test, record the result
        if let Some((variant, _)) = self.get_ab_test_parameters(course_id)? {
            let engine = self
                .engine
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            if let Some(test) = engine.get_active_ab_tests().first() {
                let ab_result = ABTestResult {
                    test_id: test.id,
                    course_id,
                    variant,
                    parameters_used,
                    user_satisfaction: rating,
                    processing_time_ms,
                    quality_score,
                    user_made_adjustments,
                    adjustment_count,
                    timestamp: chrono::Utc::now(),
                };

                drop(engine); // Release lock before calling record_ab_test_result
                self.record_ab_test_result(ab_result)?;
            }
        }

        Ok(())
    }

    /// Get feedback history for analysis
    pub fn get_feedback_history(&self) -> Result<Vec<ClusteringFeedback>> {
        self.storage.load_feedback_history()
    }

    /// Get active A/B tests
    pub fn get_active_ab_tests(&self) -> Result<Vec<ABTestConfig>> {
        let engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(engine
            .get_active_ab_tests()
            .into_iter()
            .map(|c| (*c).clone())
            .collect())
    }

    /// Update settings and refresh engine
    pub fn update_settings(&mut self, settings: AppSettings) -> Result<()> {
        self.settings = settings;

        // Update clustering preferences in engine if they changed
        let mut engine = self
            .engine
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let current_prefs = engine.get_preferences().clone();

        // If settings have different clustering preferences, update them
        if current_prefs != self.settings.clustering_preferences {
            let feedback = ClusteringFeedback {
                id: Uuid::new_v4(),
                course_id: Uuid::new_v4(), // Dummy course ID for settings change
                clustering_parameters: self.settings.clustering_preferences.clone(),
                feedback_type: FeedbackType::ParameterChange,
                rating: 0.7, // Neutral rating for settings change
                comments: Some("User updated settings".to_string()),
                manual_adjustments: Vec::new(),
                created_at: chrono::Utc::now(),
            };

            engine.update_preferences_from_feedback(feedback)?;
            self.storage.save_preferences(&engine.get_preferences())?;
        }

        Ok(())
    }

    /// Estimate course complexity based on content analysis
    fn estimate_course_complexity(&self, course: &Course) -> DifficultyLevel {
        let titles = &course.raw_titles;

        if titles.is_empty() {
            return DifficultyLevel::Intermediate;
        }

        let mut complexity_score = 0.0;
        let total_titles = titles.len() as f32;

        // Analyze title complexity
        for title in titles {
            let title_lower = title.to_lowercase();

            // Beginner indicators
            if title_lower.contains("introduction")
                || title_lower.contains("basics")
                || title_lower.contains("getting started")
                || title_lower.contains("beginner")
            {
                complexity_score -= 1.0;
            }

            // Advanced indicators
            if title_lower.contains("advanced")
                || title_lower.contains("expert")
                || title_lower.contains("deep dive")
                || title_lower.contains("architecture")
            {
                complexity_score += 2.0;
            }

            // Intermediate indicators
            if title_lower.contains("implementation")
                || title_lower.contains("practical")
                || title_lower.contains("hands-on")
            {
                complexity_score += 0.5;
            }
        }

        // Normalize score
        let normalized_score = complexity_score / total_titles;

        // Course length also affects complexity
        let length_factor = if titles.len() > 50 {
            0.5 // Longer courses tend to be more comprehensive
        } else if titles.len() < 10 {
            -0.3 // Shorter courses tend to be introductory
        } else {
            0.0
        };

        let final_score = normalized_score + length_factor;

        match final_score {
            s if s < -0.5 => DifficultyLevel::Beginner,
            s if s > 1.0 => DifficultyLevel::Expert,
            s if s > 0.3 => DifficultyLevel::Advanced,
            _ => DifficultyLevel::Intermediate,
        }
    }

    /// Check if preference learning is enabled
    pub fn is_preference_learning_enabled(&self) -> bool {
        self.settings.enable_preference_learning
    }

    /// Check if A/B testing is enabled
    pub fn is_ab_testing_enabled(&self) -> bool {
        self.settings.enable_ab_testing
    }
}

/// Auto-tuning service that runs in the background
pub struct AutoTuningService {
    preference_service: Arc<PreferenceService>,
}

impl AutoTuningService {
    /// Create a new auto-tuning service
    pub fn new(preference_service: Arc<PreferenceService>) -> Self {
        Self { preference_service }
    }

    /// Run auto-tuning analysis and update preferences
    pub async fn run_auto_tuning(&self) -> Result<()> {
        if !self.preference_service.is_preference_learning_enabled() {
            return Ok(());
        }

        log::info!("Running auto-tuning analysis...");

        // Analyze recent feedback
        let feedback_history = self.preference_service.get_feedback_history()?;
        let recent_feedback: Vec<_> = feedback_history
            .into_iter()
            .filter(|f| {
                let days_ago = chrono::Utc::now() - f.created_at;
                days_ago.num_days() <= 30 // Last 30 days
            })
            .collect();

        if recent_feedback.len() < 5 {
            log::info!("Not enough recent feedback for auto-tuning");
            return Ok(());
        }

        // Calculate average satisfaction
        let avg_satisfaction: f32 =
            recent_feedback.iter().map(|f| f.rating).sum::<f32>() / recent_feedback.len() as f32;

        log::info!("Average satisfaction over last 30 days: {avg_satisfaction:.2}");

        // If satisfaction is low, suggest parameter adjustments
        if avg_satisfaction < 0.6 {
            log::info!("Low satisfaction detected, analyzing for improvements...");

            // Analyze common issues
            let manual_adjustments: Vec<_> = recent_feedback
                .iter()
                .filter(|f| matches!(f.feedback_type, FeedbackType::ManualAdjustment))
                .collect();

            if manual_adjustments.len() > recent_feedback.len() / 2 {
                log::info!("Many manual adjustments detected, suggesting parameter changes");
                // In a real implementation, this would trigger specific parameter adjustments
            }
        }

        // Check A/B test results
        for test_config in self.preference_service.get_active_ab_tests()? {
            if test_config.current_sample_size >= test_config.target_sample_size {
                log::info!("A/B test {} completed, analyzing results", test_config.name);
                let _analysis = self.preference_service.analyze_ab_test(test_config.id)?;
                // Results are automatically applied by analyze_ab_test
            }
        }

        Ok(())
    }

    /// Schedule periodic auto-tuning
    pub async fn start_periodic_tuning(&self, interval_hours: u64) -> Result<()> {
        let interval = std::time::Duration::from_secs(interval_hours * 3600);

        loop {
            if let Err(e) = self.run_auto_tuning().await {
                log::error!("Auto-tuning failed: {e}");
            }

            tokio::time::sleep(interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_preference_service_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = crate::storage::Database::new(&db_path).unwrap();
        let settings = AppSettings::default();

        let service = PreferenceService::new(db, settings).unwrap();
        let preferences = service.get_preferences().unwrap();

        assert_eq!(preferences.similarity_threshold, 0.6);
    }

    #[test]
    fn test_course_complexity_estimation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = crate::storage::Database::new(&db_path).unwrap();
        let settings = AppSettings::default();
        let service = PreferenceService::new(db, settings).unwrap();

        // Beginner course
        let beginner_course = Course::new(
            "Beginner Course".to_string(),
            vec![
                "Introduction to Programming".to_string(),
                "Getting Started with Basics".to_string(),
                "Beginner Tutorial".to_string(),
            ],
        );
        assert_eq!(
            service.estimate_course_complexity(&beginner_course),
            DifficultyLevel::Beginner
        );

        // Advanced course
        let advanced_course = Course::new(
            "Advanced Course".to_string(),
            vec![
                "Advanced Architecture Patterns".to_string(),
                "Expert-Level Deep Dive".to_string(),
                "Advanced Implementation Techniques".to_string(),
            ],
        );
        assert_eq!(
            service.estimate_course_complexity(&advanced_course),
            DifficultyLevel::Expert
        );
    }
}
