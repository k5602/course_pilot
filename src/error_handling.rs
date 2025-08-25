//! Comprehensive error handling utilities for Course Pilot
//!
//! This module provides standardized error handling patterns, user-friendly error messages,
//! and error recovery mechanisms throughout the application.

use anyhow::{Context, Result};
use log::{error, info, warn};
use std::time::Duration;
use thiserror::Error;

/// Domain-specific error types for Course Pilot
#[derive(Error, Debug)]
pub enum CourseError {
    #[error("Course not found: {id}")]
    NotFound { id: String },

    #[error("Course validation failed: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Course creation failed: {message}")]
    CreationFailed { message: String },

    #[error("Course update failed: {message}")]
    UpdateFailed { message: String },

    #[error("Course deletion failed: {message}")]
    DeletionFailed { message: String },

    #[error("Course structure invalid: {message}")]
    StructureInvalid { message: String },

    #[error("Course metadata error: {message}")]
    MetadataError { message: String },
}

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Import source invalid: {source_name} - {message}")]
    InvalidSource { source_name: String, message: String },

    #[error("Import authentication failed: {service}")]
    AuthenticationFailed { service: String },

    #[error("Import rate limited: {service} - retry after {retry_after_seconds}s")]
    RateLimited { service: String, retry_after_seconds: u64 },

    #[error("Import parsing failed: {message}")]
    ParsingFailed { message: String },

    #[error("Import network error: {message}")]
    NetworkError { message: String },

    #[error("Import file system error: {path} - {message}")]
    FileSystemError { path: String, message: String },

    #[error("Import cancelled by user")]
    Cancelled,

    #[error("Import timeout: operation took longer than {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },
}

#[derive(Error, Debug)]
pub enum NlpError {
    #[error("NLP processing failed: {message}")]
    ProcessingFailed { message: String },

    #[error("NLP model loading failed: {model_name} - {message}")]
    ModelLoadFailed { model_name: String, message: String },

    #[error("NLP clustering failed: {algorithm} - {message}")]
    ClusteringFailed { algorithm: String, message: String },

    #[error("NLP content analysis failed: {content_type} - {message}")]
    ContentAnalysisFailed { content_type: String, message: String },

    #[error("NLP insufficient data: need at least {required} items, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    #[error("NLP configuration error: {setting} - {message}")]
    ConfigurationError { setting: String, message: String },
}

#[derive(Error, Debug)]
pub enum PlanError {
    #[error("Plan generation failed: {message}")]
    GenerationFailed { message: String },

    #[error("Plan validation failed: {constraint} - {message}")]
    ValidationFailed { constraint: String, message: String },

    #[error("Plan scheduling conflict: {message}")]
    SchedulingConflict { message: String },

    #[error("Plan optimization failed: {optimizer} - {message}")]
    OptimizationFailed { optimizer: String, message: String },

    #[error("Plan not found: {id}")]
    NotFound { id: String },

    #[error("Plan update failed: {message}")]
    UpdateFailed { message: String },

    #[error("Plan execution error: {session_id} - {message}")]
    ExecutionError { session_id: String, message: String },
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Database query failed: {query} - {message}")]
    QueryFailed { query: String, message: String },

    #[error("Database transaction failed: {message}")]
    TransactionFailed { message: String },

    #[error("Database migration failed: {version} - {message}")]
    MigrationFailed { version: String, message: String },

    #[error("Database constraint violation: {constraint} - {message}")]
    ConstraintViolation { constraint: String, message: String },

    #[error("Database corruption detected: {table} - {message}")]
    CorruptionDetected { table: String, message: String },

    #[error("Database lock timeout: waited {timeout_seconds}s")]
    LockTimeout { timeout_seconds: u64 },

    #[error("Database pool exhausted: {active_connections}/{max_connections}")]
    PoolExhausted { active_connections: u32, max_connections: u32 },

    // Legacy compatibility variants
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("Data not found: {0}")]
    NotFound(String),
}

