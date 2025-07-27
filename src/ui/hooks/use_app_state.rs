use crate::types::AppState;
use dioxus::prelude::*;

/// Hook for accessing the legacy AppState
/// This provides backward compatibility while we transition to modern state management
pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}

/// Hook for accessing the backend adapter
pub fn use_backend_adapter() -> std::sync::Arc<crate::ui::backend_adapter::Backend> {
    use_context::<std::sync::Arc<crate::ui::backend_adapter::Backend>>()
}
