//! Hierarchical clustering algorithm implementation for video content grouping
//!
//! This module provides hierarchical clustering as an alternative to k-means,
//! particularly useful for discovering natural content hierarchies and when
//! the number of clusters is not known in advance.

use super::{
    ClusteringError, ContentAnalysis, ContentClusterer, FeatureVector, OptimizedCluster,
    VideoCluster, VideoWithMetadata,
};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;

/// Hierarchical clustering implementation using agglomerative approach
pub struct HierarchicalClusterer {
    pub linkage_method: LinkageMethod,
    pub distance_threshold: f32,
    pub min_cluster_size: usize,
    pub max_clusters: usize,
}

/// Linkage methods for hierarchical clustering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkageMethod {
    /// Single linkage (minimum distance between clusters)
    Single,
    /// Complete linkage (maximum distance between clusters)
    Complete,
    /// Average linkage (average distance between clusters)
    Average,
    /// Ward linkage (minimize within-cluster variance)
    Ward,
}

/// Internal cluster node for hierarchical clustering
#[derive(Debug, Clone)]
struct ClusterNode {
    points: Vec<usize>,
    centroid: FeatureVector,
    height: f32,
    left_child: Option<Box<ClusterNode>>,
    right_child: Option<Box<ClusterNode>>,
}

/// Distance matrix for hierarchical clustering
struct DistanceMatrix {
    distances: Vec<Vec<f32>>,
    size: usize,
}

impl Default for HierarchicalClusterer {
    fn default() -> Self {
        Self {
            linkage_method: LinkageMethod::Average,
            distance_threshold: 0.7,
            min_cluster_size: 2,
            max_clusters: 10,
        }
    }
}

impl HierarchicalClusterer {
    /// Create a new hierarchical clusterer with custom parameters
    pub fn new(
        linkage_method: LinkageMethod,
        distance_threshold: f32,
        min_cluster_size: usize,
        max_clusters: usize,
    ) -> Self {
        Self {
            linkage_method,
            distance_threshold,
            min_cluster_size,
            max_clusters,
        }
    }

    /// Perform hierarchical clustering on feature vectors
    pub fn cluster_hierarchical(
        &self,
        features: &[FeatureVector],
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        if features.len() < 2 {
            return Err(ClusteringError::InsufficientContent(features.len()));
        }

        // Build distance matrix
        let distance_matrix = self.build_distance_matrix(features);

        // Perform agglomerative clustering
        let root = self.agglomerative_clustering(features, &distance_matrix)?;

        // Extract clusters from dendrogram
        let clusters = self.extract_clusters(&root, features)?;

        // Convert to VideoCluster format
        self.convert_to_video_clusters(clusters, features)
    }

    /// Build distance matrix between all pairs of feature vectors
    fn build_distance_matrix(&self, features: &[FeatureVector]) -> DistanceMatrix {
        let size = features.len();
        let mut distances = vec![vec![0.0; size]; size];

        for i in 0..size {
            for j in (i + 1)..size {
                let distance = self.calculate_distance(&features[i], &features[j]);
                distances[i][j] = distance;
                distances[j][i] = distance;
            }
        }

        DistanceMatrix { distances, size }
    }

