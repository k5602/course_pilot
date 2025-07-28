use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings that are persisted locally
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    // API Configuration
    pub youtube_api_key: Option<String>,
    pub gemini_api_key: Option<String>,

    // General Settings
    pub theme: Option<String>,
    pub auto_structure: bool,
    pub notifications_enabled: bool,

    // Course Defaults
    pub default_plan_settings: crate::types::PlanSettings,
    pub auto_create_plan: bool,

    // Analytics Preferences
    pub analytics_enabled: bool,
    pub track_study_time: bool,

    // Clustering Preferences
    pub clustering_preferences: crate::nlp::clustering::ClusteringPreferences,
    pub enable_preference_learning: bool,
    pub enable_ab_testing: bool,

    // Import Preferences
    pub import_preferences: ImportPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportPreferences {
    // Course naming
    pub default_course_prefix: Option<String>,
    pub use_playlist_title: bool,
    pub course_naming_pattern: CourseNamingPattern,

    // Video filtering
    pub skip_short_videos: bool,
    pub min_video_duration_seconds: u64,
    pub skip_long_videos: bool,
    pub max_video_duration_seconds: u64,
    pub quality_preference: VideoQualityPreference,

    // Auto-processing
    pub auto_create_plan: bool,
    pub auto_structure_course: bool,
    pub enable_ai_clustering: bool,

    // Advanced options
    pub preserve_playlist_order: bool,
    pub extract_timestamps: bool,
    pub download_thumbnails: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CourseNamingPattern {
    PlaylistTitle,
    PrefixPlusTitle,
    CustomPattern(String),
    DatePlusTitle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoQualityPreference {
    Any,
    PreferHD,
    RequireHD,
    PreferSD,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // API Configuration
            youtube_api_key: None,
            gemini_api_key: None,

            // General Settings
            theme: Some("corporate".to_string()),
            auto_structure: true,
            notifications_enabled: true,

            // Course Defaults
            default_plan_settings: crate::types::PlanSettings {
                start_date: chrono::Utc::now() + chrono::Duration::days(1),
                sessions_per_week: 3,
                session_length_minutes: 60,
                include_weekends: false,
                advanced_settings: None,
            },
            auto_create_plan: false,

            // Analytics Preferences
            analytics_enabled: true,
            track_study_time: true,

            // Clustering Preferences
            clustering_preferences: crate::nlp::clustering::ClusteringPreferences::default(),
            enable_preference_learning: true,
            enable_ab_testing: false, // Disabled by default for stability

            // Import Preferences
            import_preferences: ImportPreferences::default(),
        }
    }
}

impl Default for ImportPreferences {
    fn default() -> Self {
        Self {
            // Course naming
            default_course_prefix: None,
            use_playlist_title: true,
            course_naming_pattern: CourseNamingPattern::PlaylistTitle,

            // Video filtering
            skip_short_videos: false,
            min_video_duration_seconds: 30,
            skip_long_videos: false,
            max_video_duration_seconds: 3600, // 1 hour
            quality_preference: VideoQualityPreference::Any,

            // Auto-processing
            auto_create_plan: false,
            auto_structure_course: true,
            enable_ai_clustering: true,

            // Advanced options
            preserve_playlist_order: false,
            extract_timestamps: true,
            download_thumbnails: true,
        }
    }
}

impl AppSettings {
    /// Load settings from the persistent storage file
    pub fn load() -> Result<Self> {
        let settings_path = Self::get_settings_path();

        if !settings_path.exists() {
            // Create default settings file
            let default_settings = Self::default();
            default_settings.save()?;
            return Ok(default_settings);
        }

        let contents = fs::read_to_string(&settings_path)?;
        let settings: AppSettings = serde_json::from_str(&contents).unwrap_or_else(|_| {
            log::warn!("Failed to parse settings file, using defaults");
            Self::default()
        });

        Ok(settings)
    }

    /// Save settings to persistent storage
    pub fn save(&self) -> Result<()> {
        let settings_path = Self::get_settings_path();

        // Ensure the directory exists
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&settings_path, json)?;

