use dioxus::prelude::*;
use dioxus_signals::Signal;
use dioxus_toast::{ToastFrame as DioxusToastFrame, ToastInfo, ToastManager};

/// Toast variant for different notification types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastVariant {
    Success,
    Error,
    Warning,
    Info,
}

/// Initialize the toast manager (called in AppRoot)
pub fn provide_toast_manager() -> Signal<ToastManager> {
    let signal = Signal::new(ToastManager::default());
    use_context_provider(move || signal.clone());
    signal
}

/// Get the ToastManager signal from context
pub fn use_toast_manager() -> Signal<ToastManager> {
    use_context()
}

/// Toast container component that renders all active toasts
#[component]
pub fn ToastContainer() -> Element {
    use crate::ui::theme_unified::use_theme_context;
    let theme_ctx = use_theme_context();
    let _theme = theme_ctx.read().theme; // subscribe to theme changes

    // Use the ToastManager from context
    let manager = use_toast_manager();

    rsx! {
        div {
            class: "toast toast-end toast-bottom",
            DioxusToastFrame {
                manager: manager.clone(),
            }
        }
    }
}

/// Show a toast notification with the given message and variant
pub fn show_toast(message: impl Into<String>, variant: ToastVariant) {
    let msg = message.into();
    let mut manager = use_toast_manager();
    manager.with_mut(|manager| {
        let info = match variant {
            ToastVariant::Success => ToastInfo::success(msg.as_str(), "Success"),
            ToastVariant::Error => ToastInfo::error(msg.as_str(), "Error"),
            ToastVariant::Warning => ToastInfo::warning(msg.as_str(), "Warning"),
            ToastVariant::Info => ToastInfo::simple(msg.as_str()),
        };
        let _ = manager.popup(info);
    });
}

/// Helper functions for showing different types of toasts
pub mod toast {
    /// Show an info toast
    pub fn info(message: impl Into<String>) {
        super::show_toast(message, super::ToastVariant::Info);
    }

    /// Show a success toast
    pub fn success(message: impl Into<String>) {
        super::show_toast(message, super::ToastVariant::Success);
    }

    /// Show a warning toast
    pub fn warning(message: impl Into<String>) {
        super::show_toast(message, super::ToastVariant::Warning);
    }

    /// Show an error toast
    pub fn error(message: impl Into<String>) {
        super::show_toast(message, super::ToastVariant::Error);
    }
}
