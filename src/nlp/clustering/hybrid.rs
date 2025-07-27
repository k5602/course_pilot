//! Hybrid clustering algorithm that combines multiple clustering approaches
//!
//! This module provides a hybrid clustering strategy that intelligently selects
//! and combines different clustering algorithms based on content characteristics.

use super::{
    ClusteringError, ContentAnalysis, ContentClusterer, FeatureVector, OptimizedCluster,
    VideoCluster, hierarchical::HierarchicalClusterer, kmeans::KMeansClusterer, lda::LdaClusterer,
};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

/// Hybrid clustering strategy that combines multiple algorithms
pub struct HybridClusterer {
    pub kmeans_clusterer: KMeansClusterer,
    pub hierarchical_clusterer: HierarchicalClusterer,
    pub lda_clusterer: LdaClusterer,
    pub strategy_selection: StrategySelection,
    pub ensemble_method: EnsembleMethod,
    pub quality_threshold: f32,
}

/// Strategy for selecting which algorithms to use
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrategySelection {
    /// Automatically select based on content characteristics
    Automatic,
    /// Use all algorithms and ensemble results
    Ensemble,
    /// Use specific combination of algorithms
    Custom(bool, bool, bool), // (use_kmeans, use_hierarchical, use_lda)
}

/// Method for combining results from multiple algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnsembleMethod {
    /// Use the algorithm that produces the best quality score
    BestQuality,
    /// Combine results using voting
    Voting,
    /// Use weighted combination based on algorithm confidence
    WeightedCombination,
}

/// Content characteristics analysis for strategy selection
#[derive(Debug, Clone)]
pub struct ContentCharacteristics {
    pub document_count: usize,
    pub vocabulary_size: usize,
    pub average_document_length: f32,
    pub vocabulary_diversity: f32,
    pub topic_coherence_estimate: f32,
    pub similarity_distribution: SimilarityDistribution,
}

/// Distribution of pairwise similarities in the content
#[derive(Debug, Clone)]
pub struct SimilarityDistribution {
    pub mean: f32,
    pub std_dev: f32,
    pub min: f32,
    pub max: f32,
    pub has_clear_clusters: bool,
}

/// Results from multiple clustering algorithms
#[derive(Debug, Clone)]
pub struct EnsembleResults {
    pub kmeans_result: Option<Vec<VideoCluster>>,
    pub hierarchical_result: Option<Vec<VideoCluster>>,
    pub lda_result: Option<Vec<VideoCluster>>,
    pub quality_scores: HashMap<String, f32>,
    pub selected_algorithm: String,
    pub final_clusters: Vec<VideoCluster>,
}

impl Default for HybridClusterer {
    fn default() -> Self {
        Self {
            kmeans_clusterer: KMeansClusterer::default(),
            hierarchical_clusterer: HierarchicalClusterer::default(),
            lda_clusterer: LdaClusterer::default(),
            strategy_selection: StrategySelection::Automatic,
            ensemble_method: EnsembleMethod::BestQuality,
            quality_threshold: 0.6,
        }
    }
}

impl HybridClusterer {
    /// Create a new hybrid clusterer with custom configuration
    pub fn new(
        strategy_selection: StrategySelection,
        ensemble_method: EnsembleMethod,
        quality_threshold: f32,
    ) -> Self {
        Self {
            kmeans_clusterer: KMeansClusterer::default(),
            hierarchical_clusterer: HierarchicalClusterer::default(),
            lda_clusterer: LdaClusterer::default(),
            strategy_selection,
            ensemble_method,
            quality_threshold,
        }
    }

