//! TagBadge component - Colored label for course categorization.

use dioxus::prelude::*;

use crate::domain::entities::Tag;

/// A colored badge displaying a tag name.
#[component]
pub fn TagBadge(
    tag: Tag,
    #[props(default = false)] removable: bool,
    #[props(default)] on_remove: Option<EventHandler<()>>,
) -> Element {
    let bg_style = format!("background-color: {}; color: white;", tag.color());

    rsx! {
        span {
            class: "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium",
            style: "{bg_style}",

            "{tag.name()}"

            if removable {
                if let Some(handler) = on_remove {
                    button {
                        class: "ml-1 hover:bg-white/20 rounded-full p-0.5",
                        onclick: move |_| handler.call(()),
                        "×"
                    }
                }
            }
        }
    }
}

/// A clickable tag for filtering.
#[component]
pub fn TagFilterChip(tag: Tag, active: bool, on_click: EventHandler<()>) -> Element {
    let bg_color = tag.color();
    let (bg_style, text_class) = if active {
        (format!("background-color: {}; color: white;", bg_color), "")
    } else {
        (
            format!("background-color: transparent; border: 1px solid {};", bg_color),
            "text-base-content",
        )
    };

    rsx! {
        button {
            class: "px-3 py-1 rounded-full text-sm font-medium transition-all {text_class}",
            style: "{bg_style}",
            onclick: move |_| on_click.call(()),
            "{tag.name()}"
        }
    }
}

/// Input for creating new tags.
#[component]
pub fn TagInput(on_create: EventHandler<String>) -> Element {
    let mut input_value = use_signal(String::new);
    let mut show_input = use_signal(|| false);

    rsx! {
        if *show_input.read() {
            div { class: "flex items-center gap-2",
                input {
                    class: "input input-sm input-bordered w-32",
                    r#type: "text",
                    placeholder: "Tag name...",
                    value: "{input_value}",
                    oninput: move |e| input_value.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            let value = input_value.read().trim().to_string();
                            if !value.is_empty() {
                                on_create.call(value);
                                input_value.set(String::new());
                                show_input.set(false);
                            }
                        }
                    },
                }
                button {
                    class: "btn btn-sm btn-primary",
                    onclick: move |_| {
                        let value = input_value.read().trim().to_string();
                        if !value.is_empty() {
                            on_create.call(value);
                            input_value.set(String::new());
                            show_input.set(false);
                        }
                    },
                    "Add"
                }
                button {
                    class: "btn btn-sm btn-ghost",
                    onclick: move |_| show_input.set(false),
                    "×"
                }
            }
        } else {
            button {
                class: "btn btn-sm btn-ghost btn-circle",
                onclick: move |_| show_input.set(true),
                "+"
            }
        }
    }
}
