//! Sequential pattern detection for educational content
//!
//! This module provides functionality to detect sequential patterns in video titles
//! to determine if content should preserve its original order or be clustered.

use regex::Regex;
use std::collections::HashMap;

/// Content type analysis results for sequential pattern detection
#[derive(Debug, Clone)]
pub struct ContentTypeAnalysis {
    pub content_type: ContentType,
    pub confidence_score: f32,
    pub sequential_patterns: Vec<SequentialPattern>,
    pub module_indicators: Vec<ModuleIndicator>,
    pub naming_consistency: NamingConsistency,
    pub recommendation: ProcessingRecommendation,
}

/// Detected content type classification
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Sequential, // Numbered lessons, clear progression
    Thematic,   // Topic-based content suitable for clustering
    Mixed,      // Contains both sequential and thematic elements
    Ambiguous,  // Cannot determine clear pattern
}

/// Sequential pattern detection results
#[derive(Debug, Clone)]
pub struct SequentialPattern {
    pub pattern_type: SequentialPatternType,
    pub confidence: f32,
    pub matched_indices: Vec<usize>,
    pub pattern_description: String,
}

/// Types of sequential patterns detected
#[derive(Debug, Clone, PartialEq)]
pub enum SequentialPatternType {
    NumericSequence,    // "Lesson 1", "Part 2", etc.
    AlphabeticSequence, // "Chapter A", "Section B", etc.
    ModuleProgression,  // "Module 1", "Unit 2", etc.
    StepByStep,         // "Step 1", "Tutorial 2", etc.
    ChronologicalOrder, // Date-based or time-based ordering
}

/// Module indicator detection results
#[derive(Debug, Clone)]
pub struct ModuleIndicator {
    pub index: usize,
    pub title: String,
    pub indicator_type: ModuleIndicatorType,
    pub confidence: f32,
}

/// Types of module indicators
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleIndicatorType {
    ExplicitModule,  // "Module", "Unit", "Chapter"
    SectionBreak,    // "Introduction to", "Overview of"
    TopicTransition, // Clear topic change indicators
}

/// Naming consistency analysis
#[derive(Debug, Clone)]
pub struct NamingConsistency {
    pub consistency_score: f32,
    pub common_patterns: Vec<String>,
    pub naming_variations: usize,
    pub has_consistent_format: bool,
}

/// Processing recommendation based on analysis
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingRecommendation {
    PreserveOrder,      // Use sequential processing
    ApplyClustering,    // Use clustering algorithms
    UserChoice,         // Present options to user
    FallbackProcessing, // Use simple fallback
}

/// Detect sequential patterns in content titles for content type classification
///
/// This function analyzes video titles to identify sequential patterns that indicate
/// educational content should preserve its original order rather than be clustered.
///
/// # Arguments
/// * `titles` - Vector of video titles to analyze
///
/// # Returns
/// * `ContentTypeAnalysis` - Comprehensive analysis of content type and patterns
pub fn detect_sequential_patterns(titles: &[String]) -> ContentTypeAnalysis {
    if titles.is_empty() {
        return ContentTypeAnalysis {
            content_type: ContentType::Ambiguous,
            confidence_score: 0.0,
            sequential_patterns: Vec::new(),
            module_indicators: Vec::new(),
            naming_consistency: NamingConsistency {
                consistency_score: 0.0,
                common_patterns: Vec::new(),
                naming_variations: 0,
                has_consistent_format: false,
            },
            recommendation: ProcessingRecommendation::FallbackProcessing,
        };
    }

    // Step 1: Detect various sequential patterns
    let sequential_patterns = detect_all_sequential_patterns(titles);

    // Step 2: Identify module indicators
    let module_indicators = detect_module_indicators(titles);

    // Step 3: Analyze naming consistency
    let naming_consistency = analyze_naming_consistency(titles);

    // Step 4: Calculate overall confidence and determine content type
    let (content_type, confidence_score) = determine_content_type(
        &sequential_patterns,
        &module_indicators,
        &naming_consistency,
        titles.len(),
    );

    // Step 5: Generate processing recommendation
    let recommendation =
        generate_processing_recommendation(&content_type, confidence_score, &sequential_patterns);

    ContentTypeAnalysis {
        content_type,
        confidence_score,
        sequential_patterns,
        module_indicators,
        naming_consistency,
        recommendation,
    }
}

