//! Difficulty progression analysis for intelligent video clustering
//!
//! This module provides sophisticated difficulty scoring and progression analysis
//! to optimize learning sequences based on content complexity and user experience level.

use crate::types::{DifficultyLevel, Section};
use anyhow::Result;
use std::collections::HashMap;

/// Difficulty progression analyzer
#[derive(Debug, Clone)]
pub struct DifficultyAnalyzer {
    /// Keyword weights for difficulty scoring
    keyword_weights: HashMap<String, f32>,
    /// Duration thresholds for complexity assessment
    duration_thresholds: DurationThresholds,
    /// User experience level for adaptive scoring
    user_experience_level: DifficultyLevel,
}

/// Duration thresholds for difficulty assessment
#[derive(Debug, Clone)]
struct DurationThresholds {
    short_video_minutes: u32,
    medium_video_minutes: u32,
    long_video_minutes: u32,
}

/// Difficulty progression metrics for a sequence of videos
#[derive(Debug, Clone)]
pub struct DifficultyProgression {
    pub scores: Vec<f32>,
    pub progression_quality: f32,
    pub steep_jumps: Vec<usize>,
    pub recommended_reordering: Vec<usize>,
    pub cognitive_load_distribution: Vec<f32>,
}

/// Session difficulty analysis
#[derive(Debug, Clone)]
pub struct SessionDifficultyAnalysis {
    pub average_difficulty: f32,
    pub difficulty_variance: f32,
    pub cognitive_load_score: f32,
    pub recommended_pacing: PacingRecommendation,
    pub break_points: Vec<usize>,
}

/// Pacing recommendation for sessions
#[derive(Debug, Clone, PartialEq)]
pub enum PacingRecommendation {
    Accelerated,
    Standard,
    Decelerated,
    Mixed,
}

impl Default for SessionDifficultyAnalysis {
    fn default() -> Self {
        Self {
            average_difficulty: 0.5,
            difficulty_variance: 0.0,
            cognitive_load_score: 0.5,
            recommended_pacing: PacingRecommendation::Standard,
            break_points: Vec::new(),
        }
    }
}

impl Default for PacingRecommendation {
    fn default() -> Self {
        Self::Standard
    }
}

impl Default for DurationThresholds {
    fn default() -> Self {
        Self {
            short_video_minutes: 10,
            medium_video_minutes: 20,
            long_video_minutes: 40,
        }
    }
}

impl DifficultyAnalyzer {
    /// Create a new difficulty analyzer with default settings
    pub fn new(user_experience_level: DifficultyLevel) -> Self {
        Self {
            keyword_weights: Self::create_default_keyword_weights(),
            duration_thresholds: DurationThresholds::default(),
            user_experience_level,
        }
    }

    /// Create a new difficulty analyzer with custom keyword weights
    pub fn with_custom_weights(
        user_experience_level: DifficultyLevel,
        keyword_weights: HashMap<String, f32>,
    ) -> Self {
        Self {
            keyword_weights,
            duration_thresholds: DurationThresholds::default(),
            user_experience_level,
        }
    }

