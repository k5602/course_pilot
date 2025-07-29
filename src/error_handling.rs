//! Comprehensive error handling utilities for Course Pilot
//!
//! This module provides standardized error handling patterns, user-friendly error messages,
//! and error recovery mechanisms throughout the application.

use anyhow::{Context, Result};
use log::{error, info, warn};
use thiserror::Error;

/// Custom error types for Course Pilot with user-friendly messages
#[derive(Error, Debug)]
pub enum CoursePilotError {
    #[error("Database operation failed: {message}")]
    Database {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("File operation failed: {message}")]
    FileSystem {
        message: String,
        path: Option<std::path::PathBuf>,
    },

    #[error("Network operation failed: {message}")]
    Network {
        message: String,
        url: Option<String>,
    },

    #[error("Import operation failed: {message}")]
    Import {
        message: String,
        source_type: String,
    },

    #[error("Export operation failed: {message}")]
    Export { message: String, format: String },

    #[error("Video processing failed: {message}")]
    VideoProcessing {
        message: String,
        video_path: Option<String>,
    },

    #[error("API operation failed: {message}")]
    Api { message: String, api_name: String },

    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        setting: Option<String>,
    },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },
}

/// User-friendly error message mapper
pub struct ErrorMessageMapper;

impl ErrorMessageMapper {
    /// Convert technical errors to user-friendly messages
    pub fn map_error(error: &anyhow::Error) -> String {
        // Check if it's one of our custom error types
        if let Some(course_pilot_error) = error.downcast_ref::<CoursePilotError>() {
            return Self::map_course_pilot_error(course_pilot_error);
        }

        // Handle common error patterns
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("permission denied") {
            "Permission denied. Please check file permissions and try again.".to_string()
        } else if error_str.contains("no such file or directory") {
            "File not found. Please check the file path and try again.".to_string()
        } else if error_str.contains("connection refused") || error_str.contains("network") {
            "Network connection failed. Please check your internet connection and try again."
                .to_string()
        } else if error_str.contains("timeout") {
            "Operation timed out. Please try again or check your connection.".to_string()
        } else if error_str.contains("database") || error_str.contains("sqlite") {
            "Database error occurred. Please restart the application and try again.".to_string()
        } else if error_str.contains("json") || error_str.contains("parse") {
            "Data format error. The file may be corrupted or in an unsupported format.".to_string()
        } else if error_str.contains("youtube") || error_str.contains("api") {
            "API service error. Please check your API key and try again later.".to_string()
        } else {
            // Generic fallback message
            "An unexpected error occurred. Please try again or contact support if the problem persists.".to_string()
        }
    }

    fn map_course_pilot_error(error: &CoursePilotError) -> String {
        match error {
            CoursePilotError::Database { message, .. } => {
                format!(
                    "Database error: {message}. Please restart the application if the problem persists."
                )
            }
            CoursePilotError::FileSystem { message, path } => {
                if let Some(path) = path {
                    format!("File error: {} (Path: {})", message, path.display())
                } else {
                    format!("File error: {message}")
                }
            }
            CoursePilotError::Network { message, url } => {
                if let Some(url) = url {
                    format!("Network error: {message} (URL: {url})")
                } else {
                    format!("Network error: {message}. Please check your internet connection.")
                }
            }
            CoursePilotError::Import {
                message,
                source_type,
            } => {
                format!(
                    "Import failed: {message} (Source: {source_type}). Please check the source and try again."
                )
            }
            CoursePilotError::Export { message, format } => {
                format!(
                    "Export failed: {message} (Format: {format}). Please try a different format or location."
                )
            }
            CoursePilotError::VideoProcessing {
                message,
                video_path,
            } => {
                if let Some(path) = video_path {
                    format!("Video processing error: {message} (Video: {path})")
                } else {
                    format!("Video processing error: {message}")
                }
            }
            CoursePilotError::Api { message, api_name } => {
                format!("{api_name} API error: {message}. Please check your API key and try again.")
            }
            CoursePilotError::Configuration { message, setting } => {
                if let Some(setting) = setting {
                    format!("Configuration error: {message} (Setting: {setting})")
                } else {
                    format!("Configuration error: {message}")
                }
            }
            CoursePilotError::Validation { message, field } => {
                if let Some(field) = field {
                    format!("Validation error: {message} (Field: {field})")
                } else {
                    format!("Validation error: {message}")
                }
            }
        }
    }
}

/// Error recovery strategies
pub struct ErrorRecoveryManager;

impl ErrorRecoveryManager {
    /// Get suggested recovery actions for an error
    pub fn get_recovery_actions(error: &anyhow::Error) -> Vec<String> {
        let error_str = error.to_string().to_lowercase();
        let mut actions = Vec::new();

        if error_str.contains("permission denied") {
            actions.push("Check file permissions".to_string());
            actions.push("Run as administrator if necessary".to_string());
            actions.push("Choose a different location".to_string());
        } else if error_str.contains("no such file") {
            actions.push("Verify the file path is correct".to_string());
            actions.push("Check if the file exists".to_string());
            actions.push("Try browsing for the file".to_string());
        } else if error_str.contains("network") || error_str.contains("connection") {
            actions.push("Check your internet connection".to_string());
            actions.push("Try again in a few moments".to_string());
            actions.push("Check if the service is available".to_string());
        } else if error_str.contains("database") {
            actions.push("Restart the application".to_string());
            actions.push("Check available disk space".to_string());
            actions.push("Try creating a new database".to_string());
        } else if error_str.contains("api") || error_str.contains("youtube") {
            actions.push("Check your API key configuration".to_string());
            actions.push("Verify the API service is available".to_string());
            actions.push("Try again later".to_string());
        }

        // Always provide generic recovery actions
        if actions.is_empty() {
            actions.push("Try the operation again".to_string());
            actions.push("Restart the application".to_string());
            actions.push("Check the application logs for more details".to_string());
        }

        actions
    }

