//! Duration-aware cluster balancing for optimal session planning
//!
//! This module provides algorithms to balance cluster sizes based on video durations
//! while maintaining content coherence and respecting session time constraints.

use super::{OptimizedCluster, VideoWithMetadata};
use crate::types::PlanSettings;
use anyhow::Result;
use std::collections::HashMap;
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
    pub similarity_threshold: f32,
    pub max_duration_variance: f32,
}

impl DurationBalancer {
    /// Create a new duration balancer from plan settings
    pub fn from_plan_settings(settings: &PlanSettings) -> Self {
        let target_duration = Duration::from_secs(settings.session_length_minutes as u64 * 60);
        let max_duration =
            Duration::from_secs((settings.session_length_minutes as f64 * 1.2) as u64 * 60);

        Self {
            target_session_duration: target_duration,
            max_session_duration: max_duration,
            buffer_percentage: 0.2, // 20% buffer for breaks and notes
            min_cluster_size: 1,
            max_cluster_size: 10,
            similarity_threshold: 0.6, // Minimum similarity to maintain coherence
            max_duration_variance: 0.5, // Maximum allowed variance in cluster durations
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
            similarity_threshold: 0.6,
            max_duration_variance: 0.5,
        }
    }

    /// Create a new duration balancer with full configuration
    pub fn with_config(
        target_session_duration: Duration,
        max_session_duration: Duration,
        buffer_percentage: f32,
        similarity_threshold: f32,
        max_duration_variance: f32,
    ) -> Self {
        Self {
            target_session_duration,
            max_session_duration,
            buffer_percentage,
            min_cluster_size: 1,
            max_cluster_size: 10,
            similarity_threshold,
            max_duration_variance,
        }
    }

    /// Balance clusters by duration constraints with advanced optimization
    pub fn balance_clusters(
        &self,
        clusters: Vec<OptimizedCluster>,
    ) -> Result<Vec<BalancedCluster>> {
        let mut balanced_clusters = Vec::new();

        for cluster in clusters {
            let balanced = self.balance_single_cluster(cluster)?;
            balanced_clusters.extend(balanced);
        }

        // Apply advanced rebalancing algorithms
        let rebalanced = self.rebalance_clusters(balanced_clusters)?;
        let optimized = self.apply_bin_packing_optimization(rebalanced)?;

        // Final post-processing with content coherence checks
        self.post_process_with_coherence(optimized)
    }

    /// Advanced cluster rebalancing to avoid extremely long/short modules
    pub fn rebalance_clusters(
        &self,
        clusters: Vec<BalancedCluster>,
    ) -> Result<Vec<BalancedCluster>> {
        let mut working_clusters = clusters;
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 5;

        // Iterative rebalancing until convergence or max iterations
        while iteration < MAX_ITERATIONS {
            let mut rebalanced = Vec::new();
            let mut changed = false;
            let mut i = 0;

            while i < working_clusters.len() {
                let current_utilization = working_clusters[i].utilization_percentage;
                let needs_rebalancing = self.needs_rebalancing(&working_clusters[i]);

                // Check if cluster needs rebalancing
                if needs_rebalancing {
                    if current_utilization < 30.0 {
                        // Try to merge with similar small clusters
                        // Clone the working_clusters to avoid borrow conflicts
                        let clusters_clone = working_clusters.clone();
                        if let Some(merged_clusters) =
                            self.try_merge_small_clusters(&mut working_clusters, i)?
                        {
                            rebalanced.extend(merged_clusters);
                            changed = true;
                            continue;
                        } else if let Some((merged, merged_indices)) =
                            self.try_merge_small_clusters_advanced(&clusters_clone, i)?
                        {
                            rebalanced.extend(merged);
                            // Skip merged clusters
                            let mut skip_indices = merged_indices;
                            skip_indices.sort_by(|a, b| b.cmp(a)); // Sort in descending order
                            for &idx in &skip_indices {
                                if idx > i {
                                    working_clusters.remove(idx);
                                }
                            }
                            changed = true;
                            continue;
                        }
                    } else if current_utilization > 150.0 {
                        // Split oversized clusters using the dedicated function
                        let current_cluster = working_clusters[i].clone();
                        let split_clusters = self.split_oversized_cluster(current_cluster)?;
                        if split_clusters.len() > 1 {
                            rebalanced.extend(split_clusters);
                            changed = true;
                            i += 1;
                            continue;
                        }

                        // Fallback to advanced splitting if basic splitting didn't work
                        let current_cluster = working_clusters[i].clone();
                        let split_clusters =
                            self.split_oversized_cluster_advanced(current_cluster)?;
                        if split_clusters.len() > 1 {
                            rebalanced.extend(split_clusters);
                            changed = true;
                            i += 1;
                            continue;
                        }
                    }
                }

                rebalanced.push(working_clusters[i].clone());
                i += 1;
            }

            working_clusters = rebalanced;

            // If no changes were made, we've reached convergence
            if !changed {
                break;
            }

            iteration += 1;
        }

        // Final optimization pass
        self.final_optimization_pass(working_clusters)
    }