/// Unified error type that encompasses all domain errors
#[derive(Error, Debug)]
pub enum CoursePilotError {
    #[error("Course error: {0}")]
    Course(#[from] CourseError),

    #[error("Import error: {0}")]
    Import(#[from] ImportError),

    #[error("NLP error: {0}")]
    Nlp(#[from] NlpError),

    #[error("Plan error: {0}")]
    Plan(#[from] PlanError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("File operation failed: {message}")]
    FileSystem { message: String, path: Option<std::path::PathBuf> },

    #[error("Network operation failed: {message}")]
    Network { message: String, url: Option<String> },

    #[error("Video processing failed: {message}")]
    VideoProcessing { message: String, video_path: Option<String> },

    #[error("API operation failed: {message}")]
    Api { message: String, api_name: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String, setting: Option<String> },

    #[error("Validation error: {message}")]
    Validation { message: String, field: Option<String> },
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
            CoursePilotError::Course(course_error) => Self::map_course_error(course_error),
            CoursePilotError::Import(import_error) => Self::map_import_error(import_error),
            CoursePilotError::Nlp(nlp_error) => Self::map_nlp_error(nlp_error),
            CoursePilotError::Plan(plan_error) => Self::map_plan_error(plan_error),
            CoursePilotError::Database(db_error) => Self::map_database_error(db_error),
            CoursePilotError::FileSystem { message, path } => {
                if let Some(path) = path {
                    format!("File error: {} (Path: {})", message, path.display())
                } else {
                    format!("File error: {message}")
                }
            },
            CoursePilotError::Network { message, url } => {
                if let Some(url) = url {
                    format!("Network error: {message} (URL: {url})")
                } else {
                    format!("Network error: {message}. Please check your internet connection.")
                }
            },
            CoursePilotError::VideoProcessing { message, video_path } => {
                if let Some(path) = video_path {
                    format!("Video processing error: {message} (Video: {path})")
                } else {
                    format!("Video processing error: {message}")
                }
            },
            CoursePilotError::Api { message, api_name } => {
                format!("{api_name} API error: {message}. Please check your API key and try again.")
            },
            CoursePilotError::Configuration { message, setting } => {
                if let Some(setting) = setting {
                    format!("Configuration error: {message} (Setting: {setting})")
                } else {
                    format!("Configuration error: {message}")
                }
            },
            CoursePilotError::Validation { message, field } => {
                if let Some(field) = field {
                    format!("Validation error: {message} (Field: {field})")
                } else {
                    format!("Validation error: {message}")
                }
            },
        }
    }

    fn map_course_error(error: &CourseError) -> String {
        match error {
            CourseError::NotFound { id } => {
                format!("Course not found. The course '{id}' may have been deleted or moved.")
            },
            CourseError::Validation { field, message } => {
                format!("Course validation failed: {message} (Field: {field})")
            },
            CourseError::CreationFailed { message } => {
                format!("Failed to create course: {message}. Please try again.")
            },
            CourseError::UpdateFailed { message } => {
                format!("Failed to update course: {message}. Please try again.")
            },
            CourseError::DeletionFailed { message } => {
                format!("Failed to delete course: {message}. Please try again.")
            },
            CourseError::StructureInvalid { message } => {
                format!("Course structure is invalid: {message}. Please restructure the course.")
            },
            CourseError::MetadataError { message } => {
                format!("Course metadata error: {message}. Please check the course information.")
            },
        }
    }

    fn map_import_error(error: &ImportError) -> String {
        match error {
            ImportError::InvalidSource { source_name, message } => {
                format!(
                    "Invalid import source '{source_name}': {message}. Please check the URL or path."
                )
            },
            ImportError::AuthenticationFailed { service } => {
                format!(
                    "Authentication failed for {service}. Please check your API key or login credentials."
                )
            },
            ImportError::RateLimited { service, retry_after_seconds } => {
                format!(
                    "Rate limited by {service}. Please wait {retry_after_seconds} seconds before trying again."
                )
            },
            ImportError::ParsingFailed { message } => {
                format!(
                    "Failed to parse import data: {message}. The source may be in an unsupported format."
                )
            },
            ImportError::NetworkError { message } => {
                format!(
                    "Network error during import: {message}. Please check your internet connection."
                )
            },
            ImportError::FileSystemError { path, message } => {
                format!("File system error: {message} (Path: {path})")
            },
            ImportError::Cancelled => "Import was cancelled by user.".to_string(),
            ImportError::Timeout { timeout_seconds } => {
                format!(
                    "Import timed out after {timeout_seconds} seconds. Please try again or check your connection."
                )
            },
        }
    }

    fn map_nlp_error(error: &NlpError) -> String {
        match error {
            NlpError::ProcessingFailed { message } => {
                format!("Content analysis failed: {message}. Please try again.")
            },
            NlpError::ModelLoadFailed { model_name, message } => {
                format!("Failed to load analysis model '{model_name}': {message}")
            },
            NlpError::ClusteringFailed { algorithm, message } => {
                format!("Content clustering failed using {algorithm}: {message}")
            },
            NlpError::ContentAnalysisFailed { content_type, message } => {
                format!("Failed to analyze {content_type} content: {message}")
            },
            NlpError::InsufficientData { required, actual } => {
                format!(
                    "Not enough content to analyze. Need at least {required} items, but only {actual} available."
                )
            },
            NlpError::ConfigurationError { setting, message } => {
                format!("Analysis configuration error: {message} (Setting: {setting})")
            },
        }
    }

    fn map_plan_error(error: &PlanError) -> String {
        match error {
            PlanError::GenerationFailed { message } => {
                format!("Failed to generate study plan: {message}. Please try again.")
            },
            PlanError::ValidationFailed { constraint, message } => {
                format!("Study plan validation failed: {message} (Constraint: {constraint})")
            },
            PlanError::SchedulingConflict { message } => {
                format!("Scheduling conflict: {message}. Please adjust your preferences.")
            },
            PlanError::OptimizationFailed { optimizer, message } => {
                format!("Plan optimization failed using {optimizer}: {message}")
            },
            PlanError::NotFound { id } => {
                format!("Study plan not found. The plan '{id}' may have been deleted.")
            },
            PlanError::UpdateFailed { message } => {
                format!("Failed to update study plan: {message}. Please try again.")
            },
            PlanError::ExecutionError { session_id, message } => {
                format!("Error executing study session '{session_id}': {message}")
            },
        }
    }

    fn map_database_error(error: &DatabaseError) -> String {
        match error {
            DatabaseError::ConnectionFailed { message } => {
                format!("Database connection failed: {message}. Please restart the application.")
            },
            DatabaseError::QueryFailed { query: _, message } => {
                format!("Database operation failed: {message}. Please try again.")
            },
            DatabaseError::TransactionFailed { message } => {
                format!("Database transaction failed: {message}. Changes may not have been saved.")
            },
            DatabaseError::MigrationFailed { version, message } => {
                format!("Database migration to version {version} failed: {message}")
            },
            DatabaseError::ConstraintViolation { constraint: _, message } => {
                format!("Data validation error: {message}")
            },
            DatabaseError::CorruptionDetected { table, message } => {
                format!(
                    "Database corruption detected in {table}: {message}. Please backup and restore your data."
                )
            },
            DatabaseError::LockTimeout { timeout_seconds } => {
                format!(
                    "Database operation timed out after {timeout_seconds} seconds. Please try again."
                )
            },
            DatabaseError::PoolExhausted { active_connections, max_connections } => {
                format!(
                    "Too many database operations in progress ({active_connections}/{max_connections}). Please wait and try again."
                )
            },
            // Legacy compatibility variants
            DatabaseError::Sqlite(e) => {
                format!("Database error: {}. Please try again.", e)
            },
            DatabaseError::Serialization(e) => {
                format!("Data format error: {}. The data may be corrupted.", e)
            },
            DatabaseError::Io(e) => {
                format!("File system error: {}. Please check permissions and disk space.", e)
            },
            DatabaseError::Pool(e) => {
                format!("Database connection pool error: {}. Please restart the application.", e)
            },
            DatabaseError::NotFound(message) => {
                format!("Data not found: {}. The item may have been deleted.", message)
            },
        }
    }
}

/// Error recovery strategies with retry logic and exponential backoff
pub struct ErrorRecoveryManager;

impl ErrorRecoveryManager {
    /// Get suggested recovery actions for an error
    pub fn get_recovery_actions(error: &anyhow::Error) -> Vec<String> {
        // Check for domain-specific errors first
        if let Some(course_pilot_error) = error.downcast_ref::<CoursePilotError>() {
            return Self::get_domain_specific_actions(course_pilot_error);
        }

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

    fn get_domain_specific_actions(error: &CoursePilotError) -> Vec<String> {
        match error {
            CoursePilotError::Course(course_error) => match course_error {
                CourseError::NotFound { .. } => vec![
                    "Refresh the course list".to_string(),
                    "Check if the course was deleted".to_string(),
                    "Try importing the course again".to_string(),
                ],
                CourseError::Validation { .. } => vec![
                    "Check the course information".to_string(),
                    "Ensure all required fields are filled".to_string(),
                    "Try with different course data".to_string(),
                ],
                _ => vec!["Try the course operation again".to_string()],
            },
            CoursePilotError::Import(import_error) => match import_error {
                ImportError::AuthenticationFailed { .. } => vec![
                    "Check your API key configuration".to_string(),
                    "Verify your account permissions".to_string(),
                    "Try logging in again".to_string(),
                ],
                ImportError::RateLimited { retry_after_seconds, .. } => vec![
                    format!("Wait {} seconds before retrying", retry_after_seconds),
                    "Try importing fewer items at once".to_string(),
                ],
                ImportError::NetworkError { .. } => vec![
                    "Check your internet connection".to_string(),
                    "Try again in a few moments".to_string(),
                    "Use a different network if available".to_string(),
                ],
                _ => vec!["Try the import operation again".to_string()],
            },
            CoursePilotError::Database(db_error) => match db_error {
                DatabaseError::ConnectionFailed { .. } => vec![
                    "Restart the application".to_string(),
                    "Check available disk space".to_string(),
                    "Close other applications using the database".to_string(),
                ],
                DatabaseError::LockTimeout { .. } => vec![
                    "Wait a moment and try again".to_string(),
                    "Close other Course Pilot windows".to_string(),
                    "Restart the application if the problem persists".to_string(),
                ],
                _ => vec!["Try the database operation again".to_string()],
            },
            _ => vec!["Try the operation again".to_string()],
        }
    }

    /// Attempt automatic recovery for certain error types
    pub fn attempt_recovery(error: &anyhow::Error) -> Option<String> {
        if let Some(course_pilot_error) = error.downcast_ref::<CoursePilotError>() {
            return Self::attempt_domain_recovery(course_pilot_error);
        }

        let error_str = error.to_string().to_lowercase();

        if error_str.contains("database locked") {
            info!("Attempting database recovery...");
            Some("Database connection reset. Please try again.".to_string())
        } else if error_str.contains("temporary") || error_str.contains("timeout") {
            info!("Attempting automatic retry...");
            Some("Retrying operation automatically...".to_string())
        } else {
            None
        }
    }

    fn attempt_domain_recovery(error: &CoursePilotError) -> Option<String> {
        match error {
            CoursePilotError::Database(DatabaseError::LockTimeout { .. }) => {
                info!("Attempting database lock recovery...");
                Some("Database lock cleared. Retrying operation...".to_string())
            },
            CoursePilotError::Import(ImportError::RateLimited { retry_after_seconds, .. }) => {
                info!("Scheduling automatic retry after rate limit...");
                Some(format!("Will retry automatically in {} seconds...", retry_after_seconds))
            },
            _ => None,
        }
    }

    /// Retry an operation with exponential backoff
    pub async fn retry_with_backoff<F, T, E>(
        operation: F,
        max_retries: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Result<T>
    where
        F: Fn() -> std::result::Result<T, E> + Send + Sync,
        E: Into<anyhow::Error> + std::fmt::Debug,
    {
        let mut delay = initial_delay;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation() {
                Ok(result) => {
                    if attempt > 0 {
                        info!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                },
                Err(error) => {
                    let error = error.into();
                    warn!("Operation failed on attempt {}: {}", attempt + 1, error);
                    last_error = Some(error);

                    if attempt < max_retries {
                        info!("Retrying in {:?}...", delay);
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(delay * 2, max_delay);
                    }
                },
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("Operation failed after {} retries", max_retries)))
    }

    /// Retry an async operation with exponential backoff
    pub async fn retry_async_with_backoff<F, Fut, T, E>(
        operation: F,
        max_retries: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = std::result::Result<T, E>> + Send,
        E: Into<anyhow::Error> + std::fmt::Debug,
    {
        let mut delay = initial_delay;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!("Async operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                },
                Err(error) => {
                    let error = error.into();
                    warn!("Async operation failed on attempt {}: {}", attempt + 1, error);
                    last_error = Some(error);

                    if attempt < max_retries {
                        info!("Retrying async operation in {:?}...", delay);
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(delay * 2, max_delay);
                    }
                },
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Async operation failed after {} retries", max_retries)
        }))
    }

    /// Check if an error is retryable
    pub fn is_retryable(error: &anyhow::Error) -> bool {
        if let Some(course_pilot_error) = error.downcast_ref::<CoursePilotError>() {
            return Self::is_domain_error_retryable(course_pilot_error);
        }

        let error_str = error.to_string().to_lowercase();

        // Network errors are usually retryable
        if error_str.contains("network")
            || error_str.contains("connection")
            || error_str.contains("timeout")
        {
            return true;
        }

        // Database lock errors are retryable
        if error_str.contains("database locked") || error_str.contains("busy") {
            return true;
        }

        // Rate limiting is retryable
        if error_str.contains("rate limit") || error_str.contains("too many requests") {
            return true;
        }

        false
    }

    fn is_domain_error_retryable(error: &CoursePilotError) -> bool {
        match error {
            CoursePilotError::Import(import_error) => match import_error {
                ImportError::NetworkError { .. } => true,
                ImportError::RateLimited { .. } => true,
                ImportError::Timeout { .. } => true,
                ImportError::Cancelled => false,
                ImportError::AuthenticationFailed { .. } => false,
                ImportError::InvalidSource { .. } => false,
                _ => true,
            },
            CoursePilotError::Database(db_error) => match db_error {
                DatabaseError::ConnectionFailed { .. } => true,
                DatabaseError::LockTimeout { .. } => true,
                DatabaseError::PoolExhausted { .. } => true,
                DatabaseError::CorruptionDetected { .. } => false,
                DatabaseError::ConstraintViolation { .. } => false,
                _ => true,
            },
            CoursePilotError::Network { .. } => true,
            _ => false,
        }
    }
}

/// Structured error logging with different levels for technical vs user information
pub struct ErrorLogger;

impl ErrorLogger {
    /// Log error with context and structured information
    pub fn log_error(error: &anyhow::Error, operation: &str, context: Option<&str>) {
        let error_id = Self::generate_error_id();

        // Log technical details at error level
        error!("Operation '{operation}' failed [ID: {error_id}]: {error}");

        if let Some(ctx) = context {
            error!("Context [ID: {error_id}]: {ctx}");
        }

        // Log domain-specific error details
        if let Some(course_pilot_error) = error.downcast_ref::<CoursePilotError>() {
            Self::log_domain_error_details(course_pilot_error, &error_id);
        }

        // Log the error chain for debugging
        let mut current_error = error.source();
        let mut level = 1;
        while let Some(err) = current_error {
            error!("  Caused by [{level}] [ID: {error_id}]: {err}");
            current_error = err.source();
            level += 1;
        }
    }

    /// Log error with user-friendly message separation
    pub fn log_error_with_user_message(
        error: &anyhow::Error,
        operation: &str,
        user_message: &str,
        context: Option<&str>,
    ) {
        let error_id = Self::generate_error_id();

        // Log technical details
        error!("Operation '{operation}' failed [ID: {error_id}]: {error}");

        // Log user-friendly message separately
        info!("User message [ID: {error_id}]: {user_message}");

        if let Some(ctx) = context {
            error!("Context [ID: {error_id}]: {ctx}");
        }

        // Log error chain
        let mut current_error = error.source();
        let mut level = 1;
        while let Some(err) = current_error {
            error!("  Caused by [{level}] [ID: {error_id}]: {err}");
            current_error = err.source();
            level += 1;
        }
    }

    /// Log warning with recovery information
    pub fn log_warning_with_recovery(message: &str, recovery_action: &str) {
        let warning_id = Self::generate_warning_id();
        warn!("{message} [ID: {warning_id}] - Recovery: {recovery_action}");
    }

    /// Log retry attempts
    pub fn log_retry_attempt(operation: &str, attempt: u32, max_retries: u32, delay: Duration) {
        info!(
            "Retrying operation '{operation}' - attempt {}/{} after {:?}",
            attempt + 1,
            max_retries + 1,
            delay
        );
    }

    /// Log successful recovery
    pub fn log_recovery_success(operation: &str, attempts: u32) {
        info!("Operation '{operation}' succeeded after {attempts} attempts");
    }

    /// Log recovery failure
    pub fn log_recovery_failure(operation: &str, attempts: u32, final_error: &anyhow::Error) {
        let error_id = Self::generate_error_id();
        error!(
            "Operation '{operation}' failed permanently after {attempts} attempts [ID: {error_id}]: {final_error}"
        );
    }

    fn log_domain_error_details(error: &CoursePilotError, error_id: &str) {
        match error {
            CoursePilotError::Course(course_error) => {
                error!("Course error details [ID: {error_id}]: {course_error:?}");
            },
            CoursePilotError::Import(import_error) => {
                error!("Import error details [ID: {error_id}]: {import_error:?}");

                // Log additional context for import errors
                match import_error {
                    ImportError::RateLimited { service, retry_after_seconds } => {
                        warn!(
                            "Rate limited by {service}, retry after {retry_after_seconds}s [ID: {error_id}]"
                        );
                    },
                    ImportError::AuthenticationFailed { service } => {
                        warn!("Authentication failed for {service} [ID: {error_id}]");
                    },
                    _ => {},
                }
            },
            CoursePilotError::Database(db_error) => {
                error!("Database error details [ID: {error_id}]: {db_error:?}");

                // Log additional context for database errors
                match db_error {
                    DatabaseError::PoolExhausted { active_connections, max_connections } => {
                        warn!(
                            "Database pool exhausted: {active_connections}/{max_connections} [ID: {error_id}]"
                        );
                    },
                    DatabaseError::LockTimeout { timeout_seconds } => {
                        warn!("Database lock timeout after {timeout_seconds}s [ID: {error_id}]");
                    },
                    _ => {},
                }
            },
            CoursePilotError::Nlp(nlp_error) => {
                error!("NLP error details [ID: {error_id}]: {nlp_error:?}");
            },
            CoursePilotError::Plan(plan_error) => {
                error!("Plan error details [ID: {error_id}]: {plan_error:?}");
            },
            _ => {
                error!("General error details [ID: {error_id}]: {error:?}");
            },
        }
    }

    fn generate_error_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        format!("ERR-{timestamp}")
    }

    fn generate_warning_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        format!("WARN-{timestamp}")
    }
}

/// Async-safe error handling utilities
pub struct AsyncErrorHandler;

impl AsyncErrorHandler {
    /// Handle async errors with proper logging and user feedback
    pub async fn handle_async_error<T>(
        operation: impl std::future::Future<Output = Result<T>>,
        operation_name: &str,
        timeout_duration: Option<Duration>,
    ) -> Result<T> {
        let operation_name = operation_name.to_string();

        let result = if let Some(timeout) = timeout_duration {
            // Apply timeout to the operation
            match tokio::time::timeout(timeout, operation).await {
                Ok(result) => result,
                Err(_) => {
                    let timeout_error = anyhow::anyhow!(
                        "Operation '{}' timed out after {:?}",
                        operation_name,
                        timeout
                    );
                    ErrorLogger::log_error(&timeout_error, &operation_name, Some("timeout"));
                    return Err(timeout_error);
                },
            }
        } else {
            operation.await
        };

        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                let handled_error = ErrorHandler::handle_error(error, &operation_name, None);
                Err(handled_error)
            },
        }
    }

    /// Handle async operations with retry logic
    pub async fn handle_async_with_retry<T, F, Fut>(
        operation: F,
        operation_name: &str,
        max_retries: u32,
        timeout_duration: Option<Duration>,
    ) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let operation_name = operation_name.to_string();

        for attempt in 0..=max_retries {
            let operation_future = operation();

            let result = if let Some(timeout) = timeout_duration {
                match tokio::time::timeout(timeout, operation_future).await {
                    Ok(result) => result,
                    Err(_) => {
                        let timeout_error = anyhow::anyhow!(
                            "Operation '{}' timed out after {:?} (attempt {})",
                            operation_name,
                            timeout,
                            attempt + 1
                        );

                        if attempt < max_retries {
                            ErrorLogger::log_retry_attempt(
                                &operation_name,
                                attempt,
                                max_retries,
                                Duration::from_secs(1),
                            );
                            let delay = ErrorHandler::get_retry_delay(&timeout_error, attempt);
                            tokio::time::sleep(delay).await;
                            continue;
                        } else {
                            ErrorLogger::log_recovery_failure(
                                &operation_name,
                                attempt + 1,
                                &timeout_error,
                            );
                            return Err(timeout_error);
                        }
                    },
                }
            } else {
                operation_future.await
            };

            match result {
                Ok(value) => {
                    if attempt > 0 {
                        ErrorLogger::log_recovery_success(&operation_name, attempt + 1);
                    }
                    return Ok(value);
                },
                Err(error) => {
                    if attempt < max_retries
                        && ErrorHandler::should_retry(&error, attempt, max_retries)
                    {
                        ErrorLogger::log_retry_attempt(
                            &operation_name,
                            attempt,
                            max_retries,
                            Duration::from_secs(1),
                        );
                        let delay = ErrorHandler::get_retry_delay(&error, attempt);
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        ErrorLogger::log_recovery_failure(&operation_name, attempt + 1, &error);
                        let handled_error =
                            ErrorHandler::handle_error(error, &operation_name, None);
                        return Err(handled_error);
                    }
                },
            }
        }

        unreachable!("Loop should have returned or continued")
    }

    /// Handle UI async operations with user-friendly error messages
    pub async fn handle_ui_async<T>(
        operation: impl std::future::Future<Output = Result<T>>,
        operation_name: &str,
        show_loading: bool,
    ) -> Result<T> {
        let operation_name = operation_name.to_string();

        if show_loading {
            // In a real implementation, this would show a loading indicator
            info!("Starting async operation: {}", operation_name);
        }

        let result = operation.await;

        match result {
            Ok(value) => {
                if show_loading {
                    info!("Completed async operation: {}", operation_name);
                }
                Ok(value)
            },
            Err(error) => {
                let handled_error = ErrorHandler::handle_ui_error(error, &operation_name);
                Err(handled_error)
            },
        }
    }

    /// Create an async error boundary for UI components
    pub async fn with_error_boundary<T>(
        operation: impl std::future::Future<Output = Result<T>>,
        fallback_value: T,
        operation_name: &str,
    ) -> T {
        match operation.await {
            Ok(value) => value,
            Err(error) => {
                ErrorLogger::log_error(&error, operation_name, Some("error_boundary"));
                warn!("Error boundary caught error in '{}': {}", operation_name, error);
                fallback_value
            },
        }
    }
}

