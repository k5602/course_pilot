//! NLP processing module for Course Pilot
//!
//! Public API contract (session-group-first):
//! - NLP only groups raw titles into sessions. It must not reorder imported videos.
//! - Use `group_sessions(&[String]) -> Vec<Vec<usize>>` for the canonical grouping.
//! - If you need a `CourseStructure`, use `structure_course(Vec<String>)` which builds
//!   "Session N" modules from the original order with zero-duration sections.
//!
//! Planner integration:
//! - Feed the groups into `planner::generate_plan_from_groups(...)`.
//! - The planner is responsible for packing by duration, spacing, difficulty progression,
//!   and all scheduling optimizations. NLP does not influence timing.
//!
//! Advanced NLP (optional):
//! - Advanced clustering and topic analysis are gated behind the `advanced_nlp` feature.
//! - These are not part of the structuring contract and should not change ordering.
//!
//! In short: NLP produces groups, planner consumes them. Order in == order out.

pub mod clustering;
pub mod preference_service;

pub mod sequential_detection;
pub mod session_grouper;

// Lightweight grouping-based APIs (SoT) — preserve original import order

/// Group sessions from raw titles without reordering.
/// Returns groups of indices referencing the input titles.
pub fn group_sessions(titles: &[String]) -> Result<Vec<Vec<usize>>, NlpError> {
    if titles.is_empty() {
        return Ok(vec![]);
    }

    // Preserve order, simple chunking by count (default settings)
    let grouper = crate::nlp::SequentialGrouper::new();
    grouper.group(titles).map_err(|e| NlpError::Processing(format!("Grouping failed: {e}")))
}

/// Build a minimal CourseStructure from session groups while preserving order.
/// This does NOT perform restructuring or heavy clustering. Durations are set to 0.
pub fn structure_course(titles: Vec<String>) -> Result<CourseStructure, NlpError> {
    if titles.is_empty() {
        return Err(NlpError::InvalidInput("No titles provided".to_string()));
    }

    let groups = group_sessions(&titles)?;

    // Convert session groups to modules with sections (preserve original order)
    let mut modules = Vec::with_capacity(groups.len());
    for (i, group) in groups.iter().enumerate() {
        let mut sections = Vec::with_capacity(group.len());
        for &idx in group {
            let title = titles.get(idx).cloned().unwrap_or_else(|| format!("Video {}", idx + 1));
            sections.push(Section { title, video_index: idx, duration: Duration::from_secs(0) });
        }

        let module_title = format!("Session {}", i + 1);
        modules.push(Module::new_basic(module_title, sections));
    }

    let metadata = StructureMetadata {
        total_videos: titles.len(),
        total_duration: Duration::from_secs(0),
        estimated_duration_hours: None,
        difficulty_level: None,
        structure_quality_score: None,
        content_coherence_score: None,
        content_type_detected: Some("Sequential".to_string()),
        original_order_preserved: Some(true),
        processing_strategy_used: Some("PreserveOrder".to_string()),
    };

    Ok(CourseStructure::new_basic(modules, metadata).with_aggregated_metadata())
}

// Re-export preference service
pub use preference_service::{AutoTuningService, PreferenceService};

// Re-export sequential detection
pub use sequential_detection::{
    ContentType, ContentTypeAnalysis, ProcessingRecommendation, detect_sequential_patterns,
};

// Re-export session grouper types
pub use session_grouper::{
    SequentialGrouper, SessionGrouper, SessionGrouperConfig, SessionGrouperFactory,
    SimilarityGrouper,
};

// Re-export error types
pub use crate::NlpError;

use log::error;
use regex::Regex;
use std::sync::OnceLock;
use std::time::Duration;

use crate::types::{CourseStructure, Module, Section, StructureMetadata};

/// Common course structure keywords and patterns
pub struct StructurePatterns {
    pub module_keywords: Vec<&'static str>,
    pub section_keywords: Vec<&'static str>,
    pub numeric_patterns: Vec<Regex>,
}

