//! NLP processing module for Course Pilot
//!
//! This module provides functionality for analyzing course content and
//! structuring video titles into logical course hierarchies.

pub mod clustering;
pub mod preference_service;
pub mod processor;

// Re-export main processing function
pub use processor::structure_course;

// Re-export preference service
pub use preference_service::{AutoTuningService, PreferenceService};

// Re-export error types
pub use crate::NlpError;

use regex::Regex;
use std::sync::OnceLock;

/// Common course structure keywords and patterns
pub struct StructurePatterns {
    pub module_keywords: Vec<&'static str>,
    pub section_keywords: Vec<&'static str>,
    pub numeric_patterns: Vec<Regex>,
}

impl StructurePatterns {
    pub fn default() -> &'static Self {
        static PATTERNS: OnceLock<StructurePatterns> = OnceLock::new();
        PATTERNS.get_or_init(|| StructurePatterns {
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
            numeric_patterns: vec![
                Regex::new(r"\b(\d+)\b").unwrap(),
                Regex::new(r"\b(part|chapter|lesson|module|section)\s*(\d+)").unwrap(),
                Regex::new(r"\b(\d+)[:\.\-]\s*").unwrap(),
                Regex::new(r"\((\d+)\)").unwrap(),
            ],
        })
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

    patterns
        .module_keywords
        .iter()
        .any(|keyword| text_lower.contains(keyword))
}

/// Check if text contains section-level keywords
pub fn is_section_indicator(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    let patterns = StructurePatterns::default();

    patterns
        .section_keywords
        .iter()
        .any(|keyword| text_lower.contains(keyword))
}

/// Clean and normalize text for analysis
pub fn normalize_text(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
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

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
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
