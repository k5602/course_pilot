//! Application Context - Dependency Injection Container
//!
//! Wires all infrastructure adapters to application use cases.

use parking_lot::Mutex;
use std::sync::Arc;

use crate::application::use_cases::{
    AskCompanionUseCase, ChatUseCase, CreateModuleUseCase, DeleteModuleUseCase, IngestLocalUseCase,
    IngestPlaylistUseCase, LoadDashboardUseCase, NotesUseCase, PreferencesUseCase,
    SummarizeVideoUseCase, TakeExamUseCase, UpdatePresenceUseCase,
};
use crate::domain::ports::{
    ChatMessageRepository, CourseRepository, ExamRepository, ModuleRepository,
    ModuleTitleGenerator, NoteRepository, PresenceProvider, SearchRepository, SecretStore,
    TagRepository, UserPreferencesRepository, VideoRepository,
};
use crate::infrastructure::{
    discord::DiscordPresenceAdapter,
    keystore::NativeKeystore,
    llm::GeminiAdapter,
    local_media::LocalMediaScannerAdapter,
    persistence::{
        DbPool, SqliteChatMessageRepository, SqliteCourseRepository, SqliteExamRepository,
        SqliteModuleRepository, SqliteNoteRepository, SqliteSearchRepository, SqliteTagRepository,
        SqliteUserPreferencesRepository, SqliteVideoRepository,
    },
    transcript::TranscriptAdapter,
    youtube::RustyYtdlAdapter,
};

/// Default LLM model used when none is configured.
const DEFAULT_LLM_MODEL: &str = "gemini-3.1-flash-lite";

