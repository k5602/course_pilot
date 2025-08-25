use crate::PlanError;
use crate::types::{Course, DifficultyLevel, PlanItem, PlanSettings};
use chrono::{DateTime, Datelike, Utc, Weekday};
use std::time::Duration;

/// Adaptive strategy:
/// - Builds per-section enhanced sessions with difficulty and cognitive load analysis
/// - Optimizes session order by type, difficulty, and load
/// - Schedules sessions with adaptive spacing and timing heuristics
pub fn generate_adaptive_plan(
    course: &Course,
    settings: &PlanSettings,
) -> Result<Vec<PlanItem>, PlanError> {
    let structure = course.structure.as_ref().expect("Course must be structured for adaptive plan");
    let mut plan_items = Vec::new();
    let mut current_date = settings.start_date;

    // Create enhanced session plans with cognitive load analysis
    let mut enhanced_sessions = Vec::new();

    for module in &structure.modules {
        for section in &module.sections {
            let difficulty = analyze_section_difficulty(&section.title, section.duration);
            let cognitive_load = calculate_cognitive_load(&section.title, section.duration);
            let optimal_time = determine_optimal_time_of_day(&section.title);

            enhanced_sessions.push(EnhancedSessionPlan {
                title: section.title.clone(),
                video_indices: vec![section.video_index],
                session_type: classify_session_type(&section.title),
                difficulty_level: difficulty,
                estimated_cognitive_load: cognitive_load,
                prerequisites: Vec::new(), // Could be enhanced with dependency analysis
                optimal_time_of_day: optimal_time,
            });
        }
    }

    // Optimize session order based on cognitive load and learning principles
    optimize_session_sequence(&mut enhanced_sessions);

    // Convert enhanced sessions to plan items with intelligent scheduling
    for session in enhanced_sessions {
        // Adjust scheduling based on session characteristics
        let adjusted_date = adjust_date_for_optimal_learning(current_date, &session, settings);

        // Calculate next session date with adaptive spacing (before partial move)
        let spacing_days = calculate_adaptive_spacing(&session);

        // Find the section duration for this session
        let mut section_duration = Duration::from_secs(30 * 60); // Default 30 minutes
        for module in &structure.modules {
            for section in &module.sections {
                if session.video_indices.contains(&section.video_index) {
                    section_duration = section.duration;
                    break;
                }
            }
        }

        let estimated_completion_time =
            crate::types::duration_utils::calculate_completion_time_with_buffer(
                section_duration,
                0.25,
            );

        plan_items.push(PlanItem {
            date: adjusted_date,
            module_title: "Adaptive Learning".to_string(),
            section_title: session.title,
            video_indices: session.video_indices.clone(),
            completed: false,
            total_duration: section_duration,
            estimated_completion_time,
            overflow_warnings: Vec::new(),
        });

        current_date = crate::planner::get_next_session_date(
            adjusted_date + chrono::Duration::days(spacing_days),
            settings.sessions_per_week,
            settings.include_weekends,
        );
    }

    Ok(plan_items)
}

/// Learning session types for varied engagement
#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    Introduction, // New concept introduction
    Practice,     // Hands-on practice
    Review,       // Content review
    Assessment,   // Knowledge check
    Project,      // Applied project work
    #[allow(dead_code)]
    Break, // Rest/consolidation
}

/// Optimal time of day for different types of learning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Morning,   // 6-12: Best for complex/new concepts
    Afternoon, // 12-18: Good for practice/application
    Evening,   // 18-22: Best for review/consolidation
}

/// Enhanced session planning with cognitive load considerations
#[derive(Debug, Clone)]
pub struct EnhancedSessionPlan {
    title: String,
    video_indices: Vec<usize>,
    session_type: SessionType,
    difficulty_level: DifficultyLevel,
    estimated_cognitive_load: f32, // 0.0 to 1.0
    #[allow(dead_code)]
    prerequisites: Vec<usize>, // Session indices that must be completed first
    #[allow(dead_code)]
    optimal_time_of_day: Option<TimeOfDay>,
}

/// Cognitive load factors for different content types
const COGNITIVE_LOAD_FACTORS: &[(f32, &str)] = &[
    (0.9, "algorithm"),
    (0.8, "theory"),
    (0.7, "concept"),
    (0.6, "example"),
    (0.5, "practice"),
    (0.4, "review"),
    (0.3, "introduction"),
];

/// Heuristic difficulty analysis based on title keywords and duration.
/// Falls back to duration when keywords are absent.
pub fn analyze_section_difficulty(title: &str, duration: Duration) -> DifficultyLevel {
    let title_lower = title.to_lowercase();
    let duration_minutes = duration.as_secs() / 60;

    // Keywords indicating difficulty levels
    let expert_keywords = ["advanced", "expert", "complex", "algorithm", "optimization"];
    let advanced_keywords = ["intermediate", "deep", "detailed", "implementation"];
    let beginner_keywords = ["introduction", "basic", "getting started", "overview"];

    // Check for explicit difficulty indicators
    for keyword in expert_keywords {
        if title_lower.contains(keyword) {
            return DifficultyLevel::Expert;
        }
    }

    for keyword in advanced_keywords {
        if title_lower.contains(keyword) {
            return DifficultyLevel::Advanced;
        }
    }

    for keyword in beginner_keywords {
        if title_lower.contains(keyword) {
            return DifficultyLevel::Beginner;
        }
    }

    // Use duration as a heuristic
    match duration_minutes {
        0..=10 => DifficultyLevel::Beginner,
        11..=25 => DifficultyLevel::Intermediate,
        26..=45 => DifficultyLevel::Advanced,
        _ => DifficultyLevel::Expert,
    }
}