    /// Apply bin-packing optimization for duration distribution
    pub fn apply_bin_packing_optimization(
        &self,
        clusters: Vec<BalancedCluster>,
    ) -> Result<Vec<BalancedCluster>> {
        // Extract all videos from clusters for redistribution
        let mut all_videos: Vec<VideoWithMetadata> =
            clusters.into_iter().flat_map(|c| c.videos).collect();

        // Sort videos by duration (descending) for better bin packing
        all_videos.sort_by(|a, b| b.duration.cmp(&a.duration));

        // Apply First Fit Decreasing (FFD) bin packing algorithm
        let bins = self.first_fit_decreasing_pack(&all_videos)?;

        // Convert bins back to balanced clusters
        let mut optimized_clusters = Vec::new();
        for (i, bin) in bins.into_iter().enumerate() {
            if !bin.videos.is_empty() {
                let cluster = self.create_balanced_cluster_from_bin(bin, i)?;
                optimized_clusters.push(cluster);
            }
        }

        Ok(optimized_clusters)
    }

    /// Balance a single cluster by duration
    fn balance_single_cluster(&self, cluster: OptimizedCluster) -> Result<Vec<BalancedCluster>> {
        let effective_target = Duration::from_secs(
            (self.target_session_duration.as_secs() as f32 * (1.0 - self.buffer_percentage)) as u64,
        );

        if cluster.total_duration <= effective_target {
            // Cluster fits within target, no splitting needed
            Ok(vec![
                self.create_balanced_cluster(cluster.videos, effective_target),
            ])
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
        let size_score =
            if videos.len() >= self.min_cluster_size && videos.len() <= self.max_cluster_size {
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
            .sum::<f32>()
            / videos.len() as f32;
        let variance_score = 1.0 / (1.0 + variance / (avg_duration * avg_duration));

        // Weighted combination
        (duration_score * 0.5) + (size_score * 0.3) + (variance_score * 0.2)
    }

    /// Post-process clusters with content coherence maintenance
    fn post_process_with_coherence(
        &self,
        mut clusters: Vec<BalancedCluster>,
    ) -> Result<Vec<BalancedCluster>> {
        // Sort by balance score to identify problematic clusters
        clusters.sort_by(|a, b| {
            a.balance_score
                .partial_cmp(&b.balance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut optimized = Vec::new();
        let mut i = 0;

        while i < clusters.len() {
            let current = &clusters[i];

            // Try to merge small clusters with content coherence check
            if current.utilization_percentage < 50.0 && i + 1 < clusters.len() {
                let next = &clusters[i + 1];
                let combined_duration = current.total_duration + next.total_duration;

                if combined_duration <= self.target_session_duration {
                    // Check content coherence before merging
                    let coherence_score =
                        self.calculate_content_coherence(&current.videos, &next.videos);

                    if coherence_score >= self.similarity_threshold {
                        // Merge the clusters with good coherence
                        let mut combined_videos = current.videos.clone();
                        combined_videos.extend(next.videos.clone());

                        let merged = self
                            .create_balanced_cluster(combined_videos, self.target_session_duration);
                        optimized.push(merged);
                        i += 2; // Skip both clusters
                        continue;
                    }
                }
            }

            optimized.push(current.clone());
            i += 1;
        }

        Ok(optimized)
    }

    /// Check if a cluster needs rebalancing
    fn needs_rebalancing(&self, cluster: &BalancedCluster) -> bool {
        cluster.utilization_percentage < 30.0
            || cluster.utilization_percentage > 150.0
            || cluster.balance_score < 0.5
    }

    /// Try to merge small clusters while maintaining content coherence
    fn try_merge_small_clusters(
        &self,
        clusters: &mut [BalancedCluster],
        start_index: usize,
    ) -> Result<Option<Vec<BalancedCluster>>> {
        let current = &clusters[start_index];

        // Look for other small clusters to merge with
        for i in (start_index + 1)..clusters.len() {
            let candidate = &clusters[i];

            if candidate.utilization_percentage < 50.0 {
                let combined_duration = current.total_duration + candidate.total_duration;

                if combined_duration <= self.max_session_duration {
                    let coherence =
                        self.calculate_content_coherence(&current.videos, &candidate.videos);

                    if coherence >= self.similarity_threshold {
                        // Create merged cluster
                        let mut combined_videos = current.videos.clone();
                        combined_videos.extend(candidate.videos.clone());

                        let merged = self
                            .create_balanced_cluster(combined_videos, self.target_session_duration);

                        // Remove the merged clusters from the original list
                        // This is a simplified approach - in practice, we'd need more sophisticated handling
                        return Ok(Some(vec![merged]));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Advanced cluster merging with multiple candidate evaluation
    fn try_merge_small_clusters_advanced(
        &self,
        clusters: &[BalancedCluster],
        start_index: usize,
    ) -> Result<Option<(Vec<BalancedCluster>, Vec<usize>)>> {
        let current = &clusters[start_index];

        if current.utilization_percentage >= 50.0 {
            return Ok(None);
        }

        let mut best_merge: Option<(Vec<BalancedCluster>, Vec<usize>, f32)> = None;

        // Evaluate all possible merge candidates
        for i in (start_index + 1)..clusters.len() {
            let candidate = &clusters[i];

            if candidate.utilization_percentage < 70.0 {
                let combined_duration = current.total_duration + candidate.total_duration;

                // Check if merge is feasible
                if combined_duration <= self.max_session_duration {
                    let coherence =
                        self.calculate_content_coherence(&current.videos, &candidate.videos);

                    if coherence >= self.similarity_threshold {
                        // Calculate merge quality score
                        let merge_score =
                            self.calculate_merge_quality_score(current, candidate, coherence);

                        // Create merged cluster
                        let mut combined_videos = current.videos.clone();
                        combined_videos.extend(candidate.videos.clone());
                        let merged = self
                            .create_balanced_cluster(combined_videos, self.target_session_duration);

                        // Check if this is the best merge so far
                        if best_merge.is_none() || merge_score > best_merge.as_ref().unwrap().2 {
                            best_merge = Some((vec![merged], vec![start_index, i], merge_score));
                        }
                    }
                }
            }
        }

        // Try multi-way merges for very small clusters
        if current.utilization_percentage < 20.0 {
            if let Some(multi_merge) = self.try_multi_way_merge(clusters, start_index)? {
                let (merged_clusters, indices, score) = multi_merge;
                if best_merge.is_none() || score > best_merge.as_ref().unwrap().2 {
                    best_merge = Some((merged_clusters, indices, score));
                }
            }
        }

        Ok(best_merge.map(|(clusters, indices, _)| (clusters, indices)))
    }

    /// Calculate quality score for a potential merge
    fn calculate_merge_quality_score(
        &self,
        cluster1: &BalancedCluster,
        cluster2: &BalancedCluster,
        coherence: f32,
    ) -> f32 {
        let combined_duration = cluster1.total_duration + cluster2.total_duration;
        let target_utilization =
            combined_duration.as_secs() as f32 / self.target_session_duration.as_secs() as f32;

        // Prefer merges that result in good utilization (70-100%)
        let utilization_score = if target_utilization >= 0.7 && target_utilization <= 1.0 {
            target_utilization
        } else if target_utilization < 0.7 {
            target_utilization * 0.8 // Penalize under-utilization
        } else {
            1.0 / target_utilization // Penalize over-utilization
        };

        // Balance score components
        let size_balance = 1.0
            - ((cluster1.videos.len() as f32 - cluster2.videos.len() as f32).abs()
                / (cluster1.videos.len() + cluster2.videos.len()) as f32);

        // Weighted combination
        (coherence * 0.4) + (utilization_score * 0.4) + (size_balance * 0.2)
    }

    /// Try multi-way merge for very small clusters
    fn try_multi_way_merge(
        &self,
        clusters: &[BalancedCluster],
        start_index: usize,
    ) -> Result<Option<(Vec<BalancedCluster>, Vec<usize>, f32)>> {
        let current = &clusters[start_index];

        if current.utilization_percentage >= 20.0 {
            return Ok(None);
        }

        let mut candidates = vec![start_index];
        let mut combined_duration = current.total_duration;
        let mut combined_videos = current.videos.clone();

        // Find other very small clusters that can be merged together
        for i in (start_index + 1)..clusters.len() {
            let candidate = &clusters[i];

            if candidate.utilization_percentage < 30.0 {
                let test_duration = combined_duration + candidate.total_duration;

                if test_duration <= self.max_session_duration {
                    let coherence =
                        self.calculate_content_coherence(&combined_videos, &candidate.videos);

                    if coherence >= (self.similarity_threshold * 0.8) {
                        // Slightly relaxed threshold for multi-merge
                        candidates.push(i);
                        combined_duration = test_duration;
                        combined_videos.extend(candidate.videos.clone());

                        // Stop if we have enough content
                        if combined_duration.as_secs() as f32
                            / self.target_session_duration.as_secs() as f32
                            > 0.7
                        {
                            break;
                        }
                    }
                }
            }
        }

        // Only proceed if we're merging at least 3 clusters
        if candidates.len() >= 3 {
            let merged =
                self.create_balanced_cluster(combined_videos, self.target_session_duration);
            let score = merged.balance_score * 1.1; // Bonus for multi-way merge
            Ok(Some((vec![merged], candidates, score)))
        } else {
            Ok(None)
        }
    }

    /// Advanced cluster splitting with content coherence preservation
    fn split_oversized_cluster_advanced(
        &self,
        cluster: BalancedCluster,
    ) -> Result<Vec<BalancedCluster>> {
        if cluster.utilization_percentage <= 150.0 {
            return Ok(vec![cluster]);
        }

        let mut videos = cluster.videos;

        // Sort videos by content similarity to maintain coherence in splits
        // Create a copy of videos for similarity calculations to avoid borrow conflicts
        let videos_for_similarity = videos.clone();
        videos.sort_by(|a, b| {
            let sim_a = self.calculate_average_similarity_to_group(a, &videos_for_similarity);
            let sim_b = self.calculate_average_similarity_to_group(b, &videos_for_similarity);
            sim_b
                .partial_cmp(&sim_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate optimal split points using dynamic programming approach
        let split_points = self.find_optimal_split_points(&videos)?;

        let mut split_clusters = Vec::new();
        let mut start_idx = 0;

        for &split_point in &split_points {
            if split_point > start_idx {
                let group_videos = videos[start_idx..split_point].to_vec();
                if !group_videos.is_empty() {
                    let split_cluster =
                        self.create_balanced_cluster(group_videos, self.target_session_duration);
                    split_clusters.push(split_cluster);
                }
                start_idx = split_point;
            }
        }

        // Handle remaining videos
        if start_idx < videos.len() {
            let remaining_videos = videos[start_idx..].to_vec();
            if !remaining_videos.is_empty() {
                let final_cluster =
                    self.create_balanced_cluster(remaining_videos, self.target_session_duration);
                split_clusters.push(final_cluster);
            }
        }

        // Ensure we actually split the cluster
        if split_clusters.len() <= 1 {
            // Fallback to simple duration-based splitting
            return self.split_cluster_by_duration(videos, self.target_session_duration);
        }

        Ok(split_clusters)
    }

    /// Find optimal split points using dynamic programming
    fn find_optimal_split_points(&self, videos: &[VideoWithMetadata]) -> Result<Vec<usize>> {
        if videos.len() < 2 {
            return Ok(Vec::new());
        }

        let n = videos.len();
        let target_secs = self.target_session_duration.as_secs() as f32;

        // Calculate cumulative durations
        let mut cumulative_durations = vec![0.0; n + 1];
        for i in 0..n {
            cumulative_durations[i + 1] =
                cumulative_durations[i] + videos[i].duration.as_secs() as f32;
        }

        // Dynamic programming to find optimal splits
        let mut dp = vec![f32::INFINITY; n + 1];
        let mut splits = vec![Vec::new(); n + 1];
        dp[0] = 0.0;

        for i in 1..=n {
            for j in 0..i {
                let segment_duration = cumulative_durations[i] - cumulative_durations[j];
                let segment_videos = &videos[j..i];

                // Calculate cost for this segment
                let duration_cost = if segment_duration <= target_secs {
                    (target_secs - segment_duration) / target_secs // Under-utilization penalty
                } else {
                    (segment_duration - target_secs) / target_secs * 2.0 // Over-utilization penalty (higher)
                };

                let coherence_cost = if segment_videos.len() > 1 {
                    1.0 - self.calculate_intra_cluster_coherence(segment_videos)
                } else {
                    0.0
                };

                let total_cost = dp[j] + duration_cost + coherence_cost;

                if total_cost < dp[i] {
                    dp[i] = total_cost;
                    splits[i] = splits[j].clone();
                    if j > 0 {
                        splits[i].push(j);
                    }
                }
            }
        }

        let mut result = splits[n].clone();
        result.push(n);
        Ok(result)
    }

    /// Final optimization pass to fine-tune the balanced clusters
    fn final_optimization_pass(
        &self,
        clusters: Vec<BalancedCluster>,
    ) -> Result<Vec<BalancedCluster>> {
        let mut optimized = clusters;

        // Sort by balance score to prioritize problematic clusters
        optimized.sort_by(|a, b| {
            a.balance_score
                .partial_cmp(&b.balance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply final optimizations
        for cluster in &mut optimized {
            // Reorder videos within cluster for better flow
            self.optimize_video_order_within_cluster(cluster);

            // Recalculate balance score after reordering
            cluster.balance_score = self.calculate_balance_score(
                &cluster.videos,
                cluster.total_duration,
                cluster.target_duration,
            );
        }

        // Sort back by original order or by some logical sequence
        optimized.sort_by(|a, b| {
            if a.videos.is_empty() || b.videos.is_empty() {
                return std::cmp::Ordering::Equal;
            }
            a.videos[0].index.cmp(&b.videos[0].index)
        });

        Ok(optimized)
    }

    /// Optimize video order within a cluster for better learning flow
    fn optimize_video_order_within_cluster(&self, cluster: &mut BalancedCluster) {
        if cluster.videos.len() <= 2 {
            return;
        }

        // Sort by difficulty progression (easier to harder)
        cluster.videos.sort_by(|a, b| {
            a.difficulty_score
                .partial_cmp(&b.difficulty_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Then by duration (shorter to longer within same difficulty)
        cluster.videos.sort_by(|a, b| {
            let diff_cmp = a
                .difficulty_score
                .partial_cmp(&b.difficulty_score)
                .unwrap_or(std::cmp::Ordering::Equal);
            if diff_cmp == std::cmp::Ordering::Equal {
                a.duration.cmp(&b.duration)
            } else {
                diff_cmp
            }
        });
    }

    /// Split oversized clusters while maintaining content coherence
    fn split_oversized_cluster(&self, cluster: BalancedCluster) -> Result<Vec<BalancedCluster>> {
        let mut videos = cluster.videos;

        // Sort by content similarity to maintain coherence in splits
        // Create a copy of videos for similarity calculations to avoid borrow conflicts
        let videos_for_similarity = videos.clone();
        videos.sort_by(|a, b| {
            let sim_a = self.calculate_average_similarity_to_group(a, &videos_for_similarity);
            let sim_b = self.calculate_average_similarity_to_group(b, &videos_for_similarity);
            sim_b
                .partial_cmp(&sim_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let effective_target = Duration::from_secs(
            (self.target_session_duration.as_secs() as f32 * (1.0 - self.buffer_percentage)) as u64,
        );

        let mut split_clusters = Vec::new();
        let mut current_group = Vec::new();
        let mut current_duration = Duration::from_secs(0);

        for video in videos {
            // Check if adding this video would exceed target
            if current_duration + video.duration > effective_target && !current_group.is_empty() {
                // Create cluster from current group
                split_clusters.push(self.create_balanced_cluster(current_group, effective_target));
                current_group = Vec::new();
                current_duration = Duration::from_secs(0);
            }

            current_duration += video.duration;
            current_group.push(video);
        }

        // Handle remaining videos
        if !current_group.is_empty() {
            split_clusters.push(self.create_balanced_cluster(current_group, effective_target));
        }

        Ok(split_clusters)
    }

    /// First Fit Decreasing bin packing algorithm
    fn first_fit_decreasing_pack(&self, videos: &[VideoWithMetadata]) -> Result<Vec<DurationBin>> {
        let mut bins: Vec<DurationBin> = Vec::new();
        let effective_capacity = Duration::from_secs(
            (self.target_session_duration.as_secs() as f32 * (1.0 - self.buffer_percentage)) as u64,
        );

        for video in videos {
            let mut placed = false;

            // Try to place in existing bin
            for bin in &mut bins {
                if bin.can_fit(&video.duration) && self.maintains_content_coherence(bin, video) {
                    bin.add_video(video.clone());
                    placed = true;
                    break;
                }
            }

            // Create new bin if couldn't place in existing ones
            if !placed {
                let mut new_bin = DurationBin::new(effective_capacity);
                new_bin.add_video(video.clone());
                bins.push(new_bin);
            }
        }

        Ok(bins)
    }

    /// Create balanced cluster from duration bin
    fn create_balanced_cluster_from_bin(
        &self,
        bin: DurationBin,
        _index: usize,
    ) -> Result<BalancedCluster> {
        let cluster = self.create_balanced_cluster(bin.videos, bin.capacity);
        Ok(cluster)
    }

    /// Calculate content coherence between two video groups
    fn calculate_content_coherence(
        &self,
        group1: &[VideoWithMetadata],
        group2: &[VideoWithMetadata],
    ) -> f32 {
        if group1.is_empty() || group2.is_empty() {
            return 0.0;
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for video1 in group1 {
            for video2 in group2 {
                total_similarity += video1
                    .feature_vector
                    .cosine_similarity(&video2.feature_vector);
                count += 1;
            }
        }

        if count > 0 {
            total_similarity / count as f32
        } else {
            0.0
        }
    }

    /// Calculate average similarity of a video to a group
    fn calculate_average_similarity_to_group(
        &self,
        video: &VideoWithMetadata,
        group: &[VideoWithMetadata],
    ) -> f32 {
        if group.len() <= 1 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for other in group {
            if other.index != video.index {
                total_similarity += video
                    .feature_vector
                    .cosine_similarity(&other.feature_vector);
                count += 1;
            }
        }

        if count > 0 {
            total_similarity / count as f32
        } else {
            1.0
        }
    }

    /// Check if adding a video maintains content coherence in a bin
    fn maintains_content_coherence(&self, bin: &DurationBin, video: &VideoWithMetadata) -> bool {
        if bin.videos.is_empty() {
            return true;
        }

        let coherence = self.calculate_content_coherence(&bin.videos, &[video.clone()]);
        coherence >= self.similarity_threshold
    }

    /// Calculate overall balance metrics for a set of clusters
    pub fn calculate_balance_metrics(&self, clusters: &[BalancedCluster]) -> BalanceMetrics {
        if clusters.is_empty() {
            return BalanceMetrics::default();
        }

        let total_videos: usize = clusters.iter().map(|c| c.videos.len()).sum();
        let total_duration: Duration = clusters.iter().map(|c| c.total_duration).sum();
        let average_utilization = clusters
            .iter()
            .map(|c| c.utilization_percentage)
            .sum::<f32>()
            / clusters.len() as f32;
        let average_balance_score =
            clusters.iter().map(|c| c.balance_score).sum::<f32>() / clusters.len() as f32;

        let utilization_variance = clusters
            .iter()
            .map(|c| {
                let diff = c.utilization_percentage - average_utilization;
                diff * diff
            })
            .sum::<f32>()
            / clusters.len() as f32;

        let underutilized_clusters = clusters
            .iter()
            .filter(|c| c.utilization_percentage < 70.0)
            .count();
        let overutilized_clusters = clusters
            .iter()
            .filter(|c| c.utilization_percentage > 120.0)
            .count();

        // Calculate content coherence score
        let content_coherence_score = self.calculate_overall_content_coherence(clusters);

        // Calculate duration variance score
        let duration_variance_score = self.calculate_duration_variance_score(clusters);

        // Calculate bin packing efficiency
        let bin_packing_efficiency = self.calculate_bin_packing_efficiency(clusters);

        BalanceMetrics {
            total_clusters: clusters.len(),
            total_videos,
            total_duration,
            average_utilization,
            utilization_variance,
            average_balance_score,
            underutilized_clusters,
            overutilized_clusters,
            content_coherence_score,
            duration_variance_score,
            bin_packing_efficiency,
        }
    }

    /// Calculate overall content coherence across all clusters
    fn calculate_overall_content_coherence(&self, clusters: &[BalancedCluster]) -> f32 {
        if clusters.is_empty() {
            return 0.0;
        }

        let mut total_coherence = 0.0;
        let mut count = 0;

        for cluster in clusters {
            if cluster.videos.len() > 1 {
                let coherence = self.calculate_intra_cluster_coherence(&cluster.videos);
                total_coherence += coherence;
                count += 1;
            }
        }

        if count > 0 {
            total_coherence / count as f32
        } else {
            1.0 // Single video clusters are perfectly coherent
        }
    }

    /// Calculate coherence within a single cluster
    fn calculate_intra_cluster_coherence(&self, videos: &[VideoWithMetadata]) -> f32 {
        if videos.len() < 2 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for i in 0..videos.len() {
            for j in (i + 1)..videos.len() {
                total_similarity += videos[i]
                    .feature_vector
                    .cosine_similarity(&videos[j].feature_vector);
                count += 1;
            }
        }

        if count > 0 {
            total_similarity / count as f32
        } else {
            1.0
        }
    }

    /// Calculate duration variance score (lower variance is better)
    fn calculate_duration_variance_score(&self, clusters: &[BalancedCluster]) -> f32 {
        if clusters.is_empty() {
            return 1.0;
        }

        let durations: Vec<f32> = clusters
            .iter()
            .map(|c| c.total_duration.as_secs() as f32)
            .collect();

        let mean = durations.iter().sum::<f32>() / durations.len() as f32;
        let variance = durations
            .iter()
            .map(|d| {
                let diff = d - mean;
                diff * diff
            })
            .sum::<f32>()
            / durations.len() as f32;

        let coefficient_of_variation = if mean > 0.0 {
            (variance.sqrt() / mean).min(1.0)
        } else {
            0.0
        };

        1.0 - coefficient_of_variation // Higher score for lower variance
    }

    /// Calculate bin packing efficiency
    fn calculate_bin_packing_efficiency(&self, clusters: &[BalancedCluster]) -> f32 {
        if clusters.is_empty() {
            return 0.0;
        }

        let total_content_duration: Duration = clusters.iter().map(|c| c.total_duration).sum();

        let total_capacity: Duration =
            Duration::from_secs(clusters.len() as u64 * self.target_session_duration.as_secs());

        if total_capacity.as_secs() > 0 {
            total_content_duration.as_secs() as f32 / total_capacity.as_secs() as f32
        } else {
            0.0
        }
    }
}

/// Duration bin for bin-packing algorithm
#[derive(Debug, Clone)]
pub struct DurationBin {
    pub videos: Vec<VideoWithMetadata>,
    pub current_duration: Duration,
    pub capacity: Duration,
    pub utilization_percentage: f32,
}

impl DurationBin {
    /// Create a new duration bin with specified capacity
    pub fn new(capacity: Duration) -> Self {
        Self {
            videos: Vec::new(),
            current_duration: Duration::from_secs(0),
            capacity,
            utilization_percentage: 0.0,
        }
    }

    /// Check if a video can fit in this bin
    pub fn can_fit(&self, video_duration: &Duration) -> bool {
        self.current_duration + *video_duration <= self.capacity
    }

    /// Add a video to this bin
    pub fn add_video(&mut self, video: VideoWithMetadata) {
        self.current_duration += video.duration;
        self.videos.push(video);
        self.update_utilization();
    }

    /// Update utilization percentage
    fn update_utilization(&mut self) {
        if self.capacity.as_secs() > 0 {
            self.utilization_percentage =
                (self.current_duration.as_secs() as f32 / self.capacity.as_secs() as f32) * 100.0;
        }
    }

    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> Duration {
        if self.current_duration <= self.capacity {
            self.capacity - self.current_duration
        } else {
            Duration::from_secs(0)
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
    pub content_coherence_score: f32,
    pub duration_variance_score: f32,
    pub bin_packing_efficiency: f32,
}

impl BalanceMetrics {
    /// Check if the balance is acceptable with enhanced criteria
    pub fn is_well_balanced(&self) -> bool {
        self.average_balance_score > 0.7
            && self.utilization_variance < 500.0
            && (self.underutilized_clusters as f32 / self.total_clusters as f32) < 0.3
            && self.overutilized_clusters == 0
            && self.content_coherence_score > 0.6
            && self.duration_variance_score > 0.7
            && self.bin_packing_efficiency > 0.8
    }

    /// Get a quality assessment string with detailed analysis
    pub fn quality_assessment(&self) -> String {
        if self.is_well_balanced() {
            "Excellently balanced".to_string()
        } else if self.overutilized_clusters > 0 {
            "Some sessions exceed time limits".to_string()
        } else if self.underutilized_clusters as f32 / self.total_clusters as f32 > 0.5 {
            "Many sessions are too short".to_string()
        } else if self.content_coherence_score < 0.5 {
            "Content grouping needs improvement".to_string()
        } else if self.duration_variance_score < 0.5 {
            "Session durations are inconsistent".to_string()
        } else if self.bin_packing_efficiency < 0.7 {
            "Time utilization could be improved".to_string()
        } else {
            "Moderately balanced".to_string()
        }
    }

    /// Get detailed balance report
    pub fn detailed_report(&self) -> HashMap<String, f32> {
        let mut report = HashMap::new();
        report.insert("average_utilization".to_string(), self.average_utilization);
        report.insert(
            "content_coherence".to_string(),
            self.content_coherence_score,
        );
        report.insert(
            "duration_variance".to_string(),
            self.duration_variance_score,
        );
        report.insert(
            "bin_packing_efficiency".to_string(),
            self.bin_packing_efficiency,
        );
        report.insert("balance_score".to_string(), self.average_balance_score);
        report.insert(
            "underutilized_ratio".to_string(),
            self.underutilized_clusters as f32 / self.total_clusters as f32,
        );
        report.insert(
            "overutilized_count".to_string(),
            self.overutilized_clusters as f32,
        );
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PlanSettings;
    use chrono::Utc;

    fn create_test_video(index: usize, duration_minutes: u64) -> VideoWithMetadata {
        use crate::nlp::clustering::FeatureVector;
        use std::collections::HashMap;

        // Create a simple feature vector for testing
        let mut features = HashMap::new();
        features.insert(format!("topic_{}", index % 3), 1.0); // Group videos by topic
        features.insert("general".to_string(), 0.5);

        VideoWithMetadata {
            index,
            title: format!("Video {}", index),
            duration: Duration::from_secs(duration_minutes * 60),
            feature_vector: FeatureVector::new(features),
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
        let balancer =
            DurationBalancer::new(Duration::from_secs(3600), Duration::from_secs(4320), 0.2);

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
        let balancer =
            DurationBalancer::new(Duration::from_secs(3600), Duration::from_secs(4320), 0.2);

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
        assert!(metrics.content_coherence_score >= 0.0);
        assert!(metrics.duration_variance_score >= 0.0);
        assert!(metrics.bin_packing_efficiency >= 0.0);
    }

    #[test]
    fn test_duration_bin() {
        let capacity = Duration::from_secs(3600); // 1 hour
        let mut bin = DurationBin::new(capacity);

        assert!(bin.can_fit(&Duration::from_secs(1800))); // 30 minutes

        let video = create_test_video(0, 30);
        bin.add_video(video);

        assert_eq!(bin.utilization_percentage, 50.0);
        assert_eq!(bin.remaining_capacity(), Duration::from_secs(1800));
        assert!(bin.can_fit(&Duration::from_secs(1800)));
        assert!(!bin.can_fit(&Duration::from_secs(2400))); // 40 minutes won't fit
    }

    #[test]
    fn test_cluster_rebalancing() {
        let balancer = DurationBalancer::with_config(
            Duration::from_secs(3600), // 60 minutes
            Duration::from_secs(4320), // 72 minutes
            0.2,
            0.6, // similarity threshold
            0.5, // duration variance
        );

        // Create clusters that need rebalancing
        let clusters = vec![
            BalancedCluster {
                videos: vec![create_test_video(0, 10)], // Very short
                total_duration: Duration::from_secs(600),
                target_duration: Duration::from_secs(3600),
                utilization_percentage: 16.7,
                balance_score: 0.3,
            },
            BalancedCluster {
                videos: vec![create_test_video(1, 50), create_test_video(2, 50)], // Too long
                total_duration: Duration::from_secs(6000),
                target_duration: Duration::from_secs(3600),
                utilization_percentage: 166.7,
                balance_score: 0.4,
            },
        ];

        let result = balancer.rebalance_clusters(clusters);
        assert!(result.is_ok());

        let rebalanced = result.unwrap();
        // Should have more clusters after splitting the oversized one
        assert!(rebalanced.len() >= 2);
    }

    #[test]
    fn test_bin_packing_optimization() {
        let balancer = DurationBalancer::with_config(
            Duration::from_secs(3600),
            Duration::from_secs(4320),
            0.2,
            0.6,
            0.5,
        );

        let clusters = vec![BalancedCluster {
            videos: vec![
                create_test_video(0, 20),
                create_test_video(1, 25),
                create_test_video(2, 30),
            ],
            total_duration: Duration::from_secs(4500),
            target_duration: Duration::from_secs(3600),
            utilization_percentage: 125.0,
            balance_score: 0.6,
        }];

        let result = balancer.apply_bin_packing_optimization(clusters);
        assert!(result.is_ok());

        let optimized = result.unwrap();
        assert!(!optimized.is_empty());

        // Check that total duration is preserved
        let total_optimized_duration: Duration = optimized.iter().map(|c| c.total_duration).sum();
        assert_eq!(total_optimized_duration, Duration::from_secs(4500));
    }

    #[test]
    fn test_content_coherence_calculation() {
        let balancer =
            DurationBalancer::new(Duration::from_secs(3600), Duration::from_secs(4320), 0.2);

        let group1 = vec![create_test_video(0, 20), create_test_video(3, 25)]; // Same topic
        let group2 = vec![create_test_video(1, 30), create_test_video(4, 35)]; // Same topic

        let coherence = balancer.calculate_content_coherence(&group1, &group2);
        assert!(coherence >= 0.0 && coherence <= 1.0);
    }

    #[test]
    fn test_enhanced_quality_assessment() {
        let balancer =
            DurationBalancer::new(Duration::from_secs(3600), Duration::from_secs(4320), 0.2);

        let well_balanced_clusters = vec![
            BalancedCluster {
                videos: vec![create_test_video(0, 50)],
                total_duration: Duration::from_secs(3000),
                target_duration: Duration::from_secs(3600),
                utilization_percentage: 83.3,
                balance_score: 0.9,
            },
            BalancedCluster {
                videos: vec![create_test_video(1, 55)],
                total_duration: Duration::from_secs(3300),
                target_duration: Duration::from_secs(3600),
                utilization_percentage: 91.7,
                balance_score: 0.95,
            },
        ];

        let metrics = balancer.calculate_balance_metrics(&well_balanced_clusters);
        let assessment = metrics.quality_assessment();
        assert!(!assessment.is_empty());

        let report = metrics.detailed_report();
        assert!(report.contains_key("content_coherence"));
        assert!(report.contains_key("bin_packing_efficiency"));
        assert!(report.contains_key("duration_variance"));
    }
}