/// Detect all types of sequential patterns in titles
fn detect_all_sequential_patterns(titles: &[String]) -> Vec<SequentialPattern> {
    let mut patterns = Vec::new();

    // Detect numeric sequences (Lesson 1, Part 2, etc.)
    if let Some(pattern) = detect_numeric_sequence_pattern(titles) {
        patterns.push(pattern);
    }

    // Detect alphabetic sequences (Chapter A, Section B, etc.)
    if let Some(pattern) = detect_alphabetic_sequence_pattern(titles) {
        patterns.push(pattern);
    }

    // Detect module progression patterns
    if let Some(pattern) = detect_module_progression_pattern(titles) {
        patterns.push(pattern);
    }

    // Detect step-by-step patterns
    if let Some(pattern) = detect_step_by_step_pattern(titles) {
        patterns.push(pattern);
    }

    // Detect chronological ordering
    if let Some(pattern) = detect_chronological_pattern(titles) {
        patterns.push(pattern);
    }

    patterns
}

/// Detect numeric sequence patterns like "Lesson 1", "Part 2", "Episode 3"
fn detect_numeric_sequence_pattern(titles: &[String]) -> Option<SequentialPattern> {
    let numeric_patterns = [
        r"(?i)\b(lesson|part|episode|chapter|section|tutorial|video)\s*(\d+)",
        r"(?i)\b(\d+)\s*[-.:]\s*",
        r"(?i)\b(\d+)\s*[-.:]\s*(.*)",
        r"(?i)^(\d+)\s*[-.:]\s*",
    ];

    let mut best_pattern = None;
    let mut best_confidence = 0.0;

    for pattern_str in &numeric_patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            let mut matched_indices = Vec::new();
            let mut numbers = Vec::new();

            for (i, title) in titles.iter().enumerate() {
                if let Some(captures) = regex.captures(title) {
                    matched_indices.push(i);
                    // Extract the number from the appropriate capture group
                    if let Some(num_match) = captures.get(2).or_else(|| captures.get(1)) {
                        if let Ok(num) = num_match.as_str().parse::<i32>() {
                            numbers.push(num);
                        }
                    }
                }
            }

            if matched_indices.len() >= 3 {
                let confidence =
                    calculate_sequence_confidence(&numbers, matched_indices.len(), titles.len());

                if confidence > best_confidence {
                    best_confidence = confidence;
                    best_pattern = Some(SequentialPattern {
                        pattern_type: SequentialPatternType::NumericSequence,
                        confidence,
                        matched_indices: matched_indices.clone(),
                        pattern_description: format!(
                            "Numeric sequence detected: {} of {} titles match pattern",
                            matched_indices.len(),
                            titles.len()
                        ),
                    });
                }
            }
        }
    }

    best_pattern
}

/// Detect alphabetic sequence patterns like "Chapter A", "Section B"
fn detect_alphabetic_sequence_pattern(titles: &[String]) -> Option<SequentialPattern> {
    let alpha_pattern = r"(?i)\b(chapter|section|part|unit)\s*([a-z])\b";

    if let Ok(regex) = Regex::new(alpha_pattern) {
        let mut matched_indices = Vec::new();
        let mut letters = Vec::new();

        for (i, title) in titles.iter().enumerate() {
            if let Some(captures) = regex.captures(title) {
                if let Some(letter_match) = captures.get(2) {
                    matched_indices.push(i);
                    letters.push(
                        letter_match
                            .as_str()
                            .chars()
                            .next()
                            .unwrap()
                            .to_ascii_lowercase(),
                    );
                }
            }
        }

        if matched_indices.len() >= 3 {
            let confidence = calculate_alphabetic_sequence_confidence(
                &letters,
                matched_indices.len(),
                titles.len(),
            );

            if confidence > 0.5 {
                return Some(SequentialPattern {
                    pattern_type: SequentialPatternType::AlphabeticSequence,
                    confidence,
                    matched_indices: matched_indices.clone(),
                    pattern_description: format!(
                        "Alphabetic sequence detected: {} titles with letter progression",
                        matched_indices.len()
                    ),
                });
            }
        }
    }

    None
}

