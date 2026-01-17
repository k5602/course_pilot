//! Application Context - Dependency Injection Container
//!
//! Wires all infrastructure adapters to application use cases.

use std::sync::Arc;

use crate::application::use_cases::{
    AskCompanionUseCase, IngestPlaylistUseCase, PlanSessionUseCase, TakeExamUseCase,
};
use crate::domain::ports::SecretStore;
use crate::infrastructure::{
    keystore::NativeKeystore,
    llm::GeminiAdapter,
    persistence::{
        DbPool, SqliteCourseRepository, SqliteExamRepository, SqliteModuleRepository,
        SqliteNoteRepository, SqliteSearchRepository, SqliteTagRepository, SqliteVideoRepository,
    },
    youtube::YouTubeApiAdapter,
};

/// Configuration for the application.
///
/// Load from environment with `AppConfig::from_env()`.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Path to SQLite database file.
    pub database_url: String,
    /// YouTube Data API v3 key (required for playlist import).
    pub youtube_api_key: Option<String>,
    /// Gemini API key (optional - for AI companion and exams).
    pub gemini_api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "course_pilot.db".to_string(),
            youtube_api_key: None,
            gemini_api_key: None,
        }
    }
}

impl AppConfig {
    /// Loads configuration from environment variables.
    /// Falls back to defaults if not set.
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "course_pilot.db".to_string()),
            youtube_api_key: std::env::var("YOUTUBE_API_KEY").ok().filter(|s| !s.is_empty()),
            gemini_api_key: std::env::var("GEMINI_API_KEY").ok().filter(|s| !s.is_empty()),
        }
    }

    /// Creates a builder for programmatic configuration.
    pub fn builder() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }
}

/// Builder for AppConfig (useful for GUI).
#[derive(Default)]
pub struct AppConfigBuilder {
    config: AppConfig,
}

impl AppConfigBuilder {
    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = url.into();
        self
    }

    pub fn youtube_api_key(mut self, key: impl Into<String>) -> Self {
        self.config.youtube_api_key = Some(key.into());
        self
    }

    pub fn gemini_api_key(mut self, key: impl Into<String>) -> Self {
        self.config.gemini_api_key = Some(key.into());
        self
    }

    pub fn build(self) -> AppConfig {
        self.config
    }
}

/// Application context holding all wired dependencies.
pub struct AppContext {
    // Configuration
    pub config: AppConfig,

    // Repositories
    pub course_repo: Arc<SqliteCourseRepository>,
    pub module_repo: Arc<SqliteModuleRepository>,
    pub video_repo: Arc<SqliteVideoRepository>,
    pub exam_repo: Arc<SqliteExamRepository>,
    pub note_repo: Arc<SqliteNoteRepository>,
    pub tag_repo: Arc<SqliteTagRepository>,
    pub search_repo: Arc<SqliteSearchRepository>,

    // Infrastructure adapters
    pub youtube: Option<Arc<YouTubeApiAdapter>>,
    pub llm: Option<Arc<GeminiAdapter>>,
    pub keystore: Arc<NativeKeystore>,

    // Database pool
    pub db_pool: Arc<DbPool>,
}

impl AppContext {
    /// Creates a new application context with all dependencies wired.
    pub fn new(config: AppConfig) -> Result<Self, AppContextError> {
        // Initialize database pool
        let db_pool = Arc::new(crate::infrastructure::persistence::establish_connection(
            &config.database_url,
        ));

        // Create repositories
        let course_repo = Arc::new(SqliteCourseRepository::new(db_pool.clone()));
        let module_repo = Arc::new(SqliteModuleRepository::new(db_pool.clone()));
        let video_repo = Arc::new(SqliteVideoRepository::new(db_pool.clone()));
        let exam_repo = Arc::new(SqliteExamRepository::new(db_pool.clone()));
        let note_repo = Arc::new(SqliteNoteRepository::new(db_pool.clone()));
        let tag_repo = Arc::new(SqliteTagRepository::new(db_pool.clone()));
        let search_repo = Arc::new(SqliteSearchRepository::new(db_pool.clone()));

        // Create keystore
        let keystore = Arc::new(NativeKeystore::new());

        // Get API keys from config or keystore
        let youtube_api_key = config
            .youtube_api_key
            .clone()
            .or_else(|| keystore.retrieve("youtube_api_key").ok().flatten());

        let gemini_api_key = config
            .gemini_api_key
            .clone()
            .or_else(|| keystore.retrieve("gemini_api_key").ok().flatten());

        // Create optional adapters based on API key availability
        let youtube = youtube_api_key.map(|key| Arc::new(YouTubeApiAdapter::new(key)));
        let llm = gemini_api_key.map(|key| Arc::new(GeminiAdapter::new(key)));

        Ok(Self {
            config,
            course_repo,
            module_repo,
            video_repo,
            exam_repo,
            note_repo,
            tag_repo,
            search_repo,
            youtube,
            llm,
            keystore,
            db_pool,
        })
    }

