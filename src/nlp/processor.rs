//! Course structure analysis processor
//!
//! This module implements course structure analysis using a rule-based approach
//! to convert raw video titles into organized course hierarchies.

use crate::NlpError;
use crate::nlp::{extract_numbers, is_module_indicator, normalize_text};
use crate::types::{CourseStructure, Module, Section, StructureMetadata};
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration;

/// Structure a course from raw video titles
///
/// # Arguments
/// * `titles` - Vector of raw video titles to analyze
///
/// # Returns
/// * `Ok(CourseStructure)` - Structured course with modules and sections
/// * `Err(NlpError)` - Error if structuring fails
pub fn structure_course(titles: Vec<String>) -> Result<CourseStructure, NlpError> {
    if titles.is_empty() {
        return Err(NlpError::InvalidInput("No titles provided".to_string()));
    }

    // Step 1: Analyze titles to identify structure patterns
    let analysis = analyze_title_patterns(&titles)?;

    // Step 2: Choose the best structuring strategy based on analysis
    let strategy = choose_structuring_strategy(&analysis);

    // Step 3: Apply the chosen strategy to create structure
    let modules = match strategy {
        StructuringStrategy::Hierarchical => create_hierarchical_structure(&titles, &analysis)?,
        StructuringStrategy::Sequential => create_sequential_structure(&titles, &analysis)?,
        StructuringStrategy::Thematic => create_thematic_structure(&titles, &analysis)?,
        StructuringStrategy::Fallback => create_fallback_structure(&titles)?,
    };

    // Step 4: Generate metadata
    let metadata = generate_metadata(&titles, &modules);

    Ok(CourseStructure { modules, metadata })
}

/// Analysis results for title patterns
#[derive(Debug)]
#[allow(dead_code)]
struct TitleAnalysis {
    has_numeric_sequence: bool,
    has_explicit_modules: bool,
    has_consistent_naming: bool,
    module_boundaries: Vec<usize>,
    estimated_difficulty: DifficultyLevel,
}

/// Different difficulty levels for courses
#[derive(Debug, Clone)]
enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Mixed,
}

/// Available structuring strategies
#[derive(Debug)]
enum StructuringStrategy {
    Hierarchical, // Clear module/section hierarchy
    Sequential,   // Linear progression with grouping
    Thematic,     // Topic-based grouping
    Fallback,     // Simple chunking when no pattern is clear
}

/// Analyze title patterns to understand course structure
fn analyze_title_patterns(titles: &[String]) -> Result<TitleAnalysis, NlpError> {
    let has_numeric_sequence;
    let has_explicit_modules;
    let has_consistent_naming;
    let mut module_boundaries = Vec::new();

    // Check for numeric sequences
    let mut numeric_titles = 0;
    for title in titles {
        let numbers = extract_numbers(title);
        if !numbers.is_empty() {
            numeric_titles += 1;
        }
    }
    has_numeric_sequence = numeric_titles > titles.len() / 2;

    // Check for explicit module indicators
    let mut module_indicators = 0;
    for (i, title) in titles.iter().enumerate() {
        if is_module_indicator(title) {
            module_indicators += 1;
            module_boundaries.push(i);
        }
    }
    has_explicit_modules = module_indicators > 0;

    // Check for consistent naming patterns
    let patterns = find_naming_patterns(titles);
    has_consistent_naming = patterns.len() > 1 && patterns.values().any(|&count| count > 2);

    // Estimate difficulty based on vocabulary complexity
    let estimated_difficulty = estimate_difficulty(titles);

    Ok(TitleAnalysis {
        has_numeric_sequence,
        has_explicit_modules,
        has_consistent_naming,
        module_boundaries,
        estimated_difficulty,
    })
}

/// Find common naming patterns in titles
fn find_naming_patterns(titles: &[String]) -> HashMap<String, usize> {
    let mut patterns = HashMap::new();
    let pattern_regex = Regex::new(r"^([a-zA-Z\s]+)\s*\d+").unwrap();

    for title in titles {
        if let Some(captures) = pattern_regex.captures(title) {
            if let Some(pattern) = captures.get(1) {
                let normalized_pattern = normalize_text(pattern.as_str());
                *patterns.entry(normalized_pattern).or_insert(0) += 1;
            }
        }
    }

    patterns
}