/// Detect module progression patterns
fn detect_module_progression_pattern(titles: &[String]) -> Option<SequentialPattern> {
    let module_patterns = [
        r"(?i)\b(module|unit|course)\s*(\d+)",
        r"(?i)^(module|unit|course)\s*(\d+)",
    ];

    for pattern_str in &module_patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            let mut matched_indices = Vec::new();
            let mut numbers = Vec::new();

            for (i, title) in titles.iter().enumerate() {
                if let Some(captures) = regex.captures(title) {
                    if let Some(num_match) = captures.get(2) {
                        if let Ok(num) = num_match.as_str().parse::<i32>() {
                            matched_indices.push(i);
                            numbers.push(num);
                        }
                    }
                }
            }

            if matched_indices.len() >= 2 {
                let confidence =
                    calculate_sequence_confidence(&numbers, matched_indices.len(), titles.len());

                if confidence > 0.6 {
                    return Some(SequentialPattern {
                        pattern_type: SequentialPatternType::ModuleProgression,
                        confidence,
                        matched_indices: matched_indices.clone(),
                        pattern_description: format!(
                            "Module progression detected: {} modules in sequence",
                            matched_indices.len()
                        ),
                    });
                }
            }
        }
    }

    None
}

/// Detect step-by-step tutorial patterns
fn detect_step_by_step_pattern(titles: &[String]) -> Option<SequentialPattern> {
    let step_patterns = [
        r"(?i)\b(step|tutorial|guide)\s*(\d+)",
        r"(?i)\bhow\s*to\s*.*\s*(\d+)",
        r"(?i)^(\d+)\s*[-.:]\s*(step|tutorial|guide)",
    ];

    for pattern_str in &step_patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            let mut matched_indices = Vec::new();

            for (i, title) in titles.iter().enumerate() {
                if regex.is_match(title) {
                    matched_indices.push(i);
                }
            }

            if matched_indices.len() >= 3 {
                let confidence = matched_indices.len() as f32 / titles.len() as f32;

                if confidence > 0.4 {
                    let indices_len = matched_indices.len();
                    return Some(SequentialPattern {
                        pattern_type: SequentialPatternType::StepByStep,
                        confidence,
                        matched_indices,
                        pattern_description: format!(
                            "Step-by-step pattern detected: {} instructional titles",
                            indices_len
                        ),
                    });
                }
            }
        }
    }

    None
}

/// Detect chronological ordering patterns
fn detect_chronological_pattern(titles: &[String]) -> Option<SequentialPattern> {
    let chrono_patterns = [
        r"(?i)\b(first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth)",
        r"(?i)\b(beginning|start|introduction|basics|fundamentals)",
        r"(?i)\b(final|conclusion|summary|wrap.?up|ending)",
    ];

    let mut matched_indices = Vec::new();
    let mut pattern_matches = 0;

    for pattern_str in &chrono_patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            for (i, title) in titles.iter().enumerate() {
                if regex.is_match(title) && !matched_indices.contains(&i) {
                    matched_indices.push(i);
                    pattern_matches += 1;
                }
            }
        }
    }

    if pattern_matches >= 2 {
        let confidence = pattern_matches as f32 / titles.len() as f32;

        if confidence > 0.3 {
            return Some(SequentialPattern {
                pattern_type: SequentialPatternType::ChronologicalOrder,
                confidence,
                matched_indices,
                pattern_description: format!(
                    "Chronological ordering detected: {} temporal indicators",
                    pattern_matches
                ),
            });
        }
    }

    None
}

