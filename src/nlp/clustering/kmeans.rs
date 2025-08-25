//! K-means clustering algorithm implementation for video content grouping
//!
//! This module provides a k-means clustering implementation specifically designed
//! for grouping video content based on TF-IDF feature vectors.

use super::{
    ClusteringError, ContentAnalysis, ContentClusterer, FeatureVector, OptimizedCluster,
    VideoCluster,
};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

/// K-means clustering implementation
pub struct KMeansClusterer {
    pub max_iterations: usize,
    pub convergence_threshold: f32,
    pub random_seed: Option<u64>,
    pub min_similarity_threshold: f32,
}

impl Default for KMeansClusterer {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 0.001,
            random_seed: None,
            min_similarity_threshold: 0.3,
        }
    }
}

/// Internal cluster representation for k-means
#[derive(Debug, Clone)]
pub struct Cluster {
    pub points: Vec<usize>,
    pub centroid: FeatureVector,
    pub similarity_score: f32,
}

/// Clustering quality metrics
#[derive(Debug, Clone)]
pub struct ClusteringQuality {
    pub silhouette_score: f32,
    pub intra_cluster_similarity: f32,
    pub inter_cluster_separation: f32,
    pub duration_balance_score: f32,
}

impl KMeansClusterer {
    /// Create a new k-means clusterer with custom parameters
    pub fn new(
        max_iterations: usize,
        convergence_threshold: f32,
        random_seed: Option<u64>,
    ) -> Self {
        Self { max_iterations, convergence_threshold, random_seed, min_similarity_threshold: 0.3 }
    }

    /// Determine optimal number of clusters using elbow method
    pub fn determine_optimal_k(&self, features: &[FeatureVector]) -> usize {
        if features.len() < 5 {
            return 1;
        }

        let max_k = std::cmp::min(features.len() / 2, 10);
        let mut wcss_values = Vec::new();

        // Calculate WCSS (Within-Cluster Sum of Squares) for different k values
        for k in 1..=max_k {
            if let Ok(clusters) = self.cluster_with_k(features, k) {
                let wcss = self.calculate_wcss(features, &clusters);
                wcss_values.push((k, wcss));
            }
        }

        // Find elbow point (point with maximum rate of change decrease)
        self.find_elbow_point(&wcss_values)
    }

    /// Perform k-means clustering with a specific k value
    pub fn cluster_with_k(
        &self,
        features: &[FeatureVector],
        k: usize,
    ) -> Result<Vec<Cluster>, ClusteringError> {
        if features.is_empty() {
            return Err(ClusteringError::InsufficientContent(0));
        }

        if k == 0 || k > features.len() {
            return Err(ClusteringError::AnalysisFailed(format!("Invalid k value: {k}")));
        }

        // Initialize centroids
        let mut centroids = self.initialize_centroids(features, k)?;
        let mut assignments = vec![0; features.len()];
        let mut previous_assignments = vec![usize::MAX; features.len()];

        for iteration in 0..self.max_iterations {
            // Assign points to nearest centroids
            for (i, feature) in features.iter().enumerate() {
                let mut best_centroid = 0;
                let mut best_distance = f32::INFINITY;

                for (j, centroid) in centroids.iter().enumerate() {
                    let distance = self.calculate_distance(feature, centroid);
                    if distance < best_distance {
                        best_distance = distance;
                        best_centroid = j;
                    }
                }

                assignments[i] = best_centroid;
            }

            // Check for convergence
            if assignments == previous_assignments {
                break;
            }

            // Update centroids
            let new_centroids = self.update_centroids(features, &assignments, k)?;

            // Check for centroid convergence
            let centroid_change = centroids
                .iter()
                .zip(new_centroids.iter())
                .map(|(old, new)| self.calculate_distance(old, new))
                .fold(0.0, f32::max);

            centroids = new_centroids;
            previous_assignments = assignments.clone();

            if centroid_change < self.convergence_threshold {
                break;
            }

            if iteration == self.max_iterations - 1 {
                return Err(ClusteringError::ConvergenceFailed(self.max_iterations));
            }
        }

        // Create clusters from assignments
        self.create_clusters_from_assignments(features, &assignments, &centroids)
    }