/// Estimate the difficulty level of the course
fn estimate_difficulty(titles: &[String]) -> DifficultyLevel {
    let beginner_keywords = [
        "introduction",
        "basics",
        "fundamentals",
        "getting started",
        "beginner",
    ];
    let advanced_keywords = [
        "advanced",
        "expert",
        "master",
        "deep dive",
        "optimization",
        "architecture",
    ];

    let mut beginner_count = 0;
    let mut advanced_count = 0;

    for title in titles {
        let title_lower = title.to_lowercase();

        for keyword in &beginner_keywords {
            if title_lower.contains(keyword) {
                beginner_count += 1;
                break;
            }
        }

        for keyword in &advanced_keywords {
            if title_lower.contains(keyword) {
                advanced_count += 1;
                break;
            }
        }
    }

    match (beginner_count, advanced_count) {
        (b, a) if b > a && b > titles.len() / 4 => DifficultyLevel::Beginner,
        (b, a) if a > b && a > titles.len() / 4 => DifficultyLevel::Advanced,
        (b, a) if b > 0 && a > 0 => DifficultyLevel::Mixed,
        _ => DifficultyLevel::Intermediate,
    }
}

/// Choose the best structuring strategy based on analysis
fn choose_structuring_strategy(analysis: &TitleAnalysis) -> StructuringStrategy {
    if analysis.has_explicit_modules && analysis.module_boundaries.len() > 1 {
        StructuringStrategy::Hierarchical
    } else if analysis.has_numeric_sequence && analysis.has_consistent_naming {
        StructuringStrategy::Sequential
    } else if analysis.has_consistent_naming {
        StructuringStrategy::Thematic
    } else {
        StructuringStrategy::Fallback
    }
}

/// Create hierarchical structure based on explicit module indicators
fn create_hierarchical_structure(
    titles: &[String],
    _analysis: &TitleAnalysis,
) -> Result<Vec<Module>, NlpError> {
    let mut modules = Vec::new();
    let mut current_sections = Vec::new();
    let mut current_module_title = "Introduction".to_string();

    for (i, title) in titles.iter().enumerate() {
        if is_module_indicator(title) && !current_sections.is_empty() {
            // Save previous module
            modules.push(Module {
                title: current_module_title.clone(),
                sections: std::mem::take(&mut current_sections),
                total_duration: modules.last().map_or(Duration::from_secs(0), |m: &Module| {
                    m.sections.iter().map(|s| s.duration).sum::<Duration>()
                }),
            });
            current_module_title = extract_module_title(title);
        } else if is_module_indicator(title) {
            current_module_title = extract_module_title(title);
        }

        // Add current title as a section
        current_sections.push(Section {
            title: title.clone(),
            video_index: i,
            duration: estimate_video_duration(title)
                .unwrap_or_else(|| std::time::Duration::from_secs(0)),
        });
    }

    // Add the last module
    if !current_sections.is_empty() {
        modules.push(Module {
            title: current_module_title,
            sections: current_sections.clone(),
            total_duration: current_sections.iter().map(|s| s.duration).sum(),
        });
    }

    Ok(modules)
}

/// Create sequential structure with natural grouping
fn create_sequential_structure(
    titles: &[String],
    _analysis: &TitleAnalysis,
) -> Result<Vec<Module>, NlpError> {
    let chunk_size = calculate_optimal_chunk_size(titles.len());
    let mut modules = Vec::new();

    for (module_index, chunk) in titles.chunks(chunk_size).enumerate() {
        let module_title = generate_sequential_module_title(module_index + 1, chunk);
        let sections: Vec<Section> = chunk
            .iter()
            .enumerate()
            .map(|(section_index, title)| Section {
                title: title.clone(),
                video_index: module_index * chunk_size + section_index,
                duration: estimate_video_duration(title)
                    .unwrap_or_else(|| std::time::Duration::from_secs(0)),
            })
            .collect();

        modules.push(Module {
            title: module_title,
            sections: sections.clone(),
            total_duration: sections.iter().map(|s| s.duration).sum(),
        });
    }

    Ok(modules)
}