    /// Perform hybrid clustering with automatic algorithm selection
    pub fn cluster_hybrid(&self, titles: &[String]) -> Result<EnsembleResults, ClusteringError> {
        if titles.is_empty() {
            return Err(ClusteringError::InsufficientContent(0));
        }

        // Analyze content characteristics
        let characteristics = self.analyze_content_characteristics(titles)?;

        // Select algorithms based on characteristics and strategy
        let (use_kmeans, use_hierarchical, use_lda) = self.select_algorithms(&characteristics);

        // Run selected algorithms
        let mut results = EnsembleResults {
            kmeans_result: None,
            hierarchical_result: None,
            lda_result: None,
            quality_scores: HashMap::new(),
            selected_algorithm: String::new(),
            final_clusters: Vec::new(),
        };

        // Run K-means if selected
        if use_kmeans {
            match self.run_kmeans_clustering(titles) {
                Ok((clusters, quality)) => {
                    results.quality_scores.insert("kmeans".to_string(), quality);
                    results.kmeans_result = Some(clusters);
                }
                Err(e) => {
                    eprintln!("K-means clustering failed: {e}");
                }
            }
        }

        // Run hierarchical clustering if selected
        if use_hierarchical {
            match self.run_hierarchical_clustering(titles) {
                Ok((clusters, quality)) => {
                    results
                        .quality_scores
                        .insert("hierarchical".to_string(), quality);
                    results.hierarchical_result = Some(clusters);
                }
                Err(e) => {
                    eprintln!("Hierarchical clustering failed: {e}");
                }
            }
        }

        // Run LDA if selected
        if use_lda {
            match self.run_lda_clustering(titles) {
                Ok((clusters, quality)) => {
                    results.quality_scores.insert("lda".to_string(), quality);
                    results.lda_result = Some(clusters);
                }
                Err(e) => {
                    eprintln!("LDA clustering failed: {e}");
                }
            }
        }

        // Combine results using ensemble method
        self.combine_results(&mut results, &characteristics)?;

        Ok(results)
    }

    /// Analyze content characteristics to inform algorithm selection
    fn analyze_content_characteristics(
        &self,
        titles: &[String],
    ) -> Result<ContentCharacteristics, ClusteringError> {
        let document_count = titles.len();

        // Build vocabulary and calculate basic statistics
        let mut vocabulary = std::collections::HashSet::new();
        let mut total_words = 0;
        let mut document_lengths = Vec::new();

        for title in titles {
            let words: Vec<String> = title
                .to_lowercase()
                .split_whitespace()
                .map(|w| w.chars().filter(|c| c.is_alphabetic()).collect())
                .filter(|w: &String| w.len() > 2)
                .collect();

            document_lengths.push(words.len());
            total_words += words.len();

            for word in words {
                vocabulary.insert(word);
            }
        }

        let vocabulary_size = vocabulary.len();
        let average_document_length = if document_count > 0 {
            total_words as f32 / document_count as f32
        } else {
            0.0
        };

        // Calculate vocabulary diversity (unique words / total words)
        let vocabulary_diversity = if total_words > 0 {
            vocabulary_size as f32 / total_words as f32
        } else {
            0.0
        };

        // Estimate topic coherence by analyzing word co-occurrence
        let topic_coherence_estimate = self.estimate_topic_coherence(titles);

        // Analyze similarity distribution
        let similarity_distribution = self.analyze_similarity_distribution(titles)?;

        Ok(ContentCharacteristics {
            document_count,
            vocabulary_size,
            average_document_length,
            vocabulary_diversity,
            topic_coherence_estimate,
            similarity_distribution,
        })
    }

    /// Estimate topic coherence in the content
    fn estimate_topic_coherence(&self, titles: &[String]) -> f32 {
        if titles.len() < 2 {
            return 0.5;
        }

        // Simple heuristic: count repeated words across documents
        let mut word_counts = HashMap::new();
        let mut _total_words = 0;

        for title in titles {
            let words: Vec<String> = title
                .to_lowercase()
                .split_whitespace()
                .map(|w| w.chars().filter(|c| c.is_alphabetic()).collect())
                .filter(|w: &String| w.len() > 2)
                .collect();

            for word in words {
                *word_counts.entry(word).or_insert(0) += 1;
                _total_words += 1;
            }
        }

        // Calculate coherence as the proportion of words that appear multiple times
        let repeated_words = word_counts.values().filter(|&&count| count > 1).count();
        let unique_words = word_counts.len();

        if unique_words > 0 {
            repeated_words as f32 / unique_words as f32
        } else {
            0.0
        }
    }

