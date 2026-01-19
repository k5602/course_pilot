//! Import Playlist Dialog component

use dioxus::prelude::*;
use rfd::FileDialog;

use crate::components::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};

/// Dialog for importing a YouTube playlist.
#[component]
pub fn ImportPlaylistDialog(
    open: Signal<bool>,
    on_import_youtube: EventHandler<String>,
    on_import_local: EventHandler<String>,
    is_loading: Signal<bool>,
    status_msg: Signal<Option<String>>,
) -> Element {
    let mut url_input = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut active_tab = use_signal(|| "youtube".to_string());

    // Convert bool signal to Option<bool> for dialog
    let open_option = use_memo(move || Some(*open.read()));

    let handle_import = move |_| {
        let url = url_input.read().clone();
        if url.trim().is_empty() {
            let message = if *active_tab.read() == "local" {
                "Please choose a folder"
            } else {
                "Please enter a playlist or video URL"
            };
            error_msg.set(Some(message.to_string()));
            return;
        }

        error_msg.set(None);

        // Trigger import via callback
        if *active_tab.read() == "local" {
            on_import_local.call(url);
        } else {
            on_import_youtube.call(url);
        }

        // Status is managed by the parent
    };

    let handle_cancel = move |_| {
        if *is_loading.read() {
            return;
        }
        open.set(false);
        url_input.set(String::new());
        error_msg.set(None);
    };

    rsx! {
        DialogRoot {
            open: open_option,
            is_modal: true,
            on_open_change: move |is_open: bool| {
                if *is_loading.read() {
                    open.set(true);
                } else {
                    open.set(is_open);
                }
            },

            DialogContent {
                div { class: "p-6 bg-base-100 rounded-lg shadow-xl max-w-md w-full",

                    DialogTitle {
                        h2 { class: "text-xl font-bold mb-2", "Import Playlist" }
                    }

                    DialogDescription {
                        p { class: "text-base-content/70 mb-4",
                            "Choose a source to import your course content"
                        }
                    }

                    // Source tabs + input
                    div { class: "mb-4",

                        // Tabs
                        div { class: "tabs tabs-boxed mb-4",
                            button {
                                class: if *active_tab.read() == "youtube" { "tab tab-active" } else { "tab" },
                                onclick: move |_| active_tab.set("youtube".to_string()),
                                "YouTube"
                            }
                            button {
                                class: if *active_tab.read() == "local" { "tab tab-active" } else { "tab" },
                                onclick: move |_| active_tab.set("local".to_string()),
                                "Local"
                            }
                        }

                        // Inputs
                        if *active_tab.read() == "youtube" {
                            input {
                                class: "input input-bordered w-full",
                                r#type: "url",
                                placeholder: "https://www.youtube.com/playlist?list=... or https://youtu.be/ID",
                                value: "{url_input}",
                                oninput: move |e| url_input.set(e.value()),
                                disabled: *is_loading.read(),
                            }
                        } else {
                            div { class: "flex flex-col gap-2",
                                button {
                                    class: "btn btn-outline w-full",
                                    onclick: move |_| {
                                        if *is_loading.read() {
                                            return;
                                        }
                                        if let Some(path) = FileDialog::new().pick_folder() {
                                            url_input.set(path.display().to_string());
                                            error_msg.set(None);
                                        }
                                    },
                                    disabled: *is_loading.read(),
                                    "Choose Folder"
                                }
                                if !url_input.read().is_empty() {
                                    div { class: "text-sm text-base-content/70 break-all",
                                        "{url_input}"
                                    }
                                } else {
                                    div { class: "text-sm text-base-content/50", "No folder selected" }
                                }
                            }
                        }

                        // Error message
                        if let Some(ref err) = *error_msg.read() {
                            p { class: "text-error text-sm mt-1", "{err}" }
                        }

                        // Status message
                        if let Some(ref status) = *status_msg.read() {
                            div { class: "mt-2 text-sm text-base-content/70 flex items-center gap-2",
                                if *is_loading.read() {
                                    span { class: "loading loading-spinner loading-xs" }
                                }
                                "{status}"
                            }
                            if *is_loading.read() {
                                progress { class: "progress progress-primary w-full mt-2" }
                            }
                        }
                    }

                    // Actions
                    div { class: "flex justify-end gap-2",

                        button {
                            class: "btn btn-ghost",
                            onclick: handle_cancel,
                            disabled: *is_loading.read(),
                            "Close"
                        }

                        button {
                            class: "btn btn-primary",
                            onclick: handle_import,
                            disabled: *is_loading.read(),
                            if *is_loading.read() {
                                span { class: "loading loading-spinner loading-sm" }
                            } else {
                                "Import"
                            }
                        }
                    }
                }
            }
        }
    }
}