/// Configuration for the application.
/// Load from environment with `AppConfig::from_env()`.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Path to SQLite database file.
    pub database_url: String,
    /// Gemini API key (optional - for AI companion, exams, and summaries).
    pub gemini_api_key: Option<String>,
    /// Discord Rich Presence client ID (optional).
    pub discord_client_id: Option<String>,
    /// LLM model identifier (default: gemini/gemini-2.5-flash).
    pub llm_model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "course_pilot.db".to_string(),
            gemini_api_key: None,
            discord_client_id: None,
            llm_model: DEFAULT_LLM_MODEL.to_string(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from environment variables.
    /// Falls back to defaults if not set.
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "course_pilot.db".to_string()),
            gemini_api_key: std::env::var("GEMINI_API_KEY")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            discord_client_id: std::env::var("DISCORD_CLIENT_ID")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            llm_model: std::env::var("LLM_MODEL")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| DEFAULT_LLM_MODEL.to_string()),
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

    pub fn gemini_api_key(mut self, key: impl Into<String>) -> Self {
        self.config.gemini_api_key = Some(key.into().trim().to_string());
        self
    }

    pub fn discord_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.config.discord_client_id = Some(client_id.into());
        self
    }

    pub fn llm_model(mut self, model: impl Into<String>) -> Self {
        self.config.llm_model = model.into();
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
    pub course_repo: Arc<dyn CourseRepository>,
    pub module_repo: Arc<dyn ModuleRepository>,
    pub video_repo: Arc<dyn VideoRepository>,
    pub exam_repo: Arc<dyn ExamRepository>,
    pub note_repo: Arc<dyn NoteRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub search_repo: Arc<dyn SearchRepository>,
    pub preferences_repo: Arc<dyn UserPreferencesRepository>,
    pub chat_repo: Arc<dyn ChatMessageRepository>,

    // Infrastructure adapters
    pub local_media: Arc<LocalMediaScannerAdapter>,
    pub youtube: Arc<RustyYtdlAdapter>,
    pub transcript: Arc<TranscriptAdapter>,
    pub llm: Mutex<Option<Arc<GeminiAdapter>>>,
    pub presence: Arc<dyn PresenceProvider>,
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
        )?);

        // Create repositories
        let course_repo = Arc::new(SqliteCourseRepository::new(db_pool.clone()));
        let module_repo = Arc::new(SqliteModuleRepository::new(db_pool.clone()));
        let video_repo = Arc::new(SqliteVideoRepository::new(db_pool.clone()));
        let exam_repo = Arc::new(SqliteExamRepository::new(db_pool.clone()));
        let note_repo = Arc::new(SqliteNoteRepository::new(db_pool.clone()));
        let tag_repo = Arc::new(SqliteTagRepository::new(db_pool.clone()));
        let search_repo = Arc::new(SqliteSearchRepository::new(db_pool.clone()));
        let preferences_repo = Arc::new(SqliteUserPreferencesRepository::new(db_pool.clone()));
        let chat_repo = Arc::new(SqliteChatMessageRepository::new(db_pool.clone()));

        // Create keystore
        let keystore = Arc::new(NativeKeystore::new());

        // Local media scanner (filesystem)
        let local_media = Arc::new(
            LocalMediaScannerAdapter::new()
                .map_err(|e| AppContextError::Database(format!("Local media init failed: {e}")))?,
        );

        // YouTube adapter (always available - no API key needed)
        let cookie_path = keystore.retrieve("youtube_cookies").ok().flatten();
        let youtube = Arc::new(RustyYtdlAdapter::with_cookies(cookie_path));

        // Presence provider (Discord)
        let discord_client_id = config
            .discord_client_id
            .clone()
            .or_else(|| keystore.retrieve("discord_client_id").ok().flatten());
        let presence_adapter = discord_client_id
            .as_deref()
            .map(DiscordPresenceAdapter::new_with_client_id)
            .unwrap_or_default();
        let presence: Arc<dyn PresenceProvider> = Arc::new(presence_adapter);

        // Transcript adapter (for summaries)
        let transcript = Arc::new(
            crate::infrastructure::transcript::TranscriptAdapter::new()
                .map_err(|e| AppContextError::Transcript(e.to_string()))?,
        );

        // Get Gemini API key from environment config first, then secure keystore (ensuring no empty strings)
        let gemini_api_key =
            if let Some(env_key) = config.gemini_api_key.as_ref().filter(|s| !s.is_empty()) {
                log::info!("Gemini API key loaded from environment (.env).");
                Some(env_key.clone())
            } else if let Ok(Some(store_key)) =
                keystore.retrieve("gemini_api_key").map(|opt| opt.filter(|s| !s.is_empty()))
            {
                log::info!("Gemini API key loaded from secure OS keystore.");
                Some(store_key)
            } else {
                log::info!("No Gemini API key found in environment or keystore.");
                None
            };

        // Create LLM adapter if key is available
        let llm = Mutex::new(
            gemini_api_key.map(|key| Arc::new(GeminiAdapter::new(key, config.llm_model.clone()))),
        );

        Ok(Self {
            config,
            course_repo,
            module_repo,
            video_repo,
            exam_repo,
            note_repo,
            tag_repo,
            search_repo,
            preferences_repo,
            chat_repo,
            local_media,
            youtube,
            transcript,
            llm,
            presence,
            keystore,
            db_pool,
        })
    }

    /// Checks if the LLM is available.
    pub fn has_llm(&self) -> bool {
        self.llm.lock().is_some()
    }

    /// Stores a Gemini API key in the secure keystore and reloads the adapter.
    /// Takes `&self` because interior mutability via `Mutex` is used.
    pub fn set_gemini_api_key(&self, key: &str) -> Result<(), AppContextError> {
        let trimmed = key.trim();
        self.keystore
            .store("gemini_api_key", trimmed)
            .map_err(|e| AppContextError::Keystore(e.to_string()))?;
        *self.llm.lock() =
            Some(Arc::new(GeminiAdapter::new(trimmed.to_string(), self.config.llm_model.clone())));
        Ok(())
    }

    /// Reloads the LLM adapter from the keystore or config (for dynamic key updates).
    /// Takes `&self` because interior mutability via `Mutex` is used.
    pub fn reload_llm(&self) -> Result<(), AppContextError> {
        let key_opt =
            self.config.gemini_api_key.as_ref().filter(|s| !s.is_empty()).cloned().or_else(|| {
                self.keystore.retrieve("gemini_api_key").ok().flatten().filter(|s| !s.is_empty())
            });

        if let Some(key) = key_opt {
            let trimmed = key.trim().to_string();
            log::info!("Reloaded Gemini LLM adapter with key.");
            *self.llm.lock() =
                Some(Arc::new(GeminiAdapter::new(trimmed, self.config.llm_model.clone())));
        } else {
            log::info!("Gemini LLM adapter cleared (no key available).");
            *self.llm.lock() = None;
        }
        Ok(())
    }
}

/// Errors that can occur during context creation.
#[derive(Debug, thiserror::Error)]
pub enum AppContextError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Keystore error: {0}")]
    Keystore(String),
    #[error("Transcript error: {0}")]
    Transcript(String),
}

/// Service factory for creating use cases with injected dependencies.
pub struct ServiceFactory;

impl ServiceFactory {
    /// Creates the playlist ingestion use case.
    pub fn ingest_playlist(ctx: &AppContext) -> IngestPlaylistUseCase {
        let batch_size = ServiceFactory::preferences(ctx)
            .load()
            .map(|p| p.boundary_batch_size() as usize)
            .unwrap_or(5);

        IngestPlaylistUseCase::new(
            ctx.youtube.clone(),
            ctx.course_repo.clone(),
            ctx.module_repo.clone(),
            ctx.video_repo.clone(),
            ctx.search_repo.clone(),
            ctx.llm.lock().as_ref().map(|a| Arc::clone(a) as Arc<dyn ModuleTitleGenerator>),
            batch_size,
        )
    }

    /// Creates the local library ingestion use case.
    pub fn ingest_local(ctx: &AppContext) -> IngestLocalUseCase {
        let batch_size = ServiceFactory::preferences(ctx)
            .load()
            .map(|p| p.boundary_batch_size() as usize)
            .unwrap_or(5);

        IngestLocalUseCase::new(
            ctx.local_media.clone(),
            ctx.course_repo.clone(),
            ctx.module_repo.clone(),
            ctx.video_repo.clone(),
            ctx.search_repo.clone(),
            ctx.llm.lock().as_ref().map(|a| Arc::clone(a) as Arc<dyn ModuleTitleGenerator>),
            batch_size,
        )
    }