/// Comprehensive error handler that coordinates all error handling functionality
pub struct ErrorHandler;

impl ErrorHandler {
    /// Handle an error with full logging, user message, and recovery suggestions
    pub fn handle_error(
        error: anyhow::Error,
        operation: &str,
        context: Option<&str>,
    ) -> anyhow::Error {
        let user_message = ErrorMessageMapper::map_error(&error);

        // Log the error with both technical and user-friendly information
        ErrorLogger::log_error_with_user_message(&error, operation, &user_message, context);

        // Attempt automatic recovery if possible
        if let Some(recovery_message) = ErrorRecoveryManager::attempt_recovery(&error) {
            ErrorLogger::log_warning_with_recovery(&user_message, &recovery_message);
        }

        error
    }

    /// Handle an error in UI context with toast notification
    pub fn handle_ui_error(error: anyhow::Error, operation: &str) -> anyhow::Error {
        let user_message = ErrorMessageMapper::map_error(&error);
        ErrorLogger::log_error_with_user_message(&error, operation, &user_message, None);

        // Show user-friendly message in UI
        // Note: This would typically use a toast system, but we'll return the error for now
        error.context(user_message)
    }

    /// Handle an error with retry suggestions
    pub fn handle_error_with_retry(error: anyhow::Error, operation: &str) -> anyhow::Error {
        let user_message = ErrorMessageMapper::map_error(&error);
        let recovery_actions = ErrorRecoveryManager::get_recovery_actions(&error);

        ErrorLogger::log_error_with_user_message(&error, operation, &user_message, None);

        // Add recovery suggestions to the error
        let enhanced_message = if !recovery_actions.is_empty() {
            format!("{}\n\nSuggested actions:\n• {}", user_message, recovery_actions.join("\n• "))
        } else {
            user_message
        };

        error.context(enhanced_message)
    }