    /// Analyze the distribution of pairwise similarities
    fn analyze_similarity_distribution(
        &self,
        titles: &[String],
    ) -> Result<SimilarityDistribution, ClusteringError> {
        if titles.len() < 2 {
            return Ok(SimilarityDistribution {
                mean: 0.5,
                std_dev: 0.0,
                min: 0.5,
                max: 0.5,
                has_clear_clusters: false,
            });
        }

        // Use TF-IDF to calculate similarities
        let analysis = self.kmeans_clusterer.analyze_content(titles)?;
        let features = &analysis.feature_vectors;

        let mut similarities = Vec::new();
        for i in 0..features.len() {
            for j in (i + 1)..features.len() {
                let similarity = features[i].cosine_similarity(&features[j]);
                similarities.push(similarity);
            }
        }

        if similarities.is_empty() {
            return Ok(SimilarityDistribution {
                mean: 0.5,
                std_dev: 0.0,
                min: 0.5,
                max: 0.5,
                has_clear_clusters: false,
            });
        }

        // Calculate statistics
        let mean = similarities.iter().sum::<f32>() / similarities.len() as f32;
        let variance = similarities
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>()
            / similarities.len() as f32;
        let std_dev = variance.sqrt();
        let min = similarities.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = similarities
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        // Determine if there are clear clusters (bimodal distribution)
        let has_clear_clusters = std_dev > 0.2 && (max - min) > 0.4;

        Ok(SimilarityDistribution {
            mean,
            std_dev,
            min,
            max,
            has_clear_clusters,
        })
    }

    /// Select which algorithms to use based on content characteristics
    fn select_algorithms(&self, characteristics: &ContentCharacteristics) -> (bool, bool, bool) {
        match self.strategy_selection {
            StrategySelection::Custom(kmeans, hierarchical, lda) => (kmeans, hierarchical, lda),
            StrategySelection::Ensemble => (true, true, true),
            StrategySelection::Automatic => {
                let use_kmeans = characteristics.document_count >= 10
                    && characteristics.similarity_distribution.has_clear_clusters;

                let use_hierarchical = characteristics.document_count <= 50
                    && characteristics.similarity_distribution.std_dev > 0.15;

                let use_lda = characteristics.document_count >= 8
                    && characteristics.vocabulary_diversity > 0.3
                    && characteristics.topic_coherence_estimate > 0.2;

                // Ensure at least one algorithm is selected
                if !use_kmeans && !use_hierarchical && !use_lda {
                    // Default to k-means for general cases
                    (true, false, false)
                } else {
                    (use_kmeans, use_hierarchical, use_lda)
                }
            }
        }
    }

    /// Run K-means clustering and return results with quality score
    fn run_kmeans_clustering(
        &self,
        titles: &[String],
    ) -> Result<(Vec<VideoCluster>, f32), ClusteringError> {
        let analysis = self.kmeans_clusterer.analyze_content(titles)?;
        let optimal_k = self
            .kmeans_clusterer
            .determine_optimal_k(&analysis.feature_vectors);
        let clusters = self.kmeans_clusterer.cluster_videos(&analysis, optimal_k)?;

        // Calculate quality score
        let quality = self.calculate_clustering_quality(&clusters, &analysis.feature_vectors);

        Ok((clusters, quality))
    }

    /// Run hierarchical clustering and return results with quality score
    fn run_hierarchical_clustering(
        &self,
        titles: &[String],
    ) -> Result<(Vec<VideoCluster>, f32), ClusteringError> {
        let analysis = self.hierarchical_clusterer.analyze_content(titles)?;
        let clusters = self.hierarchical_clusterer.cluster_videos(&analysis, 0)?;

        // Calculate quality score
        let quality = self.calculate_clustering_quality(&clusters, &analysis.feature_vectors);

        Ok((clusters, quality))
    }

    /// Run LDA clustering and return results with quality score
    fn run_lda_clustering(
        &self,
        titles: &[String],
    ) -> Result<(Vec<VideoCluster>, f32), ClusteringError> {
        let optimal_topics = self.lda_clusterer.determine_optimal_topics(titles);
        let mut lda_clusterer = self.lda_clusterer.clone();
        lda_clusterer.num_topics = optimal_topics;

        let lda_model = lda_clusterer.fit_lda(titles)?;
        let clusters = lda_clusterer.cluster_by_topics(&lda_model, 0.3)?;

        // Calculate quality score based on topic coherence
        let quality = self.calculate_lda_quality(&lda_model);

        Ok((clusters, quality))
    }

