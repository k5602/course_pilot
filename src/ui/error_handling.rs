//! Enhanced error handling utilities for the UI layer

use crate::Phase3Error;
use crate::error_handling::{ErrorLogger, ErrorMessageMapper, ErrorRecoveryManager};
use crate::ui::toast_helpers;
use anyhow::Error;
use log::{error, info, warn};

/// Handle async errors in UI components with comprehensive error handling
pub fn handle_ui_error(error: Error, operation: &str) {
    // Use our comprehensive error logging
    ErrorLogger::log_error(&error, operation, Some("UI operation"));

    // Try automatic recovery first
    if let Some(recovery_message) = ErrorRecoveryManager::attempt_recovery(&error) {
        toast_helpers::info(recovery_message);
        return;
    }

    // Get user-friendly error message
    let user_message = ErrorMessageMapper::map_error(&error);

    // Get recovery suggestions
    let recovery_actions = ErrorRecoveryManager::get_recovery_actions(&error);

    // Show error with recovery suggestions if available
    if !recovery_actions.is_empty() {
        let full_message = format!(
            "{}\n\nSuggested actions:\n• {}",
            user_message,
            recovery_actions.join("\n• ")
        );
        toast_helpers::error(full_message);
    } else {
        toast_helpers::error(user_message);
    }
}

/// Handle specific Phase3 errors with custom logic
pub fn handle_phase3_error(error: &Phase3Error, operation: &str) {
    error!("Phase3 operation '{operation}' failed: {error}");

    let user_message = match error {
        Phase3Error::PlanItemNotFound {
            plan_id,
            item_index,
        } => {
            warn!("Plan item not found: plan_id={plan_id}, item_index={item_index}");
            "The item you're trying to update no longer exists. Please refresh the page."
                .to_string()
        }
        Phase3Error::Backend(e) => {
            error!("Backend error: {e}");
            "A server error occurred. Please try again in a moment.".to_string()
        }
        Phase3Error::Ingest(msg) => {
            warn!("Import operation failed: {msg}");
            format!("Import failed: {msg}")
        }
        Phase3Error::StateSyncError(msg) => {
            error!("UI state synchronization failed: {msg}");
            "UI state synchronization failed. Please refresh the page.".to_string()
        }
    };

    toast_helpers::error(user_message);
}

/// Enhanced error handler with recovery mechanisms
pub fn handle_ui_error_with_recovery(error: Error, operation: &str, context: Option<&str>) {
    // Log with context
    ErrorLogger::log_error(&error, operation, context);

    // Check for specific error types that need special handling
    if let Some(phase3_error) = error.downcast_ref::<Phase3Error>() {
        handle_phase3_error(phase3_error, operation);
        return;
    }

    // Try automatic recovery
    if let Some(recovery_message) = ErrorRecoveryManager::attempt_recovery(&error) {
        info!("Attempting automatic recovery for operation: {operation}");
        toast_helpers::info(recovery_message);
        return;
    }

    // Show user-friendly error with recovery suggestions
    let user_message = ErrorMessageMapper::map_error(&error);
    let recovery_actions = ErrorRecoveryManager::get_recovery_actions(&error);

    if !recovery_actions.is_empty() {
        let full_message = format!(
            "{}\n\nSuggested actions:\n• {}",
            user_message,
            recovery_actions.join("\n• ")
        );
        toast_helpers::error(full_message);
    } else {
        toast_helpers::error(user_message);
    }
}

/// Hook to get the enhanced error handler function
pub fn use_error_handler() -> impl Fn(Error, &str) + Clone {
    move |error, operation| {
        handle_ui_error(error, operation);
    }
}

/// Hook to get the error handler with recovery function
pub fn use_error_handler_with_recovery() -> impl Fn(Error, &str, Option<&str>) + Clone {
    move |error, operation, context| {
        handle_ui_error_with_recovery(error, operation, context);
    }
}

/// Utility function to handle Result types in UI components
pub fn handle_result_in_ui<T>(
    result: anyhow::Result<T>,
    operation: &str,
    success_message: Option<String>,
) -> Option<T> {
    match result {
        Ok(value) => {
            if let Some(msg) = success_message {
                toast_helpers::success(msg);
            }
            Some(value)
        }
        Err(error) => {
            handle_ui_error(error, operation);
            None
        }
    }
}