    /// Perform agglomerative clustering to build dendrogram
    fn agglomerative_clustering(
        &self,
        features: &[FeatureVector],
        distance_matrix: &DistanceMatrix,
    ) -> Result<ClusterNode, ClusteringError> {
        debug_assert_eq!(
            distance_matrix.size,
            features.len(),
            "DistanceMatrix.size should match features length"
        );
        let mut active_clusters: Vec<ClusterNode> = features
            .iter()
            .enumerate()
            .map(|(i, feature)| ClusterNode {
                points: vec![i],
                centroid: feature.clone(),
                height: 0.0,
                left_child: None,
                right_child: None,
            })
            .collect();

        let mut cluster_distances = distance_matrix.distances.clone();

        while active_clusters.len() > 1 {
            // Find the pair of clusters with minimum distance
            let (min_i, min_j, min_distance) = self.find_closest_clusters(&cluster_distances)?;

            // Merge the closest clusters
            let cluster_i = active_clusters.remove(std::cmp::max(min_i, min_j));
            let cluster_j = active_clusters.remove(std::cmp::min(min_i, min_j));

            let merged_cluster = self.merge_clusters(cluster_i, cluster_j, min_distance, features);

            // Update distance matrix
            self.update_distance_matrix(
                &mut cluster_distances,
                &active_clusters,
                &merged_cluster,
                min_i,
                min_j,
                features,
            );

            active_clusters.push(merged_cluster);

            // Stop if we've reached the maximum number of clusters
            if active_clusters.len() <= self.max_clusters {
                break;
            }
        }

        if active_clusters.len() == 1 {
            Ok(active_clusters.into_iter().next().unwrap())
        } else {
            // If we have multiple clusters remaining, create a root node
            let mut root_points = Vec::new();
            let mut root_centroid_features = HashMap::new();

            for cluster in &active_clusters {
                root_points.extend(&cluster.points);
                for (term, &value) in &cluster.centroid.features {
                    *root_centroid_features.entry(term.clone()).or_insert(0.0) += value;
                }
            }

            // Average the centroids
            let count = active_clusters.len() as f32;
            for value in root_centroid_features.values_mut() {
                *value /= count;
            }

            Ok(ClusterNode {
                points: root_points,
                centroid: FeatureVector::new(root_centroid_features),
                height: 1.0,
                left_child: None,
                right_child: None,
            })
        }
    }

    /// Find the pair of clusters with minimum distance
    fn find_closest_clusters(
        &self,
        distances: &[Vec<f32>],
    ) -> Result<(usize, usize, f32), ClusteringError> {
        let mut min_distance = f32::INFINITY;
        let mut min_i = 0;
        let mut min_j = 0;

        for i in 0..distances.len() {
            for j in (i + 1)..distances[i].len() {
                if distances[i][j] < min_distance {
                    min_distance = distances[i][j];
                    min_i = i;
                    min_j = j;
                }
            }
        }

        if min_distance == f32::INFINITY {
            return Err(ClusteringError::AnalysisFailed(
                "No valid cluster pairs found".to_string(),
            ));
        }

        Ok((min_i, min_j, min_distance))
    }

    /// Merge two clusters into a new cluster
    fn merge_clusters(
        &self,
        cluster_i: ClusterNode,
        cluster_j: ClusterNode,
        distance: f32,
        features: &[FeatureVector],
    ) -> ClusterNode {
        let mut merged_points = cluster_i.points.clone();
        merged_points.extend(&cluster_j.points);

        // Calculate new centroid
        let merged_centroid = self.calculate_merged_centroid(&cluster_i, &cluster_j, features);

        ClusterNode {
            points: merged_points,
            centroid: merged_centroid,
            height: distance,
            left_child: Some(Box::new(cluster_i)),
            right_child: Some(Box::new(cluster_j)),
        }
    }

    /// Calculate centroid for merged cluster
    fn calculate_merged_centroid(
        &self,
        cluster_i: &ClusterNode,
        cluster_j: &ClusterNode,
        features: &[FeatureVector],
    ) -> FeatureVector {
        let all_points: Vec<&FeatureVector> = cluster_i
            .points
            .iter()
            .chain(cluster_j.points.iter())
            .filter_map(|&idx| features.get(idx))
            .collect();

        if all_points.is_empty() {
            return FeatureVector::default();
        }

        let mut centroid_features = HashMap::new();

        // Sum all features
        for feature in &all_points {
            for (term, &value) in &feature.features {
                *centroid_features.entry(term.clone()).or_insert(0.0) += value;
            }
        }

        // Average the features
        let count = all_points.len() as f32;
        for value in centroid_features.values_mut() {
            *value /= count;
        }

        FeatureVector::new(centroid_features)
    }

