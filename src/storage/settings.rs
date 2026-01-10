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

    // Advanced options
    pub preserve_playlist_order: bool,
    pub extract_timestamps: bool,
    pub download_thumbnails: bool,

    // Preview performance
    pub preview_probe_max_concurrency: usize,
    pub preview_cancellation_enabled: bool,
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
            youtube_api_key: None,
            gemini_api_key: None,
            theme: Some("corporate".to_string()),
            auto_structure: true,
            notifications_enabled: true,
            default_plan_settings: crate::types::PlanSettings {
                start_date: chrono::Utc::now() + chrono::Duration::days(1),
                sessions_per_week: 3,
                session_length_minutes: 60,
                include_weekends: false,
                advanced_settings: None,
            },
            auto_create_plan: false,
            analytics_enabled: true,
            track_study_time: true,
            import_preferences: ImportPreferences::default(),
        }
    }
}

impl Default for ImportPreferences {
    fn default() -> Self {
        Self {
            default_course_prefix: None,
            use_playlist_title: true,
            course_naming_pattern: CourseNamingPattern::PlaylistTitle,
            skip_short_videos: false,
            min_video_duration_seconds: 30,
            skip_long_videos: false,
            max_video_duration_seconds: 3600,
            quality_preference: VideoQualityPreference::Any,
            auto_create_plan: false,
            auto_structure_course: true,
            preserve_playlist_order: false,
            extract_timestamps: true,
            download_thumbnails: true,
            preview_probe_max_concurrency: 8,
            preview_cancellation_enabled: true,
        }
    }
}

impl AppSettings {
    pub fn load() -> Result<Self> {
        let settings_path = Self::get_settings_path();
        if !settings_path.exists() {
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

    pub fn save(&self) -> Result<()> {
        let settings_path = Self::get_settings_path();
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&settings_path, json)?;
        log::info!("Settings saved to: {}", settings_path.display());
        Ok(())
    }

    fn get_settings_path() -> PathBuf {
        dirs::config_dir()
            .map(|d| d.join("course_pilot").join("settings.json"))
            .unwrap_or_else(|| PathBuf::from("settings.json"))
    }

    pub fn set_youtube_api_key(&mut self, api_key: Option<String>) -> Result<()> {
        self.youtube_api_key = api_key;
        self.save()
    }

    pub fn get_youtube_api_key(&self) -> Option<&str> {
        self.youtube_api_key.as_deref()
    }

    pub fn set_gemini_api_key(&mut self, api_key: Option<String>) -> Result<()> {
        self.gemini_api_key = api_key;
        self.save()
    }

    pub fn get_gemini_api_key(&self) -> Option<&str> {
        self.gemini_api_key.as_deref()
    }

    pub fn set_theme(&mut self, theme: String) -> Result<()> {
        self.theme = Some(theme);
        self.save()
    }

    pub fn get_theme(&self) -> &str {
        self.theme.as_deref().unwrap_or("corporate")
    }

    pub fn set_default_plan_settings(
        &mut self,
        settings: crate::types::PlanSettings,
    ) -> Result<()> {
        self.default_plan_settings = settings;
        self.save()
    }

    pub fn get_default_plan_settings(&self) -> &crate::types::PlanSettings {
        &self.default_plan_settings
    }

    pub fn set_import_preferences(&mut self, preferences: ImportPreferences) -> Result<()> {
        self.import_preferences = preferences;
        self.save()
    }

    pub fn get_import_preferences(&self) -> &ImportPreferences {
        &self.import_preferences
    }
}

pub fn use_app_settings() -> AppSettings {
    AppSettings::load().unwrap_or_default()
}

pub fn save_app_settings(settings: &AppSettings) -> Result<()> {
    settings.save()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_creation() {
        let settings = AppSettings::default();
        assert!(settings.youtube_api_key.is_none());
        assert_eq!(settings.get_theme(), "corporate");
    }
}