    /// Create default keyword weights for difficulty scoring
    fn create_default_keyword_weights() -> HashMap<String, f32> {
        let mut weights = HashMap::new();

        // Beginner indicators (negative weight = easier)
        weights.insert("introduction".to_string(), -0.3);
        weights.insert("basics".to_string(), -0.3);
        weights.insert("fundamentals".to_string(), -0.2);
        weights.insert("getting started".to_string(), -0.3);
        weights.insert("beginner".to_string(), -0.4);
        weights.insert("overview".to_string(), -0.2);
        weights.insert("what is".to_string(), -0.2);
        weights.insert("how to".to_string(), -0.1);
        weights.insert("tutorial".to_string(), -0.1);
        weights.insert("guide".to_string(), -0.1);

        // Intermediate indicators
        weights.insert("implementation".to_string(), 0.2);
        weights.insert("building".to_string(), 0.1);
        weights.insert("creating".to_string(), 0.1);
        weights.insert("developing".to_string(), 0.2);
        weights.insert("working with".to_string(), 0.1);
        weights.insert("using".to_string(), 0.0);
        weights.insert("applying".to_string(), 0.1);

        // Advanced indicators (positive weight = harder)
        weights.insert("advanced".to_string(), 0.4);
        weights.insert("expert".to_string(), 0.5);
        weights.insert("master".to_string(), 0.4);
        weights.insert("deep dive".to_string(), 0.3);
        weights.insert("optimization".to_string(), 0.3);
        weights.insert("architecture".to_string(), 0.3);
        weights.insert("complex".to_string(), 0.3);
        weights.insert("sophisticated".to_string(), 0.3);
        weights.insert("algorithm".to_string(), 0.4);
        weights.insert("theory".to_string(), 0.3);
        weights.insert("internals".to_string(), 0.4);
        weights.insert("performance".to_string(), 0.2);
        weights.insert("scaling".to_string(), 0.3);
        weights.insert("enterprise".to_string(), 0.2);

        // Technical complexity indicators
        weights.insert("debugging".to_string(), 0.2);
        weights.insert("troubleshooting".to_string(), 0.2);
        weights.insert("testing".to_string(), 0.1);
        weights.insert("deployment".to_string(), 0.2);
        weights.insert("production".to_string(), 0.2);
        weights.insert("security".to_string(), 0.3);
        weights.insert("concurrency".to_string(), 0.4);
        weights.insert("async".to_string(), 0.3);
        weights.insert("parallel".to_string(), 0.3);

        // Numerical/mathematical indicators
        weights.insert("part 1".to_string(), -0.2);
        weights.insert("part 2".to_string(), 0.0);
        weights.insert("part 3".to_string(), 0.1);
        weights.insert("part 4".to_string(), 0.2);
        weights.insert("part 5".to_string(), 0.3);

        weights
    }

    /// Calculate difficulty score for a video section
    pub fn calculate_difficulty_score(&self, section: &Section) -> f32 {
        let mut score = 0.5; // Base intermediate score
        let title_lower = section.title.to_lowercase();

        // Apply keyword-based scoring
        for (keyword, weight) in &self.keyword_weights {
            if title_lower.contains(keyword) {
                score += weight;
            }
        }

        // Apply duration-based complexity scoring
        let duration_minutes = section.duration.as_secs() / 60;
        let duration_factor = self.calculate_duration_complexity_factor(duration_minutes as u32);
        score += duration_factor;

        // Apply user experience level adjustment
        score = self.adjust_for_user_experience(score);

        // Apply numerical sequence detection
        score += self.detect_sequence_progression(&title_lower);

        // Clamp to valid range
        score.clamp(0.0, 1.0)
    }

    /// Calculate duration-based complexity factor using configured thresholds
    fn calculate_duration_complexity_factor(&self, duration_minutes: u32) -> f32 {
        let short_threshold = self.duration_thresholds.short_video_minutes;
        let medium_threshold = self.duration_thresholds.medium_video_minutes;
        let long_threshold = self.duration_thresholds.long_video_minutes;

        match duration_minutes {
            0..=5 => -0.1,                        // Very short videos are often simple
            d if d <= short_threshold => 0.0,     // Short videos are neutral
            d if d <= medium_threshold => 0.1,    // Medium videos slightly more complex
            d if d <= long_threshold => 0.2,      // Long videos more complex
            d if d <= long_threshold + 20 => 0.3, // Very long videos quite complex
            _ => 0.4,                             // Extremely long videos very complex
        }
    }

    /// Adjust difficulty score based on user experience level
    fn adjust_for_user_experience(&self, base_score: f32) -> f32 {
        match self.user_experience_level {
            DifficultyLevel::Beginner => {
                // Beginners perceive everything as more difficult
                base_score + 0.1
            }
            DifficultyLevel::Intermediate => base_score, // No adjustment
            DifficultyLevel::Advanced => {
                // Advanced users find things easier
                base_score - 0.1
            }
            DifficultyLevel::Expert => {
                // Experts find most things easy
                base_score - 0.2
            }
        }
    }

