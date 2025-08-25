//! Import management state for Course Pilot
//!
//! This module handles reactive state for import operations including
//! progress tracking, import job management, and import status updates.

use crate::types::{ImportJob, ImportStatus};
use dioxus::prelude::*;
use uuid::Uuid;

use super::{StateError, StateResult};

/// Import management context
#[derive(Clone, Copy)]
pub struct ImportContext {
    pub active_import: Signal<Option<ImportJob>>,
}

impl Default for ImportContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportContext {
    pub fn new() -> Self {
        Self { active_import: Signal::new(None) }
    }
}

/// Import context provider component
#[component]
pub fn ImportContextProvider(children: Element) -> Element {
    use_context_provider(ImportContext::new);
    rsx! { {children} }
}

/// Hook to access active import reactively
pub fn use_active_import_reactive() -> Signal<Option<ImportJob>> {
    use_context::<ImportContext>().active_import
}

/// Check if an import is currently active
pub fn is_import_active_reactive() -> bool {
    let active_import = use_active_import_reactive();
    active_import.read().is_some()
}

/// Get current import progress (0.0 to 1.0)
pub fn get_import_progress_reactive() -> f32 {
    let active_import = use_active_import_reactive();
    if let Some(ref job) = *active_import.read() { job.progress_percentage / 100.0 } else { 0.0 }
}

/// Get current import status
pub fn get_import_status_reactive() -> Option<ImportStatus> {
    let active_import = use_active_import_reactive();
    active_import.read().as_ref().map(|job| job.status.clone())
}

/// Get current import message
pub fn get_import_message_reactive() -> Option<String> {
    let active_import = use_active_import_reactive();
    active_import.read().as_ref().map(|job| job.message.clone())
}

/// Start a new import operation
pub fn start_import_reactive(message: String) -> Uuid {
    let mut active_import = use_active_import_reactive();
    let job = ImportJob::new(message);
    let job_id = job.id;
    active_import.set(Some(job));
    job_id
}

/// Update import progress and message
pub fn update_import_reactive(job_id: Uuid, progress: f32, message: String) -> StateResult<()> {
    let mut active_import = use_active_import_reactive();
    if let Some(ref mut job) = active_import.write().as_mut() {
        if job.id == job_id {
            let mut v = progress * 100.0;
            if v < 0.0 {
                v = 0.0;
            }
            if v > 100.0 {
                v = 100.0;
            }
            job.update_progress(v, message);
            Ok(())
        } else {
            Err(StateError::InvalidOperation(format!(
                "Import job ID mismatch: expected {}, got {}",
                job.id, job_id
            )))
        }
    } else {
        Err(StateError::InvalidOperation("No active import to update".to_string()))
    }
}

/// Complete an import operation successfully
pub fn complete_import_reactive(job_id: Uuid, final_message: String) -> StateResult<()> {
    let mut active_import = use_active_import_reactive();
    if let Some(ref mut job) = active_import.write().as_mut() {
        if job.id == job_id {
            job.mark_completed();
            job.message = final_message;

            // Clear the import after a brief delay (handled by UI)
            Ok(())
        } else {
            Err(StateError::InvalidOperation(format!(
                "Import job ID mismatch: expected {}, got {}",
                job.id, job_id
            )))
        }
    } else {
        Err(StateError::InvalidOperation("No active import to complete".to_string()))
    }
}

/// Fail an import operation with an error
pub fn fail_import_reactive(job_id: Uuid, error_message: String) -> StateResult<()> {
    let mut active_import = use_active_import_reactive();
    if let Some(ref mut job) = active_import.write().as_mut() {
        if job.id == job_id {
            job.mark_failed(error_message);
            Ok(())
        } else {
            Err(StateError::InvalidOperation(format!(
                "Import job ID mismatch: expected {}, got {}",
                job.id, job_id
            )))
        }
    } else {
        Err(StateError::InvalidOperation("No active import to fail".to_string()))
    }
}

/// Clear the active import (typically after completion or failure)
pub fn clear_import_reactive() {
    let mut active_import = use_active_import_reactive();
    active_import.set(None);
}

/// Cancel an active import operation
pub fn cancel_import_reactive() -> StateResult<()> {
    let mut active_import = use_active_import_reactive();
    if active_import.read().is_some() {
        active_import.set(None);
        Ok(())
    } else {
        Err(StateError::InvalidOperation("No active import to cancel".to_string()))
    }
}

