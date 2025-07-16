//! Error handling utilities for the UI layer

use crate::Phase3Error;
use crate::ui::components::toast::toast;

/// Handle async errors in UI components
pub fn handle_ui_error(error: anyhow::Error, operation: &str) {
    log::error!("UI operation '{}' failed: {}", operation, error);
    
    let user_message = match error.downcast_ref::<Phase3Error>() {
        Some(Phase3Error::PlanItemNotFound { .. }) => {
            "The item you're trying to update no longer exists. Please refresh the page."
        }
        Some(Phase3Error::Backend(_)) => {
            "A server error occurred. Please try again in a moment."
        }
        Some(Phase3Error::Ingest(msg)) => {
            &format!("Import failed: {}", msg)
        }
        Some(Phase3Error::StateSyncError(_)) => {
            "UI state synchronization failed. Please refresh the page."
        }
        _ => "An unexpected error occurred. Please try again."
    };
    
    toast::error(user_message);
}

/// Hook to get the error handler function
pub fn use_error_handler() -> impl Fn(anyhow::Error, &str) + Clone {
    move |error, operation| {
        handle_ui_error(error, operation);
    }
}