    /// Detect numerical sequence progression in titles
    fn detect_sequence_progression(&self, title_lower: &str) -> f32 {
        // Look for patterns like "part 1", "chapter 2", "lesson 3", etc.
        let patterns = [
            ("part ", 0.05),
            ("chapter ", 0.05),
            ("lesson ", 0.05),
            ("section ", 0.05),
            ("episode ", 0.05),
        ];

        for (pattern, base_increment) in &patterns {
            if let Some(pos) = title_lower.find(pattern) {
                let after_pattern = &title_lower[pos + pattern.len()..];
                if let Some(number_str) = after_pattern.split_whitespace().next() {
                    if let Ok(number) = number_str.parse::<u32>() {
                        // Higher numbers in sequence indicate progression
                        return base_increment * (number as f32 - 1.0).min(5.0);
                    }
                }
            }
        }

        0.0
    }

    /// Analyze difficulty progression for a sequence of sections
    pub fn analyze_progression(&self, sections: &[Section]) -> Result<DifficultyProgression> {
        if sections.is_empty() {
            return Ok(DifficultyProgression {
                scores: Vec::new(),
                progression_quality: 0.0,
                steep_jumps: Vec::new(),
                recommended_reordering: Vec::new(),
                cognitive_load_distribution: Vec::new(),
            });
        }

        let scores: Vec<f32> = sections
            .iter()
            .map(|section| self.calculate_difficulty_score(section))
            .collect();

        let progression_quality = self.calculate_progression_quality(&scores);
        let steep_jumps = self.detect_steep_jumps(&scores);
        let recommended_reordering = self.generate_optimal_ordering(sections, &scores)?;
        let cognitive_load_distribution = self.calculate_cognitive_load_distribution(&scores);

        Ok(DifficultyProgression {
            scores,
            progression_quality,
            steep_jumps,
            recommended_reordering,
            cognitive_load_distribution,
        })
    }

    /// Calculate quality of difficulty progression (0.0 = poor, 1.0 = excellent)
    fn calculate_progression_quality(&self, scores: &[f32]) -> f32 {
        if scores.len() < 2 {
            return 1.0; // Single item is perfectly progressive
        }

        let mut quality_score = 0.0;
        let mut total_transitions = 0;

        for window in scores.windows(2) {
            let diff = window[1] - window[0];
            total_transitions += 1;

            // Ideal progression is gradual increase
            match diff {
                d if (-0.05..=0.15).contains(&d) => quality_score += 1.0, // Good progression
                d if (-0.1..=0.25).contains(&d) => quality_score += 0.7,  // Acceptable
                d if (-0.2..=0.35).contains(&d) => quality_score += 0.4,  // Suboptimal
                _ => quality_score += 0.0,                                // Poor progression
            }
        }

        if total_transitions > 0 {
            quality_score / total_transitions as f32
        } else {
            1.0
        }
    }

    /// Detect steep difficulty jumps in the sequence
    fn detect_steep_jumps(&self, scores: &[f32]) -> Vec<usize> {
        let mut steep_jumps = Vec::new();
        let jump_threshold = 0.3; // Difficulty increase > 0.3 is considered steep

        for (i, window) in scores.windows(2).enumerate() {
            let diff = window[1] - window[0];
            if diff > jump_threshold {
                steep_jumps.push(i + 1); // Index of the video with steep jump
            }
        }

        steep_jumps
    }

    /// Generate optimal ordering for sections based on difficulty progression
    fn generate_optimal_ordering(
        &self,
        sections: &[Section],
        scores: &[f32],
    ) -> Result<Vec<usize>> {
        if sections.is_empty() {
            return Ok(Vec::new());
        }

        // Create pairs of (index, score) and sort by score
        let mut indexed_scores: Vec<(usize, f32)> = scores
            .iter()
            .enumerate()
            .map(|(i, &score)| (i, score))
            .collect();

        // Sort by difficulty score (ascending)
        indexed_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply intelligent reordering considering content relationships
        let optimal_order = self.apply_content_aware_reordering(sections, indexed_scores)?;

        Ok(optimal_order)
    }

