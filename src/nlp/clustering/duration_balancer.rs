//! Duration-aware cluster balancing for optimal session planning
//!
//! This module provides algorithms to balance cluster sizes based on video durations
//! while maintaining content coherence and respecting session time constraints.

use super::{OptimizedCluster, VideoWithMetadata};
use crate::types::PlanSettings;
use anyhow::Result;
use std::time::Duration;

/// Balanced cluster with duration optimization
#[derive(Debug, Clone)]
pub struct BalancedCluster {
    pub videos: Vec<VideoWithMetadata>,
    pub total_duration: Duration,
    pub target_duration: Duration,
    pub utilization_percentage: f32,
    pub balance_score: f32,
}

/// Duration balancer for cluster optimization
pub struct DurationBalancer {
    pub target_session_duration: Duration,
    pub max_session_duration: Duration,
    pub buffer_percentage: f32,
    pub min_cluster_size: usize,
    pub max_cluster_size: usize,
}

impl DurationBalancer {
    /// Create a new duration balancer from plan settings
    pub fn from_plan_settings(settings: &PlanSettings) -> Self {
        let target_duration = Duration::from_secs(settings.session_length_minutes as u64 * 60);
        let max_duration = Duration::from_secs((settings.session_length_minutes as f64 * 1.2) as u64 * 60);
        
        Self {
            target_session_duration: target_duration,
            max_session_duration: max_duration,
            buffer_percentage: 0.2, // 20% buffer for breaks and notes
            min_cluster_size: 1,
            max_cluster_size: 10,
        }
    }

    /// Create a new duration balancer with custom parameters
    pub fn new(
        target_session_duration: Duration,
        max_session_duration: Duration,
        buffer_percentage: f32,
    ) -> Self {
        Self {
            target_session_duration,
            max_session_duration,
            buffer_percentage,
            min_cluster_size: 1,
            max_cluster_size: 10,
        }
    }

    /// Balance clusters by duration constraints
    pub fn balance_clusters(&self, clusters: Vec<OptimizedCluster>) -> Result<Vec<BalancedCluster>> {
        let mut balanced_clusters = Vec::new();

        for cluster in clusters {
            let balanced = self.balance_single_cluster(cluster)?;
            balanced_clusters.extend(balanced);
        }

        // Post-process to merge small clusters and split large ones
        self.post_process_clusters(balanced_clusters)
    }

    /// Balance a single cluster by duration
    fn balance_single_cluster(&self, cluster: OptimizedCluster) -> Result<Vec<BalancedCluster>> {
        let effective_target = Duration::from_secs(
            (self.target_session_duration.as_secs() as f32 * (1.0 - self.buffer_percentage)) as u64
        );

        if cluster.total_duration <= effective_target {
            // Cluster fits within target, no splitting needed
            Ok(vec![self.create_balanced_cluster(cluster.videos, effective_target)])
        } else {
            // Cluster is too large, needs splitting
            self.split_cluster_by_duration(cluster.videos, effective_target)
        }
    }

    /// Split a cluster into smaller duration-balanced clusters
    fn split_cluster_by_duration(
        &self,
        mut videos: Vec<VideoWithMetadata>,
        target_duration: Duration,
    ) -> Result<Vec<BalancedCluster>> {
        // Sort videos by duration for better bin packing
        videos.sort_by_key(|v| v.duration);

        let mut clusters = Vec::new();
        let mut current_cluster = Vec::new();
        let mut current_duration = Duration::from_secs(0);

        for video in videos {
            // Check if adding this video would exceed the target
            if current_duration + video.duration > target_duration && !current_cluster.is_empty() {
                // Create cluster from current videos
                clusters.push(self.create_balanced_cluster(current_cluster, target_duration));
                current_cluster = Vec::new();
                current_duration = Duration::from_secs(0);
            }

            // Handle videos that are individually too long
            if video.duration > target_duration {
                // Create a dedicated cluster for this video
                if !current_cluster.is_empty() {
                    clusters.push(self.create_balanced_cluster(current_cluster, target_duration));
                    current_cluster = Vec::new();
                    current_duration = Duration::from_secs(0);
                }
                let video_duration = video.duration;
                clusters.push(self.create_balanced_cluster(vec![video], video_duration));
            } else {
                current_duration += video.duration;
                current_cluster.push(video);
            }
        }

        // Handle remaining videos
        if !current_cluster.is_empty() {
            clusters.push(self.create_balanced_cluster(current_cluster, target_duration));
        }

        Ok(clusters)
    }

