use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::{Hash, Hasher};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub raw_titles: Vec<String>,
    pub videos: Vec<VideoMetadata>,
    pub structure: Option<CourseStructure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoMetadata {
    pub title: String,
    pub source_url: Option<String>,
    pub video_id: Option<String>,
    pub playlist_id: Option<String>,
    pub original_index: usize,
    pub duration_seconds: Option<f64>,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
    pub upload_date: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub view_count: Option<u64>,
    pub tags: Vec<String>,
    pub is_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseStructure {
    pub modules: Vec<Module>,
    pub metadata: StructureMetadata,
    pub clustering_metadata: Option<ClusteringMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringMetadata {
    pub algorithm_used: ClusteringAlgorithm,
    pub similarity_threshold: f32,
    pub cluster_count: usize,
    pub quality_score: f32,
    pub processing_time_ms: u64,
    pub content_topics: Vec<TopicInfo>,
    pub strategy_used: ClusteringStrategy,
    pub confidence_scores: ClusteringConfidenceScores,
    pub rationale: ClusteringRationale,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringConfidenceScores {
    pub overall_confidence: f32,
    pub module_grouping_confidence: f32,
    pub similarity_confidence: f32,
    pub topic_extraction_confidence: f32,
    pub module_confidences: Vec<ModuleConfidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfidence {
    pub module_index: usize,
    pub confidence_score: f32,
    pub similarity_strength: f32,
    pub topic_coherence: f32,
    pub duration_balance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringRationale {
    pub primary_strategy: String,
    pub explanation: String,
    pub key_factors: Vec<String>,
    pub alternatives_considered: Vec<String>,
    pub module_rationales: Vec<ModuleRationale>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleRationale {
    pub module_index: usize,
    pub module_title: String,
    pub grouping_reason: String,
    pub similarity_explanation: String,
    pub topic_keywords: Vec<String>,
    pub video_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PerformanceMetrics {
    pub total_processing_time_ms: u64,
    pub content_analysis_time_ms: u64,
    pub clustering_time_ms: u64,
    pub optimization_time_ms: u64,
    pub peak_memory_usage_bytes: u64,
    pub algorithm_iterations: u32,
    pub input_metrics: InputMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InputMetrics {
    pub video_count: usize,
    pub unique_words: usize,
    pub vocabulary_size: usize,
    pub average_title_length: f32,
    pub content_diversity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ClusteringAlgorithm {
    #[default]
    TfIdf,
    KMeans,
    Hierarchical,
    Lda,
    Hybrid,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ClusteringStrategy {
    ContentBased,
    DurationBased,
    Hierarchical,
    Lda,
    #[default]
    Hybrid,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopicInfo {
    pub keyword: String,
    pub relevance_score: f32,
    pub video_count: usize,
}

impl Eq for TopicInfo {}

impl Hash for TopicInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.keyword.hash(state);
        self.relevance_score.to_bits().hash(state);
        self.video_count.hash(state);
    }
}

impl CourseStructure {
    pub fn aggregate_total_duration(&self) -> Duration {
        self.modules.iter().map(|m| m.total_duration).sum()
    }

    pub fn with_aggregated_metadata(mut self) -> Self {
        let total_videos = self.modules.iter().map(|m| m.sections.len()).sum();
        let total_duration = self.aggregate_total_duration();
        self.metadata.total_videos = total_videos;
        self.metadata.total_duration = total_duration;
        self
    }

    pub fn new_basic(modules: Vec<Module>, metadata: StructureMetadata) -> Self {
        Self { modules, metadata, clustering_metadata: None }
    }

    pub fn new_with_clustering(
        modules: Vec<Module>,
        metadata: StructureMetadata,
        clustering_metadata: ClusteringMetadata,
    ) -> Self {
        Self { modules, metadata, clustering_metadata: Some(clustering_metadata) }
    }

    pub fn is_clustered(&self) -> bool {
        self.clustering_metadata.is_some()
    }

    pub fn get_content_organization_type(&self) -> String {
        if let Some(content_type) = &self.metadata.content_type_detected {
            return content_type.clone();
        }

        if self.clustering_metadata.is_some() {
            "Clustered".to_string()
        } else if self.metadata.original_order_preserved.unwrap_or(false) {
            "Sequential".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    pub fn get_content_organization_description(&self) -> String {
        match self.get_content_organization_type().as_str() {
            "Sequential" => "Content follows original order with preserved progression".to_string(),
            "Clustered" => "Content organized by topics using intelligent clustering".to_string(),
            "Mixed" => "Content contains both sequential and thematic elements".to_string(),
            "Ambiguous" => "Content organization could not be clearly determined".to_string(),
            _ => "Content organization type unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructureMetadata {
    pub total_videos: usize,
    pub total_duration: Duration,
    pub estimated_duration_hours: Option<f32>,
    pub difficulty_level: Option<String>,
    pub structure_quality_score: Option<f32>,
    pub content_coherence_score: Option<f32>,
    pub content_type_detected: Option<String>,
    pub original_order_preserved: Option<bool>,
    pub processing_strategy_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Module {
    pub title: String,
    pub sections: Vec<Section>,
    pub total_duration: Duration,
    pub similarity_score: Option<f32>,
    pub topic_keywords: Vec<String>,
    pub difficulty_level: Option<DifficultyLevel>,
}

impl Module {
    pub fn aggregate_total_duration(&self) -> Duration {
        self.sections.iter().map(|s| s.duration).sum()
    }

    pub fn new_basic(title: String, sections: Vec<Section>) -> Self {
        let total_duration = sections.iter().map(|s| s.duration).sum();
        Self {
            title,
            sections,
            total_duration,
            similarity_score: None,
            topic_keywords: Vec::new(),
            difficulty_level: None,
        }
    }

    pub fn new_with_clustering(
        title: String,
        sections: Vec<Section>,
        similarity_score: f32,
        topic_keywords: Vec<String>,
        difficulty_level: DifficultyLevel,
    ) -> Self {
        let total_duration = sections.iter().map(|s| s.duration).sum();
        Self {
            title,
            sections,
            total_duration,
            similarity_score: Some(similarity_score),
            topic_keywords,
            difficulty_level: Some(difficulty_level),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub title: String,
    pub video_index: usize,
    #[serde(
        serialize_with = "crate::types::course::serialize_duration_as_secs",
        deserialize_with = "crate::types::course::deserialize_duration_from_secs"
    )]
    pub duration: Duration,
}

pub(crate) fn serialize_duration_as_secs<S>(
    duration: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

pub(crate) fn deserialize_duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub enum DifficultyLevel {
    Beginner,
    #[default]
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: Uuid,
    pub course_id: Uuid,
    pub video_id: Option<Uuid>,
    pub video_index: Option<usize>,
    pub content: String,
    pub timestamp: Option<u32>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Course {
    pub fn new(name: String, raw_titles: Vec<String>) -> Self {
        let videos = raw_titles
            .iter()
            .enumerate()
            .map(|(index, title)| VideoMetadata {
                title: title.clone(),
                source_url: None,
                video_id: None,
                playlist_id: None,
                original_index: index,
                duration_seconds: None,
                thumbnail_url: None,
                description: None,
                upload_date: None,
                author: None,
                view_count: None,
                tags: Vec::new(),
                is_local: false,
            })
            .collect();

        Self {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
            raw_titles,
            videos,
            structure: None,
        }
    }

    pub fn new_with_videos(name: String, videos: Vec<VideoMetadata>) -> Self {
        let raw_titles = videos.iter().map(|v| v.title.clone()).collect();
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
            raw_titles,
            videos,
            structure: None,
        }
    }

    pub fn video_count(&self) -> usize {
        self.videos.len().max(self.raw_titles.len())
    }

    pub fn is_structured(&self) -> bool {
        self.structure.is_some()
    }

    pub fn get_video_metadata(&self, index: usize) -> Option<&VideoMetadata> {
        self.videos.get(index)
    }

    pub fn get_video_title(&self, index: usize) -> Option<&str> {
        self.videos
            .get(index)
            .map(|v| v.title.as_str())
            .or_else(|| self.raw_titles.get(index).map(|s| s.as_str()))
    }
}

impl VideoMetadata {
    pub fn new_youtube(title: String, video_id: String, url: String) -> Self {
        Self {
            title,
            source_url: Some(url),
            video_id: Some(video_id),
            playlist_id: None,
            original_index: 0,
            duration_seconds: None,
            thumbnail_url: None,
            description: None,
            upload_date: None,
            author: None,
            view_count: None,
            tags: Vec::new(),
            is_local: false,
        }
    }

    pub fn new_youtube_with_playlist(
        title: String,
        video_id: String,
        url: String,
        playlist_id: Option<String>,
        original_index: usize,
    ) -> Self {
        Self {
            title,
            source_url: Some(url),
            video_id: Some(video_id),
            playlist_id,
            original_index,
            duration_seconds: None,
            thumbnail_url: None,
            description: None,
            upload_date: None,
            author: None,
            view_count: None,
            tags: Vec::new(),
            is_local: false,
        }
    }

    pub fn new_local(title: String, file_path: String) -> Self {
        Self {
            title,
            source_url: Some(file_path),
            video_id: None,
            playlist_id: None,
            original_index: 0,
            duration_seconds: None,
            thumbnail_url: None,
            description: None,
            upload_date: None,
            author: None,
            view_count: None,
            tags: Vec::new(),
            is_local: true,
        }
    }

    pub fn new_local_with_index(title: String, file_path: String, original_index: usize) -> Self {
        Self {
            title,
            source_url: Some(file_path),
            video_id: None,
            playlist_id: None,
            original_index,
            duration_seconds: None,
            thumbnail_url: None,
            description: None,
            upload_date: None,
            author: None,
            view_count: None,
            tags: Vec::new(),
            is_local: true,
        }
    }

    pub fn is_youtube(&self) -> bool {
        !self.is_local && self.video_id.is_some()
    }

    pub fn get_video_source(&self) -> Option<crate::video_player::VideoSource> {
        if self.is_local {
            if let Some(path) = &self.source_url {
                if !path.trim().is_empty() {
                    Some(crate::video_player::VideoSource::Local {
                        path: std::path::PathBuf::from(path),
                        title: self.title.clone(),
                    })
                } else {
                    log::error!("Local video has empty source_url: {}", self.title);
                    None
                }
            } else {
                log::error!("Local video missing source_url: {}", self.title);
                None
            }
        } else if let Some(video_id) = &self.video_id {
            if !video_id.trim().is_empty() && !video_id.starts_with("PLACEHOLDER_") {
                Some(crate::video_player::VideoSource::YouTube {
                    video_id: video_id.clone(),
                    playlist_id: self.playlist_id.clone(),
                    title: self.title.clone(),
                })
            } else {
                log::error!("YouTube video has invalid video_id '{}': {}", video_id, self.title);
                None
            }
        } else {
            log::error!("YouTube video missing video_id: {}", self.title);
            None
        }
    }

    pub fn is_metadata_complete(&self) -> bool {
        if self.is_local {
            !self.title.trim().is_empty()
                && self.source_url.as_ref().is_some_and(|url| !url.trim().is_empty())
        } else {
            !self.title.trim().is_empty()
                && self
                    .video_id
                    .as_ref()
                    .is_some_and(|id| !id.trim().is_empty() && !id.starts_with("PLACEHOLDER_"))
                && self.source_url.as_ref().is_some_and(|url| !url.trim().is_empty())
        }
    }

    pub fn validate_metadata(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Video title is empty".to_string());
        }

        if self.is_local {
            match &self.source_url {
                None => return Err("Local video missing file path".to_string()),
                Some(path) if path.trim().is_empty() => {
                    return Err("Local video has empty file path".to_string());
                },
                Some(_) => {},
            }
        } else {
            match &self.video_id {
                None => return Err("YouTube video missing video_id".to_string()),
                Some(id) if id.trim().is_empty() => {
                    return Err("YouTube video has empty video_id".to_string());
                },
                Some(id) if id.starts_with("PLACEHOLDER_") => {
                    return Err("YouTube video has placeholder video_id".to_string());
                },
                Some(_) => {},
            }

            match &self.source_url {
                None => return Err("YouTube video missing source URL".to_string()),
                Some(url) if url.trim().is_empty() => {
                    return Err("YouTube video has empty source URL".to_string());
                },
                Some(_) => {},
            }
        }

        Ok(())
    }
}
