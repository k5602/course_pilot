//! Clustering metadata and confidence scoring generation
//!
//! This module provides functionality to generate comprehensive metadata about clustering
//! operations, including confidence scores, rationale explanations, and performance metrics.

use super::{
    ClusteringConfidenceScores, ClusteringMetadata, ClusteringRationale,
    InputMetrics, ModuleConfidence, ModuleRationale, PerformanceMetrics,
};
use crate::types::{ClusteringAlgorithm, TopicInfo, Section};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::{BalancedCluster, OptimizedCluster};

/// Memory usage tracker for performance monitoring
pub struct MemoryTracker {
    initial_memory: u64,
    peak_memory: u64,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        let initial = Self::get_current_memory_usage();
        Self {
            initial_memory: initial,
            peak_memory: initial,
        }
    }

    /// Update peak memory usage
    pub fn update_peak(&mut self) {
        let current = Self::get_current_memory_usage();
        if current > self.peak_memory {
            self.peak_memory = current;
        }
    }

    /// Get peak memory usage since creation
    pub fn get_peak_usage(&self) -> u64 {
        self.peak_memory.saturating_sub(self.initial_memory)
    }

    /// Get current memory usage (simplified implementation)
    fn get_current_memory_usage() -> u64 {
        // In a real implementation, this would use system calls to get actual memory usage
        // For now, we'll use a simplified approach based on heap allocations
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        // Fallback: estimate based on heap size (very rough approximation)
        std::mem::size_of::<usize>() as u64 * 1024 * 1024 // 1MB baseline
    }
}

/// Performance metrics collector for clustering operations
pub struct PerformanceCollector {
    start_time: Instant,
    content_analysis_time: Option<Duration>,
    clustering_time: Option<Duration>,
    optimization_time: Option<Duration>,
    memory_tracker: MemoryTracker,
    algorithm_iterations: u32,
}

impl PerformanceCollector {
    /// Create a new performance collector
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            content_analysis_time: None,
            clustering_time: None,
            optimization_time: None,
            memory_tracker: MemoryTracker::new(),
            algorithm_iterations: 0,
        }
    }

    /// Mark content analysis phase completion
    pub fn mark_content_analysis_complete(&mut self) {
        self.content_analysis_time = Some(self.start_time.elapsed());
        self.memory_tracker.update_peak();
    }

    /// Mark clustering phase completion
    pub fn mark_clustering_complete(&mut self, iterations: u32) {
        self.clustering_time = Some(self.start_time.elapsed());
        self.algorithm_iterations = iterations;
        self.memory_tracker.update_peak();
    }

    /// Mark optimization phase completion
    pub fn mark_optimization_complete(&mut self) {
        self.optimization_time = Some(self.start_time.elapsed());
        self.memory_tracker.update_peak();
    }

    /// Generate performance metrics
    pub fn generate_metrics(&self, input_metrics: InputMetrics) -> PerformanceMetrics {
        let total_time = self.start_time.elapsed();

        PerformanceMetrics {
            total_processing_time_ms: total_time.as_millis() as u64,
            content_analysis_time_ms: self.content_analysis_time
                .unwrap_or_default()
                .as_millis() as u64,
            clustering_time_ms: self.clustering_time
                .map(|t| t.saturating_sub(self.content_analysis_time.unwrap_or_default()))
                .unwrap_or_default()
                .as_millis() as u64,
            optimization_time_ms: self.optimization_time
                .map(|t| t.saturating_sub(self.clustering_time.unwrap_or_default()))
                .unwrap_or_default()
                .as_millis() as u64,
            peak_memory_usage_bytes: self.memory_tracker.get_peak_usage(),
            algorithm_iterations: self.algorithm_iterations,
            input_metrics,
        }
    }
}

