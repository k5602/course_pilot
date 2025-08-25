use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;

// Import route components for the Routable derive
#[cfg(debug_assertions)]
use crate::ui::routes::ToastTest;
use crate::ui::routes::{AddCourse, AllCourses, Dashboard, Home, PlanView, Settings};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub raw_titles: Vec<String>,    // Keep for backward compatibility
    pub videos: Vec<VideoMetadata>, // New structured video data
    pub structure: Option<CourseStructure>,
}

/// Video metadata for courses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoMetadata {
    pub title: String,
    pub source_url: Option<String>,  // YouTube URL or local file path
    pub video_id: Option<String>,    // YouTube video ID
    pub playlist_id: Option<String>, // YouTube playlist ID for preserving playlist context
    pub original_index: usize,       // Preserve import order for sequential content detection
    pub duration_seconds: Option<f64>,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
    pub upload_date: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub view_count: Option<u64>,
    pub tags: Vec<String>,
    pub is_local: bool, // true for local files, false for YouTube
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseStructure {
    pub modules: Vec<Module>,
    pub metadata: StructureMetadata,
    pub clustering_metadata: Option<ClusteringMetadata>,
}

/// Clustering metadata for course structure
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

/// Confidence scores for clustering decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringConfidenceScores {
    /// Overall confidence in the clustering result (0.0 - 1.0)
    pub overall_confidence: f32,
    /// Confidence in module groupings (0.0 - 1.0)
    pub module_grouping_confidence: f32,
    /// Confidence in similarity calculations (0.0 - 1.0)
    pub similarity_confidence: f32,
    /// Confidence in topic extraction (0.0 - 1.0)
    pub topic_extraction_confidence: f32,
    /// Per-module confidence scores
    pub module_confidences: Vec<ModuleConfidence>,
}

/// Confidence score for individual modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfidence {
    pub module_index: usize,
    pub confidence_score: f32,
    pub similarity_strength: f32,
    pub topic_coherence: f32,
    pub duration_balance: f32,
}

/// Rationale explaining clustering decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringRationale {
    /// Primary reason for clustering approach
    pub primary_strategy: String,
    /// Detailed explanation of clustering decisions
    pub explanation: String,
    /// Key factors that influenced clustering
    pub key_factors: Vec<String>,
    /// Alternative strategies considered
    pub alternatives_considered: Vec<String>,
    /// Per-module rationale
    pub module_rationales: Vec<ModuleRationale>,
}

/// Rationale for individual module groupings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleRationale {
    pub module_index: usize,
    pub module_title: String,
    pub grouping_reason: String,
    pub similarity_explanation: String,
    pub topic_keywords: Vec<String>,
    pub video_count: usize,
}

/// Performance metrics for clustering operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PerformanceMetrics {
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
    /// Time spent on content analysis
    pub content_analysis_time_ms: u64,
    /// Time spent on clustering algorithm
    pub clustering_time_ms: u64,
    /// Time spent on optimization
    pub optimization_time_ms: u64,
    /// Peak memory usage during clustering (in bytes)
    pub peak_memory_usage_bytes: u64,
    /// Number of iterations for convergence
    pub algorithm_iterations: u32,
    /// Input data size metrics
    pub input_metrics: InputMetrics,
}

/// Metrics about the input data processed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMetrics {
    pub video_count: usize,
    pub unique_words: usize,
    pub vocabulary_size: usize,
    pub average_title_length: f32,
    pub content_diversity_score: f32,
}

/// Clustering algorithm types
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

/// Clustering strategy selection
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

/// Topic information from clustering analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopicInfo {
    pub keyword: String,
    pub relevance_score: f32,
    pub video_count: usize,
}

impl Eq for TopicInfo {}

