//! TF-IDF based content similarity analysis for video clustering
//!
//! This module implements Term Frequency-Inverse Document Frequency (TF-IDF) analysis
//! to extract semantic features from video titles and calculate content similarity.

use super::{ClusteringError, ContentClusterer, OptimizedCluster, VideoCluster};
use crate::nlp::normalize_text;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// Feature vector representation for TF-IDF analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureVector {
    pub features: HashMap<String, f32>,
    pub magnitude: f32,
}

impl FeatureVector {
    /// Create a new feature vector from term frequencies
    pub fn new(features: HashMap<String, f32>) -> Self {
        let magnitude = features.values().map(|&v| v * v).sum::<f32>().sqrt();
        Self { features, magnitude }
    }

    /// Calculate cosine similarity with another feature vector
    pub fn cosine_similarity(&self, other: &FeatureVector) -> f32 {
        if self.magnitude == 0.0 || other.magnitude == 0.0 {
            return 0.0;
        }

        let dot_product: f32 = self
            .features
            .iter()
            .filter_map(|(term, &value)| {
                other.features.get(term).map(|&other_value| value * other_value)
            })
            .sum();

        dot_product / (self.magnitude * other.magnitude)
    }

    /// Get the top N most significant features
    pub fn top_features(&self, n: usize) -> Vec<(String, f32)> {
        let mut features: Vec<_> =
            self.features.iter().map(|(term, &score)| (term.clone(), score)).collect();
        features.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        features.into_iter().take(n).collect()
    }
}

/// Similarity matrix for pairwise comparisons
#[derive(Debug, Clone)]
pub struct SimilarityMatrix {
    pub matrix: Vec<Vec<f32>>,
    pub size: usize,
}

impl SimilarityMatrix {
    /// Create a new similarity matrix
    pub fn new(size: usize) -> Self {
        Self { matrix: vec![vec![0.0; size]; size], size }
    }

    /// Set similarity score between two items
    pub fn set(&mut self, i: usize, j: usize, similarity: f32) {
        if i < self.size && j < self.size {
            self.matrix[i][j] = similarity;
            self.matrix[j][i] = similarity; // Symmetric matrix
        }
    }

    /// Get similarity score between two items
    pub fn get(&self, i: usize, j: usize) -> f32 {
        if i < self.size && j < self.size { self.matrix[i][j] } else { 0.0 }
    }

    /// Find the most similar items to a given item
    pub fn most_similar(&self, item: usize, threshold: f32) -> Vec<(usize, f32)> {
        if item >= self.size {
            return Vec::new();
        }

        let mut similar: Vec<_> = self.matrix[item]
            .iter()
            .enumerate()
            .filter(|&(i, &score)| i != item && score >= threshold)
            .map(|(i, &score)| (i, score))
            .collect();

        similar.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similar
    }

    /// Calculate average similarity for clustering quality assessment
    pub fn average_similarity(&self) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;

        for i in 0..self.size {
            for j in (i + 1)..self.size {
                sum += self.matrix[i][j];
                count += 1;
            }
        }

        if count > 0 { sum / count as f32 } else { 0.0 }
    }
}

/// Content analysis results
#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub feature_vectors: Vec<FeatureVector>,
    pub similarity_matrix: SimilarityMatrix,
    pub vocabulary: HashSet<String>,
    pub document_frequencies: HashMap<String, usize>,
    pub topic_keywords: Vec<String>,
}

/// TF-IDF analyzer implementation
pub struct TfIdfAnalyzer {
    pub min_similarity_threshold: f32,
    pub max_features: usize,
    pub stop_words: HashSet<String>,
    pub min_term_frequency: usize,
}

impl Default for TfIdfAnalyzer {
    fn default() -> Self {
        Self {
            min_similarity_threshold: 0.3,
            max_features: 1000,
            stop_words: Self::default_stop_words(),
            min_term_frequency: 1,
        }
    }
}