    /// Calculate clustering quality score
    fn calculate_clustering_quality(
        &self,
        clusters: &[VideoCluster],
        features: &[FeatureVector],
    ) -> f32 {
        if clusters.is_empty() || features.is_empty() {
            return 0.0;
        }

        // Calculate silhouette-like score
        let mut total_score = 0.0;
        let mut total_points = 0;

        for cluster in clusters {
            if cluster.videos.len() < 2 {
                continue;
            }

            // Intra-cluster similarity
            let mut intra_similarity = 0.0;
            let mut intra_count = 0;

            for &i in &cluster.videos {
                for &j in &cluster.videos {
                    if i != j && i < features.len() && j < features.len() {
                        intra_similarity += features[i].cosine_similarity(&features[j]);
                        intra_count += 1;
                    }
                }
            }

            if intra_count > 0 {
                intra_similarity /= intra_count as f32;
                total_score += intra_similarity * cluster.videos.len() as f32;
                total_points += cluster.videos.len();
            }
        }

        if total_points > 0 {
            total_score / total_points as f32
        } else {
            0.0
        }
    }

    /// Calculate LDA quality score based on topic coherence
    fn calculate_lda_quality(&self, lda_model: &super::lda::LdaModel) -> f32 {
        if lda_model.topics.is_empty() {
            return 0.0;
        }

        // Calculate average topic coherence
        let mut total_coherence = 0.0;

        for topic in &lda_model.topics {
            // Simple coherence measure: entropy of word probabilities
            let mut entropy = 0.0;
            for &prob in topic.word_probabilities.values() {
                if prob > 0.0 {
                    entropy -= prob * prob.ln();
                }
            }

            // Normalize entropy to [0, 1] range
            let max_entropy = (topic.word_probabilities.len() as f32).ln();
            let normalized_entropy = if max_entropy > 0.0 {
                entropy / max_entropy
            } else {
                0.0
            };

            total_coherence += normalized_entropy;
        }

        total_coherence / lda_model.topics.len() as f32
    }

    /// Combine results from multiple algorithms using the specified ensemble method
    fn combine_results(
        &self,
        results: &mut EnsembleResults,
        _characteristics: &ContentCharacteristics,
    ) -> Result<(), ClusteringError> {
        match self.ensemble_method {
            EnsembleMethod::BestQuality => self.select_best_quality_result(results),
            EnsembleMethod::Voting => self.combine_by_voting(results),
            EnsembleMethod::WeightedCombination => self.combine_by_weighted_average(results),
        }
    }

    /// Select the result with the best quality score
    fn select_best_quality_result(
        &self,
        results: &mut EnsembleResults,
    ) -> Result<(), ClusteringError> {
        let mut best_algorithm = String::new();
        let mut best_score = 0.0;
        let mut best_clusters = Vec::new();

        // Find the algorithm with the highest quality score
        for (algorithm, &score) in &results.quality_scores {
            if score > best_score {
                best_score = score;
                best_algorithm = algorithm.clone();

                best_clusters = match algorithm.as_str() {
                    "kmeans" => results.kmeans_result.clone().unwrap_or_default(),
                    "hierarchical" => results.hierarchical_result.clone().unwrap_or_default(),
                    "lda" => results.lda_result.clone().unwrap_or_default(),
                    _ => Vec::new(),
                };
            }
        }

        // If no algorithm produced good results, use the first available
        if best_clusters.is_empty() {
            if let Some(ref clusters) = results.kmeans_result {
                best_clusters = clusters.clone();
                best_algorithm = "kmeans".to_string();
            } else if let Some(ref clusters) = results.hierarchical_result {
                best_clusters = clusters.clone();
                best_algorithm = "hierarchical".to_string();
            } else if let Some(ref clusters) = results.lda_result {
                best_clusters = clusters.clone();
                best_algorithm = "lda".to_string();
            }
        }

        results.selected_algorithm = best_algorithm;
        results.final_clusters = best_clusters;

        Ok(())
    }

