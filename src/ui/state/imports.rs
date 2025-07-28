//! Import State Management
//!
//! Focused state management for import operations using modern Dioxus signals

use crate::types::{ImportJob, ImportStatus};
use dioxus::prelude::*;
use uuid::Uuid;

/// Import state context
#[derive(Clone, Copy)]
pub struct ImportState {
    pub active_import: Signal<Option<ImportJob>>,
}

/// Provide import context
pub fn provide_imports_context(initial_import: Option<ImportJob>) {
    use_context_provider(|| ImportState {
        active_import: Signal::new(initial_import),
    });
}

/// Hook to get import state
pub fn use_import_state() -> ImportState {
    use_context::<ImportState>()
}

/// Hook for reactive access to active import
pub fn use_active_import() -> ReadOnlySignal<Option<ImportJob>> {
    let state = use_import_state();
    state.active_import.into()
}

/// Actions for imports
pub mod actions {
    use super::*;
    use crate::state::StateError;

    /// Start an import job
    pub fn start_import(job: ImportJob) {
        let mut state = use_import_state();
        *state.active_import.write() = Some(job);
        log::info!("Import started");
    }

    /// Update import progress
    pub fn update_import(id: Uuid, progress: f32, message: String) -> Result<(), StateError> {
        let mut state = use_import_state();
        let mut import_opt = state.active_import.write();

        if let Some(ref mut import) = *import_opt {
            if import.id == id {
                import.update_progress(progress, message);
                return Ok(());
            }
        }

        Err(StateError::InvalidOperation(
            "No active import found".to_string(),
        ))
    }

    /// Complete an import job
    pub fn complete_import(id: Uuid) -> Result<(), StateError> {
        let mut state = use_import_state();
        let mut import_opt = state.active_import.write();

        if let Some(ref mut import) = *import_opt {
            if import.id == id {
                import.status = ImportStatus::Completed;
                return Ok(());
            }
        }

        Err(StateError::InvalidOperation(
            "No active import found".to_string(),
        ))
    }

    /// Fail an import job
    pub fn fail_import(id: Uuid, error: String) -> Result<(), StateError> {
        let mut state = use_import_state();
        let mut import_opt = state.active_import.write();

        if let Some(ref mut import) = *import_opt {
            if import.id == id {
                import.mark_failed(error);
                return Ok(());
            }
        }

        Err(StateError::InvalidOperation(
            "No active import found".to_string(),
        ))
    }

    /// Clear the active import
    pub fn clear_import() {
        let mut state = use_import_state();
        *state.active_import.write() = None;
        log::info!("Import cleared");
    }
}
