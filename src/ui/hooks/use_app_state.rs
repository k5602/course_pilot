use crate::types::AppState;
use dioxus::prelude::*;

/// Hook for accessing the legacy AppState
/// This provides backward compatibility while we transition to modern state management
pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}

/// Hook for accessing the backend adapter
/// This now uses the new hook-based backend instead of the old adapter
pub fn use_backend_adapter() -> super::use_backend::Backend {
    super::use_backend::use_backend()
}