    /// Combine results using voting (majority consensus)
    fn combine_by_voting(&self, results: &mut EnsembleResults) -> Result<(), ClusteringError> {
        // For now, fall back to best quality selection
        // A full voting implementation would require more complex consensus logic
        self.select_best_quality_result(results)
    }

    /// Combine results using weighted average based on quality scores
    fn combine_by_weighted_average(
        &self,
        results: &mut EnsembleResults,
    ) -> Result<(), ClusteringError> {
        // For now, fall back to best quality selection
        // A full weighted combination would require merging cluster assignments
        self.select_best_quality_result(results)
    }

    /// Get algorithm recommendations based on content characteristics
    pub fn get_algorithm_recommendations(
        &self,
        characteristics: &ContentCharacteristics,
    ) -> Vec<(String, f32, String)> {
        let mut recommendations = Vec::new();

        // K-means recommendation
        let kmeans_score = if characteristics.document_count >= 10
            && characteristics.similarity_distribution.has_clear_clusters
        {
            0.8
        } else if characteristics.document_count >= 5 {
            0.6
        } else {
            0.3
        };
        recommendations.push((
            "K-means".to_string(),
            kmeans_score,
            "Good for well-separated clusters with similar sizes".to_string(),
        ));

        // Hierarchical recommendation
        let hierarchical_score = if characteristics.document_count <= 50
            && characteristics.similarity_distribution.std_dev > 0.15
        {
            0.8
        } else if characteristics.document_count <= 30 {
            0.6
        } else {
            0.4
        };
        recommendations.push((
            "Hierarchical".to_string(),
            hierarchical_score,
            "Good for discovering natural hierarchies and nested clusters".to_string(),
        ));

        // LDA recommendation
        let lda_score = if characteristics.document_count >= 8
            && characteristics.vocabulary_diversity > 0.3
            && characteristics.topic_coherence_estimate > 0.2
        {
            0.9
        } else if characteristics.vocabulary_diversity > 0.2 {
            0.6
        } else {
            0.3
        };
        recommendations.push((
            "LDA".to_string(),
            lda_score,
            "Good for discovering latent topics in text content".to_string(),
        ));

        // Sort by score
        recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        recommendations
    }
}

impl ContentClusterer for HybridClusterer {
    fn analyze_content(&self, titles: &[String]) -> Result<ContentAnalysis, ClusteringError> {
        // Use the most appropriate analyzer based on content characteristics
        let characteristics = self.analyze_content_characteristics(titles)?;

        if characteristics.vocabulary_diversity > 0.3
            && characteristics.topic_coherence_estimate > 0.2
        {
            // Use TF-IDF for diverse content
            self.kmeans_clusterer.analyze_content(titles)
        } else {
            // Use simpler analysis for homogeneous content
            self.hierarchical_clusterer.analyze_content(titles)
        }
    }

    fn cluster_videos(
        &self,
        _analysis: &ContentAnalysis,
        _target_clusters: usize,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        // This method is not directly used for hybrid clustering
        // Instead, use cluster_hybrid for full hybrid functionality
        Err(ClusteringError::AnalysisFailed(
            "Use cluster_hybrid for hybrid clustering".to_string(),
        ))
    }