impl TfIdfAnalyzer {
    /// Create a new TF-IDF analyzer with custom parameters
    pub fn new(
        min_similarity_threshold: f32,
        max_features: usize,
        min_term_frequency: usize,
    ) -> Self {
        Self {
            min_similarity_threshold,
            max_features,
            stop_words: Self::default_stop_words(),
            min_term_frequency,
        }
    }

    /// Default English stop words for filtering
    fn default_stop_words() -> HashSet<String> {
        [
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in",
            "is", "it", "its", "of", "on", "that", "the", "to", "was", "will", "with", "the",
            "this", "but", "they", "have", "had", "what", "said", "each", "which", "she", "do",
            "how", "their", "if", "up", "out", "many", "then", "them", "these", "so", "some",
            "her", "would", "make", "like", "into", "him", "time", "two", "more", "go", "no",
            "way", "could", "my", "than", "first", "been", "call", "who", "oil", "sit", "now",
            "find", "down", "day", "did", "get", "come", "made", "may", "part",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect()
    }

    /// Preprocess text by tokenizing, normalizing, and removing stop words
    fn preprocess_text(&self, text: &str) -> Vec<String> {
        let normalized = normalize_text(text);
        normalized
            .split_whitespace()
            .filter(|word| {
                word.len() > 2
                    && !self.stop_words.contains(*word)
                    && word.chars().any(|c| c.is_alphabetic())
            })
            .map(|word| word.to_string())
            .collect()
    }

    /// Calculate term frequencies for a document
    fn calculate_term_frequencies(&self, tokens: &[String]) -> HashMap<String, f32> {
        let mut tf = HashMap::new();
        let total_terms = tokens.len() as f32;

        for token in tokens {
            *tf.entry(token.clone()).or_insert(0.0) += 1.0;
        }

        // Normalize by document length
        for value in tf.values_mut() {
            *value /= total_terms;
        }

        tf
    }

    /// Calculate document frequencies across all documents
    fn calculate_document_frequencies(&self, all_tokens: &[Vec<String>]) -> HashMap<String, usize> {
        let mut df = HashMap::new();

        for tokens in all_tokens {
            let unique_tokens: HashSet<_> = tokens.iter().collect();
            for token in unique_tokens {
                *df.entry(token.clone()).or_insert(0) += 1;
            }
        }

        // Filter by minimum frequency
        df.into_iter().filter(|(_, freq)| *freq >= self.min_term_frequency).collect()
    }

    /// Calculate TF-IDF scores for all documents
    fn calculate_tfidf_vectors(
        &self,
        all_tokens: &[Vec<String>],
        document_frequencies: &HashMap<String, usize>,
    ) -> Vec<FeatureVector> {
        let num_documents = all_tokens.len() as f32;
        let mut vectors = Vec::new();

        for tokens in all_tokens {
            let tf = self.calculate_term_frequencies(tokens);
            let mut tfidf_features = HashMap::new();

            for (term, tf_score) in tf {
                if let Some(&df) = document_frequencies.get(&term) {
                    let idf = (num_documents / df as f32).ln();
                    let tfidf = tf_score * idf;
                    tfidf_features.insert(term, tfidf);
                }
            }

            // Limit features if necessary
            if tfidf_features.len() > self.max_features {
                let mut features: Vec<_> = tfidf_features.into_iter().collect();
                features.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                tfidf_features = features.into_iter().take(self.max_features).collect();
            }

            vectors.push(FeatureVector::new(tfidf_features));
        }

        vectors
    }

    /// Extract feature vectors from titles
    pub fn extract_features(&self, titles: &[String]) -> Vec<FeatureVector> {
        let all_tokens: Vec<_> = titles.iter().map(|title| self.preprocess_text(title)).collect();

        let document_frequencies = self.calculate_document_frequencies(&all_tokens);
        self.calculate_tfidf_vectors(&all_tokens, &document_frequencies)
    }

    /// Calculate similarity matrix using cosine similarity
    pub fn calculate_similarity_matrix(&self, features: &[FeatureVector]) -> SimilarityMatrix {
        let mut matrix = SimilarityMatrix::new(features.len());

        for i in 0..features.len() {
            for j in (i + 1)..features.len() {
                let similarity = features[i].cosine_similarity(&features[j]);
                matrix.set(i, j, similarity);
            }
        }

        matrix
    }

    /// Identify topic keywords from TF-IDF features
    pub fn identify_topic_keywords(&self, titles: &[String]) -> HashMap<String, f32> {
        let all_tokens: Vec<_> = titles.iter().map(|title| self.preprocess_text(title)).collect();

        let document_frequencies = self.calculate_document_frequencies(&all_tokens);
        let feature_vectors = self.calculate_tfidf_vectors(&all_tokens, &document_frequencies);

        let mut global_scores = HashMap::new();

        // Aggregate TF-IDF scores across all documents
        for vector in &feature_vectors {
            for (term, &score) in &vector.features {
                *global_scores.entry(term.clone()).or_insert(0.0) += score;
            }
        }

        // Normalize by number of documents
        let num_docs = feature_vectors.len() as f32;
        for score in global_scores.values_mut() {
            *score /= num_docs;
        }

        global_scores
    }
}

impl ContentClusterer for TfIdfAnalyzer {
    fn analyze_content(&self, titles: &[String]) -> Result<ContentAnalysis, ClusteringError> {
        if titles.len() < 5 {
            return Err(ClusteringError::InsufficientContent(titles.len()));
        }

        let all_tokens: Vec<_> = titles.iter().map(|title| self.preprocess_text(title)).collect();

        let document_frequencies = self.calculate_document_frequencies(&all_tokens);
        let feature_vectors = self.calculate_tfidf_vectors(&all_tokens, &document_frequencies);
        let similarity_matrix = self.calculate_similarity_matrix(&feature_vectors);

        let vocabulary: HashSet<String> = document_frequencies.keys().cloned().collect();

        let topic_keywords = self.identify_topic_keywords(titles);
        let mut sorted_keywords: Vec<_> = topic_keywords.into_iter().collect();
        sorted_keywords.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let topic_keywords: Vec<String> =
            sorted_keywords.into_iter().take(10).map(|(keyword, _)| keyword).collect();

        Ok(ContentAnalysis {
            feature_vectors,
            similarity_matrix,
            vocabulary,
            document_frequencies,
            topic_keywords,
        })
    }