    /// Apply content-aware reordering that considers both difficulty and content relationships
    fn apply_content_aware_reordering(
        &self,
        sections: &[Section],
        mut indexed_scores: Vec<(usize, f32)>,
    ) -> Result<Vec<usize>> {
        // Group sections by difficulty tiers
        let mut beginner_tier = Vec::new();
        let mut intermediate_tier = Vec::new();
        let mut advanced_tier = Vec::new();

        for (index, score) in indexed_scores.drain(..) {
            match score {
                s if s < 0.33 => beginner_tier.push((index, score)),
                s if s < 0.67 => intermediate_tier.push((index, score)),
                _ => advanced_tier.push((index, score)),
            }
        }

        // Within each tier, try to maintain content relationships
        let mut optimal_order = Vec::new();

        // Process beginner tier
        optimal_order.extend(self.order_within_tier(sections, beginner_tier)?);

        // Process intermediate tier
        optimal_order.extend(self.order_within_tier(sections, intermediate_tier)?);

        // Process advanced tier
        optimal_order.extend(self.order_within_tier(sections, advanced_tier)?);

        Ok(optimal_order)
    }

    /// Order sections within a difficulty tier considering content relationships
    fn order_within_tier(
        &self,
        _sections: &[Section],
        tier: Vec<(usize, f32)>,
    ) -> Result<Vec<usize>> {
        if tier.is_empty() {
            return Ok(Vec::new());
        }

        // For now, maintain original order within tiers to preserve content flow
        // In a more sophisticated implementation, this could analyze title similarity
        let mut tier_indices: Vec<usize> = tier.into_iter().map(|(index, _)| index).collect();
        tier_indices.sort(); // Maintain original order

        Ok(tier_indices)
    }

    /// Calculate cognitive load distribution for a sequence
    fn calculate_cognitive_load_distribution(&self, scores: &[f32]) -> Vec<f32> {
        scores
            .iter()
            .map(|&score| {
                // Convert difficulty score to cognitive load (0.0 = low, 1.0 = high)
                match score {
                    s if s < 0.2 => 0.1, // Very low cognitive load
                    s if s < 0.4 => 0.3, // Low cognitive load
                    s if s < 0.6 => 0.5, // Medium cognitive load
                    s if s < 0.8 => 0.7, // High cognitive load
                    _ => 0.9,            // Very high cognitive load
                }
            })
            .collect()
    }

    /// Analyze difficulty for a session
    pub fn analyze_session_difficulty(
        &self,
        sections: &[Section],
    ) -> Result<SessionDifficultyAnalysis> {
        if sections.is_empty() {
            return Ok(SessionDifficultyAnalysis {
                average_difficulty: 0.0,
                difficulty_variance: 0.0,
                cognitive_load_score: 0.0,
                recommended_pacing: PacingRecommendation::Standard,
                break_points: Vec::new(),
            });
        }

        let scores: Vec<f32> = sections
            .iter()
            .map(|section| self.calculate_difficulty_score(section))
            .collect();

        let average_difficulty = scores.iter().sum::<f32>() / scores.len() as f32;
        let difficulty_variance = self.calculate_variance(&scores, average_difficulty);
        let cognitive_load_score = self.calculate_session_cognitive_load(&scores);
        let recommended_pacing =
            self.determine_pacing_recommendation(average_difficulty, difficulty_variance);
        let break_points = self.identify_break_points(&scores);

        Ok(SessionDifficultyAnalysis {
            average_difficulty,
            difficulty_variance,
            cognitive_load_score,
            recommended_pacing,
            break_points,
        })
    }

    /// Calculate variance of difficulty scores
    fn calculate_variance(&self, scores: &[f32], mean: f32) -> f32 {
        if scores.len() <= 1 {
            return 0.0;
        }

        let sum_squared_diffs: f32 = scores.iter().map(|&score| (score - mean).powi(2)).sum();
        sum_squared_diffs / scores.len() as f32
    }

    /// Calculate cognitive load score for a session
    fn calculate_session_cognitive_load(&self, scores: &[f32]) -> f32 {
        if scores.is_empty() {
            return 0.0;
        }

        // Cognitive load is influenced by both average difficulty and transitions
        let average_difficulty = scores.iter().sum::<f32>() / scores.len() as f32;
        let transition_complexity = self.calculate_transition_complexity(scores);

        // Combine average difficulty with transition complexity
        (average_difficulty * 0.7) + (transition_complexity * 0.3)
    }