    /// Check if an operation should be retried
    pub fn should_retry(error: &anyhow::Error, attempt: u32, max_retries: u32) -> bool {
        attempt < max_retries && ErrorRecoveryManager::is_retryable(error)
    }

    /// Get appropriate retry delay based on error type and attempt number
    pub fn get_retry_delay(error: &anyhow::Error, attempt: u32) -> Duration {
        // Check for domain-specific retry delays
        if let Some(course_pilot_error) = error.downcast_ref::<CoursePilotError>() {
            if let CoursePilotError::Import(ImportError::RateLimited {
                retry_after_seconds, ..
            }) = course_pilot_error
            {
                return Duration::from_secs(*retry_after_seconds);
            }
        }

        // Default exponential backoff: 1s, 2s, 4s, 8s, max 30s
        let base_delay = Duration::from_secs(1);
        let exponential_delay = base_delay * (2_u32.pow(attempt));
        std::cmp::min(exponential_delay, Duration::from_secs(30))
    }
}

/// Timeout handling utilities
pub struct TimeoutHandler;

impl TimeoutHandler {
    /// Apply timeout to any async operation
    pub async fn with_timeout<T>(
        operation: impl std::future::Future<Output = T>,
        timeout_duration: Duration,
        operation_name: &str,
    ) -> Result<T> {
        match tokio::time::timeout(timeout_duration, operation).await {
            Ok(result) => Ok(result),
            Err(_) => {
                let error = anyhow::anyhow!(
                    "Operation '{}' timed out after {:?}",
                    operation_name,
                    timeout_duration
                );
                ErrorLogger::log_error(&error, operation_name, Some("timeout"));
                Err(error)
            },
        }
    }