/// Confidence score calculator for clustering results
pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    /// Calculate comprehensive confidence scores for clustering results
    pub fn calculate_confidence_scores(
        _sections: &[Section],
        clusters: &[OptimizedCluster],
        similarity_threshold: f32,
        algorithm_used: &crate::types::ClusteringAlgorithm,
    ) -> ClusteringConfidenceScores {
        let module_confidences = Self::calculate_module_confidences(clusters, similarity_threshold);

        let overall_confidence = Self::calculate_overall_confidence(&module_confidences, algorithm_used);
        let module_grouping_confidence = Self::calculate_module_grouping_confidence(&module_confidences);
        let similarity_confidence = Self::calculate_similarity_confidence(clusters, similarity_threshold);
        let topic_extraction_confidence = Self::calculate_topic_extraction_confidence(clusters);

        ClusteringConfidenceScores {
            overall_confidence,
            module_grouping_confidence,
            similarity_confidence,
            topic_extraction_confidence,
            module_confidences,
        }
    }

    /// Calculate confidence scores for individual modules
    fn calculate_module_confidences(
        clusters: &[OptimizedCluster],
        similarity_threshold: f32,
    ) -> Vec<ModuleConfidence> {
        clusters
            .iter()
            .enumerate()
            .map(|(index, cluster)| {
                let similarity_strength = cluster.average_similarity;
                let topic_coherence = Self::calculate_topic_coherence(cluster);
                let duration_balance = Self::calculate_duration_balance_score(cluster);

                // Overall confidence is weighted average of individual factors
                let confidence_score = (similarity_strength * 0.4) +
                                     (topic_coherence * 0.3) +
                                     (duration_balance * 0.3);

                ModuleConfidence {
                    module_index: index,
                    confidence_score: confidence_score.clamp(0.0, 1.0),
                    similarity_strength,
                    topic_coherence,
                    duration_balance,
                }
            })
            .collect()
    }

    /// Calculate overall confidence based on module confidences and algorithm
    fn calculate_overall_confidence(
        module_confidences: &[ModuleConfidence],
        algorithm_used: &crate::types::ClusteringAlgorithm,
    ) -> f32 {
        if module_confidences.is_empty() {
            return 0.0;
        }

        // Base confidence from average module confidence
        let avg_module_confidence = module_confidences
            .iter()
            .map(|mc| mc.confidence_score)
            .sum::<f32>() / module_confidences.len() as f32;

        // Algorithm reliability factor
        let algorithm_factor = match algorithm_used {
            crate::types::ClusteringAlgorithm::TfIdf => 0.8,
            crate::types::ClusteringAlgorithm::KMeans => 0.9,
            crate::types::ClusteringAlgorithm::Hierarchical => 0.85,
            crate::types::ClusteringAlgorithm::Hybrid => 0.95,
            crate::types::ClusteringAlgorithm::Fallback => 0.5,
        };

        // Consistency bonus: higher confidence if modules have similar confidence scores
        let confidence_variance = Self::calculate_variance(
            &module_confidences.iter().map(|mc| mc.confidence_score).collect::<Vec<_>>()
        );
        let consistency_bonus = (1.0 - confidence_variance).max(0.0) * 0.1;

        ((avg_module_confidence * algorithm_factor) + consistency_bonus).clamp(0.0, 1.0)
    }

    /// Calculate module grouping confidence
    fn calculate_module_grouping_confidence(module_confidences: &[ModuleConfidence]) -> f32 {
        if module_confidences.is_empty() {
            return 0.0;
        }

        // Average similarity strength across all modules
        let avg_similarity = module_confidences
            .iter()
            .map(|mc| mc.similarity_strength)
            .sum::<f32>() / module_confidences.len() as f32;

        // Penalize if we have too many or too few modules
        let module_count_factor = match module_confidences.len() {
            1 => 0.6, // Single module suggests poor clustering
            2..=5 => 1.0, // Optimal range
            6..=10 => 0.9, // Still good
            _ => 0.7, // Too many modules
        };

        (avg_similarity * module_count_factor).clamp(0.0, 1.0)
    }

    /// Calculate similarity confidence based on clustering quality
    fn calculate_similarity_confidence(
        clusters: &[OptimizedCluster],
        similarity_threshold: f32,
    ) -> f32 {
        if clusters.is_empty() {
            return 0.0;
        }

        let avg_similarity = clusters
            .iter()
            .map(|c| c.average_similarity)
            .sum::<f32>() / clusters.len() as f32;

        // Confidence increases with similarity above threshold
        let threshold_factor = if avg_similarity > similarity_threshold {
            1.0 + ((avg_similarity - similarity_threshold) * 0.5)
        } else {
            avg_similarity / similarity_threshold
        };

        threshold_factor.clamp(0.0, 1.0)
    }

    /// Calculate topic extraction confidence
    fn calculate_topic_extraction_confidence(clusters: &[OptimizedCluster]) -> f32 {
        if clusters.is_empty() {
            return 0.0;
        }

        let total_videos = clusters.iter().map(|c| c.videos.len()).sum::<usize>();
        let videos_with_topics = clusters
            .iter()
            .map(|c| if c.videos.iter().any(|v| !v.topic_tags.is_empty()) { c.videos.len() } else { 0 })
            .sum::<usize>();

        if total_videos == 0 {
            0.0
        } else {
            (videos_with_topics as f32 / total_videos as f32).clamp(0.0, 1.0)
        }
    }

    /// Calculate topic coherence for a cluster
    fn calculate_topic_coherence(cluster: &OptimizedCluster) -> f32 {
        if cluster.videos.is_empty() {
            return 0.0;
        }

        // Count unique topic tags across all videos in cluster
        let mut all_topics = std::collections::HashSet::new();
        let mut topic_counts = HashMap::new();

        for video in &cluster.videos {
            for topic in &video.topic_tags {
                all_topics.insert(topic.clone());
                *topic_counts.entry(topic.clone()).or_insert(0) += 1;
            }
        }

        if all_topics.is_empty() {
            return 0.5; // Neutral score if no topics
        }

        // Calculate coherence based on topic overlap
        let max_count = topic_counts.values().max().copied().unwrap_or(0);
        let total_videos = cluster.videos.len();

        (max_count as f32 / total_videos as f32).clamp(0.0, 1.0)
    }

    /// Calculate duration balance score for a cluster
    fn calculate_duration_balance_score(cluster: &OptimizedCluster) -> f32 {
        if cluster.videos.len() <= 1 {
            return 1.0; // Perfect balance for single video
        }

        let durations: Vec<f32> = cluster.videos
            .iter()
            .map(|v| v.duration.as_secs() as f32)
            .collect();

        let mean = durations.iter().sum::<f32>() / durations.len() as f32;
        let variance = Self::calculate_variance(&durations);
        let coefficient_of_variation = if mean > 0.0 { variance.sqrt() / mean } else { 0.0 };

        // Lower coefficient of variation = better balance
        (1.0 - coefficient_of_variation.min(1.0)).clamp(0.0, 1.0)
    }

    /// Calculate variance of a set of values
    fn calculate_variance(values: &[f32]) -> f32 {
        if values.len() <= 1 {
            return 0.0;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / values.len() as f32;

        variance
    }
}