    /// Update distance matrix after merging clusters
    fn update_distance_matrix(
        &self,
        distances: &mut [Vec<f32>],
        active_clusters: &[ClusterNode],
        merged_cluster: &ClusterNode,
        merged_i: usize,
        merged_j: usize,
        features: &[FeatureVector],
    ) {
        let new_cluster_idx = active_clusters.len(); // Index where merged cluster will be added

        // Calculate distances from merged cluster to all other clusters
        for (k, other_cluster) in active_clusters.iter().enumerate() {
            if k == merged_i || k == merged_j {
                continue;
            }

            let distance = match self.linkage_method {
                LinkageMethod::Single => {
                    // Minimum distance between any two points in different clusters
                    self.calculate_single_linkage_distance(merged_cluster, other_cluster, features)
                }
                LinkageMethod::Complete => {
                    // Maximum distance between any two points in different clusters
                    self.calculate_complete_linkage_distance(
                        merged_cluster,
                        other_cluster,
                        features,
                    )
                }
                LinkageMethod::Average => {
                    // Average distance between centroids
                    self.calculate_distance(&merged_cluster.centroid, &other_cluster.centroid)
                }
                LinkageMethod::Ward => {
                    // Ward linkage (simplified as centroid distance for now)
                    self.calculate_distance(&merged_cluster.centroid, &other_cluster.centroid)
                }
            };

            // Update distance matrix (simplified approach)
            if k < distances.len() && new_cluster_idx < distances[k].len() {
                distances[k][new_cluster_idx] = distance;
                if new_cluster_idx < distances.len() {
                    distances[new_cluster_idx][k] = distance;
                }
            }
        }
    }

    /// Calculate single linkage distance between clusters
    fn calculate_single_linkage_distance(
        &self,
        cluster_a: &ClusterNode,
        cluster_b: &ClusterNode,
        features: &[FeatureVector],
    ) -> f32 {
        let mut min_distance = f32::INFINITY;

        for &point_a in &cluster_a.points {
            for &point_b in &cluster_b.points {
                if let (Some(feat_a), Some(feat_b)) = (features.get(point_a), features.get(point_b))
                {
                    let distance = self.calculate_distance(feat_a, feat_b);
                    min_distance = min_distance.min(distance);
                }
            }
        }

        min_distance
    }

    /// Calculate complete linkage distance between clusters
    fn calculate_complete_linkage_distance(
        &self,
        cluster_a: &ClusterNode,
        cluster_b: &ClusterNode,
        features: &[FeatureVector],
    ) -> f32 {
        let mut max_distance = 0.0;

        for &point_a in &cluster_a.points {
            for &point_b in &cluster_b.points {
                if let (Some(feat_a), Some(feat_b)) = (features.get(point_a), features.get(point_b))
                {
                    let distance = self.calculate_distance(feat_a, feat_b);
                    max_distance = f32::max(max_distance, distance);
                }
            }
        }

        max_distance
    }

    /// Extract clusters from dendrogram based on distance threshold
    fn extract_clusters(
        &self,
        root: &ClusterNode,
        features: &[FeatureVector],
    ) -> Result<Vec<ClusterNode>, ClusteringError> {
        let mut clusters = Vec::new();
        self.extract_clusters_recursive(root, &mut clusters, features);

        // Filter clusters by minimum size
        clusters.retain(|cluster| cluster.points.len() >= self.min_cluster_size);

        if clusters.is_empty() {
            // If no clusters meet the minimum size, return the root as a single cluster
            clusters.push(root.clone());
        }

        Ok(clusters)
    }

    /// Recursively extract clusters from dendrogram
    fn extract_clusters_recursive(
        &self,
        node: &ClusterNode,
        clusters: &mut Vec<ClusterNode>,
        features: &[FeatureVector],
    ) {
        // If this node's height is below threshold, treat it as a cluster
        if node.height <= self.distance_threshold || node.left_child.is_none() {
            clusters.push(node.clone());
            return;
        }

        // Otherwise, recurse into children
        if let Some(ref left) = node.left_child {
            self.extract_clusters_recursive(left, clusters, features);
        }
        if let Some(ref right) = node.right_child {
            self.extract_clusters_recursive(right, clusters, features);
        }
    }