    /// Apply timeout with retry logic
    pub async fn with_timeout_and_retry<T, F, Fut>(
        operation: F,
        timeout_duration: Duration,
        max_retries: u32,
        operation_name: &str,
    ) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        for attempt in 0..=max_retries {
            match tokio::time::timeout(timeout_duration, operation()).await {
                Ok(Ok(result)) => {
                    if attempt > 0 {
                        ErrorLogger::log_recovery_success(operation_name, attempt + 1);
                    }
                    return Ok(result);
                },
                Ok(Err(error)) => {
                    if attempt < max_retries && ErrorRecoveryManager::is_retryable(&error) {
                        ErrorLogger::log_retry_attempt(
                            operation_name,
                            attempt,
                            max_retries,
                            Duration::from_secs(1),
                        );
                        let delay = ErrorHandler::get_retry_delay(&error, attempt);
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        ErrorLogger::log_recovery_failure(operation_name, attempt + 1, &error);
                        return Err(error);
                    }
                },
                Err(_) => {
                    let timeout_error = anyhow::anyhow!(
                        "Operation '{}' timed out after {:?} (attempt {})",
                        operation_name,
                        timeout_duration,
                        attempt + 1
                    );

                    if attempt < max_retries {
                        ErrorLogger::log_retry_attempt(
                            operation_name,
                            attempt,
                            max_retries,
                            timeout_duration,
                        );
                        let delay = ErrorHandler::get_retry_delay(&timeout_error, attempt);
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        ErrorLogger::log_recovery_failure(
                            operation_name,
                            attempt + 1,
                            &timeout_error,
                        );
                        return Err(timeout_error);
                    }
                },
            }
        }