        log::info!("Settings saved to: {}", settings_path.display());
        Ok(())
    }

    /// Get the path to the settings file
    fn get_settings_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("course_pilot").join("settings.json")
        } else {
            // Fallback to current directory
            PathBuf::from("settings.json")
        }
    }

    /// Update the YouTube API key and save settings
    pub fn set_youtube_api_key(&mut self, api_key: Option<String>) -> Result<()> {
        self.youtube_api_key = api_key;
        self.save()
    }

    /// Get the YouTube API key if available
    pub fn get_youtube_api_key(&self) -> Option<&str> {
        self.youtube_api_key.as_deref()
    }

    /// Update the Gemini API key and save settings
    pub fn set_gemini_api_key(&mut self, api_key: Option<String>) -> Result<()> {
        self.gemini_api_key = api_key;
        self.save()
    }

    /// Get the Gemini API key if available
    pub fn get_gemini_api_key(&self) -> Option<&str> {
        self.gemini_api_key.as_deref()
    }

    /// Update theme and save settings
    pub fn set_theme(&mut self, theme: String) -> Result<()> {
        self.theme = Some(theme);
        self.save()
    }

    /// Get the current theme
    pub fn get_theme(&self) -> &str {
        self.theme.as_deref().unwrap_or("corporate")
    }

    /// Update default plan settings and save
    pub fn set_default_plan_settings(
        &mut self,
        settings: crate::types::PlanSettings,
    ) -> Result<()> {
        self.default_plan_settings = settings;
        self.save()
    }

    /// Get default plan settings
    pub fn get_default_plan_settings(&self) -> &crate::types::PlanSettings {
        &self.default_plan_settings
    }

    /// Update analytics preferences and save
    pub fn set_analytics_enabled(&mut self, enabled: bool) -> Result<()> {
        self.analytics_enabled = enabled;
        self.save()
    }

    /// Update notifications preference and save
    pub fn set_notifications_enabled(&mut self, enabled: bool) -> Result<()> {
        self.notifications_enabled = enabled;
        self.save()
    }

    /// Update clustering preferences and save
    pub fn set_clustering_preferences(
        &mut self,
        preferences: crate::nlp::clustering::ClusteringPreferences,
    ) -> Result<()> {
        self.clustering_preferences = preferences;
        self.save()
    }

    /// Get clustering preferences
    pub fn get_clustering_preferences(&self) -> &crate::nlp::clustering::ClusteringPreferences {
        &self.clustering_preferences
    }

    /// Update preference learning setting and save
    pub fn set_preference_learning_enabled(&mut self, enabled: bool) -> Result<()> {
        self.enable_preference_learning = enabled;
        self.save()
    }

    /// Update A/B testing setting and save
    pub fn set_ab_testing_enabled(&mut self, enabled: bool) -> Result<()> {
        self.enable_ab_testing = enabled;
        self.save()
    }

    /// Update import preferences and save
    pub fn set_import_preferences(&mut self, preferences: ImportPreferences) -> Result<()> {
        self.import_preferences = preferences;
        self.save()
    }

    /// Get import preferences
    pub fn get_import_preferences(&self) -> &ImportPreferences {
        &self.import_preferences
    }
}

/// Settings manager hook for use in Dioxus components
pub fn use_app_settings() -> AppSettings {
    AppSettings::load().unwrap_or_default()
}

/// Save settings hook for use in Dioxus components
pub fn save_app_settings(settings: &AppSettings) -> Result<()> {
    settings.save()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_settings_creation() {
        let settings = AppSettings::default();
        assert!(settings.youtube_api_key.is_none());
        assert_eq!(settings.get_theme(), "corporate");
        assert!(settings.auto_structure);
    }

    #[test]
    fn test_api_key_operations() {
        let mut settings = AppSettings::default();

        // Initially no API key
        assert!(settings.get_youtube_api_key().is_none());

        // Set API key
        settings.youtube_api_key = Some("test-key".to_string());
        assert_eq!(settings.get_youtube_api_key(), Some("test-key"));

        // Clear API key
        settings.youtube_api_key = None;
        assert!(settings.get_youtube_api_key().is_none());
    }
}