    /// Convert internal cluster nodes to VideoCluster format
    fn convert_to_video_clusters(
        &self,
        clusters: Vec<ClusterNode>,
        features: &[FeatureVector],
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        let mut video_clusters = Vec::new();

        for cluster in clusters {
            // Calculate similarity score within cluster
            let similarity_score = if cluster.points.len() > 1 {
                let mut total_similarity = 0.0;
                let mut count = 0;

                for &i in &cluster.points {
                    for &j in &cluster.points {
                        if i != j {
                            if let (Some(feat_i), Some(feat_j)) = (features.get(i), features.get(j))
                            {
                                total_similarity += feat_i.cosine_similarity(feat_j);
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
            } else {
                1.0
            };

            // Extract topic keywords from centroid
            let topic_keywords = cluster
                .centroid
                .top_features(5)
                .into_iter()
                .map(|(keyword, _)| keyword)
                .collect();

            video_clusters.push(VideoCluster {
                videos: cluster.points,
                centroid: cluster.centroid,
                similarity_score,
                topic_keywords,
            });
        }

        Ok(video_clusters)
    }

    /// Calculate distance between two feature vectors (using cosine distance)
    fn calculate_distance(&self, a: &FeatureVector, b: &FeatureVector) -> f32 {
        1.0 - a.cosine_similarity(b)
    }

    /// Determine optimal distance threshold based on data characteristics
    pub fn determine_optimal_threshold(&self, features: &[FeatureVector]) -> f32 {
        if features.len() < 2 {
            return self.distance_threshold;
        }

        // Calculate average pairwise distance
        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..features.len() {
            for j in (i + 1)..features.len() {
                total_distance += self.calculate_distance(&features[i], &features[j]);
                count += 1;
            }
        }

        if count > 0 {
            let avg_distance = total_distance / count as f32;
            // Use 70% of average distance as threshold
            (avg_distance * 0.7).clamp(0.3, 0.9)
        } else {
            self.distance_threshold
        }
    }
}

impl ContentClusterer for HierarchicalClusterer {
    fn analyze_content(&self, titles: &[String]) -> Result<ContentAnalysis, ClusteringError> {
        // Use TF-IDF analyzer for content analysis
        let tfidf_analyzer = super::TfIdfAnalyzer::default();
        tfidf_analyzer.analyze_content(titles)
    }

    fn cluster_videos(
        &self,
        analysis: &ContentAnalysis,
        _target_clusters: usize,
    ) -> Result<Vec<VideoCluster>, ClusteringError> {
        //  uses distance_threshold to determine clusters
        self.cluster_hierarchical(&analysis.feature_vectors)
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
        let features: HashMap<String, f32> = terms
            .iter()
            .map(|(term, score)| (term.to_string(), *score))
            .collect();
        FeatureVector::new(features)
    }

    #[test]
    fn test_hierarchical_clusterer_creation() {
        let clusterer = HierarchicalClusterer::default();
        assert_eq!(clusterer.linkage_method, LinkageMethod::Average);
        assert_eq!(clusterer.distance_threshold, 0.7);
        assert_eq!(clusterer.min_cluster_size, 2);
        assert_eq!(clusterer.max_clusters, 10);
    }

    #[test]
    fn test_hierarchical_clusterer_custom_creation() {
        let clusterer = HierarchicalClusterer::new(LinkageMethod::Single, 0.5, 3, 5);
        assert_eq!(clusterer.linkage_method, LinkageMethod::Single);
        assert_eq!(clusterer.distance_threshold, 0.5);
        assert_eq!(clusterer.min_cluster_size, 3);
        assert_eq!(clusterer.max_clusters, 5);
    }

    #[test]
    fn test_distance_matrix_building() {
        let clusterer = HierarchicalClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("programming", 1.0)]),
            create_test_feature_vector(&[("database", 1.0)]),
            create_test_feature_vector(&[("programming", 0.8)]),
        ];

        let distance_matrix = clusterer.build_distance_matrix(&features);
        assert_eq!(distance_matrix.size, 3);
        assert_eq!(distance_matrix.distances.len(), 3);
        assert_eq!(distance_matrix.distances[0].len(), 3);

        // Distance from a point to itself should be 0
        assert_eq!(distance_matrix.distances[0][0], 0.0);
        assert_eq!(distance_matrix.distances[1][1], 0.0);
        assert_eq!(distance_matrix.distances[2][2], 0.0);

        // Matrix should be symmetric
        assert_eq!(
            distance_matrix.distances[0][1],
            distance_matrix.distances[1][0]
        );
    }

    #[test]
    fn test_hierarchical_clustering() {
        let clusterer = HierarchicalClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("programming", 1.0), ("basics", 0.5)]),
            create_test_feature_vector(&[("programming", 0.8), ("fundamentals", 0.7)]),
            create_test_feature_vector(&[("database", 1.0), ("sql", 0.6)]),
            create_test_feature_vector(&[("database", 0.9), ("design", 0.8)]),
        ];

        let result = clusterer.cluster_hierarchical(&features);
        assert!(result.is_ok());

        let clusters = result.unwrap();
        assert!(!clusters.is_empty());

        // Verify all points are assigned
        let total_points: usize = clusters.iter().map(|c| c.videos.len()).sum();
        assert_eq!(total_points, features.len());
    }

    #[test]
    fn test_hierarchical_clustering_insufficient_data() {
        let clusterer = HierarchicalClusterer::default();
        let features = vec![create_test_feature_vector(&[("test", 1.0)])];

        let result = clusterer.cluster_hierarchical(&features);
        assert!(matches!(
            result,
            Err(ClusteringError::InsufficientContent(1))
        ));
    }

    #[test]
    fn test_optimal_threshold_determination() {
        let clusterer = HierarchicalClusterer::default();
        let features = vec![
            create_test_feature_vector(&[("programming", 1.0)]),
            create_test_feature_vector(&[("programming", 0.9)]),
            create_test_feature_vector(&[("database", 1.0)]),
            create_test_feature_vector(&[("web", 1.0)]),
        ];

        let threshold = clusterer.determine_optimal_threshold(&features);
        assert!(threshold >= 0.3 && threshold <= 0.9);
    }

    #[test]
    fn test_linkage_methods() {
        let clusterer_single = HierarchicalClusterer::new(LinkageMethod::Single, 0.7, 2, 10);
        let clusterer_complete = HierarchicalClusterer::new(LinkageMethod::Complete, 0.7, 2, 10);
        let clusterer_average = HierarchicalClusterer::new(LinkageMethod::Average, 0.7, 2, 10);
        let clusterer_ward = HierarchicalClusterer::new(LinkageMethod::Ward, 0.7, 2, 10);

        assert_eq!(clusterer_single.linkage_method, LinkageMethod::Single);
        assert_eq!(clusterer_complete.linkage_method, LinkageMethod::Complete);
        assert_eq!(clusterer_average.linkage_method, LinkageMethod::Average);
        assert_eq!(clusterer_ward.linkage_method, LinkageMethod::Ward);
    }

    #[test]
    fn test_content_clusterer_interface() {
        let clusterer = HierarchicalClusterer::default();
        let titles = vec![
            "Introduction to Programming".to_string(),
            "Programming Fundamentals".to_string(),
            "Database Design".to_string(),
            "SQL Basics".to_string(),
        ];

        let analysis_result = clusterer.analyze_content(&titles);
        match analysis_result {
            Ok(analysis) => {
                let clusters = clusterer
                    .cluster_videos(&analysis, 0)
                    .expect("clustering failed");
                assert!(!clusters.is_empty());
            }
            Err(ClusteringError::InsufficientContent(_)) => {
                // Acceptable for small inputs: analyzer may require more content
                return;
            }
            Err(e) => panic!("unexpected content analysis error: {:?}", e),
        }
    }
}