/// Rationale generator for explaining clustering decisions
pub struct RationaleGenerator;

impl RationaleGenerator {
    /// Generate comprehensive clustering rationale
    pub fn generate_rationale(
        sections: &[Section],
        clusters: &[OptimizedCluster],
        algorithm_used: &crate::types::ClusteringAlgorithm,
        similarity_threshold: f32,
        confidence_scores: &ClusteringConfidenceScores,
    ) -> ClusteringRationale {
        let primary_strategy = Self::determine_primary_strategy(algorithm_used, clusters);
        let explanation = Self::generate_explanation(algorithm_used, clusters, similarity_threshold, confidence_scores);
        let key_factors = Self::identify_key_factors(clusters, confidence_scores);
        let alternatives_considered = Self::list_alternatives_considered(algorithm_used);
        let module_rationales = Self::generate_module_rationales(sections, clusters);

        ClusteringRationale {
            primary_strategy,
            explanation,
            key_factors,
            alternatives_considered,
            module_rationales,
        }
    }

    /// Determine the primary clustering strategy used
    fn determine_primary_strategy(
        algorithm_used: &crate::types::ClusteringAlgorithm,
        clusters: &[OptimizedCluster],
    ) -> String {
        match algorithm_used {
            crate::types::ClusteringAlgorithm::TfIdf => "Content Similarity Analysis".to_string(),
            crate::types::ClusteringAlgorithm::KMeans => "K-Means Content Clustering".to_string(),
            crate::types::ClusteringAlgorithm::Hierarchical => "Hierarchical Content Clustering".to_string(),
            crate::types::ClusteringAlgorithm::Hybrid => {
                if clusters.len() <= 3 {
                    "Hybrid Approach with Content Focus".to_string()
                } else {
                    "Hybrid Approach with Structure Focus".to_string()
                }
            }
            crate::types::ClusteringAlgorithm::Fallback => "Sequential Grouping (Fallback)".to_string(),
        }
    }

    /// Generate detailed explanation of clustering approach
    fn generate_explanation(
        algorithm_used: &crate::types::ClusteringAlgorithm,
        clusters: &[OptimizedCluster],
        similarity_threshold: f32,
        confidence_scores: &ClusteringConfidenceScores,
    ) -> String {
        let base_explanation = match algorithm_used {
            ClusteringAlgorithm::TfIdf => {
                format!(
                    "Videos were grouped based on content similarity using TF-IDF analysis. \
                     Videos with similarity scores above {:.1}% were grouped together.",
                    similarity_threshold * 100.0
                )
            }
            ClusteringAlgorithm::KMeans => {
                format!(
                    "K-means clustering algorithm identified {} optimal content groups. \
                     Videos were assigned to clusters based on semantic similarity of their titles.",
                    clusters.len()
                )
            }
            ClusteringAlgorithm::Hierarchical => {
                "Hierarchical clustering built a tree of content relationships, \
                 grouping videos from most similar to least similar."
                    .to_string()
            }
            ClusteringAlgorithm::Hybrid => {
                "A hybrid approach combined content similarity analysis with duration balancing \
                 to create well-structured learning modules."
                    .to_string()
            }
            ClusteringAlgorithm::Fallback => {
                "Content clustering was not possible due to insufficient similarity patterns. \
                 Videos were grouped sequentially to maintain logical progression."
                    .to_string()
            }
        };

        let confidence_note = if confidence_scores.overall_confidence >= 0.8 {
            " The clustering shows high confidence with strong content relationships."
        } else if confidence_scores.overall_confidence >= 0.6 {
            " The clustering shows moderate confidence with some clear content patterns."
        } else {
            " The clustering shows lower confidence due to diverse or unclear content patterns."
        };

        format!("{}{}", base_explanation, confidence_note)
    }