        unreachable!("Loop should have returned or continued")
    }

    /// Get appropriate timeout duration based on operation type
    pub fn get_timeout_for_operation(operation_type: &str) -> Duration {
        match operation_type {
            "database_query" => Duration::from_secs(30),
            "database_transaction" => Duration::from_secs(60),
            "file_operation" => Duration::from_secs(120),
            "network_request" => Duration::from_secs(30),
            "import_operation" => Duration::from_secs(300), // 5 minutes
            "export_operation" => Duration::from_secs(180), // 3 minutes
            "video_processing" => Duration::from_secs(600), // 10 minutes
            "nlp_processing" => Duration::from_secs(120),   // 2 minutes
            _ => Duration::from_secs(60),                   // Default 1 minute
        }
    }
}

/// Network error recovery utilities
pub struct NetworkErrorHandler;

impl NetworkErrorHandler {
    /// Handle network operations with automatic retry and exponential backoff
    pub async fn handle_network_operation<T, F, Fut>(
        operation: F,
        operation_name: &str,
        max_retries: u32,
    ) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let timeout = TimeoutHandler::get_timeout_for_operation("network_request");

        TimeoutHandler::with_timeout_and_retry(operation, timeout, max_retries, operation_name)
            .await
    }

    /// Check if a network error is retryable
    pub fn is_network_error_retryable(error: &anyhow::Error) -> bool {
        let error_str = error.to_string().to_lowercase();

        // Network errors that are typically retryable
        error_str.contains("timeout")
            || error_str.contains("connection refused")
            || error_str.contains("connection reset")
            || error_str.contains("network unreachable")
            || error_str.contains("temporary failure")
            || error_str.contains("service unavailable")
            || error_str.contains("too many requests")
            || error_str.contains("rate limit")
    }
}

/// File system error recovery utilities
pub struct FileSystemErrorHandler;

impl FileSystemErrorHandler {
    /// Handle file operations with retry logic for transient errors
    pub async fn handle_file_operation<T, F, Fut>(
        operation: F,
        operation_name: &str,
        max_retries: u32,
    ) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let timeout = TimeoutHandler::get_timeout_for_operation("file_operation");

        for attempt in 0..=max_retries {
            match tokio::time::timeout(timeout, operation()).await {
                Ok(Ok(result)) => {
                    if attempt > 0 {
                        ErrorLogger::log_recovery_success(operation_name, attempt + 1);
                    }
                    return Ok(result);
                },
                Ok(Err(error)) => {
                    if attempt < max_retries && Self::is_file_error_retryable(&error) {
                        ErrorLogger::log_retry_attempt(
                            operation_name,
                            attempt,
                            max_retries,
                            Duration::from_secs(1),
                        );
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        ErrorLogger::log_recovery_failure(operation_name, attempt + 1, &error);
                        return Err(error);
                    }
                },
                Err(_) => {
                    let timeout_error = anyhow::anyhow!(
                        "File operation '{}' timed out after {:?} (attempt {})",
                        operation_name,
                        timeout,
                        attempt + 1
                    );

                    if attempt < max_retries {
                        ErrorLogger::log_retry_attempt(
                            operation_name,
                            attempt,
                            max_retries,
                            timeout,
                        );
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    } else {
                        ErrorLogger::log_recovery_failure(
                            operation_name,
                            attempt + 1,
                            &timeout_error,
                        );
                        return Err(timeout_error);
                    }
                },
            }
        }