    /// Calculate silhouette score for clustering quality evaluation
    pub fn calculate_silhouette_score(
        &self,
        features: &[FeatureVector],
        clusters: &[Cluster],
    ) -> f32 {
        if features.len() <= 1 || clusters.len() <= 1 {
            return 0.0;
        }

        let mut total_silhouette = 0.0;
        let mut valid_points = 0;

        for (i, feature) in features.iter().enumerate() {
            let cluster_id = self.find_cluster_for_point(i, clusters);
            if cluster_id.is_none() {
                continue;
            }
            let cluster_id = cluster_id.unwrap();

            // Calculate average distance to points in same cluster (a)
            let same_cluster_distances: Vec<f32> = clusters[cluster_id]
                .points
                .iter()
                .filter(|&&j| j != i)
                .map(|&j| self.calculate_distance(feature, &features[j]))
                .collect();

            if same_cluster_distances.is_empty() {
                continue;
            }

            let a =
                same_cluster_distances.iter().sum::<f32>() / same_cluster_distances.len() as f32;

            // Calculate minimum average distance to points in other clusters (b)
            let mut min_other_cluster_distance = f32::INFINITY;

            for (other_cluster_id, other_cluster) in clusters.iter().enumerate() {
                if other_cluster_id == cluster_id {
                    continue;
                }

                let other_cluster_distances: Vec<f32> = other_cluster
                    .points
                    .iter()
                    .map(|&j| self.calculate_distance(feature, &features[j]))
                    .collect();

                if !other_cluster_distances.is_empty() {
                    let avg_distance = other_cluster_distances.iter().sum::<f32>()
                        / other_cluster_distances.len() as f32;
                    min_other_cluster_distance = min_other_cluster_distance.min(avg_distance);
                }
            }

            if min_other_cluster_distance != f32::INFINITY {
                let b = min_other_cluster_distance;
                let silhouette = (b - a) / a.max(b);
                total_silhouette += silhouette;
                valid_points += 1;
            }
        }

        if valid_points > 0 { total_silhouette / valid_points as f32 } else { 0.0 }
    }

    /// Evaluate clustering quality with multiple metrics
    pub fn evaluate_clustering_quality(
        &self,
        features: &[FeatureVector],
        clusters: &[Cluster],
    ) -> ClusteringQuality {
        let silhouette_score = self.calculate_silhouette_score(features, clusters);
        let intra_cluster_similarity = self.calculate_intra_cluster_similarity(features, clusters);
        let inter_cluster_separation = self.calculate_inter_cluster_separation(features, clusters);

        ClusteringQuality {
            silhouette_score,
            intra_cluster_similarity,
            inter_cluster_separation,
            duration_balance_score: 0.0, // Will be calculated by duration balancer
        }
    }

    /// Initialize centroids using k-means++ algorithm
    fn initialize_centroids(
        &self,
        features: &[FeatureVector],
        k: usize,
    ) -> Result<Vec<FeatureVector>, ClusteringError> {
        if features.is_empty() {
            return Err(ClusteringError::InsufficientContent(0));
        }

        let mut centroids = Vec::new();
        let rng = self.create_rng();

        // Choose first centroid randomly
        let first_index = (rng % features.len() as u64) as usize;
        centroids.push(features[first_index].clone());

        // Choose remaining centroids using k-means++ method
        for _ in 1..k {
            let mut distances = Vec::new();
            let mut total_distance = 0.0;

            // Calculate distance to nearest centroid for each point
            for feature in features {
                let min_distance = centroids
                    .iter()
                    .map(|centroid| self.calculate_distance(feature, centroid))
                    .fold(f32::INFINITY, f32::min);

                let squared_distance = min_distance * min_distance;
                distances.push(squared_distance);
                total_distance += squared_distance;
            }

            // Choose next centroid with probability proportional to squared distance
            if total_distance > 0.0 {
                let mut cumulative = 0.0;
                let target = (self.create_rng() as f32 / u64::MAX as f32) * total_distance;

                for (i, &distance) in distances.iter().enumerate() {
                    cumulative += distance;
                    if cumulative >= target {
                        centroids.push(features[i].clone());
                        break;
                    }
                }
            } else {
                // Fallback: choose randomly
                let index = (self.create_rng() % features.len() as u64) as usize;
                centroids.push(features[index].clone());
            }
        }

        Ok(centroids)
    }