    /// Identify key factors that influenced clustering
    fn identify_key_factors(
        clusters: &[OptimizedCluster],
        confidence_scores: &ClusteringConfidenceScores,
    ) -> Vec<String> {
        let mut factors = Vec::new();

        // Content similarity factor
        if confidence_scores.similarity_confidence >= 0.7 {
            factors.push("Strong content similarity patterns detected".to_string());
        } else if confidence_scores.similarity_confidence >= 0.4 {
            factors.push("Moderate content similarity patterns found".to_string());
        } else {
            factors.push("Limited content similarity, relied on structural cues".to_string());
        }

        // Topic extraction factor
        if confidence_scores.topic_extraction_confidence >= 0.7 {
            factors.push("Clear topic themes identified across content".to_string());
        } else if confidence_scores.topic_extraction_confidence >= 0.4 {
            factors.push("Some topic patterns detected in video titles".to_string());
        }

        // Duration balancing factor
        let avg_duration_balance = confidence_scores.module_confidences
            .iter()
            .map(|mc| mc.duration_balance)
            .sum::<f32>() / confidence_scores.module_confidences.len().max(1) as f32;

        if avg_duration_balance >= 0.7 {
            factors.push("Good duration balance achieved across modules".to_string());
        } else if avg_duration_balance >= 0.4 {
            factors.push("Moderate duration balance with some variation".to_string());
        } else {
            factors.push("Duration variation present, prioritized content over time balance".to_string());
        }

        // Module count factor
        match clusters.len() {
            1 => factors.push("Single cohesive module identified".to_string()),
            2..=3 => factors.push("Optimal number of focused modules created".to_string()),
            4..=6 => factors.push("Multiple distinct content areas identified".to_string()),
            _ => factors.push("High content diversity resulted in many specialized modules".to_string()),
        }

        factors
    }

    /// List alternative strategies that were considered
    fn list_alternatives_considered(algorithm_used: &ClusteringAlgorithm) -> Vec<String> {
        match algorithm_used {
            ClusteringAlgorithm::TfIdf => vec![
                "K-means clustering for more balanced groups".to_string(),
                "Sequential grouping for simpler structure".to_string(),
            ],
            ClusteringAlgorithm::KMeans => vec![
                "TF-IDF similarity grouping for content focus".to_string(),
                "Hierarchical clustering for nested structure".to_string(),
            ],
            ClusteringAlgorithm::Hierarchical => vec![
                "K-means for balanced cluster sizes".to_string(),
                "TF-IDF for similarity-based grouping".to_string(),
            ],
            ClusteringAlgorithm::Hybrid => vec![
                "Pure content-based clustering".to_string(),
                "Pure duration-based grouping".to_string(),
                "Sequential organization".to_string(),
            ],
            ClusteringAlgorithm::Fallback => vec![
                "Content similarity clustering (insufficient data)".to_string(),
                "Duration-based grouping (no clear patterns)".to_string(),
            ],
        }
    }

    /// Generate rationale for individual modules
    fn generate_module_rationales(
        _sections: &[Section],
        clusters: &[OptimizedCluster],
    ) -> Vec<ModuleRationale> {
        clusters
            .iter()
            .enumerate()
            .map(|(index, cluster)| {
                let grouping_reason = Self::determine_grouping_reason(cluster);
                let similarity_explanation = Self::explain_similarity(cluster);
                let topic_keywords = cluster.videos
                    .iter()
                    .flat_map(|v| &v.topic_tags)
                    .take(5)
                    .cloned()
                    .collect();

                ModuleRationale {
                    module_index: index,
                    module_title: cluster.suggested_title.clone(),
                    grouping_reason,
                    similarity_explanation,
                    topic_keywords,
                    video_count: cluster.videos.len(),
                }
            })
            .collect()
    }

    /// Determine the reason for grouping videos in a cluster
    fn determine_grouping_reason(cluster: &OptimizedCluster) -> String {
        if cluster.average_similarity >= 0.8 {
            "Videos grouped due to very high content similarity".to_string()
        } else if cluster.average_similarity >= 0.6 {
            "Videos grouped due to strong thematic relationship".to_string()
        } else if cluster.average_similarity >= 0.4 {
            "Videos grouped due to moderate content overlap".to_string()
        } else if cluster.videos.len() == 1 {
            "Single video forms standalone module".to_string()
        } else {
            "Videos grouped for structural coherence and duration balance".to_string()
        }
    }

