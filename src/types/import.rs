use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub weight: f32,
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
        let stage_definitions = [
            (
                ImportStage::Fetching,
                "Fetching Data",
                "Downloading playlist information and video metadata",
                0.15,
            ),
            (
                ImportStage::Processing,
                "Processing Content",
                "Analyzing video titles and extracting content features",
                0.15,
            ),
            (
                ImportStage::TfIdfAnalysis,
                "TF-IDF Analysis",
                "Computing term frequency and semantic similarity scores",
                0.2,
            ),
            (
                ImportStage::KMeansClustering,
                "K-Means Clustering",
                "Grouping videos into coherent learning modules",
                0.2,
            ),
            (
                ImportStage::Optimization,
                "Structure Optimization",
                "Refining module boundaries and optimizing learning flow",
                0.15,
            ),
            (
                ImportStage::Saving,
                "Saving Course",
                "Persisting course structure and metadata to database",
                0.15,
            ),
        ];

        let stages = stage_definitions
            .iter()
            .map(|(stage, name, description, weight)| ImportStageInfo {
                stage: *stage,
                name: name.to_string(),
                description: description.to_string(),
                weight: *weight,
                progress: 0.0,
                status: StageStatus::Pending,
                duration_ms: None,
            })
            .collect();

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

    fn recalc_weighted_progress(&mut self) {
        let weighted: f32 = self
            .stages
            .iter()
            .map(|stage| stage.weight * (stage.progress / 100.0).clamp(0.0, 1.0))
            .sum();
        self.progress_percentage = (weighted * 100.0).clamp(0.0, 100.0);
    }

    fn refresh_current_stage(&mut self) {
        if let Some(active) =
            self.stages.iter().find(|s| matches!(s.status, StageStatus::InProgress))
        {
            self.current_stage = active.stage;
        } else if let Some(pending) =
            self.stages.iter().find(|s| matches!(s.status, StageStatus::Pending))
        {
            self.current_stage = pending.stage;
        }
    }

    fn all_stages_completed(&self) -> bool {
        self.stages.iter().all(|s| matches!(s.status, StageStatus::Completed))
    }

    pub fn update_stage_progress(&mut self, stage: ImportStage, progress: f32, message: String) {
        self.current_stage = stage;
        self.message = message;

        if let Some(stage_info) = self.stages.iter_mut().find(|s| s.stage == stage) {
            stage_info.progress = progress.clamp(0.0, 100.0);
            stage_info.status = StageStatus::InProgress;
        }

        self.status = ImportStatus::InProgress;
        self.recalc_weighted_progress();
        self.refresh_current_stage();
    }

    pub fn complete_stage(&mut self, stage: ImportStage, duration_ms: u64) {
        if let Some(stage_info) = self.stages.iter_mut().find(|s| s.stage == stage) {
            stage_info.status = StageStatus::Completed;
            stage_info.progress = 100.0;
            stage_info.duration_ms = Some(duration_ms);
        }

        self.recalc_weighted_progress();
        if self.all_stages_completed() {
            self.status = ImportStatus::Completed;
            self.progress_percentage = 100.0;
            self.can_cancel = false;
        } else {
            self.status = ImportStatus::InProgress;
            self.refresh_current_stage();
        }
    }

    pub fn fail_stage(&mut self, stage: ImportStage, error: String) {
        if let Some(stage_info) = self.stages.iter_mut().find(|s| s.stage == stage) {
            stage_info.status = StageStatus::Failed(error.clone());
        }

        self.status = ImportStatus::Failed;
        self.message = error;
        self.can_cancel = false;
        self.current_stage = stage;
        self.recalc_weighted_progress();
    }

    pub fn set_clustering_preview(&mut self, preview: ClusteringPreview) {
        self.clustering_preview = Some(preview);
    }

    pub fn mark_completed(&mut self) {
        self.status = ImportStatus::Completed;
        self.progress_percentage = 100.0;
        self.can_cancel = false;

        for stage_info in &mut self.stages {
            if stage_info.status != StageStatus::Completed {
                stage_info.status = StageStatus::Completed;
                stage_info.progress = 100.0;
            }
        }
        self.current_stage = ImportStage::Saving;
    }

    pub fn mark_cancelled(&mut self) {
        self.status = ImportStatus::Cancelled;
        self.message = "Import cancelled by user".to_string();
        self.can_cancel = false;
    }

    pub fn update_overall_progress(&mut self) {
        self.recalc_weighted_progress();
        self.refresh_current_stage();
    }

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
