//! Latent Dirichlet Allocation (LDA) topic modeling for video content analysis
//!
//! This module provides LDA-based topic modeling to discover latent topics
//! in video titles and group content based on topic distributions.

use super::{
    ClusteringError, ContentAnalysis, ContentClusterer, FeatureVector, OptimizedCluster,
    VideoCluster, VideoWithMetadata,
};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

/// LDA topic modeling implementation
#[derive(Clone)]
pub struct LdaClusterer {
    pub num_topics: usize,
    pub alpha: f32, // Document-topic concentration
    pub beta: f32,  // Topic-word concentration
    pub num_iterations: usize,
    pub random_seed: Option<u64>,
    pub min_topic_probability: f32,
}

/// Topic representation with word probabilities
#[derive(Debug, Clone)]
pub struct Topic {
    pub id: usize,
    pub word_probabilities: HashMap<String, f32>,
    pub top_words: Vec<(String, f32)>,
}

/// Document-topic distribution
#[derive(Debug, Clone)]
pub struct DocumentTopics {
    pub document_id: usize,
    pub topic_probabilities: Vec<f32>,
    pub dominant_topic: usize,
}

/// LDA model results
#[derive(Debug, Clone)]
pub struct LdaModel {
    pub topics: Vec<Topic>,
    pub document_topics: Vec<DocumentTopics>,
    pub vocabulary: Vec<String>,
    pub log_likelihood: f32,
}

impl Default for LdaClusterer {
    fn default() -> Self {
        Self {
            num_topics: 5,
            alpha: 0.1,
            beta: 0.01,
            num_iterations: 100,
            random_seed: None,
            min_topic_probability: 0.1,
        }
    }
}

impl LdaClusterer {
    /// Create a new LDA clusterer with custom parameters
    pub fn new(
        num_topics: usize,
        alpha: f32,
        beta: f32,
        num_iterations: usize,
        random_seed: Option<u64>,
    ) -> Self {
        Self {
            num_topics,
            alpha,
            beta,
            num_iterations,
            random_seed,
            min_topic_probability: 0.1,
        }
    }

    /// Perform LDA topic modeling on documents (video titles)
    pub fn fit_lda(&self, documents: &[String]) -> Result<LdaModel, ClusteringError> {
        if documents.is_empty() {
            return Err(ClusteringError::InsufficientContent(0));
        }

        if documents.len() < self.num_topics {
            return Err(ClusteringError::InsufficientContent(documents.len()));
        }

        // Preprocess documents and build vocabulary
        let (processed_docs, vocabulary) = self.preprocess_documents(documents);

        // Initialize topic assignments randomly
        let mut topic_assignments = self.initialize_topic_assignments(&processed_docs);

        // Count matrices for Gibbs sampling
        let mut doc_topic_counts = vec![vec![0; self.num_topics]; processed_docs.len()];
        let mut topic_word_counts = vec![vec![0; vocabulary.len()]; self.num_topics];
        let mut topic_counts = vec![0; self.num_topics];

        // Initialize counts
        self.initialize_counts(
            &processed_docs,
            &topic_assignments,
            &mut doc_topic_counts,
            &mut topic_word_counts,
            &mut topic_counts,
        );

        // Gibbs sampling
        for iteration in 0..self.num_iterations {
            self.gibbs_sampling_iteration(
                &processed_docs,
                &mut topic_assignments,
                &mut doc_topic_counts,
                &mut topic_word_counts,
                &mut topic_counts,
                &vocabulary,
                iteration,
            )?;
        }

        // Extract topics and document-topic distributions
        let topics = self.extract_topics(&topic_word_counts, &vocabulary, &topic_counts);
        let document_topics = self.extract_document_topics(&doc_topic_counts);

        // Calculate log likelihood
        let log_likelihood = self.calculate_log_likelihood(
            &processed_docs,
            &topic_assignments,
            &doc_topic_counts,
            &topic_word_counts,
            &topic_counts,
        );

        Ok(LdaModel {
            topics,
            document_topics,
            vocabulary,
            log_likelihood,
        })
    }