/// Calculate cognitive load for a section
pub fn calculate_cognitive_load(title: &str, duration: Duration) -> f32 {
    let title_lower = title.to_lowercase();
    let mut load = 0.5; // Base load

    // Adjust based on content type
    for (load_factor, keyword) in COGNITIVE_LOAD_FACTORS {
        if title_lower.contains(keyword) {
            load = *load_factor;
            break;
        }
    }

    // Adjust based on duration (longer content = higher cognitive load)
    let duration_minutes = duration.as_secs() / 60;
    let duration_factor = (duration_minutes as f32 / 30.0).min(1.5);

    (load * duration_factor).min(1.0)
}

/// Classify the type of learning session
pub fn classify_session_type(title: &str) -> SessionType {
    let title_lower = title.to_lowercase();

    if title_lower.contains("introduction") || title_lower.contains("overview") {
        SessionType::Introduction
    } else if title_lower.contains("practice") || title_lower.contains("exercise") {
        SessionType::Practice
    } else if title_lower.contains("review") || title_lower.contains("summary") {
        SessionType::Review
    } else if title_lower.contains("project") || title_lower.contains("build") {
        SessionType::Project
    } else if title_lower.contains("test") || title_lower.contains("quiz") {
        SessionType::Assessment
    } else {
        SessionType::Introduction // Default
    }
}

/// Determine optimal time of day for content
pub fn determine_optimal_time_of_day(title: &str) -> Option<TimeOfDay> {
    let title_lower = title.to_lowercase();

    if title_lower.contains("algorithm") || title_lower.contains("complex") {
        Some(TimeOfDay::Morning) // Complex topics in the morning
    } else if title_lower.contains("practice") || title_lower.contains("exercise") {
        Some(TimeOfDay::Afternoon) // Practice in the afternoon
    } else if title_lower.contains("review") || title_lower.contains("summary") {
        Some(TimeOfDay::Evening) // Review in the evening
    } else {
        None // No specific preference
    }
}

/// Optimize the sequence of sessions for better learning outcomes
pub fn optimize_session_sequence(sessions: &mut Vec<EnhancedSessionPlan>) {
    // Sort by type, then difficulty, then cognitive load
    sessions.sort_by(|a, b| {
        // Primary sort: session type (intro -> practice -> project -> review -> assessment -> break)
        let type_order_a = match a.session_type {
            SessionType::Introduction => 0,
            SessionType::Practice => 1,
            SessionType::Project => 2,
            SessionType::Review => 3,
            SessionType::Assessment => 4,
            SessionType::Break => 5,
        };

        let type_order_b = match b.session_type {
            SessionType::Introduction => 0,
            SessionType::Practice => 1,
            SessionType::Project => 2,
            SessionType::Review => 3,
            SessionType::Assessment => 4,
            SessionType::Break => 5,
        };

        // Secondary sort: difficulty level
        let diff_order_a = match a.difficulty_level {
            DifficultyLevel::Beginner => 0,
            DifficultyLevel::Intermediate => 1,
            DifficultyLevel::Advanced => 2,
            DifficultyLevel::Expert => 3,
        };

        let diff_order_b = match b.difficulty_level {
            DifficultyLevel::Beginner => 0,
            DifficultyLevel::Intermediate => 1,
            DifficultyLevel::Advanced => 2,
            DifficultyLevel::Expert => 3,
        };

        type_order_a
            .cmp(&type_order_b)
            .then(diff_order_a.cmp(&diff_order_b))
            .then(a.estimated_cognitive_load.partial_cmp(&b.estimated_cognitive_load).unwrap())
    });
}

/// Adjust date for optimal learning based on session characteristics
pub fn adjust_date_for_optimal_learning(
    base_date: DateTime<Utc>,
    session: &EnhancedSessionPlan,
    settings: &PlanSettings,
) -> DateTime<Utc> {
    let mut adjusted_date = base_date;

    // Avoid scheduling high cognitive load sessions on Mondays (post-weekend)
    if session.estimated_cognitive_load > 0.7 && adjusted_date.weekday() == Weekday::Mon {
        adjusted_date += chrono::Duration::days(1);
    }

    // Space out difficult sessions
    if session.difficulty_level == DifficultyLevel::Expert {
        // Ensure at least 2 days between expert sessions
        adjusted_date += chrono::Duration::days(1);
    }

    // Respect weekend preferences
    if !settings.include_weekends {
        while adjusted_date.weekday() == Weekday::Sat || adjusted_date.weekday() == Weekday::Sun {
            adjusted_date += chrono::Duration::days(1);
        }
    }

    adjusted_date
}

/// Calculate adaptive spacing between sessions (in days)
pub fn calculate_adaptive_spacing(session: &EnhancedSessionPlan) -> i64 {
    match (session.difficulty_level, session.estimated_cognitive_load) {
        (DifficultyLevel::Expert, load) if load > 0.8 => 3, // 3 days for very difficult content
        (DifficultyLevel::Advanced, load) if load > 0.7 => 2, // 2 days for advanced content
        (DifficultyLevel::Expert, _) => 2,                  // 2 days for expert content
        (DifficultyLevel::Advanced, _) => 1,                // 1 day for advanced content
        _ => 0,                                             // Normal spacing for others
    }
}