    /// Explain the similarity basis for a cluster
    fn explain_similarity(cluster: &OptimizedCluster) -> String {
        let common_topics: Vec<String> = cluster.videos
            .iter()
            .flat_map(|v| &v.topic_tags)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();

        if !common_topics.is_empty() {
            format!(
                "Videos share common topics: {}",
                common_topics.join(", ")
            )
        } else if cluster.average_similarity >= 0.5 {
            format!(
                "Videos have {:.0}% content similarity based on title analysis",
                cluster.average_similarity * 100.0
            )
        } else {
            "Videos grouped for logical learning progression".to_string()
        }
    }
}

/// Input metrics calculator
pub struct InputMetricsCalculator;

impl InputMetricsCalculator {
    /// Calculate input metrics from sections
    pub fn calculate_metrics(sections: &[Section]) -> InputMetrics {
        let video_count = sections.len();

        // Analyze titles for vocabulary metrics
        let all_words: Vec<String> = sections
            .iter()
            .flat_map(|s| {
                let title_lower = s.title.to_lowercase();
                title_lower.split_whitespace().map(|w| w.to_string()).collect::<Vec<_>>()
            })
            .collect();

        let unique_words = all_words.iter().collect::<std::collections::HashSet<_>>().len();
        let vocabulary_size = Self::calculate_vocabulary_size(&all_words);

        let average_title_length = if video_count > 0 {
            sections.iter().map(|s| s.title.len()).sum::<usize>() as f32 / video_count as f32
        } else {
            0.0
        };

        let content_diversity_score = Self::calculate_content_diversity(sections);

        InputMetrics {
            video_count,
            unique_words,
            vocabulary_size,
            average_title_length,
            content_diversity_score,
        }
    }