    /// Calculate complexity introduced by difficulty transitions
    fn calculate_transition_complexity(&self, scores: &[f32]) -> f32 {
        if scores.len() < 2 {
            return 0.0;
        }

        let mut total_complexity = 0.0;
        for window in scores.windows(2) {
            let diff = (window[1] - window[0]).abs();
            total_complexity += diff;
        }

        total_complexity / (scores.len() - 1) as f32
    }

    /// Determine pacing recommendation based on difficulty analysis
    fn determine_pacing_recommendation(
        &self,
        average_difficulty: f32,
        variance: f32,
    ) -> PacingRecommendation {
        match (average_difficulty, variance) {
            // High difficulty, low variance = consistently hard
            (avg, var) if avg > 0.7 && var < 0.1 => PacingRecommendation::Decelerated,

            // Low difficulty, low variance = consistently easy
            (avg, var) if avg < 0.3 && var < 0.1 => PacingRecommendation::Accelerated,

            // High variance = mixed difficulty
            (_, var) if var > 0.2 => PacingRecommendation::Mixed,

            // Everything else = standard pacing
            _ => PacingRecommendation::Standard,
        }
    }

    /// Identify optimal break points in a session
    fn identify_break_points(&self, scores: &[f32]) -> Vec<usize> {
        let mut break_points = Vec::new();

        if scores.len() < 3 {
            return break_points;
        }

        // Look for transitions from high to low difficulty (natural break points)
        for (i, window) in scores.windows(2).enumerate() {
            let current_difficulty = window[0];
            let next_difficulty = window[1];

            // Break point after high difficulty content
            if current_difficulty > 0.6 && next_difficulty < current_difficulty - 0.2 {
                break_points.push(i + 1);
            }
        }

        // Add break points for very long sequences
        if scores.len() > 5 {
            let mid_point = scores.len() / 2;
            if !break_points.contains(&mid_point) {
                break_points.push(mid_point);
            }
        }

        break_points.sort();
        break_points
    }

    /// Validate difficulty progression and suggest improvements
    pub fn validate_and_improve_progression(
        &self,
        sections: &[Section],
    ) -> Result<ProgressionValidation> {
        let progression = self.analyze_progression(sections)?;

        let issues = self.identify_progression_issues(&progression);
        let suggestions = self.generate_improvement_suggestions(&progression, sections);

        Ok(ProgressionValidation {
            progression,
            issues,
            suggestions,
        })
    }

    /// Identify issues in difficulty progression
    fn identify_progression_issues(
        &self,
        progression: &DifficultyProgression,
    ) -> Vec<ProgressionIssue> {
        let mut issues = Vec::new();

        // Check for steep jumps
        if !progression.steep_jumps.is_empty() {
            issues.push(ProgressionIssue::SteepDifficultyJumps(
                progression.steep_jumps.clone(),
            ));
        }

        // Check for poor progression quality
        if progression.progression_quality < 0.5 {
            issues.push(ProgressionIssue::PoorProgression(
                progression.progression_quality,
            ));
        }

        // Check for high cognitive load concentration
        let high_load_count = progression
            .cognitive_load_distribution
            .iter()
            .filter(|&&load| load > 0.8)
            .count();

        if high_load_count > progression.scores.len() / 3 {
            issues.push(ProgressionIssue::HighCognitiveLoadConcentration(
                high_load_count,
            ));
        }

        issues
    }