    /// Cluster videos based on LDA topic modeling
    pub fn cluster_by_topics(
        &self,
        lda_model: &LdaModel,
        similarity_threshold: f32,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        let mut clusters = Vec::new();

        // Group documents by dominant topic
        let mut topic_groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for doc_topics in &lda_model.document_topics {
            topic_groups
                .entry(doc_topics.dominant_topic)
                .or_default()
                .push(doc_topics.document_id);
        }

        // Create clusters from topic groups
        for (topic_id, video_indices) in topic_groups {
            if video_indices.is_empty() {
                continue;
            }

            // Calculate cluster centroid from topic distribution
            let topic = &lda_model.topics[topic_id];
            let centroid = self.topic_to_feature_vector(topic);

            // Calculate similarity score within cluster
            let similarity_score = self.calculate_cluster_similarity(
                &video_indices,
                &lda_model.document_topics,
                similarity_threshold,
            );

            // Extract topic keywords
            let topic_keywords = topic
                .top_words
                .iter()
                .take(5)
                .map(|(word, _)| word.clone())
                .collect();

            clusters.push(VideoCluster {
                videos: video_indices,
                centroid,
                similarity_score,
                topic_keywords,
            });
        }

        // Merge small clusters or reassign videos if needed
        self.post_process_clusters(clusters, lda_model, similarity_threshold)
    }

    /// Preprocess documents and build vocabulary
    fn preprocess_documents(&self, documents: &[String]) -> (Vec<Vec<usize>>, Vec<String>) {
        let mut vocabulary = HashMap::new();
        let mut vocab_list = Vec::new();
        let mut processed_docs = Vec::new();

        // Build vocabulary and convert documents to word indices
        for document in documents {
            let words = self.tokenize_and_clean(document);
            let mut doc_indices = Vec::new();

            for word in words {
                let word_id = if let Some(&id) = vocabulary.get(&word) {
                    id
                } else {
                    let id = vocab_list.len();
                    vocabulary.insert(word.clone(), id);
                    vocab_list.push(word);
                    id
                };
                doc_indices.push(word_id);
            }

            processed_docs.push(doc_indices);
        }

        (processed_docs, vocab_list)
    }

