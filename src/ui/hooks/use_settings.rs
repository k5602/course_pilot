use crate::storage::settings::{AppSettings, save_app_settings, use_app_settings};
use anyhow::Result;
use dioxus::prelude::*;

/// Settings management hook
#[derive(Clone)]
pub struct SettingsManager;

impl SettingsManager {
    pub async fn load_settings(&self) -> Result<AppSettings> {
        tokio::task::spawn_blocking(move || Ok(use_app_settings()))
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Join error: {}", e)))
    }

    pub async fn save_settings(&self, settings: AppSettings) -> Result<()> {
        tokio::task::spawn_blocking(move || save_app_settings(&settings))
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
        preferences: crate::storage::ImportPreferences,
    ) -> Result<()> {
        let mut settings = self.load_settings().await?;
        settings.import_preferences = preferences;
        self.save_settings(settings).await
    }

    pub async fn get_import_preferences(&self) -> Result<crate::storage::ImportPreferences> {
        let settings = self.load_settings().await?;
        Ok(settings.import_preferences)
    }
}

pub fn use_settings_manager() -> SettingsManager {
    SettingsManager
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