        unreachable!("Loop should have returned or continued")
    }

    /// Check if a file system error is retryable
    fn is_file_error_retryable(error: &anyhow::Error) -> bool {
        let error_str = error.to_string().to_lowercase();

        // File system errors that are typically retryable
        error_str.contains("resource temporarily unavailable")
            || error_str.contains("device or resource busy")
            || error_str.contains("interrupted system call")
            || error_str.contains("no space left on device") && error_str.contains("temporary")
    }
}

/// Convenience macros for error handling
#[macro_export]
macro_rules! handle_error {
    ($result:expr, $operation:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let handled_error =
                    $crate::error_handling::ErrorHandler::handle_error(error, $operation, None);
                return Err(handled_error);
            },
        }
    };
    ($result:expr, $operation:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let handled_error = $crate::error_handling::ErrorHandler::handle_error(
                    error,
                    $operation,
                    Some($context),
                );
                return Err(handled_error);
            },
        }
    };
}

#[macro_export]
macro_rules! handle_error_with_recovery {
    ($result:expr, $operation:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let handled_error = $crate::error_handling::ErrorHandler::handle_error_with_retry(
                    error, $operation,
                );
                return Err(handled_error);
            },
        }
    };
}

#[macro_export]
macro_rules! handle_ui_error {
    ($result:expr, $operation:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                let handled_error =
                    $crate::error_handling::ErrorHandler::handle_ui_error(error, $operation);
                return Err(handled_error);
            },
        }
    };
}

/// Macro for retrying operations with exponential backoff
#[macro_export]
macro_rules! retry_operation {
    ($operation:expr, $max_retries:expr, $operation_name:expr) => {{
        let initial_delay = std::time::Duration::from_secs(1);
        let max_delay = std::time::Duration::from_secs(30);

        $crate::error_handling::ErrorRecoveryManager::retry_with_backoff(
            $operation,
            $max_retries,
            initial_delay,
            max_delay,
        )
        .await
    }};
}

/// Macro for retrying async operations with exponential backoff
#[macro_export]
macro_rules! retry_async_operation {
    ($operation:expr, $max_retries:expr, $operation_name:expr) => {{
        let initial_delay = std::time::Duration::from_secs(1);
        let max_delay = std::time::Duration::from_secs(30);

        $crate::error_handling::ErrorRecoveryManager::retry_async_with_backoff(
            $operation,
            $max_retries,
            initial_delay,
            max_delay,
        )
        .await
    }};
}

/// Macro for handling async operations with timeout
#[macro_export]
macro_rules! handle_async_with_timeout {
    ($operation:expr, $operation_name:expr) => {{
        let timeout =
            $crate::error_handling::TimeoutHandler::get_timeout_for_operation($operation_name);
        $crate::error_handling::AsyncErrorHandler::handle_async_error(
            $operation,
            $operation_name,
            Some(timeout),
        )
        .await
    }};
    ($operation:expr, $operation_name:expr, $timeout:expr) => {{
        $crate::error_handling::AsyncErrorHandler::handle_async_error(
            $operation,
            $operation_name,
            Some($timeout),
        )
        .await
    }};
}

/// Macro for handling UI async operations
#[macro_export]
macro_rules! handle_ui_async {
    ($operation:expr, $operation_name:expr) => {{
        $crate::error_handling::AsyncErrorHandler::handle_ui_async(
            $operation,
            $operation_name,
            true,
        )
        .await
    }};
    ($operation:expr, $operation_name:expr, $show_loading:expr) => {{
        $crate::error_handling::AsyncErrorHandler::handle_ui_async(
            $operation,
            $operation_name,
            $show_loading,
        )
        .await
    }};
}

/// Macro for creating async error boundaries
#[macro_export]
macro_rules! async_error_boundary {
    ($operation:expr, $fallback:expr, $operation_name:expr) => {{
        $crate::error_handling::AsyncErrorHandler::with_error_boundary(
            $operation,
            $fallback,
            $operation_name,
        )
        .await
    }};
}

/// Macro for handling network operations with retry
#[macro_export]
macro_rules! handle_network_operation {
    ($operation:expr, $operation_name:expr) => {{
        $crate::error_handling::NetworkErrorHandler::handle_network_operation(
            $operation,
            $operation_name,
            3, // Default 3 retries
        )
        .await
    }};
    ($operation:expr, $operation_name:expr, $max_retries:expr) => {{
        $crate::error_handling::NetworkErrorHandler::handle_network_operation(
            $operation,
            $operation_name,
            $max_retries,
        )
        .await
    }};
}