/// Calculate confidence score for numeric sequences
fn calculate_sequence_confidence(numbers: &[i32], matches: usize, total: usize) -> f32 {
    if numbers.is_empty() {
        return 0.0;
    }

    // Base confidence from match ratio
    let match_ratio = matches as f32 / total as f32;

    // Check for sequential ordering
    let mut sorted_numbers = numbers.to_vec();
    sorted_numbers.sort_unstable();

    let mut sequential_count = 0;
    for i in 1..sorted_numbers.len() {
        if sorted_numbers[i] == sorted_numbers[i - 1] + 1 {
            sequential_count += 1;
        }
    }

    let sequence_ratio = if numbers.len() > 1 {
        sequential_count as f32 / (numbers.len() - 1) as f32
    } else {
        0.0
    };

    // Combine match ratio and sequence quality
    (match_ratio * 0.6 + sequence_ratio * 0.4).min(1.0)
}

/// Calculate confidence score for alphabetic sequences
fn calculate_alphabetic_sequence_confidence(letters: &[char], matches: usize, total: usize) -> f32 {
    if letters.is_empty() {
        return 0.0;
    }

    let match_ratio = matches as f32 / total as f32;

    // Check for alphabetic progression
    let mut sorted_letters = letters.to_vec();
    sorted_letters.sort_unstable();

    let mut sequential_count = 0;
    for i in 1..sorted_letters.len() {
        if (sorted_letters[i] as u8) == (sorted_letters[i - 1] as u8) + 1 {
            sequential_count += 1;
        }
    }

    let sequence_ratio = if letters.len() > 1 {
        sequential_count as f32 / (letters.len() - 1) as f32
    } else {
        0.0
    };

    (match_ratio * 0.6 + sequence_ratio * 0.4).min(1.0)
}

/// Detect module indicators in titles
fn detect_module_indicators(titles: &[String]) -> Vec<ModuleIndicator> {
    let mut indicators = Vec::new();

    let explicit_patterns = [
        r"(?i)^(module|unit|chapter|section|part)\s*\d*\s*[-:]?\s*(.*)",
        r"(?i)\b(introduction\s*to|overview\s*of|getting\s*started\s*with)\s*(.*)",
        r"(?i)^(.*)\s*[-:]\s*(introduction|overview|basics|fundamentals)",
    ];

    for (i, title) in titles.iter().enumerate() {
        for pattern_str in &explicit_patterns {
            if let Ok(regex) = Regex::new(pattern_str) {
                if let Some(captures) = regex.captures(title) {
                    let indicator_type = if captures.get(1).map_or(false, |m| {
                        let text = m.as_str().to_lowercase();
                        text.contains("module") || text.contains("unit") || text.contains("chapter")
                    }) {
                        ModuleIndicatorType::ExplicitModule
                    } else if captures.get(1).map_or(false, |m| {
                        let text = m.as_str().to_lowercase();
                        text.contains("introduction") || text.contains("overview")
                    }) {
                        ModuleIndicatorType::SectionBreak
                    } else {
                        ModuleIndicatorType::TopicTransition
                    };

                    indicators.push(ModuleIndicator {
                        index: i,
                        title: title.clone(),
                        indicator_type,
                        confidence: 0.8, // High confidence for explicit patterns
                    });
                    break;
                }
            }
        }
    }

    indicators
}