    /// Creates the presence update use case.
    pub fn update_presence(ctx: &AppContext) -> UpdatePresenceUseCase {
        UpdatePresenceUseCase::new(ctx.presence.clone())
    }

    /// Creates the companion AI use case.
    pub fn ask_companion(ctx: &AppContext) -> Option<AskCompanionUseCase> {
        let llm = ctx.llm.lock().as_ref()?.clone();

        Some(AskCompanionUseCase::new(
            llm,
            ctx.video_repo.clone(),
            ctx.module_repo.clone(),
            ctx.course_repo.clone(),
            ctx.note_repo.clone(),
        ))
    }

    /// Creates the chat use case.
    pub fn chat(ctx: &AppContext) -> ChatUseCase {
        ChatUseCase::new(ctx.chat_repo.clone(), ctx.video_repo.clone())
    }

    /// Creates the notes use case.
    pub fn notes(ctx: &AppContext) -> NotesUseCase {
        NotesUseCase::new(
            ctx.note_repo.clone(),
            ctx.video_repo.clone(),
            ctx.module_repo.clone(),
            ctx.course_repo.clone(),
            ctx.tag_repo.clone(),
            ctx.search_repo.clone(),
        )
    }

    /// Creates the update module title use case.
    pub fn update_module_title(
        ctx: &AppContext,
    ) -> crate::application::use_cases::UpdateModuleTitleUseCase {
        crate::application::use_cases::UpdateModuleTitleUseCase::new(ctx.module_repo.clone())
    }

    /// Creates the create module use case.
    pub fn create_module(ctx: &AppContext) -> CreateModuleUseCase {
        CreateModuleUseCase::new(ctx.module_repo.clone())
    }

    /// Creates the delete module use case.
    pub fn delete_module(ctx: &AppContext) -> DeleteModuleUseCase {
        DeleteModuleUseCase::new(ctx.module_repo.clone(), ctx.video_repo.clone())
    }

    /// Creates the move video use case.
    pub fn move_video_to_module(
        ctx: &AppContext,
    ) -> crate::application::use_cases::MoveVideoToModuleUseCase {
        crate::application::use_cases::MoveVideoToModuleUseCase::new(ctx.video_repo.clone())
    }

    /// Creates the dashboard analytics use case.
    pub fn dashboard(ctx: &AppContext) -> LoadDashboardUseCase {
        LoadDashboardUseCase::new(
            ctx.course_repo.clone(),
            ctx.module_repo.clone(),
            ctx.video_repo.clone(),
        )
    }

    /// Creates the preferences use case.
    pub fn preferences(ctx: &AppContext) -> PreferencesUseCase {
        PreferencesUseCase::new(ctx.preferences_repo.clone())
    }

    /// Creates the summarize video use case.
    pub fn summarize_video(ctx: &AppContext) -> Option<SummarizeVideoUseCase> {
        let llm = ctx.llm.lock().as_ref()?.clone();

        Some(SummarizeVideoUseCase::new(llm, ctx.transcript.clone(), ctx.video_repo.clone()))
    }

    /// Creates the exam use case.
    pub fn take_exam(ctx: &AppContext) -> Option<TakeExamUseCase> {
        let llm = ctx.llm.lock().as_ref()?.clone();

        Some(TakeExamUseCase::new(llm, ctx.video_repo.clone(), ctx.exam_repo.clone()))
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::Mutex;
    use std::sync::Arc;

    use crate::infrastructure::llm::GeminiAdapter;

    #[test]
    fn mutex_llm_starts_none_and_becomes_some_after_set() {
        let llm: Mutex<Option<Arc<GeminiAdapter>>> = Mutex::new(None);
        assert!(llm.lock().is_none(), "LLM should start as None");

        *llm.lock() = Some(Arc::new(GeminiAdapter::new(
            "test-key".to_string(),
            "gemini-3.1-flash-lite".to_string(),
        )));
        assert!(llm.lock().is_some(), "LLM should be Some after set");
    }

    #[test]
    fn mutex_llm_can_be_reset_to_none() {
        let llm: Mutex<Option<Arc<GeminiAdapter>>> = Mutex::new(Some(Arc::new(
            GeminiAdapter::new("test-key".to_string(), "gemini-3.1-flash-lite".to_string()),
        )));
        assert!(llm.lock().is_some(), "LLM should start as Some");

        *llm.lock() = None;
        assert!(llm.lock().is_none(), "LLM should be None after reset");
    }

    #[test]
    fn appconfig_from_env_defaults() {
        let cfg = super::AppConfig::default();
        assert!(cfg.gemini_api_key.is_none());
        assert_eq!(cfg.llm_model, "gemini-3.1-flash-lite");
    }
}
