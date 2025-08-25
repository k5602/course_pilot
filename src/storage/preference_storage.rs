//! Storage layer for clustering preferences and feedback data
//!
//! This module handles persistence of user preferences, feedback history,
//! and A/B testing data for the clustering preference learning system.

#[cfg(feature = "advanced_nlp")]
use crate::nlp::clustering::{
    ABTestConfig, ABTestResult, ClusteringFeedback, ClusteringPreferences, PreferenceLearningEngine,
};

#[cfg(not(feature = "advanced_nlp"))]
mod nlp_store_fallback {
    use super::*;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    pub struct ClusteringPreferences {}

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum ABTestVariant {
        A,
        B,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ABTestConfig {
        pub id: Uuid,
        pub name: String,
        pub description: Option<String>,
        pub created_at: Option<DateTime<Utc>>,
        pub sample_size: Option<i64>,
        pub is_active: Option<bool>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ClusteringFeedback {
        pub id: Uuid,
        pub course_id: Uuid,
        pub clustering_parameters: ClusteringPreferences,
        pub feedback_type: String,
        pub rating: f32,
        pub comments: Option<String>,
        pub manual_adjustments: Vec<String>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
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
        pub created_at: DateTime<Utc>,
    }

    pub struct PreferenceLearningEngine;

    impl PreferenceLearningEngine {
        pub fn with_preferences(_p: ClusteringPreferences) -> Self {
            Self
        }
        pub fn update_preferences_from_feedback(
            &mut self,
            _feedback: ClusteringFeedback,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }
}

use crate::storage::Database;
use anyhow::Result;
#[cfg(not(feature = "advanced_nlp"))]
use nlp_store_fallback::{
    ABTestConfig, ABTestResult, ABTestVariant, ClusteringFeedback, ClusteringPreferences,
    PreferenceLearningEngine,
};
use rusqlite::params;
use uuid::Uuid;

/// Storage manager for clustering preferences and feedback
pub struct PreferenceStorage {
    db: Database,
}

impl PreferenceStorage {
    /// Create a new preference storage instance
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Initialize the preference storage database tables
    pub fn initialize(&self) -> Result<()> {
        let conn = self.db.get_conn()?;

        // Create clustering_preferences table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clustering_preferences (
                id INTEGER PRIMARY KEY,
                similarity_threshold REAL NOT NULL,
                preferred_algorithm TEXT NOT NULL,
                preferred_strategy TEXT NOT NULL,
                user_experience_level TEXT NOT NULL,
                max_clusters INTEGER NOT NULL,
                min_cluster_size INTEGER NOT NULL,
                enable_duration_balancing BOOLEAN NOT NULL,
                content_vs_duration_weight REAL NOT NULL,
                last_updated TEXT NOT NULL,
                usage_count INTEGER NOT NULL,
                satisfaction_score REAL NOT NULL
            )",
            [],
        )?;

        // Create clustering_feedback table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clustering_feedback (
                id TEXT PRIMARY KEY,
                course_id TEXT NOT NULL,
                clustering_parameters TEXT NOT NULL,
                feedback_type TEXT NOT NULL,
                rating REAL NOT NULL,
                comments TEXT,
                manual_adjustments TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create ab_test_configs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ab_test_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                algorithm_a TEXT NOT NULL,
                algorithm_b TEXT NOT NULL,
                parameters_a TEXT NOT NULL,
                parameters_b TEXT NOT NULL,
                target_sample_size INTEGER NOT NULL,
                current_sample_size INTEGER NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT,
                is_active BOOLEAN NOT NULL
            )",
            [],
        )?;