/// Create thematic structure based on content similarity
fn create_thematic_structure(
    titles: &[String],
    _analysis: &TitleAnalysis,
) -> Result<Vec<Module>, NlpError> {
    let themes = identify_themes(titles)?;
    let mut modules = Vec::new();

    for (theme_name, video_indices) in themes {
        let sections: Vec<Section> = video_indices
            .into_iter()
            .map(|index| Section {
                title: titles[index].clone(),
                video_index: index,
                duration: estimate_video_duration(&titles[index])
                    .unwrap_or_else(|| std::time::Duration::from_secs(0)),
            })
            .collect();

        modules.push(Module {
            title: theme_name,
            sections: sections.clone(),
            total_duration: sections.iter().map(|s| s.duration).sum(),
        });
    }

    Ok(modules)
}

/// Create fallback structure with simple chunking
fn create_fallback_structure(titles: &[String]) -> Result<Vec<Module>, NlpError> {
    let chunk_size = 8; // Default chunk size for fallback
    let mut modules = Vec::new();

    for (module_index, chunk) in titles.chunks(chunk_size).enumerate() {
        let module_title = format!("Part {}", module_index + 1);
        let sections: Vec<Section> = chunk
            .iter()
            .enumerate()
            .map(|(section_index, title)| Section {
                title: title.clone(),
                video_index: module_index * chunk_size + section_index,
                duration: estimate_video_duration(title)
                    .unwrap_or_else(|| std::time::Duration::from_secs(0)),
            })
            .collect();

        modules.push(Module {
            title: module_title,
            sections: sections.clone(),
            total_duration: sections.iter().map(|s| s.duration).sum(),
        });
    }

    Ok(modules)
}

/// Extract a clean module title from a title with module indicators
fn extract_module_title(title: &str) -> String {
    let title_clean = title
        .split(':')
        .next()
        .unwrap_or(title)
        .split('-')
        .next()
        .unwrap_or(title)
        .trim();

    if title_clean.is_empty() {
        "Untitled Module".to_string()
    } else {
        title_clean.to_string()
    }
}

/// Calculate optimal chunk size for sequential structuring
fn calculate_optimal_chunk_size(total_videos: usize) -> usize {
    match total_videos {
        1..=20 => std::cmp::max(total_videos / 3, 1),
        21..=50 => total_videos / 5,
        51..=100 => total_videos / 7,
        _ => total_videos / 10,
    }
}

/// Generate a module title for sequential structure
fn generate_sequential_module_title(module_number: usize, sections: &[String]) -> String {
    // Try to extract common theme from section titles
    let first_title = &sections[0];
    let words: Vec<&str> = first_title.split_whitespace().collect();

    if words.len() > 1 {
        let theme = words[0..std::cmp::min(2, words.len())].join(" ");
        format!("Module {}: {}", module_number, theme)
    } else {
        format!("Module {}", module_number)
    }
}

/// Identify themes in titles using clustering
fn identify_themes(titles: &[String]) -> Result<Vec<(String, Vec<usize>)>, NlpError> {
    let mut themes = Vec::new();
    let mut used_indices = std::collections::HashSet::new();

    // Simple keyword-based clustering
    let keywords = extract_common_keywords(titles);

    for keyword in keywords {
        let mut theme_indices = Vec::new();

        for (i, title) in titles.iter().enumerate() {
            if !used_indices.contains(&i) && title.to_lowercase().contains(&keyword) {
                theme_indices.push(i);
                used_indices.insert(i);
            }
        }

        if theme_indices.len() > 1 {
            let theme_name = keyword
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + &keyword[1..])
                .unwrap_or_else(|| keyword.clone());

            themes.push((theme_name, theme_indices));
        }
    }

    // Handle remaining uncategorized titles
    let remaining_indices: Vec<usize> = (0..titles.len())
        .filter(|i| !used_indices.contains(i))
        .collect();

    if !remaining_indices.is_empty() {
        themes.push(("Miscellaneous".to_string(), remaining_indices));
    }

    // If no themes found, create single theme
    if themes.is_empty() {
        themes.push(("Course Content".to_string(), (0..titles.len()).collect()));
    }

    Ok(themes)
}