    /// Create a balanced cluster from videos
    fn create_balanced_cluster(
        &self,
        videos: Vec<VideoWithMetadata>,
        target_duration: Duration,
    ) -> BalancedCluster {
        let total_duration: Duration = videos.iter().map(|v| v.duration).sum();
        let utilization_percentage = if target_duration.as_secs() > 0 {
            (total_duration.as_secs() as f32 / target_duration.as_secs() as f32) * 100.0
        } else {
            0.0
        };

        let balance_score = self.calculate_balance_score(&videos, total_duration, target_duration);

        BalancedCluster {
            videos,
            total_duration,
            target_duration,
            utilization_percentage,
            balance_score,
        }
    }

    /// Calculate balance score for a cluster
    fn calculate_balance_score(
        &self,
        videos: &[VideoWithMetadata],
        total_duration: Duration,
        target_duration: Duration,
    ) -> f32 {
        if videos.is_empty() {
            return 0.0;
        }

        // Duration balance component (closer to target is better)
        let duration_ratio = total_duration.as_secs() as f32 / target_duration.as_secs() as f32;
        let duration_score = if duration_ratio <= 1.0 {
            duration_ratio // Reward higher utilization up to 100%
        } else {
            1.0 / duration_ratio // Penalize over-utilization
        };

        // Size balance component (prefer moderate cluster sizes)
        let size_score = if videos.len() >= self.min_cluster_size && videos.len() <= self.max_cluster_size {
            1.0
        } else if videos.len() < self.min_cluster_size {
            videos.len() as f32 / self.min_cluster_size as f32
        } else {
            self.max_cluster_size as f32 / videos.len() as f32
        };

        // Duration variance component (prefer consistent video lengths)
        let avg_duration = total_duration.as_secs() as f32 / videos.len() as f32;
        let variance = videos
            .iter()
            .map(|v| {
                let diff = v.duration.as_secs() as f32 - avg_duration;
                diff * diff
            })
            .sum::<f32>() / videos.len() as f32;
        let variance_score = 1.0 / (1.0 + variance / (avg_duration * avg_duration));

        // Weighted combination
        (duration_score * 0.5) + (size_score * 0.3) + (variance_score * 0.2)
    }

    /// Post-process clusters to optimize overall balance
    fn post_process_clusters(&self, mut clusters: Vec<BalancedCluster>) -> Result<Vec<BalancedCluster>> {
        // Sort by balance score to identify problematic clusters
        clusters.sort_by(|a, b| a.balance_score.partial_cmp(&b.balance_score).unwrap_or(std::cmp::Ordering::Equal));

        let mut optimized = Vec::new();
        let mut i = 0;

        while i < clusters.len() {
            let current = &clusters[i];

            // Try to merge small clusters with the next one
            if current.utilization_percentage < 50.0 && i + 1 < clusters.len() {
                let next = &clusters[i + 1];
                let combined_duration = current.total_duration + next.total_duration;

                if combined_duration <= self.target_session_duration {
                    // Merge the clusters
                    let mut combined_videos = current.videos.clone();
                    combined_videos.extend(next.videos.clone());
                    
                    let merged = self.create_balanced_cluster(combined_videos, self.target_session_duration);
                    optimized.push(merged);
                    i += 2; // Skip both clusters
                    continue;
                }
            }

            optimized.push(current.clone());
            i += 1;
        }

        Ok(optimized)
    }

    /// Calculate overall balance metrics for a set of clusters
    pub fn calculate_balance_metrics(&self, clusters: &[BalancedCluster]) -> BalanceMetrics {
        if clusters.is_empty() {
            return BalanceMetrics::default();
        }

        let total_videos: usize = clusters.iter().map(|c| c.videos.len()).sum();
        let total_duration: Duration = clusters.iter().map(|c| c.total_duration).sum();
        let average_utilization = clusters.iter().map(|c| c.utilization_percentage).sum::<f32>() / clusters.len() as f32;
        let average_balance_score = clusters.iter().map(|c| c.balance_score).sum::<f32>() / clusters.len() as f32;

        let utilization_variance = clusters
            .iter()
            .map(|c| {
                let diff = c.utilization_percentage - average_utilization;
                diff * diff
            })
            .sum::<f32>() / clusters.len() as f32;

        let underutilized_clusters = clusters.iter().filter(|c| c.utilization_percentage < 70.0).count();
        let overutilized_clusters = clusters.iter().filter(|c| c.utilization_percentage > 120.0).count();

        BalanceMetrics {
            total_clusters: clusters.len(),
            total_videos,
            total_duration,
            average_utilization,
            utilization_variance,
            average_balance_score,
            underutilized_clusters,
            overutilized_clusters,
        }
    }
}

/// Balance metrics for cluster analysis
#[derive(Debug, Clone, Default)]
pub struct BalanceMetrics {
    pub total_clusters: usize,
    pub total_videos: usize,
    pub total_duration: Duration,
    pub average_utilization: f32,
    pub utilization_variance: f32,
    pub average_balance_score: f32,
    pub underutilized_clusters: usize,
    pub overutilized_clusters: usize,
}