        // Create ab_test_results table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ab_test_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                test_id TEXT NOT NULL,
                course_id TEXT NOT NULL,
                variant TEXT NOT NULL,
                parameters_used TEXT NOT NULL,
                user_satisfaction REAL NOT NULL,
                processing_time_ms INTEGER NOT NULL,
                quality_score REAL NOT NULL,
                user_made_adjustments BOOLEAN NOT NULL,
                adjustment_count INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (test_id) REFERENCES ab_test_configs (id)
            )",
            [],
        )?;

        // Create performance indexes for preference storage
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clustering_feedback_course_id ON clustering_feedback(course_id);",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clustering_feedback_created_at ON clustering_feedback(created_at);",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ab_test_configs_active ON ab_test_configs(is_active);",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ab_test_results_test_id ON ab_test_results(test_id);",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ab_test_results_timestamp ON ab_test_results(timestamp);",
            [],
        )?;

        Ok(())
    }

    /// Save clustering preferences to database
    pub fn save_preferences(&self, preferences: &ClusteringPreferences) -> Result<()> {
        let conn = self.db.get_conn()?;

        // Delete existing preferences (we only store one set)
        conn.execute("DELETE FROM clustering_preferences", [])?;

        // Insert new preferences
        conn.execute(
            "INSERT INTO clustering_preferences (
                similarity_threshold, preferred_algorithm, preferred_strategy,
                user_experience_level, max_clusters, min_cluster_size,
                enable_duration_balancing, content_vs_duration_weight,
                last_updated, usage_count, satisfaction_score
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                preferences.similarity_threshold,
                serde_json::to_string(&preferences.preferred_algorithm)?,
                serde_json::to_string(&preferences.preferred_strategy)?,
                serde_json::to_string(&preferences.user_experience_level)?,
                preferences.max_clusters,
                preferences.min_cluster_size,
                preferences.enable_duration_balancing,
                preferences.content_vs_duration_weight,
                preferences.last_updated.to_rfc3339(),
                preferences.usage_count,
                preferences.satisfaction_score,
            ],
        )?;

        Ok(())
    }

    /// Load clustering preferences from database
    pub fn load_preferences(&self) -> Result<Option<ClusteringPreferences>> {
        let conn = self.db.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT similarity_threshold, preferred_algorithm, preferred_strategy,
                    user_experience_level, max_clusters, min_cluster_size,
                    enable_duration_balancing, content_vs_duration_weight,
                    last_updated, usage_count, satisfaction_score
             FROM clustering_preferences
             LIMIT 1",
        )?;

        let preferences = stmt.query_row([], |row| {
            Ok(ClusteringPreferences {
                similarity_threshold: row.get(0)?,
                preferred_algorithm: serde_json::from_str(&row.get::<_, String>(1)?).map_err(
                    |_e| {
                        rusqlite::Error::InvalidColumnType(
                            1,
                            "preferred_algorithm".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    },
                )?,
                preferred_strategy: serde_json::from_str(&row.get::<_, String>(2)?).map_err(
                    |_e| {
                        rusqlite::Error::InvalidColumnType(
                            2,
                            "preferred_strategy".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    },
                )?,
                user_experience_level: serde_json::from_str(&row.get::<_, String>(3)?).map_err(
                    |_e| {
                        rusqlite::Error::InvalidColumnType(
                            3,
                            "user_experience_level".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    },
                )?,
                max_clusters: row.get(4)?,
                min_cluster_size: row.get(5)?,
                enable_duration_balancing: row.get(6)?,
                content_vs_duration_weight: row.get(7)?,
                last_updated: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map_err(|_e| {
                        rusqlite::Error::InvalidColumnType(
                            8,
                            "last_updated".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&chrono::Utc),
                usage_count: row.get(9)?,
                satisfaction_score: row.get(10)?,
            })
        });

        match preferences {
            Ok(prefs) => Ok(Some(prefs)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Save clustering feedback to database
    pub fn save_feedback(&self, feedback: &ClusteringFeedback) -> Result<()> {
        let conn = self.db.get_conn()?;

        conn.execute(
            "INSERT INTO clustering_feedback (
                id, course_id, clustering_parameters, feedback_type,
                rating, comments, manual_adjustments, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                feedback.id.to_string(),
                feedback.course_id.to_string(),
                serde_json::to_string(&feedback.clustering_parameters)?,
                serde_json::to_string(&feedback.feedback_type)?,
                feedback.rating,
                feedback.comments,
                serde_json::to_string(&feedback.manual_adjustments)?,
                feedback.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// Load all clustering feedback from database
    pub fn load_feedback_history(&self) -> Result<Vec<ClusteringFeedback>> {
        let conn = self.db.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, course_id, clustering_parameters, feedback_type,
                    rating, comments, manual_adjustments, created_at
             FROM clustering_feedback
             ORDER BY created_at DESC",
        )?;

        let feedback_iter = stmt.query_map([], |row| {
            Ok(ClusteringFeedback {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "id".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                course_id: Uuid::parse_str(&row.get::<_, String>(1)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        1,
                        "course_id".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                clustering_parameters: serde_json::from_str(&row.get::<_, String>(2)?).map_err(
                    |_e| {
                        rusqlite::Error::InvalidColumnType(
                            2,
                            "clustering_parameters".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    },
                )?,
                feedback_type: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        3,
                        "feedback_type".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                rating: row.get(4)?,
                comments: row.get(5)?,
                manual_adjustments: serde_json::from_str(&row.get::<_, String>(6)?).map_err(
                    |_e| {
                        rusqlite::Error::InvalidColumnType(
                            6,
                            "manual_adjustments".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    },
                )?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .map_err(|_e| {
                        rusqlite::Error::InvalidColumnType(
                            7,
                            "created_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&chrono::Utc),
            })
        })?;

        let mut feedback_list = Vec::new();
        for feedback in feedback_iter {
            feedback_list.push(feedback?);
        }

        Ok(feedback_list)
    }

    /// Save A/B test configuration
    pub fn save_ab_test_config(&self, config: &ABTestConfig) -> Result<()> {
        let conn = self.db.get_conn()?;

        conn.execute(
            "INSERT OR REPLACE INTO ab_test_configs (
                id, name, description, algorithm_a, algorithm_b,
                parameters_a, parameters_b, target_sample_size,
                current_sample_size, start_date, end_date, is_active
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                config.id.to_string(),
                config.name,
                config.description,
                serde_json::to_string(&config.algorithm_a)?,
                serde_json::to_string(&config.algorithm_b)?,
                serde_json::to_string(&config.parameters_a)?,
                serde_json::to_string(&config.parameters_b)?,
                config.target_sample_size,
                config.current_sample_size,
                config.start_date.to_rfc3339(),
                config.end_date.map(|d| d.to_rfc3339()),
                config.is_active,
            ],
        )?;

        Ok(())
    }

    /// Load all A/B test configurations
    pub fn load_ab_test_configs(&self) -> Result<Vec<ABTestConfig>> {
        let conn = self.db.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, algorithm_a, algorithm_b,
                    parameters_a, parameters_b, target_sample_size,
                    current_sample_size, start_date, end_date, is_active
             FROM ab_test_configs
             ORDER BY start_date DESC",
        )?;

        let config_iter = stmt.query_map([], |row| {
            Ok(ABTestConfig {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "id".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                name: row.get(1)?,
                description: row.get(2)?,
                algorithm_a: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        3,
                        "algorithm_a".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                algorithm_b: serde_json::from_str(&row.get::<_, String>(4)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        4,
                        "algorithm_b".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                parameters_a: serde_json::from_str(&row.get::<_, String>(5)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        5,
                        "parameters_a".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                parameters_b: serde_json::from_str(&row.get::<_, String>(6)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        6,
                        "parameters_b".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                target_sample_size: row.get(7)?,
                current_sample_size: row.get(8)?,
                start_date: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map_err(|_e| {
                        rusqlite::Error::InvalidColumnType(
                            9,
                            "start_date".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&chrono::Utc),
                end_date: row
                    .get::<_, Option<String>>(10)?
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s))
                    .transpose()
                    .map_err(|_e| {
                        rusqlite::Error::InvalidColumnType(
                            10,
                            "end_date".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                is_active: row.get(11)?,
            })
        })?;

        let mut configs = Vec::new();
        for config in config_iter {
            configs.push(config?);
        }

        Ok(configs)
    }

    /// Save A/B test result
    pub fn save_ab_test_result(&self, result: &ABTestResult) -> Result<()> {
        let conn = self.db.get_conn()?;

        conn.execute(
            "INSERT INTO ab_test_results (
                test_id, course_id, variant, parameters_used,
                user_satisfaction, processing_time_ms, quality_score,
                user_made_adjustments, adjustment_count, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                result.test_id.to_string(),
                result.course_id.to_string(),
                serde_json::to_string(&result.variant)?,
                serde_json::to_string(&result.parameters_used)?,
                result.user_satisfaction,
                result.processing_time_ms,
                result.quality_score,
                result.user_made_adjustments,
                result.adjustment_count,
                result.timestamp.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// Load A/B test results for a specific test
    pub fn load_ab_test_results(&self, test_id: Uuid) -> Result<Vec<ABTestResult>> {
        let conn = self.db.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT test_id, course_id, variant, parameters_used,
                    user_satisfaction, processing_time_ms, quality_score,
                    user_made_adjustments, adjustment_count, timestamp
             FROM ab_test_results
             WHERE test_id = ?1
             ORDER BY timestamp DESC",
        )?;

        let result_iter = stmt.query_map([test_id.to_string()], |row| {
            Ok(ABTestResult {
                test_id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "test_id".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                course_id: Uuid::parse_str(&row.get::<_, String>(1)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        1,
                        "course_id".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                variant: serde_json::from_str(&row.get::<_, String>(2)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        2,
                        "variant".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                parameters_used: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|_e| {
                    rusqlite::Error::InvalidColumnType(
                        3,
                        "parameters_used".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                user_satisfaction: row.get(4)?,
                processing_time_ms: row.get(5)?,
                quality_score: row.get(6)?,
                user_made_adjustments: row.get(7)?,
                adjustment_count: row.get(8)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map_err(|_e| {
                        rusqlite::Error::InvalidColumnType(
                            9,
                            "timestamp".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&chrono::Utc),
            })
        })?;

        let mut results = Vec::new();
        for result in result_iter {
            results.push(result?);
        }

        Ok(results)
    }

    /// Create a preference learning engine with stored data
    pub fn create_preference_engine(&self) -> Result<PreferenceLearningEngine> {
        // Load preferences from storage or use defaults
        let preferences = self.load_preferences()?.unwrap_or_default();

        // Create engine with loaded preferences
        let mut engine = PreferenceLearningEngine::with_preferences(preferences);

        // Load feedback history
        let feedback_history = self.load_feedback_history()?;
        for feedback in feedback_history {
            engine.update_preferences_from_feedback(feedback)?;
        }

        // Load A/B test configurations
        let ab_configs = self.load_ab_test_configs()?;
        for config in ab_configs {
            // Note: This is a simplified approach. In a real implementation,
            // you'd want to properly restore the engine state including A/B tests
            if config.is_active {
                engine.create_ab_test(
                    config.name,
                    config.description,
                    config.algorithm_a,
                    config.algorithm_b,
                    config.target_sample_size,
                );
            }
        }

        Ok(engine)
    }

    /// Update A/B test sample size
    pub fn update_ab_test_sample_size(&self, test_id: Uuid, new_size: usize) -> Result<()> {
        let conn = self.db.get_conn()?;

        conn.execute(
            "UPDATE ab_test_configs SET current_sample_size = ?1 WHERE id = ?2",
            params![new_size, test_id.to_string()],
        )?;

        Ok(())
    }

    /// Mark A/B test as completed
    pub fn complete_ab_test(&self, test_id: Uuid) -> Result<()> {
        let conn = self.db.get_conn()?;

        conn.execute(
            "UPDATE ab_test_configs SET is_active = FALSE, end_date = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), test_id.to_string()],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_preference_storage_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = crate::storage::Database::new(&db_path).unwrap();
        let storage = PreferenceStorage::new(db);

        storage.initialize().unwrap();

        // Test that we can create an engine
        let engine = storage.create_preference_engine().unwrap();
        assert_eq!(engine.get_preferences().similarity_threshold, 0.6);
    }

    #[test]
    fn test_preferences_save_load() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = crate::storage::Database::new(&db_path).unwrap();
        let storage = PreferenceStorage::new(db);
        storage.initialize().unwrap();

        let mut preferences = ClusteringPreferences::default();
        preferences.similarity_threshold = 0.8;
        preferences.max_clusters = 10;

        storage.save_preferences(&preferences).unwrap();
        let loaded = storage.load_preferences().unwrap().unwrap();

        assert_eq!(loaded.similarity_threshold, 0.8);
        assert_eq!(loaded.max_clusters, 10);
    }
}