    /// Update centroids based on current assignments
    fn update_centroids(
        &self,
        features: &[FeatureVector],
        assignments: &[usize],
        k: usize,
    ) -> Result<Vec<FeatureVector>, ClusteringError> {
        let mut new_centroids = Vec::new();

        for cluster_id in 0..k {
            let cluster_points: Vec<_> = assignments
                .iter()
                .enumerate()
                .filter(|(_, assignment)| **assignment == cluster_id)
                .map(|(i, _)| &features[i])
                .collect();

            if cluster_points.is_empty() {
                // Empty cluster - reinitialize randomly
                let random_index = (self.create_rng() % features.len() as u64) as usize;
                new_centroids.push(features[random_index].clone());
            } else {
                new_centroids.push(self.calculate_centroid(&cluster_points));
            }
        }

        Ok(new_centroids)
    }

    /// Calculate centroid from a set of feature vectors
    fn calculate_centroid(&self, points: &[&FeatureVector]) -> FeatureVector {
        if points.is_empty() {
            return FeatureVector::default();
        }

        let mut centroid_features = HashMap::new();

        // Sum all features
        for point in points {
            for (term, &value) in &point.features {
                *centroid_features.entry(term.clone()).or_insert(0.0) += value;
            }
        }

        // Average the features
        let count = points.len() as f32;
        for value in centroid_features.values_mut() {
            *value /= count;
        }

        FeatureVector::new(centroid_features)
    }

    /// Calculate distance between two feature vectors (using cosine distance)
    fn calculate_distance(&self, a: &FeatureVector, b: &FeatureVector) -> f32 {
        1.0 - a.cosine_similarity(b)
    }

    /// Create clusters from assignments and centroids
    fn create_clusters_from_assignments(
        &self,
        features: &[FeatureVector],
        assignments: &[usize],
        centroids: &[FeatureVector],
    ) -> Result<Vec<Cluster>, ClusteringError> {
        let mut clusters = Vec::new();

        for (cluster_id, centroid) in centroids.iter().enumerate() {
            let points: Vec<usize> = assignments
                .iter()
                .enumerate()
                .filter(|(_, assignment)| **assignment == cluster_id)
                .map(|(i, _)| i)
                .collect();

            if !points.is_empty() {
                // Calculate average similarity within cluster
                let similarity_score = if points.len() > 1 {
                    let mut total_similarity = 0.0;
                    let mut count = 0;
                    for &i in &points {
                        for &j in &points {
                            if i != j {
                                total_similarity += features[i].cosine_similarity(&features[j]);
                                count += 1;
                            }
                        }
                    }
                    if count > 0 { total_similarity / count as f32 } else { 1.0 }
                } else {
                    1.0
                };

                clusters.push(Cluster { points, centroid: centroid.clone(), similarity_score });
            }
        }

        Ok(clusters)
    }

    /// Calculate Within-Cluster Sum of Squares (WCSS) for elbow method
    fn calculate_wcss(&self, features: &[FeatureVector], clusters: &[Cluster]) -> f32 {
        let mut wcss = 0.0;

        for cluster in clusters {
            for &point_idx in &cluster.points {
                if let Some(feature) = features.get(point_idx) {
                    let distance = self.calculate_distance(feature, &cluster.centroid);
                    wcss += distance * distance;
                }
            }
        }

        wcss
    }