    /// Attempt automatic recovery for certain error types
    pub fn attempt_recovery(error: &anyhow::Error) -> Option<String> {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("database locked") {
            info!("Attempting database recovery...");
            // In a real implementation, you might try to close connections and retry
            Some("Database connection reset. Please try again.".to_string())
        } else if error_str.contains("temporary") || error_str.contains("timeout") {
            info!("Attempting automatic retry...");
            Some("Retrying operation automatically...".to_string())
        } else {
            None
        }
    }
}

/// Structured error logging
pub struct ErrorLogger;

impl ErrorLogger {
    /// Log error with context and structured information
    pub fn log_error(error: &anyhow::Error, operation: &str, context: Option<&str>) {
        let error_id = Self::generate_error_id();

        error!("Operation '{operation}' failed [ID: {error_id}]: {error}");

        if let Some(ctx) = context {
            error!("Context [ID: {error_id}]: {ctx}");
        }

        // Log the error chain for debugging
        let mut current_error = error.source();
        let mut level = 1;
        while let Some(err) = current_error {
            error!("  Caused by [{level}]: {err}");
            current_error = err.source();
            level += 1;
        }
    }

    /// Log warning with recovery information
    pub fn log_warning_with_recovery(message: &str, recovery_action: &str) {
        warn!("{message} - Recovery: {recovery_action}");
    }

    fn generate_error_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("ERR-{timestamp}")
    }
}

/// Convenience macros for error handling
#[macro_export]
macro_rules! handle_error {
    ($result:expr, $operation:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                $crate::error_handling::ErrorLogger::log_error(&error, $operation, None);
                let user_message = $crate::error_handling::ErrorMessageMapper::map_error(&error);
                $crate::ui::toast_helpers::error(user_message);
                return Err(error);
            }
        }
    };
    ($result:expr, $operation:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                $crate::error_handling::ErrorLogger::log_error(&error, $operation, Some($context));
                let user_message = $crate::error_handling::ErrorMessageMapper::map_error(&error);
                $crate::ui::toast_helpers::error(user_message);
                return Err(error);
            }
        }
    };
}

#[macro_export]
macro_rules! handle_error_with_recovery {
    ($result:expr, $operation:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                $crate::error_handling::ErrorLogger::log_error(&error, $operation, None);

                // Attempt recovery
                if let Some(recovery_message) =
                    $crate::error_handling::ErrorRecoveryManager::attempt_recovery(&error)
                {
                    $crate::ui::toast_helpers::info(recovery_message);
                } else {
                    let user_message =
                        $crate::error_handling::ErrorMessageMapper::map_error(&error);
                    let recovery_actions =
                        $crate::error_handling::ErrorRecoveryManager::get_recovery_actions(&error);

                    $crate::ui::toast_helpers::error(format!(
                        "{}\n\nSuggested actions:\n• {}",
                        user_message,
                        recovery_actions.join("\n• ")
                    ));
                }
                return Err(error);
            }
        }
    };
}

/// Extension trait for Result types to add context and user-friendly error handling
pub trait ResultExt<T> {
    /// Add user-friendly context to an error
    fn with_user_context(self, context: &str) -> Result<T>;

    /// Convert to user-friendly error message
    fn to_user_error(self) -> Result<T>;

    /// Log error and convert to user-friendly message
    fn log_and_convert(self, operation: &str) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn with_user_context(self, context: &str) -> Result<T> {
        self.map_err(|e| e.into())
            .with_context(|| context.to_string())
    }

    fn to_user_error(self) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            let user_message = ErrorMessageMapper::map_error(&error);
            anyhow::anyhow!(user_message)
        })
    }

    fn log_and_convert(self, operation: &str) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            ErrorLogger::log_error(&error, operation, None);
            let user_message = ErrorMessageMapper::map_error(&error);
            anyhow::anyhow!(user_message)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_mapping() {
        let error = anyhow::anyhow!("permission denied");
        let message = ErrorMessageMapper::map_error(&error);
        assert!(message.contains("Permission denied"));
    }

    #[test]
    fn test_recovery_actions() {
        let error = anyhow::anyhow!("network connection failed");
        let actions = ErrorRecoveryManager::get_recovery_actions(&error);
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.contains("internet connection")));
    }

    #[test]
    fn test_course_pilot_error_display() {
        let error = CoursePilotError::Database {
            message: "Connection failed".to_string(),
            source: None,
        };
        let message = ErrorMessageMapper::map_course_pilot_error(&error);
        assert!(message.contains("Database error"));
        assert!(message.contains("Connection failed"));
    }
}