/// Extract common keywords from titles
fn extract_common_keywords(titles: &[String]) -> Vec<String> {
    let mut word_counts = HashMap::new();

    for title in titles {
        for word in normalize_text(title).split_whitespace() {
            if word.len() > 3 {
                // Only consider words longer than 3 characters
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut keywords: Vec<_> = word_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect();

    keywords.sort_by(|a, b| b.1.cmp(&a.1));
    keywords.into_iter().map(|(word, _)| word).take(5).collect()
}

/// Estimate video duration based on title content
fn estimate_video_duration(title: &str) -> Option<Duration> {
    // Simple heuristic based on title length and keywords
    let _base_duration = Duration::from_secs(600); // 10 minutes default

    let duration_minutes = if title.to_lowercase().contains("introduction") {
        5 // Shorter for introductions
    } else if title.to_lowercase().contains("project") || title.to_lowercase().contains("exercise")
    {
        20 // Longer for practical work
    } else {
        10 // Default
    };

    Some(Duration::from_secs(duration_minutes * 60))
}

/// Generate metadata for the course structure
fn generate_metadata(titles: &[String], modules: &[Module]) -> StructureMetadata {
    let total_videos = titles.len();

    let estimated_duration_hours = modules
        .iter()
        .flat_map(|m| &m.sections)
        .map(|s| s.duration)
        .map(|d| d.as_secs_f32() / 3600.0)
        .sum::<f32>();

    let difficulty_level = match estimate_difficulty(titles) {
        DifficultyLevel::Beginner => Some("Beginner".to_string()),
        DifficultyLevel::Intermediate => Some("Intermediate".to_string()),
        DifficultyLevel::Advanced => Some("Advanced".to_string()),
        DifficultyLevel::Mixed => Some("Mixed".to_string()),
    };

    StructureMetadata {
        total_videos,
        total_duration: modules.iter().map(|m| m.total_duration).sum(),
        estimated_duration_hours: Some(estimated_duration_hours),
        difficulty_level,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_course_basic() {
        let titles = vec![
            "Introduction to Programming".to_string(),
            "Variables and Data Types".to_string(),
            "Control Structures".to_string(),
            "Functions".to_string(),
        ];

        let result = structure_course(titles).unwrap();
        assert!(!result.modules.is_empty());
        assert_eq!(result.metadata.total_videos, 4);
    }

    #[test]
    fn test_structure_course_with_modules() {
        let titles = vec![
            "Module 1: Introduction".to_string(),
            "Lesson 1: Getting Started".to_string(),
            "Lesson 2: Basic Concepts".to_string(),
            "Module 2: Advanced Topics".to_string(),
            "Lesson 3: Complex Examples".to_string(),
        ];

        let result = structure_course(titles).unwrap();
        assert_eq!(result.modules.len(), 2);
    }

    #[test]
    fn test_empty_titles() {
        let result = structure_course(vec![]);
        assert!(matches!(result, Err(NlpError::InvalidInput(_))));
    }

    #[test]
    fn test_difficulty_estimation() {
        let beginner_titles = vec!["Introduction to Basics".to_string()];
        let advanced_titles = vec!["Advanced Optimization Techniques".to_string()];

        assert!(matches!(
            estimate_difficulty(&beginner_titles),
            DifficultyLevel::Beginner
        ));
        assert!(matches!(
            estimate_difficulty(&advanced_titles),
            DifficultyLevel::Advanced
        ));
    }

    #[test]
    fn test_chunk_size_calculation() {
        assert_eq!(calculate_optimal_chunk_size(10), 3);
        assert_eq!(calculate_optimal_chunk_size(30), 6);
        assert_eq!(calculate_optimal_chunk_size(100), 14);
    }
}