    /// Tokenize and clean document text
    fn tokenize_and_clean(&self, text: &str) -> Vec<String> {
        let stop_words = self.get_stop_words();

        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                // Remove punctuation and numbers
                word.chars()
                    .filter(|c| c.is_alphabetic())
                    .collect::<String>()
            })
            .filter(|word| word.len() > 2 && !stop_words.contains(word))
            .collect()
    }

    /// Get common stop words
    fn get_stop_words(&self) -> std::collections::HashSet<String> {
        [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "can", "this", "that",
            "these", "those", "i", "you", "he", "she", "it", "we", "they", "me", "him", "her",
            "us", "them", "my", "your", "his", "her", "its", "our", "their", "video", "part",
            "episode", "tutorial", "lesson", "chapter", "section",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect()
    }

    /// Initialize topic assignments randomly
    fn initialize_topic_assignments(&self, processed_docs: &[Vec<usize>]) -> Vec<Vec<usize>> {
        let mut assignments = Vec::new();
        let mut rng_state = self.create_rng();

        for doc in processed_docs {
            let mut doc_assignments = Vec::new();
            for _ in doc {
                let topic = (rng_state % self.num_topics as u64) as usize;
                doc_assignments.push(topic);
                rng_state = self.next_random(rng_state);
            }
            assignments.push(doc_assignments);
        }

        assignments
    }

    /// Initialize count matrices
    fn initialize_counts(
        &self,
        processed_docs: &[Vec<usize>],
        topic_assignments: &[Vec<usize>],
        doc_topic_counts: &mut [Vec<usize>],
        topic_word_counts: &mut [Vec<usize>],
        topic_counts: &mut [usize],
    ) {
        for (doc_id, (doc, assignments)) in processed_docs
            .iter()
            .zip(topic_assignments.iter())
            .enumerate()
        {
            for (&word_id, &topic) in doc.iter().zip(assignments.iter()) {
                doc_topic_counts[doc_id][topic] += 1;
                topic_word_counts[topic][word_id] += 1;
                topic_counts[topic] += 1;
            }
        }
    }

    /// Perform one iteration of Gibbs sampling
    fn gibbs_sampling_iteration(
        &self,
        processed_docs: &[Vec<usize>],
        topic_assignments: &mut [Vec<usize>],
        doc_topic_counts: &mut [Vec<usize>],
        topic_word_counts: &mut [Vec<usize>],
        topic_counts: &mut [usize],
        vocabulary: &[String],
        iteration: usize,
    ) -> Result<(), ClusteringError> {
        let mut rng_state = self.create_rng() + iteration as u64;

        for (doc_id, (doc, assignments)) in processed_docs
            .iter()
            .zip(topic_assignments.iter_mut())
            .enumerate()
        {
            for (&word_id, topic) in doc.iter().zip(assignments.iter_mut()) {
                // Remove current assignment from counts
                doc_topic_counts[doc_id][*topic] -= 1;
                topic_word_counts[*topic][word_id] -= 1;
                topic_counts[*topic] -= 1;

                // Calculate probabilities for each topic
                let mut topic_probs = Vec::new();
                let mut total_prob = 0.0;

                for k in 0..self.num_topics {
                    // P(topic|doc) * P(word|topic)
                    let doc_topic_prob = (doc_topic_counts[doc_id][k] as f32 + self.alpha)
                        / (doc.len() as f32 + self.num_topics as f32 * self.alpha);

                    let topic_word_prob = (topic_word_counts[k][word_id] as f32 + self.beta)
                        / (topic_counts[k] as f32 + vocabulary.len() as f32 * self.beta);

                    let prob = doc_topic_prob * topic_word_prob;
                    topic_probs.push(prob);
                    total_prob += prob;
                }

                // Sample new topic
                let new_topic = if total_prob > 0.0 {
                    let mut cumulative = 0.0;
                    let target = (rng_state as f32 / u64::MAX as f32) * total_prob;
                    rng_state = self.next_random(rng_state);

                    let mut selected_topic = 0;
                    for (k, &prob) in topic_probs.iter().enumerate() {
                        cumulative += prob;
                        if cumulative >= target {
                            selected_topic = k;
                            break;
                        }
                    }
                    selected_topic
                } else {
                    (rng_state % self.num_topics as u64) as usize
                };

                // Update assignment and counts
                *topic = new_topic;
                doc_topic_counts[doc_id][new_topic] += 1;
                topic_word_counts[new_topic][word_id] += 1;
                topic_counts[new_topic] += 1;
            }
        }

        Ok(())
    }

    /// Extract topics from word-topic counts
    fn extract_topics(
        &self,
        topic_word_counts: &[Vec<usize>],
        vocabulary: &[String],
        topic_counts: &[usize],
    ) -> Vec<Topic> {
        let mut topics = Vec::new();

        for (topic_id, word_counts) in topic_word_counts.iter().enumerate() {
            let mut word_probabilities = HashMap::new();
            let mut word_prob_pairs = Vec::new();

            let total_count = topic_counts[topic_id] as f32;

            for (word_id, &count) in word_counts.iter().enumerate() {
                if let Some(word) = vocabulary.get(word_id) {
                    let prob = (count as f32 + self.beta)
                        / (total_count + vocabulary.len() as f32 * self.beta);
                    word_probabilities.insert(word.clone(), prob);
                    word_prob_pairs.push((word.clone(), prob));
                }
            }

            // Sort by probability and take top words
            word_prob_pairs
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let top_words = word_prob_pairs.into_iter().take(10).collect();

            topics.push(Topic {
                id: topic_id,
                word_probabilities,
                top_words,
            });
        }

        topics
    }

    /// Extract document-topic distributions
    fn extract_document_topics(&self, doc_topic_counts: &[Vec<usize>]) -> Vec<DocumentTopics> {
        let mut document_topics = Vec::new();

        for (doc_id, topic_counts) in doc_topic_counts.iter().enumerate() {
            let total_count: usize = topic_counts.iter().sum();
            let mut topic_probabilities = Vec::new();
            let mut max_prob = 0.0;
            let mut dominant_topic = 0;

            for (topic_id, &count) in topic_counts.iter().enumerate() {
                let prob = (count as f32 + self.alpha)
                    / (total_count as f32 + self.num_topics as f32 * self.alpha);
                topic_probabilities.push(prob);

                if prob > max_prob {
                    max_prob = prob;
                    dominant_topic = topic_id;
                }
            }

            document_topics.push(DocumentTopics {
                document_id: doc_id,
                topic_probabilities,
                dominant_topic,
            });
        }

        document_topics
    }

    /// Convert topic to feature vector for clustering
    fn topic_to_feature_vector(&self, topic: &Topic) -> FeatureVector {
        FeatureVector::new(topic.word_probabilities.clone())
    }

    /// Calculate similarity score within a cluster
    fn calculate_cluster_similarity(
        &self,
        video_indices: &[usize],
        document_topics: &[DocumentTopics],
        _similarity_threshold: f32,
    ) -> f32 {
        if video_indices.len() < 2 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for &i in video_indices {
            for &j in video_indices {
                if i != j {
                    if let (Some(doc_i), Some(doc_j)) =
                        (document_topics.get(i), document_topics.get(j))
                    {
                        let similarity = self.calculate_topic_similarity(
                            &doc_i.topic_probabilities,
                            &doc_j.topic_probabilities,
                        );
                        total_similarity += similarity;
                        count += 1;
                    }
                }
            }
        }

        if count > 0 {
            total_similarity / count as f32
        } else {
            1.0
        }
    }

    /// Calculate similarity between two topic distributions
    fn calculate_topic_similarity(&self, topics_a: &[f32], topics_b: &[f32]) -> f32 {
        if topics_a.len() != topics_b.len() {
            return 0.0;
        }

        // Use cosine similarity
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (a, b) in topics_a.iter().zip(topics_b.iter()) {
            dot_product += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }

        if norm_a > 0.0 && norm_b > 0.0 {
            dot_product / (norm_a.sqrt() * norm_b.sqrt())
        } else {
            0.0
        }
    }

    /// Post-process clusters to handle small clusters and improve quality
    fn post_process_clusters(
        &self,
        mut clusters: Vec<VideoCluster>,
        lda_model: &LdaModel,
        _similarity_threshold: f32,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        // Remove clusters that are too small
        clusters.retain(|cluster| cluster.videos.len() >= 2);

        // If we have very few clusters, try to split large ones
        if clusters.len() < 2 && !clusters.is_empty() {
            let large_cluster = clusters.remove(0);
            if large_cluster.videos.len() >= 4 {
                // Split based on secondary topic assignments
                let split_clusters =
                    self.split_cluster_by_secondary_topics(&large_cluster, lda_model)?;
                clusters.extend(split_clusters);
            } else {
                clusters.push(large_cluster);
            }
        }

        // Ensure we have at least one cluster
        if clusters.is_empty() {
            let all_videos: Vec<usize> = (0..lda_model.document_topics.len()).collect();
            let default_centroid = FeatureVector::default();
            clusters.push(VideoCluster {
                videos: all_videos,
                centroid: default_centroid,
                similarity_score: 0.5,
                topic_keywords: vec!["Mixed Content".to_string()],
            });
        }

        Ok(clusters)
    }

    /// Split a large cluster based on secondary topic assignments
    fn split_cluster_by_secondary_topics(
        &self,
        cluster: &VideoCluster,
        lda_model: &LdaModel,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        let mut secondary_groups: HashMap<usize, Vec<usize>> = HashMap::new();

        // Group by secondary topic (second highest probability)
        for &video_id in &cluster.videos {
            if let Some(doc_topics) = lda_model.document_topics.get(video_id) {
                let mut topic_probs: Vec<(usize, f32)> = doc_topics
                    .topic_probabilities
                    .iter()
                    .enumerate()
                    .map(|(i, &prob)| (i, prob))
                    .collect();

                topic_probs
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                let secondary_topic = if topic_probs.len() > 1 {
                    topic_probs[1].0
                } else {
                    topic_probs[0].0
                };

                secondary_groups
                    .entry(secondary_topic)
                    .or_default()
                    .push(video_id);
            }
        }

        // Create clusters from secondary groups
        let mut split_clusters = Vec::new();
        for (topic_id, video_indices) in secondary_groups {
            if video_indices.len() >= 2 {
                let topic = &lda_model.topics[topic_id];
                let centroid = self.topic_to_feature_vector(topic);
                let topic_keywords = topic
                    .top_words
                    .iter()
                    .take(5)
                    .map(|(word, _)| word.clone())
                    .collect();

                split_clusters.push(VideoCluster {
                    videos: video_indices,
                    centroid,
                    similarity_score: cluster.similarity_score,
                    topic_keywords,
                });
            }
        }

        if split_clusters.is_empty() {
            Ok(vec![cluster.clone()])
        } else {
            Ok(split_clusters)
        }
    }

    /// Calculate log likelihood of the model
    fn calculate_log_likelihood(
        &self,
        processed_docs: &[Vec<usize>],
        topic_assignments: &[Vec<usize>],
        doc_topic_counts: &[Vec<usize>],
        topic_word_counts: &[Vec<usize>],
        topic_counts: &[usize],
    ) -> f32 {
        let mut log_likelihood = 0.0;

        // Document-topic likelihood
        for (doc_id, doc) in processed_docs.iter().enumerate() {
            for &topic in &topic_assignments[doc_id] {
                let prob = (doc_topic_counts[doc_id][topic] as f32 + self.alpha)
                    / (doc.len() as f32 + self.num_topics as f32 * self.alpha);
                log_likelihood += prob.ln();
            }
        }

        // Topic-word likelihood
        for (doc, assignments) in processed_docs.iter().zip(topic_assignments.iter()) {
            for (&word_id, &topic) in doc.iter().zip(assignments.iter()) {
                let prob = (topic_word_counts[topic][word_id] as f32 + self.beta)
                    / (topic_counts[topic] as f32 + processed_docs.len() as f32 * self.beta);
                log_likelihood += prob.ln();
            }
        }

        log_likelihood
    }

    /// Simple random number generator
    fn create_rng(&self) -> u64 {
        if let Some(seed) = self.random_seed {
            seed
        } else {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .hash(&mut hasher);
            hasher.finish()
        }
    }

    /// Generate next random number
    fn next_random(&self, current: u64) -> u64 {
        // Simple linear congruential generator
        current.wrapping_mul(1103515245).wrapping_add(12345)
    }

    /// Determine optimal number of topics based on data characteristics
    pub fn determine_optimal_topics(&self, documents: &[String]) -> usize {
        if documents.len() < 5 {
            return 2;
        }

        // Use a heuristic: sqrt(num_documents) capped between 2 and 10
        let optimal = (documents.len() as f32).sqrt() as usize;
        optimal.clamp(2, 10)
    }
}

