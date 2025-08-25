use anyhow::Result;
use dioxus::prelude::*;

use crate::gemini::{ChatbotResponse, ConversationHistory, GeminiClient};
use crate::ui::hooks::use_settings::SettingsManager;

/// GeminiManager owns the GeminiClient lifecycle and provides a hooks-driven API
/// aligned with the project's Backend pattern:
/// - Initialization from SettingsManager (no direct settings reads inside the client)
/// - Reactive signals for init state and last error
/// - Async operations that can be called from UI with spawn(async move { ... })
#[derive(Clone)]
pub struct GeminiManager {
    client: Signal<GeminiClient>,
    is_initialized: Signal<bool>,
    last_error: Signal<Option<String>>,
    settings: SettingsManager,
}

impl GeminiManager {
    /// Initialize the Gemini client from persisted settings (API key).
    /// Sets is_initialized when a valid key is configured.
    pub async fn initialize(&self) -> Result<()> {
        // Load key through SettingsManager (spawn_blocking under the hood)
        let key = self.settings.get_gemini_api_key().await?;
        {
            let mut client_signal = self.client;
            let mut client = client_signal.write();
            match key {
                Some(k) if !k.trim().is_empty() => {
                    client.set_api_key(k);
                }
                _ => {
                    client.clear_api_key();
                }
            }
        }
        // Validate local configuration state; client just checks presence for now
        let init_res = {
            let mut client = self.client.read().clone();
            client.initialize().await
        };

        match init_res {
            Ok(_) => {
                let mut is_init = self.is_initialized;
                is_init.set(true);
                let mut last = self.last_error;
                last.set(None);
                Ok(())
            }
            Err(e) => {
                let mut is_init = self.is_initialized;
                is_init.set(false);
                let mut last = self.last_error;
                last.set(Some(format!("Gemini initialization failed: {}", e)));
                Err(e)
            }
        }
    }

    /// Return whether the manager considers the client initialized (has an API key set).
    pub fn is_initialized(&self) -> bool {
        self.is_initialized.read().clone()
    }

    /// Expose a reactive signal for initialized state.
    pub fn initialized_signal(&self) -> Signal<bool> {
        self.is_initialized
    }

    /// Expose a reactive signal for last error (if any).
    pub fn last_error_signal(&self) -> Signal<Option<String>> {
        self.last_error
    }

    /// Validate a candidate API key by performing a lightweight API call.
    pub async fn validate_api_key(&self, api_key: &str) -> Result<bool> {
        GeminiClient::validate_api_key(api_key).await
    }

    /// Persist the API key via SettingsManager and update the in-memory client configuration.
    /// Does not mark initialized automatically; call initialize() after setting the key.
    pub async fn set_api_key(&self, api_key: Option<String>) -> Result<()> {
        self.settings.set_gemini_api_key(api_key.clone()).await?;
        {
            let mut client_signal = self.client;
            let mut client = client_signal.write();
            match api_key {
                Some(k) if !k.trim().is_empty() => client.set_api_key(k),
                _ => client.clear_api_key(),
            }
        }
        Ok(())
    }

    /// Read the API key via SettingsManager (if set).
    pub async fn get_api_key(&self) -> Result<Option<String>> {
        self.settings.get_gemini_api_key().await
    }

    /// Send a message to Gemini with the current conversation history.
    /// The client is cloned before the async call to avoid holding locks across await points.
    pub async fn send_message(
        &self,
        message: &str,
        history: &ConversationHistory,
    ) -> Result<ChatbotResponse> {
        let client = self.client.read().clone();
        client.send_message(message, history).await
    }

    /// Return whether the underlying client currently has a configured API key.
    pub fn is_configured(&self) -> bool {
        self.client.read().is_configured()
    }
}

/// Hook to get a GeminiManager instance bound to the component's lifecycle.
/// - Keeps a GeminiClient in a reactive Signal
/// - Provides SettingsManager for persistence
pub fn use_gemini_manager() -> GeminiManager {
    let client = use_signal(GeminiClient::new);
    let is_initialized = use_signal(|| false);
    let last_error = use_signal(|| None::<String>);
    let settings = crate::ui::hooks::use_settings_manager();

    GeminiManager {
        client,
        is_initialized,
        last_error,
        settings,
    }
}
