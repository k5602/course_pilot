pub mod settings_view;
pub mod general_settings;
pub mod api_keys_settings;
pub mod course_defaults_settings;

pub use settings_view::{SettingsView, SettingsTab};
pub use general_settings::GeneralSettings;
pub use api_keys_settings::APIKeysSettings;
pub use course_defaults_settings::CourseDefaultSettings;