//! Multi-factor session optimizer for intelligent study planning
//!
//! This module provides sophisticated optimization algorithms that consider multiple
//! factors including content similarity, difficulty progression, cognitive load,
//! and user preferences to create optimal learning sequences.

use crate::nlp::clustering::{DifficultyAnalyzer, SessionDifficultyAnalysis};
use crate::types::{Course, DifficultyLevel, Plan, PlanItem, Section};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

/// Multi-factor session optimizer with configurable weights
#[derive(Debug, Clone)]
pub struct MultiFactorOptimizer {
    /// Weight for content similarity factor (0.0 - 1.0)
    pub content_weight: f32,
    /// Weight for duration balancing factor (0.0 - 1.0)
    pub duration_weight: f32,
    /// Weight for difficulty progression factor (0.0 - 1.0)
    pub difficulty_weight: f32,
    /// Weight for user preference factor (0.0 - 1.0)
    pub user_preference_weight: f32,
    /// Difficulty analyzer for progression analysis
    difficulty_analyzer: DifficultyAnalyzer,
    /// User experience level for adaptive optimization
    user_experience_level: DifficultyLevel,
    /// Maximum cognitive load per session
    max_cognitive_load: f32,
}

/// Optimization result with detailed metrics
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub optimized_items: Vec<PlanItem>,
    pub optimization_score: f32,
    pub factor_scores: FactorScores,
    pub cognitive_load_distribution: Vec<f32>,
    pub improvements: Vec<OptimizationImprovement>,
    pub warnings: Vec<String>,
}

/// Individual factor scores for transparency
#[derive(Debug, Clone)]
pub struct FactorScores {
    pub content_similarity_score: f32,
    pub duration_balance_score: f32,
    pub difficulty_progression_score: f32,
    pub user_preference_score: f32,
    pub overall_score: f32,
}

/// Optimization improvement description
#[derive(Debug, Clone)]
pub struct OptimizationImprovement {
    pub session_index: usize,
    pub improvement_type: ImprovementType,
    pub description: String,
    pub impact_score: f32,
}

/// Types of optimization improvements
#[derive(Debug, Clone, PartialEq)]
pub enum ImprovementType {
    ContentGrouping,
    DurationBalancing,
    DifficultySmoothing,
    CognitiveLoadReduction,
    UserPreferenceAlignment,
}

/// Cognitive load balancing configuration
#[derive(Debug, Clone)]
pub struct CognitiveLoadConfig {
    pub max_load_per_session: f32,
    pub ideal_load_distribution: LoadDistribution,
    pub break_threshold: f32,
    pub recovery_sessions_enabled: bool,
}

/// Load distribution patterns
#[derive(Debug, Clone, PartialEq)]
pub enum LoadDistribution {
    Uniform,     // Even distribution across sessions
    Progressive, // Gradually increasing load
    Alternating, // High-low alternating pattern
    Adaptive,    // Based on user performance
}

/// User preference learning data
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub preferred_session_length: std::time::Duration,
    pub difficulty_preference: DifficultyPreference,
    pub content_grouping_preference: ContentGroupingPreference,
    pub pacing_preference: PacingPreference,
    pub learning_style: LearningStyle,
}

/// Difficulty preference settings
#[derive(Debug, Clone, PartialEq)]
pub enum DifficultyPreference {
    GradualProgression,
    SteepLearningCurve,
    MixedDifficulty,
    ConsistentLevel,
}

/// Content grouping preferences
#[derive(Debug, Clone, PartialEq)]
pub enum ContentGroupingPreference {
    TopicBased,
    DurationBased,
    DifficultyBased,
    Mixed,
}

/// Pacing preferences
#[derive(Debug, Clone, PartialEq)]
pub enum PacingPreference {
    Intensive,
    Relaxed,
    Adaptive,
    Consistent,
}

/// Learning style preferences
#[derive(Debug, Clone, PartialEq)]
pub enum LearningStyle {
    Sequential,   // Linear progression through content
    Exploratory,  // Jump between related topics
    Repetitive,   // Multiple passes through content
    ProjectBased, // Focus on practical applications
}

impl Default for MultiFactorOptimizer {
    fn default() -> Self {
        Self::new(DifficultyLevel::Intermediate)
    }
}

impl Default for CognitiveLoadConfig {
    fn default() -> Self {
        Self {
            max_load_per_session: 0.8,
            ideal_load_distribution: LoadDistribution::Progressive,
            break_threshold: 0.7,
            recovery_sessions_enabled: true,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_session_length: std::time::Duration::from_secs(3600), // 1 hour
            difficulty_preference: DifficultyPreference::GradualProgression,
            content_grouping_preference: ContentGroupingPreference::TopicBased,
            pacing_preference: PacingPreference::Adaptive,
            learning_style: LearningStyle::Sequential,
        }
    }
}

