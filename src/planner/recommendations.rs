/*!
Study recommendations module

This module provides:
- StudyRecommendations and DifficultyProgression types
- Public API to generate personalized recommendations based on course/content and user settings
- Internal helpers for session frequency/length, time management tips, difficulty progression, and completion estimate

This code was moved from planner/mod.rs to reduce its size and keep responsibilities focused.
*/

use crate::planner::strategies::adaptive::analyze_section_difficulty;
use crate::planner::strategy::{analyze_course_complexity, infer_user_experience_level};
use crate::types::{Course, DifficultyLevel, PlanSettings};

/// Public API: generate personalized study recommendations for a course and user settings.
pub fn generate_study_recommendations(
    course: &Course,
    settings: &PlanSettings,
) -> StudyRecommendations {
    let complexity = analyze_course_complexity(course);
    let user_level = infer_user_experience_level(settings);
    let optimal_frequency = calculate_optimal_frequency(course, user_level);

    let mut recommendations = StudyRecommendations {
        optimal_sessions_per_week: optimal_frequency,
        recommended_session_length: calculate_optimal_session_length(course, user_level),
        study_strategy: recommend_study_strategy(complexity, user_level),
        time_management_tips: generate_time_management_tips(settings),
        difficulty_progression: analyze_difficulty_progression(course),
        estimated_completion_weeks: estimate_completion_time(course, settings),
    };

    recommendations.add_personalized_tips(user_level, complexity);
    recommendations
}

/// High-level recommendations summary for the user.
#[derive(Debug, Clone)]
pub struct StudyRecommendations {
    pub optimal_sessions_per_week: u8,
    pub recommended_session_length: u32,
    pub study_strategy: String,
    pub time_management_tips: Vec<String>,
    pub difficulty_progression: DifficultyProgression,
    pub estimated_completion_weeks: u32,
}

impl StudyRecommendations {
    fn add_personalized_tips(&mut self, user_level: DifficultyLevel, complexity: f32) {
        match user_level {
            DifficultyLevel::Beginner => {
                self.time_management_tips.extend(vec![
                    "Start with shorter sessions to build consistency".to_string(),
                    "Take notes during each session for better retention".to_string(),
                    "Don't hesitate to pause and replay difficult sections".to_string(),
                ]);
            }
            DifficultyLevel::Expert => {
                self.time_management_tips.extend(vec![
                    "Focus on practical application over passive watching".to_string(),
                    "Create projects to reinforce learning".to_string(),
                    "Consider teaching concepts to others for deeper understanding".to_string(),
                ]);
            }
            _ => {}
        }

        if complexity > 0.7 {
            self.time_management_tips.push(
                "This course has high complexity - consider extending your timeline".to_string(),
            );
        }
    }
}

/// Describes how difficulty evolves across the course.
#[derive(Debug, Clone)]
pub struct DifficultyProgression {
    pub starts_easy: bool,
    pub has_steep_learning_curve: bool,
    pub complexity_peaks: Vec<String>, // Module names with high complexity
    pub recommended_break_points: Vec<String>,
}

/* =========================
Internal helper functions
========================= */

/// Calculate optimal session length (minutes) based on course complexity and user experience.
fn calculate_optimal_session_length(course: &Course, user_level: DifficultyLevel) -> u32 {
    let complexity = analyze_course_complexity(course);
    let base_length = match user_level {
        DifficultyLevel::Beginner => 30,
        DifficultyLevel::Intermediate => 45,
        DifficultyLevel::Advanced => 60,
        DifficultyLevel::Expert => 90,
    };

    // Adjust based on complexity; cap to two hours
    let complexity_adjustment = (complexity * 30.0) as u32;
    (base_length + complexity_adjustment).min(120)
}

/// Recommend a high-level study strategy label.
fn recommend_study_strategy(complexity: f32, user_level: DifficultyLevel) -> String {
    match (complexity > 0.7, user_level) {
        (true, DifficultyLevel::Beginner) => {
            "Spaced Repetition: This complex course benefits from frequent review sessions"
                .to_string()
        }
        (true, _) => "Adaptive Learning: Adjust pace based on topic difficulty".to_string(),
        (false, DifficultyLevel::Expert) => {
            "Accelerated Learning: Focus on practical application and projects".to_string()
        }
        _ => "Balanced Approach: Steady progress with regular reviews".to_string(),
    }
}