    /// Find elbow point in WCSS values for optimal k determination
    fn find_elbow_point(&self, wcss_values: &[(usize, f32)]) -> usize {
        if wcss_values.len() < 3 {
            return wcss_values.first().map(|(k, _)| *k).unwrap_or(1);
        }

        let mut max_rate_change = 0.0;
        let mut optimal_k = 1;

        for i in 1..wcss_values.len() - 1 {
            let prev_wcss = wcss_values[i - 1].1;
            let curr_wcss = wcss_values[i].1;
            let next_wcss = wcss_values[i + 1].1;

            // Calculate rate of change decrease
            let rate_change = (prev_wcss - curr_wcss) - (curr_wcss - next_wcss);

            if rate_change > max_rate_change {
                max_rate_change = rate_change;
                optimal_k = wcss_values[i].0;
            }
        }

        optimal_k
    }

    /// Find which cluster a point belongs to
    fn find_cluster_for_point(&self, point_idx: usize, clusters: &[Cluster]) -> Option<usize> {
        for (cluster_idx, cluster) in clusters.iter().enumerate() {
            if cluster.points.contains(&point_idx) {
                return Some(cluster_idx);
            }
        }
        None
    }

    /// Calculate average intra-cluster similarity
    fn calculate_intra_cluster_similarity(
        &self,
        features: &[FeatureVector],
        clusters: &[Cluster],
    ) -> f32 {
        if clusters.is_empty() {
            return 0.0;
        }

        let mut total_similarity = 0.0;
        let mut total_pairs = 0;

        for cluster in clusters {
            if cluster.points.len() < 2 {
                continue;
            }

            for &i in &cluster.points {
                for &j in &cluster.points {
                    if i != j {
                        if let (Some(feat_i), Some(feat_j)) = (features.get(i), features.get(j)) {
                            total_similarity += feat_i.cosine_similarity(feat_j);
                            total_pairs += 1;
                        }
                    }
                }
            }
        }

        if total_pairs > 0 { total_similarity / total_pairs as f32 } else { 0.0 }
    }

    /// Calculate average inter-cluster separation
    fn calculate_inter_cluster_separation(
        &self,
        _features: &[FeatureVector],
        clusters: &[Cluster],
    ) -> f32 {
        if clusters.len() < 2 {
            return 1.0;
        }

        let mut total_distance = 0.0;
        let mut total_pairs = 0;

        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let distance =
                    self.calculate_distance(&clusters[i].centroid, &clusters[j].centroid);
                total_distance += distance;
                total_pairs += 1;
            }
        }

        if total_pairs > 0 { total_distance / total_pairs as f32 } else { 0.0 }
    }

    /// Create a simple random number generator
    fn create_rng(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        if let Some(seed) = self.random_seed {
            seed
        } else {
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .hash(&mut hasher);
            hasher.finish()
        }
    }

    /// Handle edge cases for clustering
    pub fn handle_edge_cases(
        &self,
        features: &[FeatureVector],
    ) -> Result<Vec<Cluster>, ClusteringError> {
        if features.is_empty() {
            return Err(ClusteringError::InsufficientContent(0));
        }

        if features.len() < 5 {
            return Err(ClusteringError::InsufficientContent(features.len()));
        }

        // Check for identical content
        let mut unique_features = Vec::new();
        let mut feature_map = HashMap::new();

        for (i, feature) in features.iter().enumerate() {
            let mut is_duplicate = false;

            for (j, unique_feature) in unique_features.iter().enumerate() {
                if feature.cosine_similarity(unique_feature) > 0.99 {
                    feature_map.entry(j).or_insert_with(Vec::new).push(i);
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                let unique_idx = unique_features.len();
                unique_features.push(feature.clone());
                feature_map.entry(unique_idx).or_insert_with(Vec::new).push(i);
            }
        }

        // If all content is identical, create a single cluster
        if unique_features.len() == 1 {
            let all_points: Vec<usize> = (0..features.len()).collect();
            return Ok(vec![Cluster {
                points: all_points,
                centroid: features[0].clone(),
                similarity_score: 1.0,
            }]);
        }

        // If we have very few unique features, adjust k accordingly
        let optimal_k =
            std::cmp::min(unique_features.len(), self.determine_optimal_k(&unique_features));
        self.cluster_with_k(features, optimal_k)
    }
}

