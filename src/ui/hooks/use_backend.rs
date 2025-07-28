use crate::export::{ExportFormat, ExportResult};
use crate::storage::settings::AppSettings;
use crate::types::{
    AdvancedSchedulerSettings, Course, DifficultyLevel, DistributionStrategy, Note, Plan,
    PlanSettings,
};
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

use super::{
    FolderValidation, ProgressInfo, use_analytics_manager, use_course_manager, use_export_manager,
    use_import_manager, use_notes_manager, use_plan_manager, use_settings_manager,
};

use super::use_analytics::AnalyticsManager;
use super::use_courses::CourseManager;
use super::use_export::ExportManager;
use super::use_import::ImportManager;
use super::use_notes::NotesManager;
use super::use_plans::PlanManager;
use super::use_settings::SettingsManager;

/// Unified backend interface that combines all the specialized hooks
/// This provides the same interface as the old BackendAdapter but using the new hook architecture
#[derive(Clone)]
pub struct Backend {
    pub courses: CourseManager,
    pub plans: PlanManager,
    pub notes: NotesManager,
    pub analytics: AnalyticsManager,
    pub import: ImportManager,
    pub export: ExportManager,
    pub settings: SettingsManager,
}

impl Backend {
    // --- Courses ---
    pub async fn list_courses(&self) -> Result<Vec<Course>> {
        self.courses.list_courses().await
    }

    pub async fn get_course(&self, id: Uuid) -> Result<Option<Course>> {
        self.courses.get_course(id).await
    }

    pub async fn create_course(&self, course: Course) -> Result<()> {
        self.courses.create_course(course).await
    }

    pub async fn update_course(&self, course: Course) -> Result<()> {
        self.courses.update_course(course).await
    }

    pub async fn delete_course(&self, course_id: Uuid) -> Result<()> {
        self.courses.delete_course(course_id).await
    }

    // --- Plans ---
    pub async fn get_plan_by_course(&self, course_id: Uuid) -> Result<Option<Plan>> {
        self.plans.get_plan_by_course(course_id).await
    }

    pub async fn save_plan(&self, plan: Plan) -> Result<()> {
        self.plans.save_plan(plan).await
    }

    pub async fn delete_plan(&self, plan_id: Uuid) -> Result<()> {
        self.plans.delete_plan(plan_id).await
    }

    pub async fn update_plan_item_completion(
        &self,
        plan_id: Uuid,
        item_index: usize,
        completed: bool,
    ) -> Result<()> {
        self.plans
            .update_plan_item_completion(plan_id, item_index, completed)
            .await
    }

    pub async fn get_plan_progress(&self, plan_id: Uuid) -> Result<ProgressInfo> {
        self.plans.get_plan_progress(plan_id).await
    }

    pub async fn get_course_progress(&self, course_id: Uuid) -> Result<Option<ProgressInfo>> {
        self.plans.get_course_progress(course_id).await
    }

    pub async fn generate_plan(&self, course_id: Uuid, settings: PlanSettings) -> Result<Plan> {
        self.plans.generate_plan(course_id, settings).await
    }

    pub async fn regenerate_plan(&self, plan_id: Uuid, new_settings: PlanSettings) -> Result<Plan> {
        self.plans.regenerate_plan(plan_id, new_settings).await
    }

    // --- Notes ---
    pub async fn list_all_notes(&self) -> Result<Vec<Note>> {
        self.notes.list_all_notes().await
    }

    pub async fn list_notes_by_course(&self, course_id: Uuid) -> Result<Vec<Note>> {
        self.notes.list_notes_by_course(course_id).await
    }

    pub async fn list_notes_by_course_and_video_index(
        &self,
        course_id: Uuid,
        video_index: Option<usize>,
    ) -> Result<Vec<Note>> {
        self.notes
            .list_notes_by_course_and_video_index(course_id, video_index)
            .await
    }

    pub async fn list_notes_by_video(&self, video_id: Uuid) -> Result<Vec<Note>> {
        self.notes.list_notes_by_video(video_id).await
    }

    pub async fn list_notes_by_video_index(
        &self,
        course_id: Uuid,
        video_index: usize,
    ) -> Result<Vec<Note>> {
        self.notes
            .list_notes_by_video_index(course_id, video_index)
            .await
    }