/// Analyze naming consistency across titles
fn analyze_naming_consistency(titles: &[String]) -> NamingConsistency {
    let mut pattern_counts = HashMap::new();
    let mut total_patterns = 0;

    // Common educational content patterns
    let consistency_patterns = [
        r"(?i)^(lesson|part|episode|chapter|section|tutorial|video)\s*\d+",
        r"(?i)^(module|unit|course)\s*\d+",
        r"(?i)^\d+\s*[-.:]\s*",
        r"(?i)\b(step|tutorial|guide)\s*\d+",
    ];

    for pattern_str in &consistency_patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            let matches = titles.iter().filter(|title| regex.is_match(title)).count();
            if matches > 0 {
                pattern_counts.insert(pattern_str.to_string(), matches);
                total_patterns += matches;
            }
        }
    }

    let consistency_score = if titles.is_empty() {
        0.0
    } else {
        total_patterns as f32 / titles.len() as f32
    };

    let common_patterns: Vec<String> = pattern_counts
        .iter()
        .filter(|(_, count)| **count >= 2)
        .map(|(pattern, _)| pattern.clone())
        .collect();

    let naming_variations = pattern_counts.len();
    let has_consistent_format = consistency_score > 0.6 && naming_variations <= 2;

    NamingConsistency {
        consistency_score,
        common_patterns,
        naming_variations,
        has_consistent_format,
    }
}

/// Determine overall content type based on all analysis results
fn determine_content_type(
    sequential_patterns: &[SequentialPattern],
    module_indicators: &[ModuleIndicator],
    naming_consistency: &NamingConsistency,
    total_titles: usize,
) -> (ContentType, f32) {
    let mut sequential_score = 0.0;
    let mut thematic_score = 0.0;

    // Score from sequential patterns
    for pattern in sequential_patterns {
        sequential_score += pattern.confidence * 0.4;
    }

    // Score from module indicators
    let module_ratio = module_indicators.len() as f32 / total_titles as f32;
    if module_ratio > 0.3 {
        // Many module indicators suggest thematic organization
        thematic_score += module_ratio * 0.3;
    } else if module_ratio > 0.0 {
        // Few module indicators might indicate sequential with breaks
        sequential_score += module_ratio * 0.2;
    }

    // Score from naming consistency
    if naming_consistency.has_consistent_format {
        sequential_score += naming_consistency.consistency_score * 0.3;
    } else if naming_consistency.naming_variations > 3 {
        // High variation suggests thematic content
        thematic_score += 0.2;
    }

    // Normalize scores
    sequential_score = sequential_score.min(1.0);
    thematic_score = thematic_score.min(1.0);

    // Determine content type and confidence
    let confidence_threshold = 0.6;

    if sequential_score > confidence_threshold && sequential_score > thematic_score {
        (ContentType::Sequential, sequential_score)
    } else if thematic_score > confidence_threshold && thematic_score > sequential_score {
        (ContentType::Thematic, thematic_score)
    } else if (sequential_score - thematic_score).abs() < 0.2 && sequential_score > 0.3 {
        (
            ContentType::Mixed,
            (sequential_score + thematic_score) / 2.0,
        )
    } else {
        (
            ContentType::Ambiguous,
            (sequential_score + thematic_score) / 2.0,
        )
    }
}

/// Generate processing recommendation based on content analysis
fn generate_processing_recommendation(
    content_type: &ContentType,
    confidence_score: f32,
    sequential_patterns: &[SequentialPattern],
) -> ProcessingRecommendation {
    match content_type {
        ContentType::Sequential if confidence_score > 0.7 => {
            ProcessingRecommendation::PreserveOrder
        }
        ContentType::Sequential if confidence_score > 0.5 => {
            // Check if we have strong sequential patterns
            if sequential_patterns.iter().any(|p| p.confidence > 0.8) {
                ProcessingRecommendation::PreserveOrder
            } else {
                ProcessingRecommendation::UserChoice
            }
        }
        ContentType::Thematic if confidence_score > 0.6 => {
            ProcessingRecommendation::ApplyClustering
        }
        ContentType::Mixed => ProcessingRecommendation::UserChoice,
        ContentType::Ambiguous if confidence_score < 0.3 => {
            ProcessingRecommendation::FallbackProcessing
        }
        _ => ProcessingRecommendation::UserChoice,
    }
}
