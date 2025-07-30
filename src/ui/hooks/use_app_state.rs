use crate::types::AppState;
use dioxus::prelude::*;

/// Hook for accessing the legacy AppState
pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}

/// Hook for accessing the backend adapter
pub fn use_backend_adapter() -> super::use_backend::Backend {
    super::use_backend::use_backend()
}
