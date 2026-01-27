//! Update Presence Use Case
//!
//! Sends activity updates to the configured presence provider.

use std::sync::Arc;

use crate::domain::ports::{Activity, PresenceProvider};

/// Input for updating presence.
#[derive(Debug, Clone)]
pub struct UpdatePresenceInput {
    pub activity: Activity,
}

/// Use case for updating external presence.
pub struct UpdatePresenceUseCase {
    presence: Arc<dyn PresenceProvider>,
}

impl UpdatePresenceUseCase {
    /// Creates a new use case.
    pub fn new(presence: Arc<dyn PresenceProvider>) -> Self {
        Self { presence }
    }

    /// Updates the external presence.
    pub fn execute(&self, input: UpdatePresenceInput) {
        self.presence.update_activity(input.activity);
    }

    /// Clears the external presence.
    pub fn clear(&self) {
        self.presence.clear_activity();
    }
}