    /// Calculate vocabulary size (unique meaningful words)
    fn calculate_vocabulary_size(words: &[String]) -> usize {
        let stop_words: std::collections::HashSet<&str> = [
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in",
            "is", "it", "its", "of", "on", "that", "the", "to", "was", "will", "with", "this",
            "but", "they", "have", "had", "what", "said", "each", "which", "she", "do", "how",
            "their", "if", "up", "out", "many", "then", "them", "these", "so", "some", "her",
            "would", "make", "like", "into", "him", "time", "two", "more", "go", "no", "way",
            "could", "my", "than", "first", "been", "call", "who", "oil", "sit", "now", "find",
            "down", "day", "did", "get", "come", "made", "may", "part",
        ].iter().copied().collect();

        words
            .iter()
            .filter(|word| word.len() > 2 && !stop_words.contains(word.as_str()))
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    /// Calculate content diversity score
    fn calculate_content_diversity(sections: &[Section]) -> f32 {
        if sections.len() <= 1 {
            return 0.0;
        }

        // Calculate pairwise similarity between all titles
        let mut total_similarity = 0.0;
        let mut pair_count = 0;

        for i in 0..sections.len() {
            for j in (i + 1)..sections.len() {
                let similarity = Self::calculate_title_similarity(&sections[i].title, &sections[j].title);
                total_similarity += similarity;
                pair_count += 1;
            }
        }

        let average_similarity = if pair_count > 0 {
            total_similarity / pair_count as f32
        } else {
            0.0
        };

        // Diversity is inverse of similarity
        1.0 - average_similarity.clamp(0.0, 1.0)
    }

    /// Calculate similarity between two titles
    fn calculate_title_similarity(title1: &str, title2: &str) -> f32 {
        let title1_lower = title1.to_lowercase();
        let title2_lower = title2.to_lowercase();
        let words1: std::collections::HashSet<String> = title1_lower.split_whitespace().map(|s| s.to_string()).collect();
        let words2: std::collections::HashSet<String> = title2_lower.split_whitespace().map(|s| s.to_string()).collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

/// Convert BalancedCluster to OptimizedCluster for metadata generation
fn convert_balanced_to_optimized_cluster(balanced: &BalancedCluster) -> OptimizedCluster {
    OptimizedCluster {
        videos: balanced.videos.clone(),
        total_duration: balanced.total_duration,
        average_similarity: balanced.balance_score, // Use balance score as similarity proxy
        difficulty_level: crate::types::DifficultyLevel::Intermediate, // Default difficulty
        suggested_title: format!("Module {}", balanced.videos.len()), // Generate a basic title
    }
}

/// Main metadata generator that orchestrates all components
pub struct MetadataGenerator;

impl MetadataGenerator {
    /// Generate complete clustering metadata from BalancedCluster
    pub fn generate_complete_metadata_from_balanced(
        sections: &[Section],
        balanced_clusters: &[BalancedCluster],
        algorithm_used: crate::types::ClusteringAlgorithm,
        strategy_used: crate::types::ClusteringStrategy,
        similarity_threshold: f32,
        content_topics: Vec<TopicInfo>,
        performance_metrics: PerformanceMetrics,
    ) -> ClusteringMetadata {
        let optimized_clusters: Vec<OptimizedCluster> = balanced_clusters
            .iter()
            .map(convert_balanced_to_optimized_cluster)
            .collect();

        Self::generate_complete_metadata(
            sections,
            &optimized_clusters,
            algorithm_used,
            strategy_used,
            similarity_threshold,
            content_topics,
            performance_metrics,
        )
    }

    /// Generate complete clustering metadata
    pub fn generate_complete_metadata(
        sections: &[Section],
        clusters: &[OptimizedCluster],
        algorithm_used: crate::types::ClusteringAlgorithm,
        strategy_used: crate::types::ClusteringStrategy,
        similarity_threshold: f32,
        content_topics: Vec<TopicInfo>,
        performance_metrics: PerformanceMetrics,
    ) -> ClusteringMetadata {
        let confidence_scores = ConfidenceCalculator::calculate_confidence_scores(
            sections,
            clusters,
            similarity_threshold,
            &algorithm_used,
        );

        let rationale = RationaleGenerator::generate_rationale(
            sections,
            clusters,
            &algorithm_used,
            similarity_threshold,
            &confidence_scores,
        );

        let quality_score = Self::calculate_overall_quality_score(&confidence_scores, &performance_metrics);

        ClusteringMetadata {
            algorithm_used,
            strategy_used,
            similarity_threshold,
            cluster_count: clusters.len(),
            quality_score,
            processing_time_ms: performance_metrics.total_processing_time_ms,
            content_topics,
            confidence_scores,
            rationale,
            performance_metrics,
        }
    }

    /// Calculate overall quality score from various metrics
    fn calculate_overall_quality_score(
        confidence_scores: &ClusteringConfidenceScores,
        performance_metrics: &PerformanceMetrics,
    ) -> f32 {
        // Base quality from confidence scores
        let confidence_quality = confidence_scores.overall_confidence * 0.6 +
                                confidence_scores.similarity_confidence * 0.2 +
                                confidence_scores.topic_extraction_confidence * 0.2;

        // Performance penalty for very slow processing
        let performance_penalty = if performance_metrics.total_processing_time_ms > 10000 {
            0.1 // 10% penalty for processing > 10 seconds
        } else if performance_metrics.total_processing_time_ms > 5000 {
            0.05 // 5% penalty for processing > 5 seconds
        } else {
            0.0
        };

        // Memory usage penalty for excessive memory consumption
        let memory_penalty = if performance_metrics.peak_memory_usage_bytes > 100 * 1024 * 1024 {
            0.05 // 5% penalty for > 100MB usage
        } else {
            0.0
        };

        (confidence_quality - performance_penalty - memory_penalty).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DifficultyLevel;
    use std::time::Duration;

    fn create_test_section(title: &str, index: usize, duration_secs: u64) -> Section {
        Section {
            title: title.to_string(),
            video_index: index,
            duration: Duration::from_secs(duration_secs),
        }
    }

    fn create_test_cluster(title: &str, video_count: usize, similarity: f32) -> OptimizedCluster {
        let videos = (0..video_count)
            .map(|i| super::super::VideoWithMetadata {
                index: i,
                title: format!("{} - Part {}", title, i + 1),
                duration: Duration::from_secs(600),
                feature_vector: super::super::FeatureVector::default(),
                difficulty_score: 0.5,
                topic_tags: vec!["test".to_string()],
            })
            .collect();

        OptimizedCluster {
            videos,
            total_duration: Duration::from_secs(600 * video_count as u64),
            average_similarity: similarity,
            difficulty_level: DifficultyLevel::Intermediate,
            suggested_title: title.to_string(),
        }
    }

    #[test]
    fn test_confidence_calculation() {
        let sections = vec![
            create_test_section("Introduction to Rust", 0, 600),
            create_test_section("Rust Basics", 1, 720),
            create_test_section("Advanced Rust", 2, 900),
        ];

        let clusters = vec![
            create_test_cluster("Rust Fundamentals", 2, 0.8),
            create_test_cluster("Advanced Topics", 1, 0.6),
        ];

        let confidence_scores = ConfidenceCalculator::calculate_confidence_scores(
            &sections,
            &clusters,
            0.6,
            &ClusteringAlgorithm::KMeans,
        );

        assert!(confidence_scores.overall_confidence > 0.0);
        assert!(confidence_scores.similarity_confidence > 0.0);
        assert_eq!(confidence_scores.module_confidences.len(), 2);
    }

    #[test]
    fn test_rationale_generation() {
        let sections = vec![
            create_test_section("Python Basics", 0, 600),
            create_test_section("Python Functions", 1, 720),
        ];

        let clusters = vec![
            create_test_cluster("Python Fundamentals", 2, 0.7),
        ];

        let confidence_scores = ClusteringConfidenceScores {
            overall_confidence: 0.8,
            module_grouping_confidence: 0.7,
            similarity_confidence: 0.75,
            topic_extraction_confidence: 0.6,
            module_confidences: vec![ModuleConfidence {
                module_index: 0,
                confidence_score: 0.8,
                similarity_strength: 0.7,
                topic_coherence: 0.8,
                duration_balance: 0.9,
            }],
        };

        let rationale = RationaleGenerator::generate_rationale(
            &sections,
            &clusters,
            &ClusteringAlgorithm::KMeans,
            0.6,
            &confidence_scores,
        );

        assert!(!rationale.primary_strategy.is_empty());
        assert!(!rationale.explanation.is_empty());
        assert!(!rationale.key_factors.is_empty());
        assert_eq!(rationale.module_rationales.len(), 1);
    }

    #[test]
    fn test_performance_collector() {
        let mut collector = PerformanceCollector::new();

        // Simulate processing phases
        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.mark_content_analysis_complete();

        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.mark_clustering_complete(5);

        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.mark_optimization_complete();

        let input_metrics = InputMetrics {
            video_count: 10,
            unique_words: 50,
            vocabulary_size: 40,
            average_title_length: 25.0,
            content_diversity_score: 0.7,
        };

        let metrics = collector.generate_metrics(input_metrics);

        assert!(metrics.total_processing_time_ms >= 30);
        assert_eq!(metrics.algorithm_iterations, 5);
        assert!(metrics.peak_memory_usage_bytes > 0);
    }

    #[test]
    fn test_input_metrics_calculation() {
        let sections = vec![
            create_test_section("Introduction to Machine Learning", 0, 600),
            create_test_section("Deep Learning Fundamentals", 1, 720),
            create_test_section("Neural Networks Basics", 2, 800),
        ];

        let metrics = InputMetricsCalculator::calculate_metrics(&sections);

        assert_eq!(metrics.video_count, 3);
        assert!(metrics.unique_words > 0);
        assert!(metrics.vocabulary_size > 0);
        assert!(metrics.average_title_length > 0.0);
        assert!(metrics.content_diversity_score >= 0.0);
        assert!(metrics.content_diversity_score <= 1.0);
    }

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();

        // Simulate memory usage
        let _data: Vec<u8> = vec![0; 1024]; // Allocate some memory
        tracker.update_peak();

        let peak_usage = tracker.get_peak_usage();
        assert!(peak_usage >= 0); // Should be non-negative
    }

    #[test]
    fn test_complete_metadata_generation() {
        let sections = vec![
            create_test_section("JavaScript Basics", 0, 600),
            create_test_section("JavaScript Functions", 1, 720),
            create_test_section("JavaScript Objects", 2, 800),
        ];

        let clusters = vec![
            create_test_cluster("JavaScript Fundamentals", 3, 0.75),
        ];

        let content_topics = vec![
            TopicInfo {
                keyword: "javascript".to_string(),
                relevance_score: 0.9,
                video_count: 3,
            },
        ];

        let performance_metrics = PerformanceMetrics {
            total_processing_time_ms: 1500,
            content_analysis_time_ms: 500,
            clustering_time_ms: 700,
            optimization_time_ms: 300,
            peak_memory_usage_bytes: 1024 * 1024, // 1MB
            algorithm_iterations: 10,
            input_metrics: InputMetrics {
                video_count: 3,
                unique_words: 15,
                vocabulary_size: 12,
                average_title_length: 20.0,
                content_diversity_score: 0.6,
            },
        };

        let metadata = MetadataGenerator::generate_complete_metadata(
            &sections,
            &clusters,
            ClusteringAlgorithm::KMeans,
            crate::types::ClusteringStrategy::ContentBased,
            0.6,
            content_topics,
            performance_metrics,
        );

        assert_eq!(metadata.algorithm_used, ClusteringAlgorithm::KMeans);
        assert_eq!(metadata.cluster_count, 1);
        assert!(metadata.quality_score > 0.0);
        assert!(metadata.confidence_scores.overall_confidence > 0.0);
        assert!(!metadata.rationale.explanation.is_empty());
        assert_eq!(metadata.performance_metrics.total_processing_time_ms, 1500);
    }

    #[test]
    fn test_performance_collector() {
        let mut collector = PerformanceCollector::new();

        std::thread::sleep(Duration::from_millis(10));
        collector.mark_content_analysis_complete();

        std::thread::sleep(Duration::from_millis(10));
        collector.mark_clustering_complete(5);

        std::thread::sleep(Duration::from_millis(10));
        collector.mark_optimization_complete();

        let input_metrics = InputMetrics {
            video_count: 10,
            unique_words: 50,
            vocabulary_size: 40,
            average_title_length: 25.0,
            content_diversity_score: 0.7,
        };

        let metrics = collector.generate_metrics(input_metrics);

        assert!(metrics.total_processing_time_ms >= 30);
        assert_eq!(metrics.algorithm_iterations, 5);
        assert!(metrics.content_analysis_time_ms > 0);
    }

    #[test]
    fn test_confidence_calculation() {
        let sections = vec![
            create_test_section("Introduction to Programming", 0, 300),
            create_test_section("Advanced Programming", 1, 400),
            create_test_section("Database Basics", 2, 350),
        ];

        let clusters = vec![
            create_test_cluster("Programming", 2, 0.8),
            create_test_cluster("Database", 1, 1.0),
        ];

        let confidence = ConfidenceCalculator::calculate_confidence_scores(
            &sections,
            &clusters,
            0.6,
            &ClusteringAlgorithm::KMeans,
        );

        assert!(confidence.overall_confidence > 0.0);
        assert!(confidence.overall_confidence <= 1.0);
        assert_eq!(confidence.module_confidences.len(), 2);
        assert!(confidence.similarity_confidence > 0.0);
    }

    #[test]
    fn test_rationale_generation() {
        let sections = vec![
            create_test_section("Programming Basics", 0, 300),
            create_test_section("Advanced Programming", 1, 400),
        ];

        let clusters = vec![create_test_cluster("Programming", 2, 0.8)];

        let confidence_scores = ClusteringConfidenceScores {
            overall_confidence: 0.8,
            module_grouping_confidence: 0.7,
            similarity_confidence: 0.8,
            topic_extraction_confidence: 0.6,
            module_confidences: vec![ModuleConfidence {
                module_index: 0,
                confidence_score: 0.8,
                similarity_strength: 0.8,
                topic_coherence: 0.7,
                duration_balance: 0.9,
            }],
        };

        let rationale = RationaleGenerator::generate_rationale(
            &sections,
            &clusters,
            &ClusteringAlgorithm::KMeans,
            0.6,
            &confidence_scores,
        );

        assert_eq!(rationale.primary_strategy, "K-Means Content Clustering");
        assert!(!rationale.explanation.is_empty());
        assert!(!rationale.key_factors.is_empty());
        assert_eq!(rationale.module_rationales.len(), 1);
    }

    #[test]
    fn test_input_metrics_calculation() {
        let sections = vec![
            create_test_section("Introduction to Programming Basics", 0, 300),
            create_test_section("Advanced Programming Concepts", 1, 400),
            create_test_section("Database Design Fundamentals", 2, 350),
        ];

        let metrics = InputMetricsCalculator::calculate_metrics(&sections);

        assert_eq!(metrics.video_count, 3);
        assert!(metrics.unique_words > 0);
        assert!(metrics.vocabulary_size > 0);
        assert!(metrics.average_title_length > 0.0);
        assert!(metrics.content_diversity_score >= 0.0);
        assert!(metrics.content_diversity_score <= 1.0);
    }

    #[test]
    fn test_complete_metadata_generation() {
        let sections = vec![
            create_test_section("Programming Basics", 0, 300),
            create_test_section("Advanced Programming", 1, 400),
        ];

        let clusters = vec![create_test_cluster("Programming", 2, 0.8)];

        let performance_metrics = PerformanceMetrics {
            total_processing_time_ms: 1000,
            content_analysis_time_ms: 300,
            clustering_time_ms: 500,
            optimization_time_ms: 200,
            peak_memory_usage_bytes: 1024 * 1024,
            algorithm_iterations: 10,
            input_metrics: InputMetricsCalculator::calculate_metrics(&sections),
        };

        let content_topics = vec![TopicInfo {
            keyword: "programming".to_string(),
            relevance_score: 0.8,
            video_count: 2,
        }];

        let metadata = MetadataGenerator::generate_complete_metadata(
            &sections,
            &clusters,
            ClusteringAlgorithm::KMeans,
            crate::types::ClusteringStrategy::Hybrid,
            0.6,
            content_topics,
            performance_metrics,
        );

        assert_eq!(metadata.algorithm_used, ClusteringAlgorithm::KMeans);
        assert_eq!(metadata.cluster_count, 1);
        assert!(metadata.quality_score > 0.0);
        assert!(metadata.quality_score <= 1.0);
        assert!(!metadata.rationale.explanation.is_empty());
        assert!(!metadata.confidence_scores.module_confidences.is_empty());
    }
}