    fn cluster_videos(
        &self,
        analysis: &ContentAnalysis,
        target_clusters: usize,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        // Simple similarity-based clustering
        let mut clusters = Vec::new();
        let mut assigned = vec![false; analysis.feature_vectors.len()];

        for i in 0..analysis.feature_vectors.len() {
            if assigned[i] {
                continue;
            }

            let mut cluster_videos = vec![i];
            assigned[i] = true;

            // Find similar videos
            let similar = analysis.similarity_matrix.most_similar(i, self.min_similarity_threshold);
            for (j, _similarity) in similar {
                if !assigned[j] && cluster_videos.len() < target_clusters {
                    cluster_videos.push(j);
                    assigned[j] = true;
                }
            }

            // Calculate cluster centroid (average of feature vectors)
            let centroid = self.calculate_centroid(&analysis.feature_vectors, &cluster_videos);

            // Calculate average similarity within cluster
            let similarity_score = if cluster_videos.len() > 1 {
                let mut total_similarity = 0.0;
                let mut count = 0;
                for &a in &cluster_videos {
                    for &b in &cluster_videos {
                        if a != b {
                            total_similarity += analysis.similarity_matrix.get(a, b);
                            count += 1;
                        }
                    }
                }
                if count > 0 { total_similarity / count as f32 } else { 0.0 }
            } else {
                1.0
            };

            // Extract topic keywords for this cluster
            let topic_keywords =
                centroid.top_features(5).into_iter().map(|(keyword, _)| keyword).collect();

            clusters.push(VideoCluster {
                videos: cluster_videos,
                centroid,
                similarity_score,
                topic_keywords,
            });
        }

        Ok(clusters)
    }

