//! NLP module for Course Pilot
//!


use crate::types::{Course, CourseStructure, Module, Section, StructureMetadata};
use std::time::Duration;

// Re-export error types
pub use crate::NlpError;


pub fn structure_course(course: &Course) -> Result<CourseStructure, NlpError> {
    if course.raw_titles.is_empty() && course.videos.is_empty() {
        return Err(NlpError::InvalidInput("No videos in course".to_string()));
    }

    // Use raw_titles if available, otherwise fall back to video titles
    let titles: Vec<String> = if !course.raw_titles.is_empty() {
        course.raw_titles.clone()
    } else {
        course.videos.iter().map(|v| v.title.clone()).collect()
    };

    let group_size = 10;
    let groups: Vec<Vec<usize>> = titles
        .iter()
        .enumerate()
        .collect::<Vec<_>>()
        .chunks(group_size)
        .map(|chunk| chunk.iter().map(|(i, _)| *i).collect())
        .collect();

    // Convert groups to modules
    let modules: Vec<Module> = groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let sections: Vec<Section> = group
                .iter()
                .map(|&idx| {
                    let title =
                        titles.get(idx).cloned().unwrap_or_else(|| format!("Video {}", idx + 1));
                    let duration = course
                        .get_video_metadata(idx)
                        .and_then(|v| v.duration_seconds)
                        .map(|s| Duration::from_secs_f64(s.max(0.0)))
                        .unwrap_or_default();
                    Section { title, video_index: idx, duration }
                })
                .collect();
            Module::new_basic(format!("Session {}", i + 1), sections)
        })
        .collect();

    let total_duration: Duration =
        modules.iter().flat_map(|m| &m.sections).map(|s| s.duration).sum();

    let metadata = StructureMetadata {
        total_videos: titles.len(),
        total_duration,
        estimated_duration_hours: if total_duration.as_secs() > 0 {
            Some(total_duration.as_secs_f32() / 3600.0)
        } else {
            None
        },
        difficulty_level: None,
        structure_quality_score: None,
        content_coherence_score: None,
        content_type_detected: Some("Sequential".to_string()),
        original_order_preserved: Some(true),
        processing_strategy_used: Some("Grouping".to_string()),
    };

    Ok(CourseStructure::new_basic(modules, metadata))
}

/// Calculate similarity between two text strings (Jaccard similarity)
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
    fn test_text_similarity() {
        assert_eq!(text_similarity("hello world", "hello world"), 1.0);
        assert_eq!(text_similarity("hello", "world"), 0.0);
        assert!(text_similarity("hello world", "hello there") > 0.0);
    }
}