impl MultiFactorOptimizer {
    /// Create a new multi-factor optimizer with default weights
    pub fn new(user_experience_level: DifficultyLevel) -> Self {
        Self {
            content_weight: 0.3,
            duration_weight: 0.2,
            difficulty_weight: 0.3,
            user_preference_weight: 0.2,
            difficulty_analyzer: DifficultyAnalyzer::new(user_experience_level),
            user_experience_level,
            max_cognitive_load: 0.8,
        }
    }

    /// Create optimizer with custom weights
    pub fn with_weights(
        user_experience_level: DifficultyLevel,
        content_weight: f32,
        duration_weight: f32,
        difficulty_weight: f32,
        user_preference_weight: f32,
    ) -> Result<Self> {
        let total_weight =
            content_weight + duration_weight + difficulty_weight + user_preference_weight;
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(anyhow::anyhow!(
                "Weights must sum to 1.0, got {}",
                total_weight
            ));
        }

        Ok(Self {
            content_weight,
            duration_weight,
            difficulty_weight,
            user_preference_weight,
            difficulty_analyzer: DifficultyAnalyzer::new(user_experience_level),
            user_experience_level,
            max_cognitive_load: 0.8,
        })
    }

    /// Optimize session sequence considering multiple factors
    pub fn optimize_session_sequence(
        &self,
        course: &Course,
        plan: &Plan,
        user_preferences: &UserPreferences,
    ) -> Result<OptimizationResult> {
        let structure = course
            .structure
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Course structure not available"))?;

        // Analyze current plan sessions
        let session_analyses = self.analyze_current_sessions(structure, &plan.items)?;

        // Calculate current factor scores for baseline
        let current_factor_scores = self.calculate_factor_scores(
            structure,
            &plan.items,
            &session_analyses,
            user_preferences,
        )?;

        // Generate optimized session sequence
        let optimized_items = self.generate_optimized_sequence(
            structure,
            &plan.items,
            &session_analyses,
            user_preferences,
        )?;

        // Analyze optimized plan
        let optimized_session_analyses =
            self.analyze_current_sessions(structure, &optimized_items)?;

        // Calculate optimized factor scores
        let optimized_factor_scores = self.calculate_factor_scores(
            structure,
            &optimized_items,
            &optimized_session_analyses,
            user_preferences,
        )?;

        // Apply cognitive load balancing
        let cognitive_load_config = CognitiveLoadConfig::default();
        let balanced_items = self.apply_cognitive_load_constraints(
            &optimized_items,
            &optimized_session_analyses,
            &cognitive_load_config,
            user_preferences,
        )?;

        // Calculate cognitive load distribution
        let cognitive_load_distribution =
            self.calculate_cognitive_load_distribution(&optimized_session_analyses);

        // Identify improvements
        let improvements = self.identify_improvements(
            &plan.items,
            &balanced_items,
            &current_factor_scores,
            &optimized_factor_scores,
        );

        // Generate optimization warnings
        let warnings =
            self.generate_optimization_warnings(&optimized_session_analyses, user_preferences);

        Ok(OptimizationResult {
            optimized_items: balanced_items,
            optimization_score: optimized_factor_scores.overall_score,
            factor_scores: optimized_factor_scores,
            cognitive_load_distribution,
            improvements,
            warnings,
        })
    }

    /// Balance cognitive load across sessions
    pub fn balance_cognitive_load(
        &self,
        course: &Course,
        plan: &Plan,
        config: &CognitiveLoadConfig,
    ) -> Result<OptimizationResult> {
        let user_preferences = UserPreferences::default();
        let mut result = self.optimize_session_sequence(course, plan, &user_preferences)?;

        // Apply additional cognitive load balancing
        result.optimized_items = self.apply_adaptive_load_distribution(
            &result.optimized_items,
            &result.cognitive_load_distribution,
            config,
        )?;

        Ok(result)
    }

    /// Analyze current session difficulty and characteristics
    fn analyze_current_sessions(
        &self,
        structure: &crate::types::CourseStructure,
        items: &[PlanItem],
    ) -> Result<Vec<SessionDifficultyAnalysis>> {
        let mut analyses = Vec::new();

        for item in items {
            let sections = self.get_sections_for_plan_item(structure, item)?;
            let analysis = self
                .difficulty_analyzer
                .analyze_session_difficulty(&sections)?;
            analyses.push(analysis);
        }

        Ok(analyses)
    }

    /// Get sections for a plan item
    fn get_sections_for_plan_item(
        &self,
        structure: &crate::types::CourseStructure,
        item: &PlanItem,
    ) -> Result<Vec<Section>> {
        let mut sections = Vec::new();

        for &video_index in &item.video_indices {
            for module in &structure.modules {
                for section in &module.sections {
                    if section.video_index == video_index {
                        sections.push(section.clone());
                        break;
                    }
                }
            }
        }

        Ok(sections)
    }

    /// Calculate factor scores for a plan
    fn calculate_factor_scores(
        &self,
        structure: &crate::types::CourseStructure,
        items: &[PlanItem],
        session_analyses: &[SessionDifficultyAnalysis],
        user_preferences: &UserPreferences,
    ) -> Result<FactorScores> {
        let content_similarity_score = self.calculate_content_similarity_score(structure, items)?;
        let duration_balance_score = self.calculate_duration_balance_score(items)?;
        let difficulty_progression_score =
            self.calculate_difficulty_progression_score(session_analyses)?;
        let user_preference_score =
            self.calculate_user_preference_score(items, session_analyses, user_preferences)?;

        let overall_score = self.content_weight * content_similarity_score
            + self.duration_weight * duration_balance_score
            + self.difficulty_weight * difficulty_progression_score
            + self.user_preference_weight * user_preference_score;

        Ok(FactorScores {
            content_similarity_score,
            duration_balance_score,
            difficulty_progression_score,
            user_preference_score,
            overall_score,
        })
    }

    /// Calculate content similarity score
    fn calculate_content_similarity_score(
        &self,
        structure: &crate::types::CourseStructure,
        items: &[PlanItem],
    ) -> Result<f32> {
        if items.len() < 2 {
            return Ok(1.0); // Perfect score for single session
        }

        let mut total_similarity = 0.0;
        let mut comparisons = 0;

        for (i, item) in items.iter().enumerate() {
            let sections_a = self.get_sections_for_plan_item(structure, item)?;

            // Compare with next session if available
            if i + 1 < items.len() {
                let sections_b = self.get_sections_for_plan_item(structure, &items[i + 1])?;
                let similarity =
                    self.calculate_session_content_similarity(&sections_a, &sections_b);
                total_similarity += similarity;
                comparisons += 1;
            }
        }

        if comparisons == 0 {
            Ok(1.0)
        } else {
            Ok(total_similarity / comparisons as f32)
        }
    }

    /// Calculate similarity between two sessions based on content
    fn calculate_session_content_similarity(
        &self,
        sections_a: &[Section],
        sections_b: &[Section],
    ) -> f32 {
        if sections_a.is_empty() || sections_b.is_empty() {
            return 0.0;
        }

        let mut similarity_sum = 0.0;
        let mut comparisons = 0;

        for section_a in sections_a {
            for section_b in sections_b {
                // Simple title-based similarity (can be enhanced with TF-IDF)
                let similarity =
                    self.calculate_title_similarity(&section_a.title, &section_b.title);
                similarity_sum += similarity;
                comparisons += 1;
            }
        }

        if comparisons == 0 {
            0.0
        } else {
            similarity_sum / comparisons as f32
        }
    }

    /// Calculate title similarity using simple word overlap
    fn calculate_title_similarity(&self, title_a: &str, title_b: &str) -> f32 {
        let title_a_lower = title_a.to_lowercase();
        let title_b_lower = title_b.to_lowercase();
        let words_a: std::collections::HashSet<&str> = title_a_lower.split_whitespace().collect();
        let words_b: std::collections::HashSet<&str> = title_b_lower.split_whitespace().collect();

        if words_a.is_empty() && words_b.is_empty() {
            return 1.0;
        }

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Calculate duration balance score
    fn calculate_duration_balance_score(&self, items: &[PlanItem]) -> Result<f32> {
        if items.len() < 2 {
            return Ok(1.0);
        }

        let durations: Vec<f32> = items
            .iter()
            .map(|item| item.total_duration.as_secs() as f32)
            .collect();

        let mean = durations.iter().sum::<f32>() / durations.len() as f32;
        let variance = durations
            .iter()
            .map(|&duration| (duration - mean).powi(2))
            .sum::<f32>()
            / durations.len() as f32;

        let coefficient_of_variation = if mean > 0.0 {
            variance.sqrt() / mean
        } else {
            0.0
        };

        // Lower coefficient of variation = better balance = higher score
        Ok((1.0f32 - coefficient_of_variation.min(1.0)).max(0.0))
    }

    /// Calculate difficulty progression score
    fn calculate_difficulty_progression_score(
        &self,
        session_analyses: &[SessionDifficultyAnalysis],
    ) -> Result<f32> {
        if session_analyses.len() < 2 {
            return Ok(1.0);
        }

        let mut progression_score = 0.0;
        let mut valid_progressions = 0;

        for window in session_analyses.windows(2) {
            let current_difficulty = window[0].average_difficulty;
            let next_difficulty = window[1].average_difficulty;

            // Ideal progression: gradual increase or maintain
            let progression = if next_difficulty >= current_difficulty {
                let increase = next_difficulty - current_difficulty;
                if increase <= 0.2 {
                    // Gradual increase is good
                    1.0 - increase * 2.0 // Penalize steep jumps
                } else {
                    0.5 // Steep jump penalty
                }
            } else {
                0.8 // Slight penalty for regression but not terrible
            };

            progression_score += progression;
            valid_progressions += 1;
        }

        if valid_progressions == 0 {
            Ok(1.0)
        } else {
            Ok(progression_score / valid_progressions as f32)
        }
    }

    /// Calculate user preference alignment score
    fn calculate_user_preference_score(
        &self,
        items: &[PlanItem],
        session_analyses: &[SessionDifficultyAnalysis],
        user_preferences: &UserPreferences,
    ) -> Result<f32> {
        let mut score = 0.0;

        // Adjust base score based on user experience level
        let experience_adjustment = match self.user_experience_level {
            DifficultyLevel::Beginner => 0.1, // Beginners need more structured approach
            DifficultyLevel::Intermediate => 0.0, // No adjustment
            DifficultyLevel::Advanced => -0.1, // Advanced users are more flexible
            DifficultyLevel::Expert => -0.2,  // Experts are very flexible with optimization
        };

        // Factor in difficulty preference
        score += match user_preferences.difficulty_preference {
            DifficultyPreference::GradualProgression => {
                if session_analyses
                    .iter()
                    .map(|a| a.average_difficulty)
                    .collect::<Vec<_>>()
                    .windows(2)
                    .all(|w| w[1] >= w[0] && (w[1] - w[0]) <= 0.2)
                {
                    1.0
                } else {
                    0.6
                }
            }
            DifficultyPreference::SteepLearningCurve => {
                let steep_jumps = self.count_steep_difficulty_jumps(session_analyses);
                if steep_jumps > 0 { 1.0 } else { 0.7 }
            }
            DifficultyPreference::MixedDifficulty => {
                let variance = session_analyses.len() as f32
                    / session_analyses
                        .iter()
                        .map(|a| {
                            (a.average_difficulty
                                - session_analyses
                                    .iter()
                                    .map(|a| a.average_difficulty)
                                    .sum::<f32>()
                                    / session_analyses.len() as f32)
                                .powi(2)
                        })
                        .sum::<f32>();
                if variance > 0.1 { 1.0 } else { 0.6 }
            }
            DifficultyPreference::ConsistentLevel => {
                let variance = session_analyses
                    .iter()
                    .map(|a| {
                        (a.average_difficulty
                            - session_analyses
                                .iter()
                                .map(|a| a.average_difficulty)
                                .sum::<f32>()
                                / session_analyses.len() as f32)
                            .powi(2)
                    })
                    .sum::<f32>()
                    / session_analyses.len() as f32;
                if variance < 0.1 { 1.0 } else { 0.5 }
            }
        };

        // Factor in pacing preference
        score += match user_preferences.pacing_preference {
            PacingPreference::Intensive => {
                let avg_duration = items
                    .iter()
                    .map(|i| i.total_duration.as_secs())
                    .sum::<u64>() as f32
                    / items.len() as f32;
                if avg_duration > 3600.0 {
                    // > 1 hour
                    1.0
                } else {
                    0.7
                }
            }
            PacingPreference::Relaxed => {
                let avg_duration = items
                    .iter()
                    .map(|i| i.total_duration.as_secs())
                    .sum::<u64>() as f32
                    / items.len() as f32;
                if avg_duration < 2400.0 {
                    // < 40 minutes
                    1.0
                } else {
                    0.7
                }
            }
            PacingPreference::Adaptive => 0.8, // Neutral score for adaptive
            PacingPreference::Consistent => self.calculate_duration_balance_score(items)?,
        };

        // Apply experience level adjustment
        score = (score / 2.0 + experience_adjustment).clamp(0.0, 1.0);

        Ok(score) // Average of the two factors with experience adjustment
    }

    /// Count steep difficulty jumps in session analyses
    fn count_steep_difficulty_jumps(
        &self,
        session_analyses: &[SessionDifficultyAnalysis],
    ) -> usize {
        if session_analyses.len() < 2 {
            return 0;
        }

        let _avg_difficulty = session_analyses
            .iter()
            .map(|a| a.average_difficulty)
            .sum::<f32>()
            / session_analyses.len() as f32;

        session_analyses
            .windows(2)
            .filter(|window| (window[1].average_difficulty - window[0].average_difficulty) > 0.3)
            .count()
    }

    /// Generate optimized session sequence
    fn generate_optimized_sequence(
        &self,
        structure: &crate::types::CourseStructure,
        items: &[PlanItem],
        session_analyses: &[SessionDifficultyAnalysis],
        user_preferences: &UserPreferences,
    ) -> Result<Vec<PlanItem>> {
        let mut optimized_items = items.to_vec();

        // Create optimization candidates with scores
        let mut optimization_candidates: HashMap<usize, f32> = HashMap::new();
        for (i, item) in optimized_items.iter().enumerate() {
            let score = self.calculate_session_optimization_score(
                structure,
                item,
                session_analyses
                    .get(i)
                    .unwrap_or(&SessionDifficultyAnalysis::default()),
                user_preferences,
            )?;
            optimization_candidates.insert(i, score);
        }

        // Sort sessions by optimization potential (lowest scores first - need most improvement)
        let mut sorted_sessions: Vec<(usize, f32)> = optimization_candidates.into_iter().collect();
        sorted_sessions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply optimizations based on user preferences
        let balanced_order = self.apply_cognitive_load_balancing(
            &optimized_items,
            session_analyses,
            user_preferences,
        )?;

        // Create optimized plan items with updated dates
        let mut current_date = optimized_items
            .first()
            .map(|item| item.date)
            .unwrap_or_else(Utc::now);
        for (i, &original_index) in balanced_order.iter().enumerate() {
            let mut item = optimized_items[original_index].clone();
            item.date = current_date;
            optimized_items[i] = item;

            // Calculate next session date (simplified - in real implementation would use plan settings)
            current_date += chrono::Duration::days(2);
        }

        Ok(optimized_items)
    }

    /// Calculate optimization score for a single session
    fn calculate_session_optimization_score(
        &self,
        _structure: &crate::types::CourseStructure,
        _item: &PlanItem,
        analysis: &SessionDifficultyAnalysis,
        user_preferences: &UserPreferences,
    ) -> Result<f32> {
        let mut score = 0.0;

        // Factor in cognitive load appropriateness
        score += match user_preferences.pacing_preference {
            PacingPreference::Intensive => {
                if analysis.cognitive_load_score < 0.3 && analysis.average_difficulty > 0.4 {
                    1.0 - analysis.cognitive_load_score
                } else {
                    0.5
                }
            }
            PacingPreference::Relaxed => {
                if analysis.cognitive_load_score > 0.7 {
                    analysis.cognitive_load_score
                } else {
                    1.0 - analysis.cognitive_load_score
                }
            }
            PacingPreference::Adaptive => 0.7, // Neutral score
            PacingPreference::Consistent => {
                // Prefer sessions that maintain consistent load
                if analysis.cognitive_load_score > 0.4 && analysis.cognitive_load_score < 0.8 {
                    1.0
                } else {
                    0.5
                }
            }
        };

        Ok(score)
    }

    /// Apply cognitive load balancing to session order
    fn apply_cognitive_load_balancing(
        &self,
        _items: &[PlanItem],
        session_analyses: &[SessionDifficultyAnalysis],
        user_preferences: &UserPreferences,
    ) -> Result<Vec<usize>> {
        let mut remaining_sessions: HashMap<usize, f32> = HashMap::new();
        for (i, analysis) in session_analyses.iter().enumerate() {
            remaining_sessions.insert(i, analysis.cognitive_load_score);
        }

        let mut balanced_order = Vec::new();
        let mut prefer_high_load = false;

        let cognitive_load_config = CognitiveLoadConfig::default();

        while !remaining_sessions.is_empty() {
            let next_session = match cognitive_load_config.ideal_load_distribution {
                LoadDistribution::Uniform => {
                    // Distribute load evenly
                    let sorted: Vec<(usize, f32)> =
                        remaining_sessions.iter().map(|(&k, &v)| (k, v)).collect();
                    sorted.into_iter().next().map(|(i, _)| i)
                }
                LoadDistribution::Progressive => {
                    // Start with easier sessions, progress to harder
                    self.find_next_progressive_session(&remaining_sessions, session_analyses)?
                }
                LoadDistribution::Alternating => {
                    // Alternate between high and low cognitive load
                    if prefer_high_load {
                        self.find_high_load_session(&remaining_sessions, session_analyses)?
                    } else {
                        self.find_low_load_session(&remaining_sessions, session_analyses)?
                    }
                }
                LoadDistribution::Adaptive => {
                    // Use user preferences to guide distribution
                    self.apply_adaptive_load_distribution_order(
                        &remaining_sessions,
                        session_analyses,
                        user_preferences,
                    )?
                }
            };

            if let Some(session) = next_session {
                balanced_order.push(session);
                remaining_sessions.remove(&session);
                prefer_high_load = !prefer_high_load;
            } else {
                // Fallback to any remaining session
                let fallback_session = *remaining_sessions
                    .keys()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No remaining sessions"))?;
                balanced_order.push(fallback_session);
                remaining_sessions.remove(&fallback_session);
            }
        }

        Ok(balanced_order)
    }

    /// Find next session for progressive load distribution
    fn find_next_progressive_session(
        &self,
        remaining_sessions: &HashMap<usize, f32>,
        session_analyses: &[SessionDifficultyAnalysis],
    ) -> Result<Option<usize>> {
        if remaining_sessions.is_empty() {
            return Ok(None);
        }

        // Find session with lowest cognitive load among remaining
        let session = remaining_sessions
            .keys()
            .min_by(|&&a, &&b| {
                session_analyses[a]
                    .cognitive_load_score
                    .partial_cmp(&session_analyses[b].cognitive_load_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();

        Ok(session)
    }

    /// Find session with low cognitive load
    fn find_low_load_session(
        &self,
        remaining_sessions: &HashMap<usize, f32>,
        session_analyses: &[SessionDifficultyAnalysis],
    ) -> Result<Option<usize>> {
        if remaining_sessions.is_empty() {
            return Ok(None);
        }

        let session = remaining_sessions
            .keys()
            .filter(|&&index| session_analyses[index].cognitive_load_score < 0.4)
            .min_by(|&&a, &&b| {
                session_analyses[a]
                    .cognitive_load_score
                    .partial_cmp(&session_analyses[b].cognitive_load_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();

        Ok(session)
    }

    /// Find session with high cognitive load
    fn find_high_load_session(
        &self,
        remaining_sessions: &HashMap<usize, f32>,
        session_analyses: &[SessionDifficultyAnalysis],
    ) -> Result<Option<usize>> {
        if remaining_sessions.is_empty() {
            return Ok(None);
        }

        let session = remaining_sessions
            .keys()
            .filter(|&&index| session_analyses[index].cognitive_load_score > 0.6)
            .max_by(|&&a, &&b| {
                session_analyses[a]
                    .cognitive_load_score
                    .partial_cmp(&session_analyses[b].cognitive_load_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();

        Ok(session)
    }

    /// Apply adaptive load distribution based on user preferences
    fn apply_adaptive_load_distribution_order(
        &self,
        remaining_sessions: &HashMap<usize, f32>,
        session_analyses: &[SessionDifficultyAnalysis],
        user_preferences: &UserPreferences,
    ) -> Result<Option<usize>> {
        if remaining_sessions.is_empty() {
            return Ok(None);
        }

        match user_preferences.pacing_preference {
            PacingPreference::Intensive => {
                // Prefer high cognitive load sessions
                let sorted_sessions: Vec<(usize, f32)> =
                    remaining_sessions.iter().map(|(&k, &v)| (k, v)).collect();
                let mut sorted = sorted_sessions;
                sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                Ok(sorted.into_iter().next().map(|(i, _)| i))
            }
            PacingPreference::Relaxed => {
                // Prefer low cognitive load sessions
                let sorted_sessions: Vec<(usize, f32)> =
                    remaining_sessions.iter().map(|(&k, &v)| (k, v)).collect();
                let mut sorted = sorted_sessions;
                sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                Ok(sorted.into_iter().next().map(|(i, _)| i))
            }
            PacingPreference::Adaptive => {
                // Use progressive distribution as default
                self.find_next_progressive_session(remaining_sessions, session_analyses)
            }
            PacingPreference::Consistent => {
                // Find session closest to average load
                let avg_load = session_analyses
                    .iter()
                    .map(|a| a.cognitive_load_score)
                    .sum::<f32>()
                    / session_analyses.len() as f32;
                let session = remaining_sessions
                    .keys()
                    .min_by(|&&a, &&b| {
                        (session_analyses[a].cognitive_load_score - avg_load)
                            .abs()
                            .partial_cmp(
                                &(session_analyses[b].cognitive_load_score - avg_load).abs(),
                            )
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied();
                Ok(session)
            }
        }
    }

    /// Apply cognitive load constraints to sessions
    fn apply_cognitive_load_constraints(
        &self,
        items: &[PlanItem],
        session_analyses: &[SessionDifficultyAnalysis],
        config: &CognitiveLoadConfig,
        user_preferences: &UserPreferences,
    ) -> Result<Vec<PlanItem>> {
        let adjusted_items = items.to_vec();

        // Check for high cognitive load sessions
        let high_load_count = session_analyses
            .iter()
            .filter(|analysis| analysis.cognitive_load_score > config.max_load_per_session)
            .count();

        if high_load_count > 0 {
            // For now, just add a warning - in full implementation would split sessions
            // This would involve more complex session restructuring
        }

        // Apply load distribution adjustments based on user preferences
        match user_preferences.pacing_preference {
            PacingPreference::Intensive => {
                // Allow higher cognitive loads
                for analysis in session_analyses.iter() {
                    if analysis.cognitive_load_score < 0.5 {
                        // Could combine with next session if available
                        // Simplified implementation for now
                    }
                }
            }
            PacingPreference::Relaxed => {
                // Ensure no session exceeds moderate load
                for analysis in session_analyses.iter() {
                    if analysis.cognitive_load_score > 0.6 {
                        // Would split session in full implementation
                        // For now, just note the need for adjustment
                    }
                }
            }
            _ => {
                // Use default constraints
            }
        }

        Ok(adjusted_items)
    }

    /// Apply adaptive load distribution to items
    fn apply_adaptive_load_distribution(
        &self,
        items: &[PlanItem],
        cognitive_loads: &[f32],
        config: &CognitiveLoadConfig,
    ) -> Result<Vec<PlanItem>> {
        let adjusted_items = items.to_vec();

        match config.ideal_load_distribution {
            LoadDistribution::Progressive => {
                // Ensure gradual increase in cognitive load
                for i in 1..cognitive_loads.len() {
                    if cognitive_loads[i] < cognitive_loads[i - 1] - 0.2 {
                        // Would reorder sessions in full implementation
                    }
                }
            }
            LoadDistribution::Alternating => {
                // Ensure alternating high-low pattern
                for i in 1..cognitive_loads.len() {
                    let should_be_high = i % 2 == 1;
                    let is_high = cognitive_loads[i] > 0.6;
                    if should_be_high != is_high {
                        // Would reorder sessions in full implementation
                    }
                }
            }
            _ => {
                // Use default distribution
            }
        }

        Ok(adjusted_items)
    }

    /// Calculate cognitive load distribution
    fn calculate_cognitive_load_distribution(
        &self,
        session_analyses: &[SessionDifficultyAnalysis],
    ) -> Vec<f32> {
        session_analyses
            .iter()
            .map(|analysis| analysis.cognitive_load_score)
            .collect()
    }

    /// Identify improvements made by optimization
    fn identify_improvements(
        &self,
        original_items: &[PlanItem],
        optimized_items: &[PlanItem],
        original_scores: &FactorScores,
        optimized_scores: &FactorScores,
    ) -> Vec<OptimizationImprovement> {
        let mut improvements = Vec::new();

        // Check for content grouping improvements
        if optimized_scores.content_similarity_score > original_scores.content_similarity_score {
            improvements.push(OptimizationImprovement {
                session_index: 0,
                improvement_type: ImprovementType::ContentGrouping,
                description: "Improved content similarity within sessions".to_string(),
                impact_score: optimized_scores.content_similarity_score
                    - original_scores.content_similarity_score,
            });
        }

        // Check for duration balancing improvements
        if optimized_scores.duration_balance_score > original_scores.duration_balance_score {
            improvements.push(OptimizationImprovement {
                session_index: 0,
                improvement_type: ImprovementType::DurationBalancing,
                description: "Better duration balance achieved".to_string(),
                impact_score: optimized_scores.duration_balance_score
                    - original_scores.duration_balance_score,
            });
        }

        // Check for difficulty progression improvements
        if optimized_scores.difficulty_progression_score
            > original_scores.difficulty_progression_score
        {
            improvements.push(OptimizationImprovement {
                session_index: 0,
                improvement_type: ImprovementType::DifficultySmoothing,
                description: "Smoother difficulty progression created".to_string(),
                impact_score: optimized_scores.difficulty_progression_score
                    - original_scores.difficulty_progression_score,
            });
        }

        // Check for user preference improvements
        if optimized_scores.user_preference_score > original_scores.user_preference_score {
            improvements.push(OptimizationImprovement {
                session_index: 0,
                improvement_type: ImprovementType::UserPreferenceAlignment,
                description: "Better alignment with user preferences".to_string(),
                impact_score: optimized_scores.user_preference_score
                    - original_scores.user_preference_score,
            });
        }

        // Overall improvement
        if optimized_scores.overall_score > original_scores.overall_score {
            improvements.push(OptimizationImprovement {
                session_index: 0,
                improvement_type: ImprovementType::CognitiveLoadReduction,
                description: format!(
                    "Overall optimization score improved from {:.3} to {:.3}",
                    original_scores.overall_score, optimized_scores.overall_score
                ),
                impact_score: optimized_scores.overall_score - original_scores.overall_score,
            });
        }

        // Check for session reordering
        let original_order: Vec<usize> = (0..original_items.len()).collect();
        let optimized_order: Vec<usize> =
            optimized_items.iter().enumerate().map(|(i, _)| i).collect();
        if original_order != optimized_order {
            improvements.push(OptimizationImprovement {
                session_index: 0,
                improvement_type: ImprovementType::CognitiveLoadReduction,
                description: "Sessions reordered for optimal learning progression".to_string(),
                impact_score: 0.1, // Fixed impact score for reordering
            });
        }

        improvements
    }

    /// Generate optimization warnings
    fn generate_optimization_warnings(
        &self,
        session_analyses: &[SessionDifficultyAnalysis],
        user_preferences: &UserPreferences,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for high cognitive load sessions
        let high_load_count = session_analyses
            .iter()
            .filter(|analysis| analysis.cognitive_load_score > self.max_cognitive_load)
            .count();

        if high_load_count > 0 {
            warnings.push(format!(
                "{} sessions have high cognitive load ({:.1}) - consider breaking them down",
                high_load_count, self.max_cognitive_load
            ));
        }

        // Check for difficulty jumps
        let steep_jumps = self.count_steep_difficulty_jumps(session_analyses);
        if steep_jumps > 0
            && user_preferences.difficulty_preference == DifficultyPreference::GradualProgression
        {
            warnings.push(format!(
                    "{steep_jumps} steep difficulty jumps detected - may conflict with gradual progression preference"
                ));
        }

        // Check for preference conflicts
        if user_preferences.difficulty_preference == DifficultyPreference::ConsistentLevel {
            let variance = session_analyses.len() as f32
                / session_analyses
                    .iter()
                    .map(|a| {
                        (a.average_difficulty
                            - session_analyses
                                .iter()
                                .map(|a| a.average_difficulty)
                                .sum::<f32>()
                                / session_analyses.len() as f32)
                            .powi(2)
                    })
                    .sum::<f32>();
            if variance > 0.1 {
                warnings.push("High difficulty variance detected - may conflict with consistent level preference".to_string());
            }
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Course, CourseStructure, Module, Plan, PlanSettings, Section, StructureMetadata,
    };
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_course() -> Course {
        let sections = vec![
            Section {
                video_index: 0,
                title: "Introduction to Programming".to_string(),
                duration: Duration::from_secs(600),
            },
            Section {
                video_index: 1,
                title: "Advanced Algorithms".to_string(),
                duration: Duration::from_secs(1800),
            },
            Section {
                video_index: 2,
                title: "Basic Data Structures".to_string(),
                duration: Duration::from_secs(900),
            },
        ];

        let module = Module::new_basic("Programming Course".to_string(), sections);

        let structure = CourseStructure {
            modules: vec![module],
            metadata: StructureMetadata {
                total_videos: 3,
                total_duration: Duration::from_secs(3300),
                estimated_duration_hours: Some(0.92),
                structure_quality_score: None,
                difficulty_level: None,
                content_coherence_score: None,
            },
            clustering_metadata: None,
        };

    fn create_test_plan(course: &Course) -> Plan {
        let items = vec![
            PlanItem {
                date: Utc::now(),
                module_title: "Programming Course".to_string(),
                section_title: "Introduction to Programming".to_string(),
                video_indices: vec![0],
                total_duration: Duration::from_secs(600),
                estimated_completion_time: Duration::from_secs(750),
                completed: false,
                overflow_warnings: Vec::new(),
            },
            PlanItem {
                date: Utc::now() + chrono::Duration::days(2),
                module_title: "Programming Course".to_string(),
                section_title: "Advanced Algorithms".to_string(),
                video_indices: vec![1],
                total_duration: Duration::from_secs(1800),
                estimated_completion_time: Duration::from_secs(2250),
                completed: false,
                overflow_warnings: Vec::new(),
            },
            PlanItem {
                date: Utc::now() + chrono::Duration::days(4),
                module_title: "Programming Course".to_string(),
                section_title: "Basic Data Structures".to_string(),
                video_indices: vec![2],
                total_duration: Duration::from_secs(900),
                estimated_completion_time: Duration::from_secs(1125),
                completed: false,
                overflow_warnings: Vec::new(),
            },
        ];

        Plan {
            id: uuid::Uuid::new_v4(),
            course_id: course.id,
            settings: PlanSettings {
                sessions_per_week: 3,
                session_length_minutes: 60,
                start_date: Utc::now(),
                include_weekends: false,
                advanced_settings: None,
            },
            created_at: Utc::now(),
            items,
        }
    }

    #[test]
    fn test_multi_factor_optimization() {
        let optimizer = MultiFactorOptimizer::new(DifficultyLevel::Intermediate);
        let course = create_test_course();
        let plan = create_test_plan(&course);
        let user_preferences = UserPreferences::default();

        let result = optimizer.optimize_session_sequence(&course, &plan, &user_preferences);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.optimized_items.len(), 3);
        assert!(result.optimization_score >= 0.0);
        assert!(result.optimization_score <= 1.0);
    }

    #[test]
    fn test_factor_score_calculation() {
        let optimizer = MultiFactorOptimizer::new(DifficultyLevel::Intermediate);
        let course = create_test_course();
        let plan = create_test_plan(&course);
        let user_preferences = UserPreferences::default();

        let structure = course.structure.as_ref().unwrap();
        let session_analyses = optimizer
            .analyze_current_sessions(structure, &plan.items)
            .unwrap();
        let scores = optimizer
            .calculate_factor_scores(structure, &plan.items, &session_analyses, &user_preferences)
            .unwrap();

        assert!(scores.content_similarity_score >= 0.0);
        assert!(scores.content_similarity_score <= 1.0);
        assert!(scores.duration_balance_score >= 0.0);
        assert!(scores.duration_balance_score <= 1.0);
        assert!(scores.difficulty_progression_score >= 0.0);
        assert!(scores.difficulty_progression_score <= 1.0);
        assert!(scores.user_preference_score >= 0.0);
        assert!(scores.user_preference_score <= 1.0);
    }

    #[test]
    fn test_cognitive_load_balancing() {
        let optimizer = MultiFactorOptimizer::new(DifficultyLevel::Intermediate);
        let course = create_test_course();
        let plan = create_test_plan(&course);
        let config = CognitiveLoadConfig::default();

        let result = optimizer.balance_cognitive_load(&course, &plan, &config);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(!result.cognitive_load_distribution.is_empty());
        assert!(result.cognitive_load_distribution.len() <= 3);
    }
}
