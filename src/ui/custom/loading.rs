//! Loading state components - Spinners and Skeletons

use dioxus::prelude::*;

/// A centered loading spinner with optional message.
#[component]
pub fn Spinner(message: Option<String>) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-8",
            span { class: "loading loading-spinner loading-lg text-primary" }
            if let Some(msg) = message {
                p { class: "mt-4 text-base-content/60 animate-pulse", "{msg}" }
            }
        }
    }
}

/// A skeleton loader for course cards.
#[component]
pub fn CardSkeleton() -> Element {
    rsx! {
        div {
            class: "card bg-base-200 animate-pulse",
            div {
                class: "card-body",
                // Title skeleton
                div { class: "h-5 bg-base-300 rounded w-3/4 mb-3" }
                // Subtitle skeleton
                div { class: "h-4 bg-base-300 rounded w-1/2 mb-4" }
                // Progress bar skeleton
                div { class: "h-2 bg-base-300 rounded w-full" }
            }
        }
    }
}

/// A skeleton loader for a page with title and content.
#[component]
pub fn PageSkeleton() -> Element {
    rsx! {
        div {
            class: "p-6 animate-pulse",
            // Title skeleton
            div { class: "h-8 bg-base-300 rounded w-1/3 mb-6" }
            // Content skeleton
            div { class: "space-y-4",
                div { class: "h-4 bg-base-300 rounded w-full" }
                div { class: "h-4 bg-base-300 rounded w-5/6" }
                div { class: "h-4 bg-base-300 rounded w-4/5" }
            }
        }
    }
}

/// A skeleton loader for video list items.
#[component]
pub fn VideoItemSkeleton() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-4 p-3 rounded-lg bg-base-200 animate-pulse",
            // Play button skeleton
            div { class: "w-8 h-8 bg-base-300 rounded-full" }
            // Content skeleton
            div { class: "flex-1",
                div { class: "h-4 bg-base-300 rounded w-3/4 mb-2" }
                div { class: "h-3 bg-base-300 rounded w-1/4" }
            }
        }
    }
}

/// An inline loading indicator for buttons or small areas.
#[component]
pub fn InlineSpinner() -> Element {
    rsx! {
        span { class: "loading loading-spinner loading-sm" }
    }
}

/// An error alert component.
#[component]
pub fn ErrorAlert(message: String, on_dismiss: Option<EventHandler<()>>) -> Element {
    rsx! {
        div {
            class: "alert alert-error shadow-lg mb-4",
            div {
                class: "flex-1",
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "stroke-current flex-shrink-0 h-6 w-6",
                    fill: "none",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
                    }
                }
                span { "{message}" }
            }
            if let Some(handler) = on_dismiss {
                button {
                    class: "btn btn-sm btn-ghost",
                    onclick: move |_| handler.call(()),
                    "✕"
                }
            }
        }
    }
}

/// A success alert component.
#[component]
pub fn SuccessAlert(message: String, on_dismiss: Option<EventHandler<()>>) -> Element {
    rsx! {
        div {
            class: "alert alert-success shadow-lg mb-4",
            div {
                class: "flex-1",
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    class: "stroke-current flex-shrink-0 h-6 w-6",
                    fill: "none",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
                    }
                }
                span { "{message}" }
            }
            if let Some(handler) = on_dismiss {
                button {
                    class: "btn btn-sm btn-ghost",
                    onclick: move |_| handler.call(()),
                    "✕"
                }
            }
        }
    }
}