impl ContentClusterer for KMeansClusterer {
    fn analyze_content(&self, titles: &[String]) -> Result<ContentAnalysis, ClusteringError> {
        // Use TF-IDF analyzer for content analysis
        let tfidf_analyzer = super::TfIdfAnalyzer::default();
        tfidf_analyzer.analyze_content(titles)
    }

    fn cluster_videos(
        &self,
        analysis: &ContentAnalysis,
        target_clusters: usize,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        // Handle edge cases first
        self.handle_edge_cases(&analysis.feature_vectors)?;

        // Determine optimal k if target_clusters is 0
        let k = if target_clusters == 0 {
            self.determine_optimal_k(&analysis.feature_vectors)
        } else {
            std::cmp::min(target_clusters, analysis.feature_vectors.len())
        };

        // Perform k-means clustering
        let clusters = self.cluster_with_k(&analysis.feature_vectors, k)?;

        // Convert internal clusters to VideoCluster format
        let mut video_clusters = Vec::new();
        for cluster in clusters {
            let topic_keywords =
                cluster.centroid.top_features(5).into_iter().map(|(keyword, _)| keyword).collect();

            video_clusters.push(VideoCluster {
                videos: cluster.points,
                centroid: cluster.centroid,
                similarity_score: cluster.similarity_score,
                topic_keywords,
            });
        }

        Ok(video_clusters)
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_feature_vector(terms: &[(&str, f32)]) -> FeatureVector {
        let features: HashMap<String, f32> =
            terms.iter().map(|(term, score)| (term.to_string(), *score)).collect();
        FeatureVector::new(features)
    }

    #[test]
    fn test_kmeans_clusterer_creation() {
        let clusterer = KMeansClusterer::default();
        assert_eq!(clusterer.max_iterations, 100);
        assert_eq!(clusterer.convergence_threshold, 0.001);
        assert_eq!(clusterer.min_similarity_threshold, 0.3);
    }

    #[test]
    fn test_kmeans_clusterer_custom_creation() {
        let clusterer = KMeansClusterer::new(50, 0.01, Some(12345));
        assert_eq!(clusterer.max_iterations, 50);
        assert_eq!(clusterer.convergence_threshold, 0.01);
        assert_eq!(clusterer.random_seed, Some(12345));
    }

    #[test]
    fn test_optimal_k_determination() {
        let clusterer = KMeansClusterer::default();

        // Test with insufficient data
        let small_features = vec![
            create_test_feature_vector(&[("test", 1.0)]),
            create_test_feature_vector(&[("example", 1.0)]),
        ];
        assert_eq!(clusterer.determine_optimal_k(&small_features), 1);

        // Test with sufficient data
        let features = vec![
            create_test_feature_vector(&[("programming", 1.0), ("basics", 0.5)]),
            create_test_feature_vector(&[("programming", 0.8), ("advanced", 0.7)]),
            create_test_feature_vector(&[("database", 1.0), ("sql", 0.6)]),
            create_test_feature_vector(&[("database", 0.9), ("design", 0.8)]),
            create_test_feature_vector(&[("web", 1.0), ("html", 0.7)]),
            create_test_feature_vector(&[("web", 0.8), ("css", 0.9)]),
        ];
        let k = clusterer.determine_optimal_k(&features);
        assert!(k >= 1 && k <= features.len() / 2);
    }

    #[test]
    fn test_clustering_with_k() {
        let clusterer = KMeansClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("programming", 1.0), ("basics", 0.5)]),
            create_test_feature_vector(&[("programming", 0.8), ("fundamentals", 0.7)]),
            create_test_feature_vector(&[("database", 1.0), ("sql", 0.6)]),
            create_test_feature_vector(&[("database", 0.9), ("design", 0.8)]),
        ];

        let result = clusterer.cluster_with_k(&features, 2);
        assert!(result.is_ok());

        let clusters = result.unwrap();
        assert_eq!(clusters.len(), 2);

        // Verify all points are assigned
        let total_points: usize = clusters.iter().map(|c| c.points.len()).sum();
        assert_eq!(total_points, features.len());
    }

    #[test]
    fn test_clustering_invalid_k() {
        let clusterer = KMeansClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("test", 1.0)]),
            create_test_feature_vector(&[("example", 1.0)]),
        ];

        // Test k = 0
        let result = clusterer.cluster_with_k(&features, 0);
        assert!(result.is_err());

        // Test k > features.len()
        let result = clusterer.cluster_with_k(&features, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_silhouette_score_calculation() {
        let clusterer = KMeansClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("a", 1.0)]),
            create_test_feature_vector(&[("a", 0.9)]),
            create_test_feature_vector(&[("b", 1.0)]),
            create_test_feature_vector(&[("b", 0.9)]),
        ];

        let clusters = vec![
            Cluster {
                points: vec![0, 1],
                centroid: create_test_feature_vector(&[("a", 0.95)]),
                similarity_score: 0.9,
            },
            Cluster {
                points: vec![2, 3],
                centroid: create_test_feature_vector(&[("b", 0.95)]),
                similarity_score: 0.9,
            },
        ];

        let silhouette = clusterer.calculate_silhouette_score(&features, &clusters);
        assert!(silhouette >= -1.0 && silhouette <= 1.0);
    }

    #[test]
    fn test_silhouette_score_edge_cases() {
        let clusterer = KMeansClusterer::default();

        // Single feature
        let single_feature = vec![create_test_feature_vector(&[("test", 1.0)])];
        let single_cluster = vec![Cluster {
            points: vec![0],
            centroid: create_test_feature_vector(&[("test", 1.0)]),
            similarity_score: 1.0,
        }];
        assert_eq!(clusterer.calculate_silhouette_score(&single_feature, &single_cluster), 0.0);

        // Single cluster
        let features = vec![
            create_test_feature_vector(&[("a", 1.0)]),
            create_test_feature_vector(&[("b", 1.0)]),
        ];
        let single_cluster = vec![Cluster {
            points: vec![0, 1],
            centroid: create_test_feature_vector(&[("a", 0.5), ("b", 0.5)]),
            similarity_score: 0.5,
        }];
        assert_eq!(clusterer.calculate_silhouette_score(&features, &single_cluster), 0.0);
    }

    #[test]
    fn test_clustering_quality_evaluation() {
        let clusterer = KMeansClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("programming", 1.0)]),
            create_test_feature_vector(&[("programming", 0.9)]),
            create_test_feature_vector(&[("database", 1.0)]),
            create_test_feature_vector(&[("database", 0.9)]),
        ];

        let clusters = vec![
            Cluster {
                points: vec![0, 1],
                centroid: create_test_feature_vector(&[("programming", 0.95)]),
                similarity_score: 0.9,
            },
            Cluster {
                points: vec![2, 3],
                centroid: create_test_feature_vector(&[("database", 0.95)]),
                similarity_score: 0.9,
            },
        ];

        let quality = clusterer.evaluate_clustering_quality(&features, &clusters);
        assert!(quality.silhouette_score >= -1.0 && quality.silhouette_score <= 1.0);
        assert!(quality.intra_cluster_similarity >= 0.0 && quality.intra_cluster_similarity <= 1.0);
        assert!(quality.inter_cluster_separation >= 0.0);
    }

    #[test]
    fn test_edge_case_handling() {
        let clusterer = KMeansClusterer::default();

        // Test empty features
        let empty_features = vec![];
        let result = clusterer.handle_edge_cases(&empty_features);
        assert!(matches!(result, Err(ClusteringError::InsufficientContent(0))));

        // Test insufficient features
        let few_features = vec![
            create_test_feature_vector(&[("test", 1.0)]),
            create_test_feature_vector(&[("example", 1.0)]),
        ];
        let result = clusterer.handle_edge_cases(&few_features);
        assert!(matches!(result, Err(ClusteringError::InsufficientContent(2))));

        // Test identical content
        let identical_features = vec![
            create_test_feature_vector(&[("same", 1.0)]),
            create_test_feature_vector(&[("same", 1.0)]),
            create_test_feature_vector(&[("same", 1.0)]),
            create_test_feature_vector(&[("same", 1.0)]),
            create_test_feature_vector(&[("same", 1.0)]),
        ];
        let result = clusterer.handle_edge_cases(&identical_features);
        assert!(result.is_ok());
        let clusters = result.unwrap();
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].points.len(), 5);
    }

    #[test]
    fn test_wcss_calculation() {
        let clusterer = KMeansClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("a", 1.0)]),
            create_test_feature_vector(&[("b", 1.0)]),
        ];

        let clusters = vec![
            Cluster {
                points: vec![0],
                centroid: create_test_feature_vector(&[("a", 1.0)]),
                similarity_score: 1.0,
            },
            Cluster {
                points: vec![1],
                centroid: create_test_feature_vector(&[("b", 1.0)]),
                similarity_score: 1.0,
            },
        ];

        let wcss = clusterer.calculate_wcss(&features, &clusters);
        assert!(wcss >= 0.0);
    }

    #[test]
    fn test_elbow_point_finding() {
        let clusterer = KMeansClusterer::default();

        // Test with insufficient data points
        let few_points = vec![(1, 10.0), (2, 5.0)];
        assert_eq!(clusterer.find_elbow_point(&few_points), 1);

        // Test with clear elbow
        let wcss_values = vec![(1, 100.0), (2, 50.0), (3, 40.0), (4, 38.0), (5, 37.0)];
        let optimal_k = clusterer.find_elbow_point(&wcss_values);
        assert!(optimal_k >= 1 && optimal_k <= 5);
    }

    #[test]
    fn test_centroid_calculation() {
        let clusterer = KMeansClusterer::default();

        // Test empty points
        let empty_points: Vec<&FeatureVector> = vec![];
        let centroid = clusterer.calculate_centroid(&empty_points);
        assert!(centroid.features.is_empty());

        // Test single point
        let feature = create_test_feature_vector(&[("test", 1.0), ("example", 0.5)]);
        let single_point = vec![&feature];
        let centroid = clusterer.calculate_centroid(&single_point);
        assert_eq!(centroid.features.get("test"), Some(&1.0));
        assert_eq!(centroid.features.get("example"), Some(&0.5));

        // Test multiple points
        let feature1 = create_test_feature_vector(&[("test", 1.0), ("example", 0.5)]);
        let feature2 = create_test_feature_vector(&[("test", 0.5), ("example", 1.0)]);
        let multiple_points = vec![&feature1, &feature2];
        let centroid = clusterer.calculate_centroid(&multiple_points);
        assert_eq!(centroid.features.get("test"), Some(&0.75));
        assert_eq!(centroid.features.get("example"), Some(&0.75));
    }

    #[test]
    fn test_distance_calculation() {
        let clusterer = KMeansClusterer::default();
        let feature1 = create_test_feature_vector(&[("test", 1.0)]);
        let feature2 = create_test_feature_vector(&[("test", 1.0)]);
        let feature3 = create_test_feature_vector(&[("different", 1.0)]);

        // Identical features should have distance 0
        assert!((clusterer.calculate_distance(&feature1, &feature2) - 0.0).abs() < 0.001);

        // Different features should have distance > 0
        assert!(clusterer.calculate_distance(&feature1, &feature3) > 0.0);
    }

    #[test]
    fn test_content_clusterer_implementation() {
        let clusterer = KMeansClusterer::default();
        let titles = vec![
            "Introduction to Programming".to_string(),
            "Advanced Programming Concepts".to_string(),
            "Database Design Fundamentals".to_string(),
            "SQL Query Optimization".to_string(),
            "Web Development Basics".to_string(),
        ];

        let analysis_result = clusterer.analyze_content(&titles);
        assert!(analysis_result.is_ok());

        let analysis = analysis_result.unwrap();
        let cluster_result = clusterer.cluster_videos(&analysis, 0);
        assert!(cluster_result.is_ok());

        let clusters = cluster_result.unwrap();
        assert!(!clusters.is_empty());

        // Test optimization
        let durations = vec![Duration::from_secs(300); titles.len()];
        let optimize_result = clusterer.optimize_clusters(clusters, &durations);
        assert!(optimize_result.is_ok());
    }

    #[test]
    fn test_convergence_failure() {
        let mut clusterer = KMeansClusterer::default();
        clusterer.max_iterations = 1; // Force early termination
        clusterer.convergence_threshold = 0.0; // Never converge

        let features = vec![
            create_test_feature_vector(&[("a", 1.0)]),
            create_test_feature_vector(&[("b", 1.0)]),
            create_test_feature_vector(&[("c", 1.0)]),
            create_test_feature_vector(&[("d", 1.0)]),
        ];

        let result = clusterer.cluster_with_k(&features, 2);
        // Should either succeed or fail with convergence error
        if let Err(e) = result {
            assert!(matches!(e, ClusteringError::ConvergenceFailed(_)));
        }
    }

    #[test]
    fn test_random_seed_consistency() {
        let clusterer1 = KMeansClusterer::new(100, 0.001, Some(12345));
        let clusterer2 = KMeansClusterer::new(100, 0.001, Some(12345));

        let features = vec![
            create_test_feature_vector(&[("programming", 1.0)]),
            create_test_feature_vector(&[("database", 1.0)]),
            create_test_feature_vector(&[("web", 1.0)]),
            create_test_feature_vector(&[("mobile", 1.0)]),
            create_test_feature_vector(&[("ai", 1.0)]),
        ];

        let result1 = clusterer1.cluster_with_k(&features, 2);
        let result2 = clusterer2.cluster_with_k(&features, 2);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // With same seed, results should be deterministic
        let clusters1 = result1.unwrap();
        let clusters2 = result2.unwrap();
        assert_eq!(clusters1.len(), clusters2.len());
    }

    #[test]
    fn test_cluster_quality_metrics() {
        let clusterer = KMeansClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("programming", 1.0), ("basics", 0.8)]),
            create_test_feature_vector(&[("programming", 0.9), ("fundamentals", 0.7)]),
            create_test_feature_vector(&[("database", 1.0), ("sql", 0.8)]),
            create_test_feature_vector(&[("database", 0.9), ("design", 0.7)]),
        ];

        let clusters = vec![
            Cluster {
                points: vec![0, 1],
                centroid: create_test_feature_vector(&[("programming", 0.95), ("basics", 0.4)]),
                similarity_score: 0.85,
            },
            Cluster {
                points: vec![2, 3],
                centroid: create_test_feature_vector(&[("database", 0.95), ("sql", 0.4)]),
                similarity_score: 0.85,
            },
        ];

        let intra_similarity = clusterer.calculate_intra_cluster_similarity(&features, &clusters);
        assert!(intra_similarity >= 0.0 && intra_similarity <= 1.0);

        let inter_separation = clusterer.calculate_inter_cluster_separation(&features, &clusters);
        assert!(inter_separation >= 0.0);
    }

    #[test]
    fn test_find_cluster_for_point() {
        let clusterer = KMeansClusterer::default();
        let clusters = vec![
            Cluster {
                points: vec![0, 1, 2],
                centroid: create_test_feature_vector(&[("test", 1.0)]),
                similarity_score: 0.8,
            },
            Cluster {
                points: vec![3, 4],
                centroid: create_test_feature_vector(&[("example", 1.0)]),
                similarity_score: 0.7,
            },
        ];

        assert_eq!(clusterer.find_cluster_for_point(0, &clusters), Some(0));
        assert_eq!(clusterer.find_cluster_for_point(1, &clusters), Some(0));
        assert_eq!(clusterer.find_cluster_for_point(3, &clusters), Some(1));
        assert_eq!(clusterer.find_cluster_for_point(5, &clusters), None);
    }
}