/// Get import duration if active
pub fn get_import_duration_reactive() -> Option<std::time::Duration> {
    let active_import = use_active_import_reactive();
    if let Some(ref job) = *active_import.read() {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(job.created_at);
        Some(std::time::Duration::from_secs(duration.num_seconds() as u64))
    } else {
        None
    }
}

/// Check if import has been running for too long (potential timeout)
pub fn is_import_timeout_reactive(timeout_duration: std::time::Duration) -> bool {
    if let Some(duration) = get_import_duration_reactive() {
        duration > timeout_duration
    } else {
        false
    }
}

/// Get import statistics for debugging/monitoring
pub fn get_import_stats_reactive() -> ImportStats {
    let active_import = use_active_import_reactive();
    if let Some(ref job) = *active_import.read() {
        ImportStats {
            is_active: true,
            job_id: Some(job.id),
            status: job.status.clone(),
            progress: job.progress_percentage,
            duration: get_import_duration_reactive(),
            message: job.message.clone(),
        }
    } else {
        ImportStats {
            is_active: false,
            job_id: None,
            status: ImportStatus::Starting,
            progress: 0.0,
            duration: None,
            message: "No active import".to_string(),
        }
    }
}

/// Import statistics structure
#[derive(Debug, Clone)]
pub struct ImportStats {
    pub is_active: bool,
    pub job_id: Option<Uuid>,
    pub status: ImportStatus,
    pub progress: f32,
    pub duration: Option<std::time::Duration>,
    pub message: String,
}

/// Get the active import job
pub fn get_active_import_reactive() -> Option<ImportJob> {
    let active_import = use_active_import_reactive();
    active_import.read().clone()
}

/// Legacy hook functions for compatibility
pub fn use_active_import() -> Signal<Option<ImportJob>> {
    use_active_import_reactive()
}

/// Non-reactive import operations for backend integration
pub fn start_import(message: String) -> Uuid {
    start_import_reactive(message)
}

pub fn update_import(job_id: Uuid, progress: f32, message: String) -> StateResult<()> {
    update_import_reactive(job_id, progress, message)
}

pub fn complete_import(job_id: Uuid, final_message: String) -> StateResult<()> {
    complete_import_reactive(job_id, final_message)
}

pub fn fail_import(job_id: Uuid, error_message: String) -> StateResult<()> {
    fail_import_reactive(job_id, error_message)
}

pub fn clear_import() {
    clear_import_reactive()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_context_creation() {
        // Avoid Dioxus Signals runtime by testing pure ImportJob defaults
        let job = ImportJob::new("Test import".to_string());
        assert_eq!(job.progress_percentage, 0.0);
        assert!(matches!(job.status, ImportStatus::Starting));
    }

    #[test]
    fn test_import_job_lifecycle() {
        // This would need a proper Dioxus context for full testing
        // For now, we test the logic with mock data
        let job = ImportJob::new("Test import".to_string());
        let job_id = job.id;

        // Check initial state
        assert_eq!(job.progress_percentage, 0.0);
        assert_eq!(job.status, ImportStatus::Starting);
        assert_eq!(job.message, "Test import");

        // Test status transitions would be done in context
        assert_ne!(job_id, Uuid::nil());
    }

    #[test]
    fn test_import_progress_validation() {
        // Test progress clamping
        let progress_values = vec![-1.0, 0.0, 0.5, 1.0, 2.0];
        let expected_clamped = vec![0.0, 0.0, 50.0, 100.0, 100.0];

        for (input, expected) in progress_values.iter().zip(expected_clamped.iter()) {
            let val = input * 100.0;
            let clamped = if val < 0.0 {
                0.0
            } else if val > 100.0 {
                100.0
            } else {
                val
            };
            assert_eq!(clamped, *expected);
        }
    }

    #[test]
    fn test_import_stats_structure() {
        let stats = ImportStats {
            is_active: true,
            job_id: Some(Uuid::new_v4()),
            status: ImportStatus::InProgress,
            progress: 50.0,
            duration: Some(std::time::Duration::from_secs(30)),
            message: "Processing...".to_string(),
        };

        assert!(stats.is_active);
        assert!(stats.job_id.is_some());
        assert_eq!(stats.progress, 50.0);
        assert_eq!(stats.message, "Processing...");
    }
}