    fn optimize_clusters(
        &self,
        clusters: Vec<VideoCluster>,
        durations: &[Duration],
    ) -> Result<Vec<OptimizedCluster>, ClusteringError> {
        if durations.len() != clusters.iter().map(|c| c.videos.len()).sum::<usize>() {
            return Err(ClusteringError::InvalidDurations(durations.len()));
        }

        let mut optimized = Vec::new();

        for cluster in clusters {
            let videos: Vec<_> = cluster
                .videos
                .into_iter()
                .map(|index| super::VideoWithMetadata {
                    index,
                    title: format!("Video {index}"), // This will be populated by caller
                    duration: durations.get(index).copied().unwrap_or_default(),
                    feature_vector: cluster.centroid.clone(),
                    difficulty_score: 0.5, // Will be calculated by caller
                    topic_tags: cluster.topic_keywords.clone(),
                })
                .collect();

            let total_duration = videos.iter().map(|v| v.duration).sum();
            let suggested_title = if cluster.topic_keywords.is_empty() {
                format!("Module {}", optimized.len() + 1)
            } else {
                cluster.topic_keywords[0].clone()
            };

            optimized.push(OptimizedCluster {
                videos,
                total_duration,
                average_similarity: cluster.similarity_score,
                difficulty_level: crate::types::DifficultyLevel::Intermediate, // Will be calculated
                suggested_title,
            });
        }

        Ok(optimized)
    }
}

impl TfIdfAnalyzer {
    /// Calculate centroid of feature vectors
    fn calculate_centroid(&self, vectors: &[FeatureVector], indices: &[usize]) -> FeatureVector {
        if indices.is_empty() {
            return FeatureVector::default();
        }

        let mut centroid_features = HashMap::new();

        // Sum all features
        for &index in indices {
            if let Some(vector) = vectors.get(index) {
                for (term, &value) in &vector.features {
                    *centroid_features.entry(term.clone()).or_insert(0.0) += value;
                }
            }
        }

        // Average the features
        let count = indices.len() as f32;
        for value in centroid_features.values_mut() {
            *value /= count;
        }

        FeatureVector::new(centroid_features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector_cosine_similarity() {
        let mut features1 = HashMap::new();
        features1.insert("hello".to_string(), 1.0);
        features1.insert("world".to_string(), 1.0);
        let vec1 = FeatureVector::new(features1);

        let mut features2 = HashMap::new();
        features2.insert("hello".to_string(), 1.0);
        features2.insert("world".to_string(), 1.0);
        let vec2 = FeatureVector::new(features2);

        assert!((vec1.cosine_similarity(&vec2) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_text_preprocessing() {
        let analyzer = TfIdfAnalyzer::default();
        let tokens = analyzer.preprocess_text("Hello, World! This is a test.");

        assert!(!tokens.contains(&"is".to_string())); // Stop word removed
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }

    #[test]
    fn test_similarity_matrix() {
        let mut matrix = SimilarityMatrix::new(3);
        matrix.set(0, 1, 0.8);
        matrix.set(0, 2, 0.3);
        matrix.set(1, 2, 0.6);

        assert_eq!(matrix.get(0, 1), 0.8);
        assert_eq!(matrix.get(1, 0), 0.8); // Symmetric

        let similar = matrix.most_similar(0, 0.5);
        assert_eq!(similar.len(), 1);
        assert_eq!(similar[0].0, 1);
    }

    #[test]
    fn test_content_analysis() {
        let analyzer = TfIdfAnalyzer::default();
        let titles = vec![
            "Introduction to Programming".to_string(),
            "Advanced Programming Concepts".to_string(),
            "Data Structures and Algorithms".to_string(),
            "Database Design Fundamentals".to_string(),
            "Web Development Basics".to_string(),
        ];

        let result = analyzer.analyze_content(&titles);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.feature_vectors.len(), 5);
        assert_eq!(analysis.similarity_matrix.size, 5);
        assert!(!analysis.topic_keywords.is_empty());
    }

    #[test]
    fn test_insufficient_content_error() {
        let analyzer = TfIdfAnalyzer::default();
        let titles = vec!["Title 1".to_string(), "Title 2".to_string()];

        let result = analyzer.analyze_content(&titles);
        assert!(matches!(result, Err(ClusteringError::InsufficientContent(2))));
    }
}