    /// Checks if YouTube integration is available.
    pub fn has_youtube(&self) -> bool {
        self.youtube.is_some()
    }

    /// Checks if the LLM is available.
    pub fn has_llm(&self) -> bool {
        self.llm.is_some()
    }

    /// Stores a YouTube API key in the secure keystore.
    pub fn set_youtube_api_key(&mut self, key: &str) -> Result<(), AppContextError> {
        self.keystore
            .store("youtube_api_key", key)
            .map_err(|e| AppContextError::Keystore(e.to_string()))?;
        self.youtube = Some(Arc::new(YouTubeApiAdapter::new(key.to_string())));
        Ok(())
    }

    /// Stores a Gemini API key in the secure keystore.
    pub fn set_gemini_api_key(&mut self, key: &str) -> Result<(), AppContextError> {
        self.keystore
            .store("gemini_api_key", key)
            .map_err(|e| AppContextError::Keystore(e.to_string()))?;
        self.llm = Some(Arc::new(GeminiAdapter::new(key.to_string())));
        Ok(())
    }

    /// Gets user preferences from the database.
    pub fn get_preferences(&self) -> Result<u32, AppContextError> {
        use crate::infrastructure::persistence::models::UserPreferencesRow;
        use crate::schema::user_preferences;
        use diesel::prelude::*;

        let mut conn = self.db_pool.get().map_err(|e| AppContextError::Database(e.to_string()))?;

        let result: Option<UserPreferencesRow> = user_preferences::table
            .find("default")
            .first(&mut conn)
            .optional()
            .map_err(|e| AppContextError::Database(e.to_string()))?;

        match result {
            Some(pref) => Ok(pref.cognitive_limit_minutes as u32),
            None => Ok(45), // Default cognitive limit
        }
    }
}

/// Errors that can occur during context creation.
#[derive(Debug, thiserror::Error)]
pub enum AppContextError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Keystore error: {0}")]
    Keystore(String),
}

/// Service factory for creating use cases with injected dependencies.
pub struct ServiceFactory;

impl ServiceFactory {
    /// Creates the playlist ingestion use case.
    /// Returns None if YouTube is not configured.
    pub fn ingest_playlist(
        ctx: &AppContext,
    ) -> Option<
        IngestPlaylistUseCase<
            YouTubeApiAdapter,
            SqliteCourseRepository,
            SqliteModuleRepository,
            SqliteVideoRepository,
        >,
    > {
        let youtube = ctx.youtube.as_ref()?.clone();

        Some(IngestPlaylistUseCase::new(
            youtube,
            ctx.course_repo.clone(),
            ctx.module_repo.clone(),
            ctx.video_repo.clone(),
        ))
    }

    /// Creates the session planning use case.
    pub fn plan_session(ctx: &AppContext) -> PlanSessionUseCase<SqliteVideoRepository> {
        PlanSessionUseCase::new(ctx.video_repo.clone())
    }

    /// Creates the companion AI use case.
    pub fn ask_companion(
        ctx: &AppContext,
    ) -> Option<
        AskCompanionUseCase<
            GeminiAdapter,
            SqliteVideoRepository,
            SqliteModuleRepository,
            SqliteCourseRepository,
        >,
    > {
        let llm = ctx.llm.as_ref()?.clone();

        Some(AskCompanionUseCase::new(
            llm,
            ctx.video_repo.clone(),
            ctx.module_repo.clone(),
            ctx.course_repo.clone(),
        ))
    }

    /// Creates the exam use case.
    pub fn take_exam(
        ctx: &AppContext,
    ) -> Option<TakeExamUseCase<GeminiAdapter, SqliteVideoRepository, SqliteExamRepository>> {
        let llm = ctx.llm.as_ref()?.clone();

        Some(TakeExamUseCase::new(llm, ctx.video_repo.clone(), ctx.exam_repo.clone()))
    }
}