    fn optimize_clusters(
        &self,
        clusters: Vec<VideoCluster>,
        durations: &[Duration],
    ) -> Result<Vec<OptimizedCluster>, ClusteringError> {
        // Use the k-means optimizer as it's the most general
        self.kmeans_clusterer.optimize_clusters(clusters, durations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_clusterer_creation() {
        let clusterer = HybridClusterer::default();
        assert_eq!(clusterer.strategy_selection, StrategySelection::Automatic);
        assert_eq!(clusterer.ensemble_method, EnsembleMethod::BestQuality);
        assert_eq!(clusterer.quality_threshold, 0.6);
    }

    #[test]
    fn test_hybrid_clusterer_custom_creation() {
        let clusterer = HybridClusterer::new(
            StrategySelection::Custom(true, false, true),
            EnsembleMethod::Voting,
            0.8,
        );
        assert_eq!(
            clusterer.strategy_selection,
            StrategySelection::Custom(true, false, true)
        );
        assert_eq!(clusterer.ensemble_method, EnsembleMethod::Voting);
        assert_eq!(clusterer.quality_threshold, 0.8);
    }

    #[test]
    fn test_content_characteristics_analysis() {
        let clusterer = HybridClusterer::default();
        let titles = vec![
            "Introduction to Programming".to_string(),
            "Advanced Programming Techniques".to_string(),
            "Database Design Fundamentals".to_string(),
            "Database Optimization".to_string(),
            "Web Development Basics".to_string(),
        ];

        let result = clusterer.analyze_content_characteristics(&titles);
        assert!(result.is_ok());

        let characteristics = result.unwrap();
        assert_eq!(characteristics.document_count, 5);
        assert!(characteristics.vocabulary_size > 0);
        assert!(characteristics.average_document_length > 0.0);
        assert!(characteristics.vocabulary_diversity > 0.0);
    }

    #[test]
    fn test_algorithm_selection() {
        let clusterer = HybridClusterer::default();

        // Test automatic selection with different characteristics
        let small_characteristics = ContentCharacteristics {
            document_count: 3,
            vocabulary_size: 10,
            average_document_length: 2.0,
            vocabulary_diversity: 0.5,
            topic_coherence_estimate: 0.1,
            similarity_distribution: SimilarityDistribution {
                mean: 0.5,
                std_dev: 0.1,
                min: 0.4,
                max: 0.6,
                has_clear_clusters: false,
            },
        };

        let (use_kmeans, use_hierarchical, use_lda) =
            clusterer.select_algorithms(&small_characteristics);
        // Should default to k-means when no algorithm is clearly suitable
        assert!(use_kmeans || use_hierarchical || use_lda);
    }

    #[test]
    fn test_custom_algorithm_selection() {
        let clusterer = HybridClusterer::new(
            StrategySelection::Custom(true, false, true),
            EnsembleMethod::BestQuality,
            0.6,
        );

        let characteristics = ContentCharacteristics {
            document_count: 10,
            vocabulary_size: 50,
            average_document_length: 3.0,
            vocabulary_diversity: 0.4,
            topic_coherence_estimate: 0.3,
            similarity_distribution: SimilarityDistribution {
                mean: 0.6,
                std_dev: 0.2,
                min: 0.2,
                max: 0.9,
                has_clear_clusters: true,
            },
        };

        let (use_kmeans, use_hierarchical, use_lda) = clusterer.select_algorithms(&characteristics);
        assert!(use_kmeans);
        assert!(!use_hierarchical);
        assert!(use_lda);
    }

    #[test]
    fn test_algorithm_recommendations() {
        let clusterer = HybridClusterer::default();
        let characteristics = ContentCharacteristics {
            document_count: 20,
            vocabulary_size: 100,
            average_document_length: 4.0,
            vocabulary_diversity: 0.4,
            topic_coherence_estimate: 0.3,
            similarity_distribution: SimilarityDistribution {
                mean: 0.6,
                std_dev: 0.2,
                min: 0.2,
                max: 0.9,
                has_clear_clusters: true,
            },
        };

        let recommendations = clusterer.get_algorithm_recommendations(&characteristics);
        assert_eq!(recommendations.len(), 3);

        // Should be sorted by score
        for i in 1..recommendations.len() {
            assert!(recommendations[i - 1].1 >= recommendations[i].1);
        }
    }

    #[test]
    fn test_hybrid_clustering() {
        let clusterer = HybridClusterer::new(
            StrategySelection::Custom(true, false, false), // Only k-means for simplicity
            EnsembleMethod::BestQuality,
            0.6,
        );

        let titles = vec![
            "Programming Basics".to_string(),
            "Programming Fundamentals".to_string(),
            "Database Design".to_string(),
            "Database Optimization".to_string(),
            "Web Development".to_string(),
            "Web Frontend".to_string(),
        ];

        let result = clusterer.cluster_hybrid(&titles);
        assert!(result.is_ok());

        let ensemble_results = result.unwrap();
        assert!(!ensemble_results.final_clusters.is_empty());
        assert!(!ensemble_results.selected_algorithm.is_empty());
    }
}
