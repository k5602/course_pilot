//! Session Grouper - Simple NLP contract for Course Pilot
//!
//! This module provides a clean interface for grouping video titles into sessions
//! without complex structuring or heavy dependencies. The goal is to keep NLP
//! focused on simple grouping operations that other modules can build upon.

use anyhow::Result;
use log::{debug, info};

/// Simple session grouping trait for NLP operations
///
/// This trait provides a clean contract for grouping video titles into sessions.
/// Implementations should be lightweight and focused only on grouping logic.
pub trait SessionGrouper {
    /// Group video titles into sessions
    ///
    /// Returns a vector of vectors, where each inner vector contains indices
    /// of videos that should be grouped together in a session.
    ///
    /// # Arguments
    /// * `titles` - Vector of video titles to group
    ///
    /// # Returns
    /// Vector of session groups, where each group is a vector of video indices
    fn group(&self, titles: &[String]) -> Result<Vec<Vec<usize>>>;

    /// Get a human-readable name for this grouper
    fn name(&self) -> &'static str;

    /// Get configuration information for this grouper
    fn config(&self) -> SessionGrouperConfig {
        SessionGrouperConfig::default()
    }
}

/// Configuration for session groupers
#[derive(Debug, Clone)]
pub struct SessionGrouperConfig {
    /// Maximum number of videos per session
    pub max_videos_per_session: usize,
    /// Minimum number of videos per session
    pub min_videos_per_session: usize,
    /// Whether to preserve original order
    pub preserve_order: bool,
    /// Additional configuration parameters
    pub params: std::collections::HashMap<String, String>,
}

impl Default for SessionGrouperConfig {
    fn default() -> Self {
        Self {
            max_videos_per_session: 10,
            min_videos_per_session: 1,
            preserve_order: true,
            params: std::collections::HashMap::new(),
        }
    }
}

/// Sequential grouper that preserves video order
///
/// Groups videos sequentially by count or duration estimates.
/// This is the simplest and most predictable grouping strategy.
#[derive(Debug, Clone)]
pub struct SequentialGrouper {
    config: SessionGrouperConfig,
}

impl SequentialGrouper {
    /// Create a new sequential grouper with default configuration
    pub fn new() -> Self {
        Self {
            config: SessionGrouperConfig::default(),
        }
    }

    /// Create a new sequential grouper with custom session size
    pub fn with_session_size(max_size: usize) -> Self {
        let mut config = SessionGrouperConfig::default();
        config.max_videos_per_session = max_size;
        Self { config }
    }

    /// Create a new sequential grouper with custom configuration
    pub fn with_config(config: SessionGrouperConfig) -> Self {
        Self { config }
    }
}

impl Default for SequentialGrouper {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionGrouper for SequentialGrouper {
    fn group(&self, titles: &[String]) -> Result<Vec<Vec<usize>>> {
        if titles.is_empty() {
            return Ok(vec![]);
        }

        debug!(
            "Sequential grouping {} titles with max {} videos per session",
            titles.len(),
            self.config.max_videos_per_session
        );

        let mut groups = Vec::new();
        let mut current_group = Vec::new();

        for (index, title) in titles.iter().enumerate() {
            current_group.push(index);

            // Check if we should start a new group
            if current_group.len() >= self.config.max_videos_per_session {
                debug!(
                    "Completed session {} with {} videos (last: '{}')",
                    groups.len() + 1,
                    current_group.len(),
                    title
                );
                groups.push(current_group);
                current_group = Vec::new();
            }
        }

        // Add remaining videos as the last group
        if !current_group.is_empty() {
            debug!(
                "Final session {} with {} videos",
                groups.len() + 1,
                current_group.len()
            );
            groups.push(current_group);
        }

        info!(
            "Sequential grouping complete: {} videos grouped into {} sessions",
            titles.len(),
            groups.len()
        );

        Ok(groups)
    }

    fn name(&self) -> &'static str {
        "Sequential"
    }

    fn config(&self) -> SessionGrouperConfig {
        self.config.clone()
    }
}

/// Simple similarity-based grouper using basic text matching
///
/// Groups videos based on simple keyword similarity without complex ML algorithms.
/// This provides thematic grouping while remaining lightweight.
#[derive(Debug, Clone)]
pub struct SimilarityGrouper {
    config: SessionGrouperConfig,
    similarity_threshold: f32,
}

impl SimilarityGrouper {
    /// Create a new similarity grouper with default settings
    pub fn new() -> Self {
        Self {
            config: SessionGrouperConfig {
                preserve_order: false,
                ..SessionGrouperConfig::default()
            },
            similarity_threshold: 0.3,
        }
    }