/// Macro for handling file operations with retry
#[macro_export]
macro_rules! handle_file_operation {
    ($operation:expr, $operation_name:expr) => {{
        $crate::error_handling::FileSystemErrorHandler::handle_file_operation(
            $operation,
            $operation_name,
            2, // Default 2 retries for file operations
        )
        .await
    }};
    ($operation:expr, $operation_name:expr, $max_retries:expr) => {{
        $crate::error_handling::FileSystemErrorHandler::handle_file_operation(
            $operation,
            $operation_name,
            $max_retries,
        )
        .await
    }};
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
        self.map_err(|e| e.into()).with_context(|| context.to_string())
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
    fn test_domain_specific_error_mapping() {
        let course_error = CourseError::NotFound { id: "test-123".to_string() };
        let course_pilot_error = CoursePilotError::Course(course_error);
        let message = ErrorMessageMapper::map_course_pilot_error(&course_pilot_error);
        assert!(message.contains("Course not found"));
        assert!(message.contains("test-123"));
    }

    #[test]
    fn test_import_error_mapping() {
        let import_error =
            ImportError::RateLimited { service: "YouTube".to_string(), retry_after_seconds: 60 };
        let course_pilot_error = CoursePilotError::Import(import_error);
        let message = ErrorMessageMapper::map_course_pilot_error(&course_pilot_error);
        assert!(message.contains("Rate limited"));
        assert!(message.contains("YouTube"));
        assert!(message.contains("60 seconds"));
    }

    #[test]
    fn test_database_error_mapping() {
        let db_error = DatabaseError::PoolExhausted { active_connections: 10, max_connections: 10 };
        let course_pilot_error = CoursePilotError::Database(db_error);
        let message = ErrorMessageMapper::map_course_pilot_error(&course_pilot_error);
        assert!(message.contains("Too many database operations"));
        assert!(message.contains("10/10"));
    }

    #[test]
    fn test_nlp_error_mapping() {
        let nlp_error = NlpError::InsufficientData { required: 10, actual: 3 };
        let course_pilot_error = CoursePilotError::Nlp(nlp_error);
        let message = ErrorMessageMapper::map_course_pilot_error(&course_pilot_error);
        assert!(message.contains("Not enough content"));
        assert!(message.contains("10 items"));
        assert!(message.contains("3 available"));
    }

    #[test]
    fn test_plan_error_mapping() {
        let plan_error =
            PlanError::SchedulingConflict { message: "Session overlap detected".to_string() };
        let course_pilot_error = CoursePilotError::Plan(plan_error);
        let message = ErrorMessageMapper::map_course_pilot_error(&course_pilot_error);
        assert!(message.contains("Scheduling conflict"));
        assert!(message.contains("Session overlap detected"));
    }

    #[test]
    fn test_retryable_error_detection() {
        let network_error =
            CoursePilotError::Network { message: "Connection timeout".to_string(), url: None };
        assert!(ErrorRecoveryManager::is_retryable(&anyhow::anyhow!(network_error)));

        let validation_error = CoursePilotError::Course(CourseError::Validation {
            field: "name".to_string(),
            message: "Name is required".to_string(),
        });
        assert!(!ErrorRecoveryManager::is_retryable(&anyhow::anyhow!(validation_error)));
    }

    #[test]
    fn test_domain_specific_recovery_actions() {
        let auth_error = CoursePilotError::Import(ImportError::AuthenticationFailed {
            service: "YouTube".to_string(),
        });
        let actions = ErrorRecoveryManager::get_domain_specific_actions(&auth_error);
        assert!(actions.iter().any(|a| a.contains("API key")));
        assert!(actions.iter().any(|a| a.contains("logging in")));
    }

    #[test]
    fn test_error_handler_retry_delay() {
        let network_error = anyhow::anyhow!("network timeout");

        // Test exponential backoff
        let delay1 = ErrorHandler::get_retry_delay(&network_error, 0);
        let delay2 = ErrorHandler::get_retry_delay(&network_error, 1);
        let delay3 = ErrorHandler::get_retry_delay(&network_error, 2);

        assert_eq!(delay1, Duration::from_secs(1));
        assert_eq!(delay2, Duration::from_secs(2));
        assert_eq!(delay3, Duration::from_secs(4));
    }

    #[test]
    fn test_rate_limit_specific_delay() {
        let rate_limit_error = CoursePilotError::Import(ImportError::RateLimited {
            service: "YouTube".to_string(),
            retry_after_seconds: 120,
        });
        let delay = ErrorHandler::get_retry_delay(&anyhow::anyhow!(rate_limit_error), 0);
        assert_eq!(delay, Duration::from_secs(120));
    }

    #[tokio::test]
    async fn test_retry_with_backoff_failure() {
        let operation = || Err::<(), _>(anyhow::anyhow!("permanent failure"));

        let result = ErrorRecoveryManager::retry_with_backoff(
            operation,
            2,
            Duration::from_millis(10),
            Duration::from_millis(100),
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("permanent failure"));
    }

    #[tokio::test]
    async fn test_async_error_handler_with_timeout() {
        // Test successful operation
        let operation = async { Ok::<i32, anyhow::Error>(42) };
        let result = AsyncErrorHandler::handle_async_error(
            operation,
            "test_operation",
            Some(Duration::from_secs(1)),
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test timeout
        let slow_operation = async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<i32, anyhow::Error>(42)
        };
        let result = AsyncErrorHandler::handle_async_error(
            slow_operation,
            "slow_operation",
            Some(Duration::from_millis(100)),
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_network_error_handler() {
        let attempt_count = std::sync::atomic::AtomicUsize::new(0);
        let network_operation = || {
            let current = attempt_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            async move {
                if current < 2 {
                    Err(anyhow::anyhow!("connection timeout"))
                } else {
                    Ok("network_success")
                }
            }
        };

        let result =
            NetworkErrorHandler::handle_network_operation(network_operation, "test_network", 3)
                .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "network_success");
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::Relaxed), 2);
    }

    #[test]
    fn test_network_error_retryable() {
        let timeout_error = anyhow::anyhow!("connection timeout");
        assert!(NetworkErrorHandler::is_network_error_retryable(&timeout_error));

        let auth_error = anyhow::anyhow!("authentication failed");
        assert!(!NetworkErrorHandler::is_network_error_retryable(&auth_error));
    }

    #[tokio::test]
    async fn test_async_error_boundary() {
        // Test successful operation
        let success_operation = async { Ok::<i32, anyhow::Error>(42) };
        let result =
            AsyncErrorHandler::with_error_boundary(success_operation, -1, "boundary_test").await;
        assert_eq!(result, 42);

        // Test error boundary fallback
        let error_operation = async { Err::<i32, anyhow::Error>(anyhow::anyhow!("test error")) };
        let result =
            AsyncErrorHandler::with_error_boundary(error_operation, -1, "boundary_test_error")
                .await;
        assert_eq!(result, -1);
    }
}