    /// Generate suggestions for improving difficulty progression
    fn generate_improvement_suggestions(
        &self,
        progression: &DifficultyProgression,
        sections: &[Section],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if !progression.steep_jumps.is_empty() {
            suggestions.push(
                "Consider adding intermediate content before steep difficulty increases"
                    .to_string(),
            );
        }

        if progression.progression_quality < 0.3 {
            suggestions
                .push("Reorder content to create smoother difficulty progression".to_string());
        }

        if progression.recommended_reordering != (0..sections.len()).collect::<Vec<_>>() {
            suggestions.push("Consider reordering videos based on difficulty analysis".to_string());
        }

        let avg_cognitive_load = progression.cognitive_load_distribution.iter().sum::<f32>()
            / progression.cognitive_load_distribution.len() as f32;
        if avg_cognitive_load > 0.7 {
            suggestions.push(
                "Consider breaking high-difficulty content into smaller sessions".to_string(),
            );
        }

        if suggestions.is_empty() {
            suggestions.push("Difficulty progression looks good!".to_string());
        }

        suggestions
    }
}

/// Progression validation result
#[derive(Debug, Clone)]
pub struct ProgressionValidation {
    pub progression: DifficultyProgression,
    pub issues: Vec<ProgressionIssue>,
    pub suggestions: Vec<String>,
}

/// Issues identified in difficulty progression
#[derive(Debug, Clone)]
pub enum ProgressionIssue {
    SteepDifficultyJumps(Vec<usize>),
    PoorProgression(f32),
    HighCognitiveLoadConcentration(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_section(title: &str, duration_minutes: u32) -> Section {
        Section {
            title: title.to_string(),
            video_index: 0,
            duration: Duration::from_secs(duration_minutes as u64 * 60),
        }
    }

    #[test]
    fn test_difficulty_scoring() {
        let analyzer = DifficultyAnalyzer::new(DifficultyLevel::Intermediate);

        let beginner_section = create_test_section("Introduction to Programming", 10);
        let advanced_section = create_test_section("Advanced Algorithm Optimization", 45);

        let beginner_score = analyzer.calculate_difficulty_score(&beginner_section);
        let advanced_score = analyzer.calculate_difficulty_score(&advanced_section);

        assert!(beginner_score < advanced_score);
        assert!(beginner_score < 0.5);
        assert!(advanced_score > 0.5);
    }

    #[test]
    fn test_progression_analysis() {
        let analyzer = DifficultyAnalyzer::new(DifficultyLevel::Intermediate);

        let sections = vec![
            create_test_section("Basics of Programming", 10),
            create_test_section("Intermediate Concepts", 20),
            create_test_section("Advanced Techniques", 30),
        ];

        let progression = analyzer.analyze_progression(&sections).unwrap();

        assert_eq!(progression.scores.len(), 3);
        assert!(progression.scores[0] < progression.scores[1]);
        assert!(progression.scores[1] < progression.scores[2]);
        assert!(progression.progression_quality > 0.5);
    }

    #[test]
    fn test_steep_jump_detection() {
        let analyzer = DifficultyAnalyzer::new(DifficultyLevel::Intermediate);

        let sections = vec![
            create_test_section("Introduction", 10),
            create_test_section("Expert Level Algorithms", 60), // Steep jump
            create_test_section("More Advanced Topics", 30),
        ];

        let progression = analyzer.analyze_progression(&sections).unwrap();

        assert!(!progression.steep_jumps.is_empty());
        assert!(progression.steep_jumps.contains(&1));
    }

    #[test]
    fn test_session_difficulty_analysis() {
        let analyzer = DifficultyAnalyzer::new(DifficultyLevel::Intermediate);

        let sections = vec![
            create_test_section("Advanced Topic 1", 30),
            create_test_section("Advanced Topic 2", 30),
            create_test_section("Expert Level Content", 45),
        ];

        let analysis = analyzer.analyze_session_difficulty(&sections).unwrap();

        assert!(analysis.average_difficulty > 0.5);
        assert_eq!(
            analysis.recommended_pacing,
            PacingRecommendation::Decelerated
        );
    }

    #[test]
    fn test_user_experience_adjustment() {
        let beginner_analyzer = DifficultyAnalyzer::new(DifficultyLevel::Beginner);
        let expert_analyzer = DifficultyAnalyzer::new(DifficultyLevel::Expert);

        let section = create_test_section("Intermediate Programming", 20);

        let beginner_score = beginner_analyzer.calculate_difficulty_score(&section);
        let expert_score = expert_analyzer.calculate_difficulty_score(&section);

        assert!(beginner_score > expert_score);
    }
}