impl ContentClusterer for LdaClusterer {
    fn analyze_content(&self, titles: &[String]) -> Result<ContentAnalysis, ClusteringError> {
        // For LDA, we'll create a simple analysis structure
        // The actual analysis happens in fit_lda
        let feature_vectors: Vec<FeatureVector> = titles
            .iter()
            .enumerate()
            .map(|(i, _)| {
                // Create placeholder feature vectors - these will be replaced by topic distributions
                let mut features = HashMap::new();
                features.insert(format!("doc_{i}"), 1.0);
                FeatureVector::new(features)
            })
            .collect();

        use super::SimilarityMatrix;
        use std::collections::HashSet;

        Ok(ContentAnalysis {
            feature_vectors,
            similarity_matrix: SimilarityMatrix::new(0), // Not used in LDA
            vocabulary: HashSet::new(),
            document_frequencies: HashMap::new(),
            topic_keywords: Vec::new(), // Will be populated by LDA
        })
    }

    fn cluster_videos(
        &self,
        _analysis: &ContentAnalysis,
        _target_clusters: usize,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        // This method is not directly used for LDA
        // Instead, use fit_lda followed by cluster_by_topics
        Err(ClusteringError::AnalysisFailed(
            "Use fit_lda and cluster_by_topics for LDA clustering".to_string(),
        ))
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
                .map(|index| VideoWithMetadata {
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
                format!("Topic {}", optimized.len() + 1)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lda_clusterer_creation() {
        let clusterer = LdaClusterer::default();
        assert_eq!(clusterer.num_topics, 5);
        assert_eq!(clusterer.alpha, 0.1);
        assert_eq!(clusterer.beta, 0.01);
        assert_eq!(clusterer.num_iterations, 100);
    }

    #[test]
    fn test_lda_clusterer_custom_creation() {
        let clusterer = LdaClusterer::new(3, 0.5, 0.1, 50, Some(12345));
        assert_eq!(clusterer.num_topics, 3);
        assert_eq!(clusterer.alpha, 0.5);
        assert_eq!(clusterer.beta, 0.1);
        assert_eq!(clusterer.num_iterations, 50);
        assert_eq!(clusterer.random_seed, Some(12345));
    }

    #[test]
    fn test_document_preprocessing() {
        let clusterer = LdaClusterer::default();
        let documents = vec![
            "Introduction to Programming Basics".to_string(),
            "Advanced Programming Techniques".to_string(),
            "Database Design Fundamentals".to_string(),
        ];

        let (processed_docs, vocabulary) = clusterer.preprocess_documents(&documents);

        assert_eq!(processed_docs.len(), 3);
        assert!(!vocabulary.is_empty());

        // Check that stop words are filtered out
        assert!(!vocabulary.contains(&"to".to_string()));
        assert!(!vocabulary.contains(&"the".to_string()));
    }

    #[test]
    fn test_tokenize_and_clean() {
        let clusterer = LdaClusterer::default();
        let text = "Introduction to Programming: The Basics!";
        let tokens = clusterer.tokenize_and_clean(text);

        // Should filter out stop words and punctuation
        assert!(tokens.contains(&"introduction".to_string()));
        assert!(tokens.contains(&"programming".to_string()));
        assert!(tokens.contains(&"basics".to_string()));
        assert!(!tokens.contains(&"to".to_string()));
        assert!(!tokens.contains(&"the".to_string()));
    }

    #[test]
    fn test_lda_fitting() {
        let clusterer = LdaClusterer::new(2, 0.1, 0.01, 10, Some(42));
        let documents = vec![
            "programming basics fundamentals".to_string(),
            "programming advanced techniques".to_string(),
            "database design principles".to_string(),
            "database optimization performance".to_string(),
            "web development frontend".to_string(),
            "web development backend".to_string(),
        ];

        let result = clusterer.fit_lda(&documents);
        assert!(result.is_ok());

        let model = result.unwrap();
        assert_eq!(model.topics.len(), 2);
        assert_eq!(model.document_topics.len(), 6);
        assert!(!model.vocabulary.is_empty());
    }

    #[test]
    fn test_lda_insufficient_data() {
        let clusterer = LdaClusterer::new(5, 0.1, 0.01, 10, Some(42));
        let documents = vec!["test document".to_string(), "another test".to_string()];

        let result = clusterer.fit_lda(&documents);
        assert!(matches!(
            result,
            Err(ClusteringError::InsufficientContent(2))
        ));
    }

    #[test]
    fn test_optimal_topics_determination() {
        let clusterer = LdaClusterer::default();

        // Small dataset
        let small_docs = vec!["doc1".to_string(), "doc2".to_string()];
        assert_eq!(clusterer.determine_optimal_topics(&small_docs), 2);

        // Medium dataset
        let medium_docs = vec!["doc".to_string(); 25];
        let optimal = clusterer.determine_optimal_topics(&medium_docs);
        assert!(optimal >= 2 && optimal <= 10);

        // Large dataset
        let large_docs = vec!["doc".to_string(); 100];
        assert_eq!(clusterer.determine_optimal_topics(&large_docs), 10);
    }

    #[test]
    fn test_topic_similarity_calculation() {
        let clusterer = LdaClusterer::default();
        let topics_a = vec![0.8, 0.1, 0.1];
        let topics_b = vec![0.7, 0.2, 0.1];
        let topics_c = vec![0.1, 0.1, 0.8];

        let sim_ab = clusterer.calculate_topic_similarity(&topics_a, &topics_b);
        let sim_ac = clusterer.calculate_topic_similarity(&topics_a, &topics_c);

        // Similar topics should have higher similarity
        assert!(sim_ab > sim_ac);
        assert!(sim_ab >= 0.0 && sim_ab <= 1.0);
        assert!(sim_ac >= 0.0 && sim_ac <= 1.0);
    }
}