impl BalanceMetrics {
    /// Check if the balance is acceptable
    pub fn is_well_balanced(&self) -> bool {
        self.average_balance_score > 0.7 &&
        self.utilization_variance < 500.0 &&
        (self.underutilized_clusters as f32 / self.total_clusters as f32) < 0.3 &&
        self.overutilized_clusters == 0
    }

    /// Get a quality assessment string
    pub fn quality_assessment(&self) -> String {
        if self.is_well_balanced() {
            "Well balanced".to_string()
        } else if self.overutilized_clusters > 0 {
            "Some sessions too long".to_string()
        } else if self.underutilized_clusters as f32 / self.total_clusters as f32 > 0.5 {
            "Many sessions too short".to_string()
        } else {
            "Moderately balanced".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PlanSettings;
    use chrono::Utc;

    fn create_test_video(index: usize, duration_minutes: u64) -> VideoWithMetadata {
        VideoWithMetadata {
            index,
            title: format!("Video {}", index),
            duration: Duration::from_secs(duration_minutes * 60),
            feature_vector: Default::default(),
            difficulty_score: 0.5,
            topic_tags: Vec::new(),
        }
    }

    fn create_test_cluster(videos: Vec<VideoWithMetadata>) -> OptimizedCluster {
        let total_duration = videos.iter().map(|v| v.duration).sum();
        OptimizedCluster {
            videos,
            total_duration,
            average_similarity: 0.8,
            difficulty_level: crate::types::DifficultyLevel::Intermediate,
            suggested_title: "Test Cluster".to_string(),
        }
    }

    #[test]
    fn test_duration_balancer_creation() {
        let settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        };

        let balancer = DurationBalancer::from_plan_settings(&settings);
        assert_eq!(balancer.target_session_duration, Duration::from_secs(3600));
        assert_eq!(balancer.buffer_percentage, 0.2);
    }

    #[test]
    fn test_cluster_fits_target() {
        let balancer = DurationBalancer::new(
            Duration::from_secs(3600), // 60 minutes
            Duration::from_secs(4320), // 72 minutes
            0.2,
        );

        let videos = vec![
            create_test_video(0, 15),
            create_test_video(1, 15),
            create_test_video(2, 15),
        ];
        let cluster = create_test_cluster(videos);

        let result = balancer.balance_single_cluster(cluster).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].videos.len(), 3);
    }

    #[test]
    fn test_cluster_splitting() {
        let balancer = DurationBalancer::new(
            Duration::from_secs(3600), // 60 minutes
            Duration::from_secs(4320), // 72 minutes
            0.2,
        );

        let videos = vec![
            create_test_video(0, 30),
            create_test_video(1, 35),
            create_test_video(2, 40),
            create_test_video(3, 25),
        ];
        let cluster = create_test_cluster(videos);

        let result = balancer.balance_single_cluster(cluster).unwrap();
        assert!(result.len() > 1); // Should be split into multiple clusters
    }

    #[test]
    fn test_balance_score_calculation() {
        let balancer = DurationBalancer::new(
            Duration::from_secs(3600),
            Duration::from_secs(4320),
            0.2,
        );

        let videos = vec![
            create_test_video(0, 20),
            create_test_video(1, 20),
            create_test_video(2, 20),
        ];

        let total_duration = Duration::from_secs(3600); // 60 minutes
        let target_duration = Duration::from_secs(3600);
        
        let score = balancer.calculate_balance_score(&videos, total_duration, target_duration);
        assert!(score > 0.8); // Should be well balanced
    }

    #[test]
    fn test_balance_metrics() {
        let balancer = DurationBalancer::new(
            Duration::from_secs(3600),
            Duration::from_secs(4320),
            0.2,
        );

        let clusters = vec![
            BalancedCluster {
                videos: vec![create_test_video(0, 30)],
                total_duration: Duration::from_secs(1800),
                target_duration: Duration::from_secs(3600),
                utilization_percentage: 50.0,
                balance_score: 0.8,
            },
            BalancedCluster {
                videos: vec![create_test_video(1, 60)],
                total_duration: Duration::from_secs(3600),
                target_duration: Duration::from_secs(3600),
                utilization_percentage: 100.0,
                balance_score: 0.9,
            },
        ];

        let metrics = balancer.calculate_balance_metrics(&clusters);
        assert_eq!(metrics.total_clusters, 2);
        assert_eq!(metrics.total_videos, 2);
        assert_eq!(metrics.underutilized_clusters, 1);
        assert_eq!(metrics.overutilized_clusters, 0);
    }
}