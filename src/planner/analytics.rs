/*!
Planner analytics: plan quality analysis, velocity, cognitive and temporal distributions.

This module provides:
- Learning velocity analysis and recommendations
- Cognitive load distribution analysis across sessions
- Temporal distribution analysis (gaps, weekend utilization, consistency)
- Overall plan score and improvement suggestions
- Aggregated PlanAnalysis struct and analyze_plan_effectiveness entrypoint
*/

use crate::types::Plan;
use chrono::{Datelike, Weekday};
use std::time::Duration;

// Reuse cognitive load estimator from adaptive strategy helpers
use crate::planner::strategies::adaptive::calculate_cognitive_load;

/// Learning velocity analysis structure
#[derive(Debug, Clone, PartialEq)]
pub struct LearningVelocityAnalysis {
    pub videos_per_day: f32,
    pub velocity_category: VelocityCategory,
    pub total_duration_days: i64,
    pub recommended_adjustments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VelocityCategory {
    Slow,
    Moderate,
    Fast,
    Intensive,
}

/// Analyze learning velocity and suggest adjustments
pub fn analyze_learning_velocity(plan: &Plan) -> LearningVelocityAnalysis {
    let total_videos = plan
        .items
        .iter()
        .map(|item| item.video_indices.len())
        .sum::<usize>();
    let total_days = if let (Some(first), Some(last)) = (plan.items.first(), plan.items.last()) {
        (last.date - first.date).num_days()
    } else {
        0
    };

    let videos_per_day = if total_days > 0 {
        total_videos as f32 / total_days as f32
    } else {
        0.0
    };

    let velocity_category = match videos_per_day {
        v if v < 0.5 => VelocityCategory::Slow,
        v if v < 1.0 => VelocityCategory::Moderate,
        v if v < 2.0 => VelocityCategory::Fast,
        _ => VelocityCategory::Intensive,
    };

    LearningVelocityAnalysis {
        videos_per_day,
        velocity_category: velocity_category.clone(),
        total_duration_days: total_days,
        recommended_adjustments: generate_velocity_recommendations(velocity_category, total_videos),
    }
}

fn generate_velocity_recommendations(
    category: VelocityCategory,
    total_videos: usize,
) -> Vec<String> {
    match category {
        VelocityCategory::Slow => vec![
            "Consider increasing session frequency for better momentum".to_string(),
            "Add more practice sessions to reinforce learning".to_string(),
            if total_videos > 50 {
                "Course may take longer than expected - consider breaking into phases".to_string()
            } else {
                "Pace is suitable for deep learning".to_string()
            },
        ],
        VelocityCategory::Moderate => vec![
            "Good balance between depth and progress".to_string(),
            "Consider adding review sessions every 2 weeks".to_string(),
        ],
        VelocityCategory::Fast => vec![
            "Fast pace - ensure adequate time for practice".to_string(),
            "Add buffer days for complex topics".to_string(),
            "Consider spaced repetition for better retention".to_string(),
        ],
        VelocityCategory::Intensive => vec![
            "Very intensive pace - monitor for burnout".to_string(),
            "Ensure adequate breaks between sessions".to_string(),
            "Consider extending session length instead of frequency".to_string(),
            "Add consolidation days every week".to_string(),
        ],
    }
}

/// Aggregated analysis of a plan
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAnalysis {
    pub velocity_analysis: LearningVelocityAnalysis,
    pub load_distribution: LoadDistribution,
    pub temporal_distribution: TemporalDistribution,
    pub overall_score: f32, // 0.0 to 1.0
    pub improvement_suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadDistribution {
    pub average_load: f32,
    pub load_variance: f32,
    pub overloaded_sessions: usize,
    pub underloaded_sessions: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalDistribution {
    pub average_gap_days: f32,
    pub longest_gap_days: i64,
    pub weekend_utilization: f32,
    pub consistency_score: f32,
}

/// Entry point: compute overall plan analysis
pub fn analyze_plan_effectiveness(plan: &Plan) -> PlanAnalysis {
    let velocity_analysis = analyze_learning_velocity(plan);
    let load_distribution = analyze_cognitive_load_distribution(plan);
    let temporal_distribution = analyze_temporal_distribution(plan);

    PlanAnalysis {
        velocity_analysis,
        load_distribution,
        temporal_distribution,
        overall_score: calculate_plan_score(plan),
        improvement_suggestions: generate_improvement_suggestions(plan),
    }
}

/// Analyze cognitive load distribution across sessions
pub fn analyze_cognitive_load_distribution(plan: &Plan) -> LoadDistribution {
    let mut loads = Vec::new();

    for item in &plan.items {
        // Title-only load (duration 0 here; load estimator still considers content keywords)
        let load = calculate_cognitive_load(&item.section_title, Duration::from_secs(0));
        loads.push(load);
    }

    if loads.is_empty() {
        return LoadDistribution {
            average_load: 0.0,
            load_variance: 0.0,
            overloaded_sessions: 0,
            underloaded_sessions: 0,
        };
    }

    let average_load = loads.iter().sum::<f32>() / loads.len() as f32;
    let variance = loads
        .iter()
        .map(|&load| (load - average_load).powi(2))
        .sum::<f32>()
        / loads.len() as f32;

    let overloaded = loads
        .iter()
        .filter(|&&load| load > average_load * 1.5)
        .count();
    let underloaded = loads
        .iter()
        .filter(|&&load| load < average_load * 0.5)
        .count();

    LoadDistribution {
        average_load,
        load_variance: variance,
        overloaded_sessions: overloaded,
        underloaded_sessions: underloaded,
    }
}

/// Analyze temporal distribution of sessions
pub fn analyze_temporal_distribution(plan: &Plan) -> TemporalDistribution {
    if plan.items.len() < 2 {
        return TemporalDistribution {
            average_gap_days: 0.0,
            longest_gap_days: 0,
            weekend_utilization: 0.0,
            consistency_score: 1.0,
        };
    }

    let mut gaps = Vec::new();
    let mut weekend_sessions = 0;

    for i in 1..plan.items.len() {
        let gap = (plan.items[i].date - plan.items[i - 1].date).num_days();
        gaps.push(gap);

        if matches!(plan.items[i].date.weekday(), Weekday::Sat | Weekday::Sun) {
            weekend_sessions += 1;
        }
    }

    let average_gap = gaps.iter().sum::<i64>() as f32 / gaps.len() as f32;
    let longest_gap = *gaps.iter().max().unwrap_or(&0);
    let weekend_util = weekend_sessions as f32 / plan.items.len() as f32;

    // Consistency score based on gap variance
    let gap_variance = gaps
        .iter()
        .map(|&gap| (gap as f32 - average_gap).powi(2))
        .sum::<f32>()
        / gaps.len() as f32;
    let consistency = (1.0f32 / (1.0 + gap_variance)).clamp(0.0, 1.0);

    TemporalDistribution {
        average_gap_days: average_gap,
        longest_gap_days: longest_gap,
        weekend_utilization: weekend_util,
        consistency_score: consistency,
    }
}

/// Calculate overall plan quality score
pub fn calculate_plan_score(plan: &Plan) -> f32 {
    let velocity = analyze_learning_velocity(plan);
    let load_dist = analyze_cognitive_load_distribution(plan);
    let temporal_dist = analyze_temporal_distribution(plan);

    // Weighted scoring
    let velocity_score = match velocity.velocity_category {
        VelocityCategory::Moderate => 1.0,
        VelocityCategory::Fast => 0.8,
        VelocityCategory::Slow => 0.6,
        VelocityCategory::Intensive => 0.4,
    };

    let load_score = (1.0 - load_dist.load_variance).max(0.0);
    let temporal_score = temporal_dist.consistency_score;

    // Weighted average
    (velocity_score * 0.4 + load_score * 0.3 + temporal_score * 0.3)
        .max(0.0)
        .min(1.0)
}

/// Generate improvement suggestions for the plan
pub fn generate_improvement_suggestions(plan: &Plan) -> Vec<String> {
    let mut suggestions = Vec::new();
    let analysis = analyze_learning_velocity(plan);
    let load_dist = analyze_cognitive_load_distribution(plan);
    let temporal_dist = analyze_temporal_distribution(plan);

    // Velocity-based suggestions
    match analysis.velocity_category {
        VelocityCategory::Intensive => {
            suggestions.push("Consider reducing session frequency to prevent burnout".to_string());
        }
        VelocityCategory::Slow => {
            suggestions
                .push("Consider increasing session frequency for better momentum".to_string());
        }
        _ => {}
    }

    // Load distribution suggestions
    if load_dist.overloaded_sessions > plan.items.len() / 4 {
        suggestions
            .push("Many sessions are overloaded - consider redistributing content".to_string());
    }

    if load_dist.underloaded_sessions > plan.items.len() / 4 {
        suggestions
            .push("Many sessions are underloaded - consider consolidating content".to_string());
    }

    // Temporal suggestions
    if temporal_dist.longest_gap_days > 7 {
        suggestions.push(
            "Long gaps between sessions may affect retention - consider more consistent scheduling"
                .to_string(),
        );
    }

    if temporal_dist.consistency_score < 0.7 {
        suggestions
            .push("Irregular session spacing - try to maintain consistent intervals".to_string());
    }

    suggestions
}