impl std::hash::Hash for TopicInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.keyword.hash(state);
        // Hash f32 as bits to make it hashable
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

    /// Create a new CourseStructure without clustering metadata (for fallback)
    pub fn new_basic(modules: Vec<Module>, metadata: StructureMetadata) -> Self {
        Self { modules, metadata, clustering_metadata: None }
    }

    /// Create a new CourseStructure with clustering metadata
    pub fn new_with_clustering(
        modules: Vec<Module>,
        metadata: StructureMetadata,
        clustering_metadata: ClusteringMetadata,
    ) -> Self {
        Self { modules, metadata, clustering_metadata: Some(clustering_metadata) }
    }

    /// Check if this structure was created using clustering
    pub fn is_clustered(&self) -> bool {
        self.clustering_metadata.is_some()
    }

    /// Get the content organization type for UI display
    pub fn get_content_organization_type(&self) -> String {
        // First check if we have explicit content type information
        if let Some(content_type) = &self.metadata.content_type_detected {
            return content_type.clone();
        }

        // Fallback to determining from clustering metadata
        if self.clustering_metadata.is_some() {
            "Clustered".to_string()
        } else {
            // Check if original order was preserved
            if self.metadata.original_order_preserved.unwrap_or(false) {
                "Sequential".to_string()
            } else {
                "Unknown".to_string()
            }
        }
    }

    /// Get a user-friendly description of the content organization
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
    pub total_duration: std::time::Duration,
    pub estimated_duration_hours: Option<f32>,
    pub difficulty_level: Option<String>,
    pub structure_quality_score: Option<f32>,
    pub content_coherence_score: Option<f32>,
    pub content_type_detected: Option<String>, // "Sequential", "Clustered", "Mixed", "Ambiguous"
    pub original_order_preserved: Option<bool>, // true if content follows original order
    pub processing_strategy_used: Option<String>, // "PreserveOrder", "ApplyClustering", "UserChoice", "FallbackProcessing"
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

    /// Create a new basic module without clustering metadata
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

    /// Create a new module with clustering metadata
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

use serde::{Deserializer, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub title: String,
    pub video_index: usize,
    #[serde(
        serialize_with = "serialize_duration_as_secs",
        deserialize_with = "deserialize_duration_from_secs"
    )]
    pub duration: Duration,
}

// Custom serde for Duration as seconds
fn serialize_duration_as_secs<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

fn deserialize_duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    pub id: Uuid,
    pub course_id: Uuid,
    pub settings: PlanSettings,
    pub items: Vec<PlanItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanSettings {
    pub start_date: DateTime<Utc>,
    pub sessions_per_week: u8,
    pub session_length_minutes: u32,
    pub include_weekends: bool,
    pub advanced_settings: Option<AdvancedSchedulerSettings>,
}

/// Advanced scheduler settings for sophisticated planning algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedSchedulerSettings {
    pub strategy: DistributionStrategy,
    pub difficulty_adaptation: bool,
    pub spaced_repetition_enabled: bool,
    pub cognitive_load_balancing: bool,
    pub user_experience_level: DifficultyLevel,
    pub custom_intervals: Option<Vec<i64>>,
    pub max_session_duration_minutes: Option<u32>,
    pub min_break_between_sessions_hours: Option<u32>,
    pub prioritize_difficult_content: bool,
    pub adaptive_pacing: bool,
}

/// Distribution strategies for course content scheduling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum DistributionStrategy {
    ModuleBased,
    TimeBased,
    #[default]
    Hybrid,
    DifficultyBased,
    SpacedRepetition,
    Adaptive,
}

/// Content difficulty levels for adaptive scheduling
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

/// Plan regeneration status for progress tracking
#[derive(Debug, Clone, PartialEq)]
pub enum RegenerationStatus {
    Idle,
    InProgress { progress: f32, message: String },
    Completed,
    Failed { error: String },
}

/// State management for the PlanView component
#[derive(Debug, Clone, PartialEq)]
pub struct PlanViewState {
    pub expanded_sessions: HashSet<usize>,
    pub selected_videos: HashSet<usize>,
    pub regeneration_status: RegenerationStatus,
    pub last_update: DateTime<Utc>,
}

/// Video progress update for tracking completion status
#[derive(Debug, Clone, PartialEq)]
pub struct VideoProgressUpdate {
    pub plan_id: Uuid,
    pub session_index: usize,
    pub video_index: usize,
    pub completed: bool,
    pub timestamp: DateTime<Utc>,
}