/// Generate time management tips derived from user settings.
fn generate_time_management_tips(settings: &PlanSettings) -> Vec<String> {
    let mut tips = vec![
        "Set a consistent study schedule".to_string(),
        "Eliminate distractions during study sessions".to_string(),
        "Use the Pomodoro Technique for better focus".to_string(),
    ];

    if settings.sessions_per_week >= 5 {
        tips.push("High frequency schedule - ensure adequate rest between sessions".to_string());
    }

    if settings.session_length_minutes >= 90 {
        tips.push("Long sessions - take 10-minute breaks every hour".to_string());
    }

    if !settings.include_weekends {
        tips.push("Weekend-free schedule - use weekends for review and practice".to_string());
    }

    tips
}

/// Analyze how difficulty changes across modules to suggest breakpoints.
fn analyze_difficulty_progression(course: &Course) -> DifficultyProgression {
    let structure = match course.structure.as_ref() {
        Some(s) => s,
        None => {
            // Fallback when structure is absent: neutral progression
            return DifficultyProgression {
                starts_easy: true,
                has_steep_learning_curve: false,
                complexity_peaks: Vec::new(),
                recommended_break_points: Vec::new(),
            };
        }
    };

    let mut module_difficulties = Vec::new();
    let mut complexity_peaks = Vec::new();
    let mut recommended_breaks = Vec::new();

    for module in &structure.modules {
        let mut module_complexity = 0.0;
        let mut section_count = 0;

        for section in &module.sections {
            let difficulty = analyze_section_difficulty(&section.title, section.duration);
            module_complexity += match difficulty {
                DifficultyLevel::Beginner => 0.25,
                DifficultyLevel::Intermediate => 0.5,
                DifficultyLevel::Advanced => 0.75,
                DifficultyLevel::Expert => 1.0,
            };
            section_count += 1;
        }

        if section_count > 0 {
            module_complexity /= section_count as f32;
        }

        module_difficulties.push(module_complexity);

        // Identify complexity peaks
        if module_complexity > 0.7 {
            complexity_peaks.push(module.title.clone());
            recommended_breaks.push(format!(
                "Consider a break after completing: {}",
                module.title
            ));
        }
    }

    let starts_easy = module_difficulties.first().is_some_and(|&d| d < 0.4);
    let has_steep_curve = module_difficulties.windows(2).any(|w| (w[1] - w[0]) > 0.3);

    DifficultyProgression {
        starts_easy,
        has_steep_learning_curve: has_steep_curve,
        complexity_peaks,
        recommended_break_points: recommended_breaks,
    }
}

/// Estimate completion time (weeks), accounting for session frequency and buffers.
fn estimate_completion_time(course: &Course, settings: &PlanSettings) -> u32 {
    let total_videos = course.video_count();
    let videos_per_session =
        crate::planner::capacity::estimated_videos_per_session(course, settings);
    let total_sessions = total_videos.div_ceil(videos_per_session.max(1));

    let weeks = (total_sessions as f32 / settings.sessions_per_week as f32).ceil() as u32;

    // Add buffer for reviews and breaks (20%)
    let buffer_weeks = (weeks as f32 * 0.2).ceil() as u32;

    weeks + buffer_weeks
}

/// Decide recommended sessions per week (frequency) from course complexity and user level.
fn calculate_optimal_frequency(course: &Course, user_level: DifficultyLevel) -> u8 {
    let complexity = analyze_course_complexity(course);
    let total_videos = course.video_count();

    match (user_level, complexity, total_videos) {
        // Beginners with complex content need more frequent, shorter sessions
        (DifficultyLevel::Beginner, c, _) if c > 0.7 => 5,
        (DifficultyLevel::Beginner, _, v) if v > 50 => 4,
        (DifficultyLevel::Beginner, _, _) => 3,

        // Intermediate users can handle moderate frequency
        (DifficultyLevel::Intermediate, c, _) if c > 0.8 => 4,
        (DifficultyLevel::Intermediate, _, _) => 3,

        // Advanced users can handle intensive schedules
        (DifficultyLevel::Advanced, _, v) if v > 100 => 5,
        (DifficultyLevel::Advanced, _, _) => 4,

        // Expert users prefer intensive, focused sessions
        (DifficultyLevel::Expert, _, _) => 3, // Fewer but longer sessions
    }
}