    pub async fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        self.notes.search_notes(query).await
    }

    pub async fn search_notes_by_tags(&self, tags: &[String]) -> Result<Vec<Note>> {
        self.notes.search_notes_by_tags(tags).await
    }

    pub async fn get_note(&self, note_id: Uuid) -> Result<Option<Note>> {
        self.notes.get_note(note_id).await
    }

    pub async fn save_note(&self, note: Note) -> Result<()> {
        self.notes.save_note(note).await
    }

    pub async fn delete_note(&self, note_id: Uuid) -> Result<()> {
        self.notes.delete_note(note_id).await
    }

    // --- Export ---
    pub async fn export_course(
        &self,
        course_id: Uuid,
        format: ExportFormat,
    ) -> Result<ExportResult> {
        self.export.export_course(course_id, format).await
    }

    pub async fn export_plan(&self, plan_id: Uuid, format: ExportFormat) -> Result<ExportResult> {
        self.export.export_plan(plan_id, format).await
    }

    pub async fn export_notes(
        &self,
        course_id: Uuid,
        format: ExportFormat,
    ) -> Result<ExportResult> {
        self.export.export_notes(course_id, format).await
    }

    pub async fn export_course_with_progress<F>(
        &self,
        course_id: Uuid,
        format: ExportFormat,
        progress_callback: F,
    ) -> Result<ExportResult>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        self.export
            .export_course_with_progress(course_id, format, Box::new(progress_callback))
            .await
    }

    // --- Analytics ---
    pub async fn get_available_scheduling_strategies(&self) -> Result<Vec<DistributionStrategy>> {
        self.analytics.get_available_scheduling_strategies().await
    }

    pub async fn get_available_difficulty_levels(&self) -> Result<Vec<DifficultyLevel>> {
        self.analytics.get_available_difficulty_levels().await
    }

    pub async fn validate_advanced_scheduler_settings(
        &self,
        settings: &AdvancedSchedulerSettings,
    ) -> Result<Vec<String>> {
        self.analytics
            .validate_advanced_scheduler_settings(settings)
            .await
    }

    pub async fn get_recommended_advanced_settings(
        &self,
        course_id: Uuid,
        user_experience: DifficultyLevel,
    ) -> Result<AdvancedSchedulerSettings> {
        self.analytics
            .get_recommended_advanced_settings(course_id, user_experience)
            .await
    }

    pub async fn structure_course(&self, course_id: Uuid) -> Result<Course> {
        self.analytics.structure_course(course_id).await
    }

    pub async fn structure_course_with_progress<F>(
        &self,
        course_id: Uuid,
        progress_callback: F,
    ) -> Result<Course>
    where
        F: Fn(f32, String) + Send + Sync + 'static,
    {
        self.analytics
            .structure_course_with_progress(course_id, progress_callback)
            .await
    }

    // --- Import ---
    pub async fn browse_folder(&self) -> Result<Option<PathBuf>> {
        self.import.browse_folder().await
    }

    pub async fn validate_folder(&self, path: PathBuf) -> Result<FolderValidation> {
        self.import.validate_folder(path).await
    }

    pub async fn import_from_local_folder(
        &self,
        folder_path: PathBuf,
        course_title: Option<String>,
    ) -> Result<Course> {
        self.import
            .import_from_local_folder(folder_path, course_title)
            .await
    }

    // --- Settings ---
    pub async fn load_settings(&self) -> Result<AppSettings> {
        self.settings.load_settings().await
    }

    pub async fn save_settings(&self, settings: AppSettings) -> Result<()> {
        self.settings.save_settings(settings).await
    }

    pub async fn get_youtube_api_key(&self) -> Result<Option<String>> {
        self.settings.get_youtube_api_key().await
    }

    pub async fn set_youtube_api_key(&self, api_key: Option<String>) -> Result<()> {
        self.settings.set_youtube_api_key(api_key).await
    }

    pub async fn get_gemini_api_key(&self) -> Result<Option<String>> {
        self.settings.get_gemini_api_key().await
    }

    pub async fn set_gemini_api_key(&self, api_key: Option<String>) -> Result<()> {
        self.settings.set_gemini_api_key(api_key).await
    }

    pub async fn reset_settings(&self) -> Result<()> {
        self.settings.reset_settings().await
    }
}

/// Hook for accessing the unified backend interface
pub fn use_backend() -> Backend {
    Backend {
        courses: use_course_manager(),
        plans: use_plan_manager(),
        notes: use_notes_manager(),
        analytics: use_analytics_manager(),
        import: use_import_manager(),
        export: use_export_manager(),
        settings: use_settings_manager(),
    }
}