impl StructurePatterns {
    pub fn default() -> &'static Self {
        static PATTERNS: OnceLock<StructurePatterns> = OnceLock::new();
        PATTERNS.get_or_init(|| {
            // Create regex patterns with proper error handling
            let numeric_patterns = Self::create_numeric_patterns();

            StructurePatterns {
                module_keywords: vec![
                    "module",
                    "chapter",
                    "part",
                    "unit",
                    "section",
                    "week",
                    "day",
                    "lesson",
                    "tutorial",
                    "course",
                    "introduction",
                    "conclusion",
                    "overview",
                    "summary",
                    "review",
                    "project",
                    "assignment",
                ],
                section_keywords: vec![
                    "lecture",
                    "video",
                    "demo",
                    "example",
                    "exercise",
                    "practice",
                    "lab",
                    "workshop",
                    "seminar",
                    "discussion",
                    "quiz",
                    "test",
                    "exam",
                    "homework",
                    "reading",
                    "study",
                    "guide",
                ],
                numeric_patterns,
            }
        })
    }

    fn create_numeric_patterns() -> Vec<Regex> {
        let pattern_strings = vec![
            r"\b(\d+)\b",
            r"\b(part|chapter|lesson|module|section)\s*(\d+)",
            r"\b(\d+)[:\.\-]\s*",
            r"\((\d+)\)",
        ];

        let mut patterns = Vec::new();
        for pattern_str in pattern_strings {
            match Regex::new(pattern_str) {
                Ok(regex) => patterns.push(regex),
                Err(e) => {
                    error!("Failed to compile regex pattern '{pattern_str}': {e}");
                    // Continue with other patterns instead of panicking
                },
            }
        }

        // If no patterns compiled successfully, provide a basic fallback
        if patterns.is_empty() {
            error!("No regex patterns compiled successfully, using basic fallback");
            if let Ok(fallback) = Regex::new(r"\d+") {
                patterns.push(fallback);
            }
        }

        patterns
    }
}

/// Extract numeric indicators from text
pub fn extract_numbers(text: &str) -> Vec<u32> {
    let patterns = StructurePatterns::default();
    let mut numbers = Vec::new();

    for pattern in &patterns.numeric_patterns {
        for cap in pattern.captures_iter(text) {
            if let Some(num_str) = cap.get(1).or_else(|| cap.get(2)) {
                if let Ok(num) = num_str.as_str().parse::<u32>() {
                    numbers.push(num);
                }
            }
        }
    }

    numbers.sort_unstable();
    numbers.dedup();
    numbers
}

/// Check if text contains module-level keywords
pub fn is_module_indicator(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    let patterns = StructurePatterns::default();

    patterns.module_keywords.iter().any(|keyword| text_lower.contains(keyword))
}

/// Check if text contains section-level keywords
pub fn is_section_indicator(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    let patterns = StructurePatterns::default();

    patterns.section_keywords.iter().any(|keyword| text_lower.contains(keyword))
}

/// Clean and normalize text for analysis
pub fn normalize_text(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Calculate similarity between two text strings
pub fn text_similarity(text1: &str, text2: &str) -> f32 {
    let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 { 0.0 } else { intersection as f32 / union as f32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_extraction() {
        assert_eq!(extract_numbers("Module 1: Introduction"), vec![1]);
        assert_eq!(extract_numbers("Chapter 5 - Part 2"), vec![2, 5]);
        assert_eq!(extract_numbers("No numbers here"), vec![] as Vec<u32>);
    }

    #[test]
    fn test_module_detection() {
        assert!(is_module_indicator("Module 1: Introduction"));
        assert!(is_module_indicator("Chapter Overview"));
        assert!(!is_module_indicator("Just a regular video"));
    }

    #[test]
    fn test_section_detection() {
        assert!(is_section_indicator("Lecture 5: Advanced Topics"));
        assert!(is_section_indicator("Practice Exercise"));
        assert!(!is_section_indicator("Module Introduction"));
    }

    #[test]
    fn test_text_normalization() {
        assert_eq!(normalize_text("  Hello, World!  123  "), "hello world 123");
    }

    #[test]
    fn test_text_similarity() {
        assert_eq!(text_similarity("hello world", "hello world"), 1.0);
        assert_eq!(text_similarity("hello", "world"), 0.0);
        assert!(text_similarity("hello world", "hello there") > 0.0);
    }
}
