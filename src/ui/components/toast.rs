use dioxus::prelude::*;
use dioxus_signals::Signal;
use std::collections::VecDeque;
use std::time::Duration;
use uuid::Uuid;

/// Toast variant for different notification types using DaisyUI alert classes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastVariant {
    Success, // alert-success
    Error,   // alert-error
    Warning, // alert-warning
    Info,    // alert-info
}

impl ToastVariant {
    /// Get the DaisyUI alert class for this variant
    pub fn alert_class(&self) -> &'static str {
        match self {
            ToastVariant::Success => "alert-success",
            ToastVariant::Error => "alert-error",
            ToastVariant::Warning => "alert-warning",
            ToastVariant::Info => "alert-info",
        }
    }

    /// Get the icon for this variant
    pub fn icon(&self) -> &'static str {
        match self {
            ToastVariant::Success => "✓",
            ToastVariant::Error => "✕",
            ToastVariant::Warning => "⚠",
            ToastVariant::Info => "ℹ",
        }
    }
}

/// Individual toast item
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: Uuid,
    pub message: String,
    pub variant: ToastVariant,
    pub duration: Duration,
    pub created_at: instant::Instant,
    pub dismissible: bool,
}

impl Toast {
    pub fn new(message: String, variant: ToastVariant) -> Self {
        let duration = match variant {
            ToastVariant::Error => Duration::from_secs(8), // Errors stay longer
            ToastVariant::Warning => Duration::from_secs(6), // Warnings stay longer
            ToastVariant::Success => Duration::from_secs(4), // Success shorter
            ToastVariant::Info => Duration::from_secs(3),  // Info shortest
        };

        Self {
            id: Uuid::new_v4(),
            message,
            variant,
            duration,
            created_at: instant::Instant::now(),
            dismissible: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

/// Toast manager state
#[derive(Debug, Clone, PartialEq)]
pub struct ToastManager {
    pub toasts: VecDeque<Toast>,
    pub max_toasts: usize,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self {
            toasts: VecDeque::new(),
            max_toasts: 5, // Limit to 5 toasts at once
        }
    }
}

impl ToastManager {
    pub fn add_toast(&mut self, toast: Toast) {
        // Remove expired toasts
        self.toasts.retain(|t| !t.is_expired());

        // Ensure we don't exceed max toasts
        while self.toasts.len() >= self.max_toasts {
            self.toasts.pop_front();
        }

        self.toasts.push_back(toast);
    }

    pub fn remove_toast(&mut self, id: Uuid) {
        self.toasts.retain(|t| t.id != id);
    }

    pub fn clear_expired(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }
}

/// Initialize the toast manager (called in AppRoot)
pub fn provide_toast_manager() -> Signal<ToastManager> {
    let signal = Signal::new(ToastManager::default());
    use_context_provider(move || signal);
    signal
}

/// Get the ToastManager signal from context
pub fn use_toast_manager() -> Signal<ToastManager> {
    use_context()
}

/// Toast container component using DaisyUI classes
#[component]
pub fn ToastContainer() -> Element {
    let mut manager = use_toast_manager();

    // Auto-cleanup expired toasts using tokio interval (desktop-appropriate)
    use_effect(move || {
        spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                manager.with_mut(|m| m.clear_expired());
            }
        });
    });

    // Get current toasts to avoid lifetime issues
    let current_toasts = manager.read().toasts.clone();

    rsx! {
        div {
            // DaisyUI toast positioning - bottom right with proper z-index
            class: "toast toast-end toast-bottom z-[100]",
            // Render each toast
            {current_toasts.iter().map(|toast| {
                let toast_id = toast.id;
                let message = toast.message.clone();
                let variant = toast.variant;
                let icon = variant.icon();
                let alert_class = variant.alert_class();

                rsx! {
                    div {
                        key: "{toast_id}",
                        class: "alert {alert_class} shadow-lg mb-2 max-w-sm animate-in slide-in-from-right duration-300",
                        div {
                            class: "flex items-center gap-2",
                            span { class: "text-lg", "{icon}" }
                            span { class: "flex-1", "{message}" }
                            button {
                                class: "btn btn-xs btn-ghost ml-auto",
                                onclick: move |_| {
                                    manager.with_mut(|m| m.remove_toast(toast_id));
                                },
                                "✕"
                            }
                        }
                    }
                }
            })}
        }
    }
}

/// Show a toast notification with the given message and variant
pub fn show_toast(message: impl Into<String>, variant: ToastVariant) {
    let msg = message.into();
    let toast = Toast::new(msg, variant);

    // Use spawn to ensure this runs in the correct async context
    spawn(async move {
        // Try to get the toast manager from context
        if let Some(mut manager) = try_consume_context::<Signal<ToastManager>>() {
            manager.with_mut(|toast_manager| {
                toast_manager.add_toast(toast);
            });
        } else {
            // Fallback: log the toast if context is not available
            log::info!(
                "Toast: {} - {}",
                match variant {
                    ToastVariant::Success => "SUCCESS",
                    ToastVariant::Error => "ERROR",
                    ToastVariant::Warning => "WARNING",
                    ToastVariant::Info => "INFO",
                },
                toast.message
            );
        }
    });
}

/// Helper functions for showing different types of toasts
pub mod toast {
    use super::{ToastVariant, show_toast};

    /// Show an info toast
    pub fn info(message: impl Into<String>) {
        show_toast(message, ToastVariant::Info);
    }

    /// Show a success toast
    pub fn success(message: impl Into<String>) {
        show_toast(message, ToastVariant::Success);
    }

    /// Show a warning toast
    pub fn warning(message: impl Into<String>) {
        show_toast(message, ToastVariant::Warning);
    }

    /// Show an error toast
    pub fn error(message: impl Into<String>) {
        show_toast(message, ToastVariant::Error);
    }
}