impl VideoProgressUpdate {
    /// Create a new video progress update
    pub fn new(plan_id: Uuid, session_index: usize, video_index: usize, completed: bool) -> Self {
        Self { plan_id, session_index, video_index, completed, timestamp: Utc::now() }
    }

    /// Create a completion update
    pub fn completed(plan_id: Uuid, session_index: usize, video_index: usize) -> Self {
        Self::new(plan_id, session_index, video_index, true)
    }

    /// Create an uncomplete update
    pub fn uncompleted(plan_id: Uuid, session_index: usize, video_index: usize) -> Self {
        Self::new(plan_id, session_index, video_index, false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanItem {
    pub date: DateTime<Utc>,
    pub module_title: String,
    pub section_title: String,
    pub video_indices: Vec<usize>,
    pub completed: bool,
    #[serde(
        serialize_with = "serialize_duration_as_secs",
        deserialize_with = "deserialize_duration_from_secs"
    )]
    pub total_duration: Duration,
    #[serde(
        serialize_with = "serialize_duration_as_secs",
        deserialize_with = "deserialize_duration_from_secs"
    )]
    pub estimated_completion_time: Duration,
    pub overflow_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportJob {
    pub id: Uuid,
    pub status: ImportStatus,
    pub progress_percentage: f32,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub current_stage: ImportStage,
    pub stages: Vec<ImportStageInfo>,
    pub clustering_preview: Option<ClusteringPreview>,
    pub can_cancel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportStage {
    Fetching,
    Processing,
    TfIdfAnalysis,
    KMeansClustering,
    Optimization,
    Saving,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportStageInfo {
    pub stage: ImportStage,
    pub name: String,
    pub description: String,
    pub progress: f32,
    pub status: StageStatus,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusteringPreview {
    pub quality_score: f32,
    pub confidence_level: f32,
    pub cluster_count: usize,
    pub rationale: String,
    pub key_topics: Vec<String>,
    pub estimated_modules: Vec<EstimatedModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EstimatedModule {
    pub title: String,
    pub video_count: usize,
    pub confidence: f32,
    pub key_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportStatus {
    Starting,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl ImportJob {
    pub fn new(message: String) -> Self {
        let stages = vec![
            ImportStageInfo {
                stage: ImportStage::Fetching,
                name: "Fetching Data".to_string(),
                description: "Downloading playlist information and video metadata".to_string(),
                progress: 0.0,
                status: StageStatus::Pending,
                duration_ms: None,
            },
            ImportStageInfo {
                stage: ImportStage::Processing,
                name: "Processing Content".to_string(),
                description: "Analyzing video titles and extracting content features".to_string(),
                progress: 0.0,
                status: StageStatus::Pending,
                duration_ms: None,
            },
            ImportStageInfo {
                stage: ImportStage::TfIdfAnalysis,
                name: "TF-IDF Analysis".to_string(),
                description: "Computing term frequency and semantic similarity scores".to_string(),
                progress: 0.0,
                status: StageStatus::Pending,
                duration_ms: None,
            },
            ImportStageInfo {
                stage: ImportStage::KMeansClustering,
                name: "K-Means Clustering".to_string(),
                description: "Grouping videos into coherent learning modules".to_string(),
                progress: 0.0,
                status: StageStatus::Pending,
                duration_ms: None,
            },
            ImportStageInfo {
                stage: ImportStage::Optimization,
                name: "Structure Optimization".to_string(),
                description: "Refining module boundaries and optimizing learning flow".to_string(),
                progress: 0.0,
                status: StageStatus::Pending,
                duration_ms: None,
            },
            ImportStageInfo {
                stage: ImportStage::Saving,
                name: "Saving Course".to_string(),
                description: "Persisting course structure and metadata to database".to_string(),
                progress: 0.0,
                status: StageStatus::Pending,
                duration_ms: None,
            },
        ];

        Self {
            id: Uuid::new_v4(),
            status: ImportStatus::Starting,
            progress_percentage: 0.0,
            message,
            created_at: Utc::now(),
            current_stage: ImportStage::Fetching,
            stages,
            clustering_preview: None,
            can_cancel: true,
        }
    }

    pub fn update_stage_progress(&mut self, stage: ImportStage, progress: f32, message: String) {
        self.current_stage = stage.clone();
        self.message = message;

        // Update the specific stage
        if let Some(stage_info) = self.stages.iter_mut().find(|s| s.stage == stage) {
            stage_info.progress = progress;
            stage_info.status = StageStatus::InProgress;
        }

        // Calculate overall progress based on stage completion
        let stage_weight = 100.0 / self.stages.len() as f32;
        let mut total_progress = 0.0;

        for stage_info in &self.stages {
            match stage_info.status {
                StageStatus::Completed => total_progress += stage_weight,
                StageStatus::InProgress => {
                    total_progress += stage_weight * (stage_info.progress / 100.0)
                },
                _ => {},
            }
        }

        self.progress_percentage = total_progress;
        self.status = ImportStatus::InProgress;
    }

    pub fn complete_stage(&mut self, stage: ImportStage, duration_ms: u64) {
        if let Some(stage_info) = self.stages.iter_mut().find(|s| s.stage == stage) {
            stage_info.status = StageStatus::Completed;
            stage_info.progress = 100.0;
            stage_info.duration_ms = Some(duration_ms);
        }

        // Recalculate overall progress
        self.update_overall_progress();
    }

    pub fn fail_stage(&mut self, stage: ImportStage, error: String) {
        if let Some(stage_info) = self.stages.iter_mut().find(|s| s.stage == stage) {
            stage_info.status = StageStatus::Failed(error.clone());
        }

        self.status = ImportStatus::Failed;
        self.message = error;
        self.can_cancel = false;
    }

    pub fn set_clustering_preview(&mut self, preview: ClusteringPreview) {
        self.clustering_preview = Some(preview);
    }

    pub fn mark_completed(&mut self) {
        self.status = ImportStatus::Completed;
        self.progress_percentage = 100.0;
        self.can_cancel = false;

        // Mark all stages as completed
        for stage_info in &mut self.stages {
            if stage_info.status != StageStatus::Completed {
                stage_info.status = StageStatus::Completed;
                stage_info.progress = 100.0;
            }
        }
    }

    pub fn mark_cancelled(&mut self) {
        self.status = ImportStatus::Cancelled;
        self.message = "Import cancelled by user".to_string();
        self.can_cancel = false;
    }

    fn update_overall_progress(&mut self) {
        let stage_weight = 100.0 / self.stages.len() as f32;
        let mut total_progress = 0.0;

        for stage_info in &self.stages {
            match stage_info.status {
                StageStatus::Completed => total_progress += stage_weight,
                StageStatus::InProgress => {
                    total_progress += stage_weight * (stage_info.progress / 100.0)
                },
                _ => {},
            }
        }

        self.progress_percentage = total_progress;
    }

    // Legacy methods for backward compatibility
    pub fn update_progress(&mut self, percentage: f32, message: String) {
        self.progress_percentage = percentage.clamp(0.0, 100.0);
        self.message = message;
        if percentage >= 100.0 {
            self.status = ImportStatus::Completed;
        } else {
            self.status = ImportStatus::InProgress;
        }
    }

    pub fn mark_failed(&mut self, error_message: String) {
        self.status = ImportStatus::Failed;
        self.message = error_message;
        self.can_cancel = false;
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub courses: Vec<Course>,
    pub plans: Vec<Plan>,
    pub notes: Vec<Note>,
    pub active_import: Option<ImportJob>,
    pub contextual_panel: ContextualPanelState,
    pub sidebar_open_mobile: bool,
}

#[derive(Clone, Debug, PartialEq, dioxus_router::prelude::Routable)]
pub enum Route {
    #[route("/")]
    Home {},

    #[route("/dashboard")]
    Dashboard {},

    #[route("/courses")]
    AllCourses {},

    #[route("/plan/:course_id")]
    PlanView { course_id: String },

    #[route("/settings")]
    Settings {},

    #[route("/import")]
    AddCourse {},

    #[cfg(debug_assertions)]
    #[route("/toast-test")]
    ToastTest {},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CourseStatus {
    Structured,
    Unstructured,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextualPanelTab {
    Notes,
    Chatbot,
}

/// Video context for notes integration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VideoContext {
    pub course_id: Uuid,
    pub video_index: usize,
    pub video_title: String,
    pub module_title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextualPanelState {
    pub is_open: bool,
    pub active_tab: ContextualPanelTab,
    pub video_context: Option<VideoContext>,
}

impl Default for ContextualPanelState {
    fn default() -> Self {
        Self {
            is_open: false, // Closed by default, user can open via button
            active_tab: ContextualPanelTab::Notes,
            video_context: None,
        }
    }
}

/// Represents a user note tied to a specific video (section) in a course.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: Uuid,
    pub course_id: Uuid, // Always present: which course this note belongs to
    pub video_id: Option<Uuid>, // None for course-level notes, Some for video-level notes
    pub video_index: Option<usize>, // Video index within the course for plan view integration
    pub content: String, // Markdown or rich text
    pub timestamp: Option<u32>, // Seconds into the video
    pub tags: Vec<String>, // Tagging support
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Course {
    pub fn new(name: String, raw_titles: Vec<String>) -> Self {
        // Create basic video metadata from raw titles for backward compatibility
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
            original_index: 0, // Will be set properly during import
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
            original_index: 0, // Will be set properly during import
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
            // For local videos, we need a valid file path
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
        } else {
            // For YouTube videos, we need a valid video_id
            if let Some(video_id) = &self.video_id {
                if !video_id.trim().is_empty() && !video_id.starts_with("PLACEHOLDER_") {
                    Some(crate::video_player::VideoSource::YouTube {
                        video_id: video_id.clone(),
                        playlist_id: self.playlist_id.clone(),
                        title: self.title.clone(),
                    })
                } else {
                    log::error!(
                        "YouTube video has invalid video_id '{}': {}",
                        video_id,
                        self.title
                    );
                    None
                }
            } else {
                log::error!("YouTube video missing video_id: {}", self.title);
                None
            }
        }
    }

    /// Check if metadata is complete for the video type (YouTube vs local)
    pub fn is_metadata_complete(&self) -> bool {
        if self.is_local {
            // Local videos need at least title and source_url (file path)
            !self.title.trim().is_empty()
                && self.source_url.as_ref().map_or(false, |url| !url.trim().is_empty())
        } else {
            // YouTube videos need at least title, video_id, and source_url
            !self.title.trim().is_empty()
                && self
                    .video_id
                    .as_ref()
                    .map_or(false, |id| !id.trim().is_empty() && !id.starts_with("PLACEHOLDER_"))
                && self.source_url.as_ref().map_or(false, |url| !url.trim().is_empty())
        }
    }

    /// Validate metadata and return detailed error information if incomplete
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
                Some(_) => {}, // Valid
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
                Some(_) => {}, // Valid
            }

            match &self.source_url {
                None => return Err("YouTube video missing source URL".to_string()),
                Some(url) if url.trim().is_empty() => {
                    return Err("YouTube video has empty source URL".to_string());
                },
                Some(_) => {}, // Valid
            }
        }

        Ok(())
    }
}

impl Plan {
    pub fn new(course_id: Uuid, settings: PlanSettings) -> Self {
        Self { id: Uuid::new_v4(), course_id, settings, items: Vec::new(), created_at: Utc::now() }
    }

    pub fn total_sessions(&self) -> usize {
        self.items.len()
    }

    pub fn completed_sessions(&self) -> usize {
        self.items.iter().filter(|item| item.completed).count()
    }

    pub fn progress_percentage(&self) -> f32 {
        if self.items.is_empty() {
            0.0
        } else {
            (self.completed_sessions() as f32 / self.total_sessions() as f32) * 100.0
        }
    }
}

/// Identifier for a plan item using composite key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanItemIdentifier {
    pub plan_id: Uuid,
    pub item_index: usize,
}

impl PlanItemIdentifier {
    pub fn new(plan_id: Uuid, item_index: usize) -> Self {
        Self { plan_id, item_index }
    }
}

/// Extension trait for Plan operations
pub trait PlanExt {
    fn get_item_identifier(&self, index: usize) -> PlanItemIdentifier;
    fn update_item_completion(&mut self, index: usize, completed: bool) -> Result<(), String>;
    fn calculate_progress(&self) -> (usize, usize, f32);
}

impl PlanExt for Plan {
    fn get_item_identifier(&self, index: usize) -> PlanItemIdentifier {
        PlanItemIdentifier::new(self.id, index)
    }

    fn update_item_completion(&mut self, index: usize, completed: bool) -> Result<(), String> {
        if let Some(item) = self.items.get_mut(index) {
            item.completed = completed;
            Ok(())
        } else {
            Err(format!("Plan item index {index} out of bounds"))
        }
    }

    fn calculate_progress(&self) -> (usize, usize, f32) {
        let total_count = self.items.len();
        let completed_count = self.items.iter().filter(|item| item.completed).count();
        let percentage = if total_count > 0 {
            (completed_count as f32 / total_count as f32) * 100.0
        } else {
            0.0
        };

        (completed_count, total_count, percentage)
    }
}

impl Default for AdvancedSchedulerSettings {
    fn default() -> Self {
        Self {
            strategy: DistributionStrategy::Hybrid,
            difficulty_adaptation: true,
            spaced_repetition_enabled: false,
            cognitive_load_balancing: true,
            user_experience_level: DifficultyLevel::Intermediate,
            custom_intervals: None,
            max_session_duration_minutes: None,
            min_break_between_sessions_hours: None,
            prioritize_difficult_content: false,
            adaptive_pacing: true,
        }
    }
}

impl AdvancedSchedulerSettings {
    /// Create new settings with a specific strategy
    pub fn with_strategy(strategy: DistributionStrategy) -> Self {
        Self { strategy, ..Self::default() }
    }

    /// Create settings optimized for beginners
    pub fn for_beginner() -> Self {
        Self {
            strategy: DistributionStrategy::SpacedRepetition,
            difficulty_adaptation: true,
            spaced_repetition_enabled: true,
            cognitive_load_balancing: true,
            user_experience_level: DifficultyLevel::Beginner,
            prioritize_difficult_content: false,
            adaptive_pacing: true,
            ..Self::default()
        }
    }

    /// Create settings optimized for advanced users
    pub fn for_advanced() -> Self {
        Self {
            strategy: DistributionStrategy::Adaptive,
            difficulty_adaptation: true,
            spaced_repetition_enabled: false,
            cognitive_load_balancing: false,
            user_experience_level: DifficultyLevel::Advanced,
            prioritize_difficult_content: true,
            adaptive_pacing: true,
            ..Self::default()
        }
    }

    /// Validate the settings for consistency
    pub fn validate(&self) -> Result<(), String> {
        if self.spaced_repetition_enabled && self.strategy != DistributionStrategy::SpacedRepetition
        {
            return Err(
                "Spaced repetition enabled but strategy is not SpacedRepetition".to_string()
            );
        }

        if let Some(max_duration) = self.max_session_duration_minutes {
            if !(15..=300).contains(&max_duration) {
                return Err("Session duration must be between 15 and 300 minutes".to_string());
            }
        }

        if let Some(min_break) = self.min_break_between_sessions_hours {
            if min_break > 168 {
                // 1 week
                return Err("Minimum break between sessions cannot exceed 1 week".to_string());
            }
        }

        if let Some(ref intervals) = self.custom_intervals {
            if intervals.is_empty() {
                return Err("Custom intervals cannot be empty".to_string());
            }
            if intervals.iter().any(|&i| i <= 0) {
                return Err("All custom intervals must be positive".to_string());
            }
        }

        Ok(())
    }

    /// Get recommended settings based on course complexity and user level
    pub fn recommend_for_course(
        user_level: DifficultyLevel,
        course_complexity: DifficultyLevel,
        total_duration_hours: f32,
    ) -> Self {
        let strategy = match (user_level, course_complexity) {
            (DifficultyLevel::Beginner, _) => DistributionStrategy::SpacedRepetition,
            (
                DifficultyLevel::Intermediate,
                DifficultyLevel::Advanced | DifficultyLevel::Expert,
            ) => DistributionStrategy::Adaptive,
            (DifficultyLevel::Advanced | DifficultyLevel::Expert, _) => {
                DistributionStrategy::Hybrid
            },
            _ => DistributionStrategy::Hybrid,
        };

        let spaced_repetition = matches!(user_level, DifficultyLevel::Beginner);
        let prioritize_difficult =
            matches!(user_level, DifficultyLevel::Advanced | DifficultyLevel::Expert);

        // Adjust session duration based on course length
        let max_session_duration = if total_duration_hours > 20.0 {
            Some(90) // Longer sessions for extensive courses
        } else if total_duration_hours < 5.0 {
            Some(45) // Shorter sessions for brief courses
        } else {
            Some(60) // Standard session length
        };

        Self {
            strategy,
            difficulty_adaptation: true,
            spaced_repetition_enabled: spaced_repetition,
            cognitive_load_balancing: true,
            user_experience_level: user_level,
            max_session_duration_minutes: max_session_duration,
            prioritize_difficult_content: prioritize_difficult,
            adaptive_pacing: true,
            ..Self::default()
        }
    }
}

impl Default for ClusteringMetadata {
    fn default() -> Self {
        Self {
            algorithm_used: ClusteringAlgorithm::Fallback,
            similarity_threshold: 0.6,
            cluster_count: 0,
            quality_score: 0.0,
            processing_time_ms: 0,
            content_topics: Vec::new(),
            strategy_used: ClusteringStrategy::Fallback,
            confidence_scores: ClusteringConfidenceScores::default(),
            rationale: ClusteringRationale::default(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }
}

impl Default for ClusteringConfidenceScores {
    fn default() -> Self {
        Self {
            overall_confidence: 0.0,
            module_grouping_confidence: 0.0,
            similarity_confidence: 0.0,
            topic_extraction_confidence: 0.0,
            module_confidences: Vec::new(),
        }
    }
}

impl Default for ClusteringRationale {
    fn default() -> Self {
        Self {
            primary_strategy: "Fallback".to_string(),
            explanation: "No clustering applied".to_string(),
            key_factors: Vec::new(),
            alternatives_considered: Vec::new(),
            module_rationales: Vec::new(),
        }
    }
}

impl Default for InputMetrics {
    fn default() -> Self {
        Self {
            video_count: 0,
            unique_words: 0,
            vocabulary_size: 0,
            average_title_length: 0.0,
            content_diversity_score: 0.0,
        }
    }
}

impl DistributionStrategy {
    /// Get all available distribution strategies
    pub fn all() -> Vec<Self> {
        vec![
            Self::ModuleBased,
            Self::TimeBased,
            Self::Hybrid,
            Self::DifficultyBased,
            Self::SpacedRepetition,
            Self::Adaptive,
        ]
    }

    /// Get human-readable name for the strategy
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ModuleBased => "Module-based",
            Self::TimeBased => "Time-based",
            Self::Hybrid => "Hybrid",
            Self::DifficultyBased => "Difficulty-based",
            Self::SpacedRepetition => "Spaced Repetition",
            Self::Adaptive => "Adaptive",
        }
    }

    /// Get description for the strategy
    pub fn description(&self) -> &'static str {
        match self {
            Self::ModuleBased => "Respects module boundaries and logical content grouping",
            Self::TimeBased => "Focuses on even time distribution across sessions",
            Self::Hybrid => "Balances both module structure and time constraints",
            Self::DifficultyBased => "Adapts pacing based on content difficulty",
            Self::SpacedRepetition => "Optimizes for memory retention with review sessions",
            Self::Adaptive => "AI-driven scheduling based on learning patterns",
        }
    }
}

impl DifficultyLevel {
    /// Get all available difficulty levels
    pub fn all() -> Vec<Self> {
        vec![Self::Beginner, Self::Intermediate, Self::Advanced, Self::Expert]
    }

    /// Get human-readable name for the difficulty level
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
            Self::Expert => "Expert",
        }
    }
}

impl Default for PlanViewState {
    fn default() -> Self {
        Self {
            expanded_sessions: HashSet::new(),
            selected_videos: HashSet::new(),
            regeneration_status: RegenerationStatus::Idle,
            last_update: Utc::now(),
        }
    }
}

impl PlanViewState {
    /// Create a new PlanViewState instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle session expansion state
    pub fn toggle_session(&mut self, session_index: usize) {
        if self.expanded_sessions.contains(&session_index) {
            self.expanded_sessions.remove(&session_index);
        } else {
            self.expanded_sessions.insert(session_index);
        }
        self.last_update = Utc::now();
    }

    /// Check if a session is expanded
    pub fn is_session_expanded(&self, session_index: usize) -> bool {
        self.expanded_sessions.contains(&session_index)
    }

    /// Toggle video selection state
    pub fn toggle_video_selection(&mut self, video_index: usize) {
        if self.selected_videos.contains(&video_index) {
            self.selected_videos.remove(&video_index);
        } else {
            self.selected_videos.insert(video_index);
        }
        self.last_update = Utc::now();
    }

    /// Check if a video is selected
    pub fn is_video_selected(&self, video_index: usize) -> bool {
        self.selected_videos.contains(&video_index)
    }

    /// Update regeneration status
    pub fn set_regeneration_status(&mut self, status: RegenerationStatus) {
        self.regeneration_status = status;
        self.last_update = Utc::now();
    }

    /// Clear all selections and expanded states
    pub fn clear_selections(&mut self) {
        self.expanded_sessions.clear();
        self.selected_videos.clear();
        self.last_update = Utc::now();
    }

    /// Get count of expanded sessions
    pub fn expanded_session_count(&self) -> usize {
        self.expanded_sessions.len()
    }

    /// Get count of selected videos
    pub fn selected_video_count(&self) -> usize {
        self.selected_videos.len()
    }
}

/// Duration formatting utilities
pub mod duration_utils {
    use std::time::Duration;

    /// Format duration as "Xh Ym" or "Ym" or "Xs"
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            if minutes > 0 { format!("{hours}h {minutes}m") } else { format!("{hours}h") }
        } else if minutes > 0 {
            format!("{minutes}m")
        } else {
            format!("{seconds}s")
        }
    }

    /// Format duration as "X minutes" or "X hours Y minutes"
    pub fn format_duration_verbose(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;

        if hours > 0 {
            if minutes > 0 {
                format!("{hours} hours {minutes} minutes")
            } else {
                format!("{hours} hours")
            }
        } else if minutes > 0 {
            format!("{minutes} minutes")
        } else {
            format!("{total_seconds} seconds")
        }
    }

    /// Format duration as decimal hours (e.g., "1.5 hours")
    pub fn format_duration_decimal_hours(duration: Duration) -> String {
        let hours = duration.as_secs() as f32 / 3600.0;
        if hours >= 1.0 {
            format!("{hours:.1} hours")
        } else {
            let minutes = duration.as_secs() / 60;
            format!("{minutes} minutes")
        }
    }

    /// Check if duration exceeds a reasonable session length
    pub fn is_duration_excessive(duration: Duration, session_limit_minutes: u32) -> bool {
        let session_limit = Duration::from_secs(session_limit_minutes as u64 * 60);
        duration > session_limit
    }

    /// Calculate estimated completion time with buffer
    pub fn calculate_completion_time_with_buffer(
        video_duration: Duration,
        buffer_percentage: f32,
    ) -> Duration {
        let buffer_time =
            Duration::from_secs((video_duration.as_secs() as f32 * buffer_percentage) as u64);
        video_duration + buffer_time
    }

    /// Validate session duration and generate overflow warnings
    pub fn validate_session_duration(
        sections: &[&crate::types::Section],
        settings: &crate::types::PlanSettings,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        let total_duration: Duration = sections.iter().map(|s| s.duration).sum();
        let session_limit = Duration::from_secs(settings.session_length_minutes as u64 * 60);

        if total_duration > session_limit {
            warnings.push(format!(
                "Session duration ({}) exceeds target ({})",
                format_duration(total_duration),
                format_duration(session_limit)
            ));
        }

        // Check for individual videos that are very long
        for section in sections {
            if section.duration.as_secs() > (settings.session_length_minutes as u64 * 60) / 2 {
                warnings.push(format!(
                    "Video '{}' is very long ({}) for session length",
                    section.title,
                    format_duration(section.duration)
                ));
            }
        }

        warnings
    }
}