    /// Create a similarity grouper with custom threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            config: SessionGrouperConfig {
                preserve_order: false,
                ..SessionGrouperConfig::default()
            },
            similarity_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Calculate simple word-based similarity between two titles
    fn calculate_similarity(&self, title1: &str, title2: &str) -> f32 {
        let title1_lower = title1.to_lowercase();
        let title2_lower = title2.to_lowercase();
        let words1: std::collections::HashSet<&str> = title1_lower.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = title2_lower.split_whitespace().collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

impl Default for SimilarityGrouper {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionGrouper for SimilarityGrouper {
    fn group(&self, titles: &[String]) -> Result<Vec<Vec<usize>>> {
        if titles.is_empty() {
            return Ok(vec![]);
        }

        if titles.len() == 1 {
            return Ok(vec![vec![0]]);
        }

        debug!(
            "Similarity grouping {} titles with threshold {}",
            titles.len(),
            self.similarity_threshold
        );

        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; titles.len()];

        for i in 0..titles.len() {
            if assigned[i] {
                continue;
            }

            let mut current_group = vec![i];
            assigned[i] = true;

            // Find similar titles to add to this group
            for j in (i + 1)..titles.len() {
                if assigned[j] {
                    continue;
                }

                if current_group.len() >= self.config.max_videos_per_session {
                    break;
                }

                // Check similarity with any title in the current group
                let should_add = current_group.iter().any(|&group_idx| {
                    self.calculate_similarity(&titles[group_idx], &titles[j])
                        >= self.similarity_threshold
                });

                if should_add {
                    current_group.push(j);
                    assigned[j] = true;
                }
            }

            debug!(
                "Created similarity group {} with {} videos (starting with: '{}')",
                groups.len() + 1,
                current_group.len(),
                titles[i]
            );

            groups.push(current_group);
        }

        info!(
            "Similarity grouping complete: {} videos grouped into {} sessions",
            titles.len(),
            groups.len()
        );

        Ok(groups)
    }

    fn name(&self) -> &'static str {
        "Similarity"
    }

    fn config(&self) -> SessionGrouperConfig {
        self.config.clone()
    }
}

/// Factory for creating session groupers
pub struct SessionGrouperFactory;

impl SessionGrouperFactory {
    /// Create a grouper by name
    pub fn create(name: &str) -> Result<Box<dyn SessionGrouper>> {
        match name.to_lowercase().as_str() {
            "sequential" => Ok(Box::new(SequentialGrouper::new())),
            "similarity" => Ok(Box::new(SimilarityGrouper::new())),
            _ => Err(anyhow::anyhow!("Unknown session grouper: {}", name)),
        }
    }

    /// Get list of available grouper names
    pub fn available_groupers() -> Vec<&'static str> {
        vec!["sequential", "similarity"]
    }

    /// Create the recommended grouper for typical use cases
    pub fn create_default() -> Box<dyn SessionGrouper> {
        Box::new(SequentialGrouper::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_grouper_basic() {
        let grouper = SequentialGrouper::with_session_size(3);
        let titles = vec![
            "Video 1".to_string(),
            "Video 2".to_string(),
            "Video 3".to_string(),
            "Video 4".to_string(),
            "Video 5".to_string(),
        ];

        let groups = grouper.group(&titles).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec![0, 1, 2]);
        assert_eq!(groups[1], vec![3, 4]);
    }

    #[test]
    fn test_sequential_grouper_empty() {
        let grouper = SequentialGrouper::new();
        let titles = vec![];
        let groups = grouper.group(&titles).unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_sequential_grouper_single() {
        let grouper = SequentialGrouper::new();
        let titles = vec!["Single Video".to_string()];
        let groups = grouper.group(&titles).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], vec![0]);
    }

    #[test]
    fn test_similarity_grouper_basic() {
        let grouper = SimilarityGrouper::with_threshold(0.3);
        let titles = vec![
            "Introduction to Python".to_string(),
            "Python Basics".to_string(),
            "Advanced JavaScript".to_string(),
            "JavaScript Functions".to_string(),
            "Python Data Types".to_string(),
        ];

        let groups = grouper.group(&titles).unwrap();

        // Should group Python videos and JavaScript videos separately
        assert!(groups.len() >= 2);

        // Verify that all videos are assigned
        let total_assigned: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total_assigned, titles.len());
    }

    #[test]
    fn test_similarity_calculation() {
        let grouper = SimilarityGrouper::new();

        // Identical titles
        assert_eq!(
            grouper.calculate_similarity("Hello World", "Hello World"),
            1.0
        );

        // Completely different titles
        assert_eq!(grouper.calculate_similarity("Hello", "Goodbye"), 0.0);

        // Partial overlap
        let similarity = grouper.calculate_similarity("Hello World", "Hello There");
        assert!(similarity > 0.0 && similarity < 1.0);
    }

    #[test]
    fn test_session_grouper_factory() {
        let sequential = SessionGrouperFactory::create("sequential").unwrap();
        assert_eq!(sequential.name(), "Sequential");

        let similarity = SessionGrouperFactory::create("similarity").unwrap();
        assert_eq!(similarity.name(), "Similarity");

        let invalid = SessionGrouperFactory::create("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_available_groupers() {
        let groupers = SessionGrouperFactory::available_groupers();
        assert!(groupers.contains(&"sequential"));
        assert!(groupers.contains(&"similarity"));
    }

    #[test]
    fn test_default_grouper() {
        let grouper = SessionGrouperFactory::create_default();
        assert_eq!(grouper.name(), "Sequential");
    }

    #[test]
    fn test_config_defaults() {
        let config = SessionGrouperConfig::default();
        assert_eq!(config.max_videos_per_session, 10);
        assert_eq!(config.min_videos_per_session, 1);
        assert!(config.preserve_order);
        assert!(config.params.is_empty());
    }

    #[test]
    fn test_grouper_config() {
        let grouper = SequentialGrouper::new();
        let config = grouper.config();
        assert_eq!(config.max_videos_per_session, 10);
        assert!(config.preserve_order);
    }
}
