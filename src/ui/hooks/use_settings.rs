use crate::storage::settings::AppSettings;
use anyhow::Result;
use dioxus::prelude::*;

/// Settings management hook with improved async patterns
#[derive(Clone)]
pub struct SettingsManager;

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_settings(&self) -> Result<AppSettings> {
        tokio::task::spawn_blocking(|| Ok(crate::storage::settings::use_app_settings()))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn save_settings(&self, settings: AppSettings) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            crate::storage::settings::save_app_settings(&settings)
                .map_err(|e| anyhow::anyhow!("Settings error: {}", e))
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn get_youtube_api_key(&self) -> Result<Option<String>> {
        let settings = self.load_settings().await?;
        Ok(settings.youtube_api_key)
    }

    pub async fn set_youtube_api_key(&self, api_key: Option<String>) -> Result<()> {
        let mut settings = self.load_settings().await?;
        settings.youtube_api_key = api_key;
        self.save_settings(settings).await
    }

    pub async fn get_gemini_api_key(&self) -> Result<Option<String>> {
        let settings = self.load_settings().await?;
        Ok(settings.gemini_api_key)
    }

    pub async fn set_gemini_api_key(&self, api_key: Option<String>) -> Result<()> {
        let mut settings = self.load_settings().await?;
        settings.gemini_api_key = api_key;
        self.save_settings(settings).await
    }

    pub async fn reset_settings(&self) -> Result<()> {
        let default_settings = AppSettings::default();
        self.save_settings(default_settings).await
    }

    pub async fn set_import_preferences(
        &self,
        preferences: crate::storage::settings::ImportPreferences,
    ) -> Result<()> {
        let mut settings = self.load_settings().await?;
        settings.import_preferences = preferences;
        self.save_settings(settings).await
    }

    pub async fn get_import_preferences(
        &self,
    ) -> Result<crate::storage::settings::ImportPreferences> {
        let settings = self.load_settings().await?;
        Ok(settings.import_preferences)
    }

    pub async fn set_theme(&self, theme: String) -> Result<()> {
        let mut settings = self.load_settings().await?;
        settings.theme = Some(theme);
        self.save_settings(settings).await
    }

    pub async fn set_analytics_enabled(&self, enabled: bool) -> Result<()> {
        let mut settings = self.load_settings().await?;
        settings.analytics_enabled = enabled;
        self.save_settings(settings).await
    }

    pub async fn set_notifications_enabled(&self, enabled: bool) -> Result<()> {
        let mut settings = self.load_settings().await?;
        settings.notifications_enabled = enabled;
        self.save_settings(settings).await
    }

    /// Batch update multiple settings efficiently
    pub async fn update_settings<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppSettings) -> Result<()> + Send + 'static,
    {
        let mut settings = self.load_settings().await?;
        updater(&mut settings)?;
        self.save_settings(settings).await
    }
}

pub fn use_settings_manager() -> SettingsManager {
    SettingsManager::new()
}

/// Hook for reactive settings loading
pub fn use_settings_resource() -> Resource<Result<AppSettings, anyhow::Error>> {
    let settings_manager = use_settings_manager();

    use_resource(move || {
        let settings_manager = settings_manager.clone();
        async move { settings_manager.load_settings().await }
    })
}

/// Hook for API key management
pub fn use_api_key_manager() -> (
    Resource<Result<Option<String>, anyhow::Error>>, // youtube_key
    Resource<Result<Option<String>, anyhow::Error>>, // gemini_key
) {
    let settings_manager = use_settings_manager();

    let youtube_key = use_resource({
        let settings_manager = settings_manager.clone();
        move || {
            let settings_manager = settings_manager.clone();
            async move { settings_manager.get_youtube_api_key().await }
        }
    });

    let gemini_key = use_resource({
        let settings_manager = settings_manager.clone();
        move || {
            let settings_manager = settings_manager.clone();
            async move { settings_manager.get_gemini_api_key().await }
        }
    });

    (youtube_key, gemini_key)
}
