//! Topic extraction utilities for clustering analysis
//!
//! This module provides functionality to extract and analyze topics from video titles
//! to support clustering decisions and generate meaningful cluster names.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Topic information extracted from content analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub keyword: String,
    pub relevance_score: f32,
    pub frequency: usize,
    pub related_videos: Vec<usize>,
}

/// Topic extractor for identifying themes in video content
pub struct TopicExtractor {
    min_frequency: usize,
    min_relevance: f32,
}

impl Default for TopicExtractor {
    fn default() -> Self {
        Self { min_frequency: 2, min_relevance: 0.1 }
    }
}

impl TopicExtractor {
    /// Create a new topic extractor with custom parameters
    pub fn new(min_frequency: usize, min_relevance: f32) -> Self {
        Self { min_frequency, min_relevance }
    }

    /// Extract topics from TF-IDF analysis results
    pub fn extract_topics(
        &self,
        titles: &[String],
        tfidf_scores: &HashMap<String, f32>,
    ) -> Vec<TopicInfo> {
        let mut topics = Vec::new();

        for (keyword, &relevance_score) in tfidf_scores {
            if relevance_score < self.min_relevance {
                continue;
            }

            let related_videos = self.find_videos_with_keyword(titles, keyword);

            if related_videos.len() >= self.min_frequency {
                topics.push(TopicInfo {
                    keyword: keyword.clone(),
                    relevance_score,
                    frequency: related_videos.len(),
                    related_videos,
                });
            }
        }

        // Sort by relevance score descending
        topics.sort_by(|a, b| {
            b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        topics
    }

    /// Find video indices that contain a specific keyword
    fn find_videos_with_keyword(&self, titles: &[String], keyword: &str) -> Vec<usize> {
        titles
            .iter()
            .enumerate()
            .filter(|(_, title)| title.to_lowercase().contains(&keyword.to_lowercase()))
            .map(|(index, _)| index)
            .collect()
    }

    /// Generate cluster title from topic keywords
    pub fn generate_cluster_title(&self, topic_keywords: &[String]) -> String {
        if topic_keywords.is_empty() {
            return "Miscellaneous".to_string();
        }

        // Take the most relevant keyword and capitalize it
        let primary_keyword = &topic_keywords[0];
        let capitalized = primary_keyword
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>() + &primary_keyword[1..])
            .unwrap_or_else(|| primary_keyword.clone());

        // If we have multiple keywords, create a compound title
        if topic_keywords.len() > 1 {
            format!("{} and {}", capitalized, topic_keywords[1])
        } else {
            capitalized
        }
    }

    /// Analyze topic distribution across clusters
    pub fn analyze_topic_distribution(&self, topics: &[TopicInfo]) -> TopicDistributionAnalysis {
        let total_videos: usize = topics.iter().map(|t| t.related_videos.len()).sum();
        let unique_videos: std::collections::HashSet<usize> =
            topics.iter().flat_map(|t| &t.related_videos).copied().collect();

        let coverage =
            if total_videos > 0 { unique_videos.len() as f32 / total_videos as f32 } else { 0.0 };

        let average_relevance = if !topics.is_empty() {
            topics.iter().map(|t| t.relevance_score).sum::<f32>() / topics.len() as f32
        } else {
            0.0
        };

        TopicDistributionAnalysis {
            total_topics: topics.len(),
            total_video_coverage: unique_videos.len(),
            coverage_percentage: coverage * 100.0,
            average_relevance_score: average_relevance,
            most_relevant_topic: topics.first().map(|t| t.keyword.clone()),
        }
    }
}

/// Analysis results for topic distribution
#[derive(Debug, Clone)]
pub struct TopicDistributionAnalysis {
    pub total_topics: usize,
    pub total_video_coverage: usize,
    pub coverage_percentage: f32,
    pub average_relevance_score: f32,
    pub most_relevant_topic: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_extraction() {
        let extractor = TopicExtractor::default();
        let titles = vec![
            "Introduction to Programming".to_string(),
            "Advanced Programming Concepts".to_string(),
            "Programming Best Practices".to_string(),
            "Database Design".to_string(),
            "Web Development".to_string(),
        ];

        let mut tfidf_scores = HashMap::new();
        tfidf_scores.insert("programming".to_string(), 0.8);
        tfidf_scores.insert("database".to_string(), 0.3);
        tfidf_scores.insert("web".to_string(), 0.2);

        let topics = extractor.extract_topics(&titles, &tfidf_scores);

        assert!(!topics.is_empty());
        assert_eq!(topics[0].keyword, "programming");
        assert_eq!(topics[0].frequency, 3);
    }

    #[test]
    fn test_cluster_title_generation() {
        let extractor = TopicExtractor::default();

        let title1 = extractor.generate_cluster_title(&["programming".to_string()]);
        assert_eq!(title1, "Programming");

        let title2 =
            extractor.generate_cluster_title(&["web".to_string(), "development".to_string()]);
        assert_eq!(title2, "Web and development");

        let title3 = extractor.generate_cluster_title(&[]);
        assert_eq!(title3, "Miscellaneous");
    }

    #[test]
    fn test_topic_distribution_analysis() {
        let extractor = TopicExtractor::default();
        let topics = vec![
            TopicInfo {
                keyword: "programming".to_string(),
                relevance_score: 0.8,
                frequency: 3,
                related_videos: vec![0, 1, 2],
            },
            TopicInfo {
                keyword: "database".to_string(),
                relevance_score: 0.3,
                frequency: 1,
                related_videos: vec![3],
            },
        ];

        let analysis = extractor.analyze_topic_distribution(&topics);
        assert_eq!(analysis.total_topics, 2);
        assert_eq!(analysis.total_video_coverage, 4);
        assert_eq!(analysis.most_relevant_topic, Some("programming".to_string()));
    }
}
