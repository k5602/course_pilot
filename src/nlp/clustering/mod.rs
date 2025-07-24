//! Content clustering algorithms for intelligent video grouping
//!
//! This module provides content-aware clustering algorithms that analyze video titles
//! to group related content together while respecting duration constraints.

pub mod content_similarity;
pub mod duration_balancer;
pub mod kmeans;
pub mod metadata_generator;
pub mod topic_extractor;

// Re-export main clustering types and functions
pub use content_similarity::{ContentAnalysis, FeatureVector, SimilarityMatrix, TfIdfAnalyzer};
pub use duration_balancer::{BalancedCluster, DurationBalancer};
pub use kmeans::{Cluster, KMeansClusterer};
pub use topic_extractor::TopicExtractor;

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
    sections
        .iter()
        .enumerate()
        .map(|(_i, section)| VideoWithMetadata {
            index: section.video_index,
            title: section.title.clone(),
            duration: section.duration,
            feature_vector: FeatureVector::default(), // Will be populated during analysis
            difficulty_score: estimate_difficulty_score(&section.title),
            topic_tags: Vec::new(), // Will be populated during analysis
        })
        .collect()
}

/// Estimate difficulty score based on title content
fn estimate_difficulty_score(title: &str) -> f32 {
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
        assert_eq!(metadata.algorithm_used, ClusteringAlgorithm::TfIdf);
        assert_eq!(metadata.similarity_threshold, 0.6);
        assert_eq!(metadata.cluster_count, 0);
    }
}


