use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaPlus;
use dioxus_free_icons::icons::fa_brands_icons::FaXing as FaTimes;
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;
use std::collections::HashSet;

/// TagInput: A component for entering and managing tags with autocomplete
#[derive(Props,Clone,PartialEq)]
pub struct TagInputProps {
    /// Current tags
    pub tags: Vec<String>,
    /// Suggested tags for autocomplete
    #[props(default = Vec::new())]
    pub suggestions: Vec<String>,
    /// Callback when tags change
    pub on_tags_change: EventHandler<Vec<String>>,
    /// Optional class for styling
    #[props(default)]
    pub class: Option<String>,
    /// Optional placeholder text
    #[props(default = "Add tags...".to_string())]
    pub placeholder: String,
    /// Maximum number of tags allowed
    #[props(default = 10)]
    pub max_tags: usize,
}

#[component]
pub fn TagInput(props: TagInputProps) -> Element {
    let mut input_value = use_signal(|| String::new());
    let mut filtered_suggestions = use_signal(Vec::new);
    let mut show_suggestions = use_signal(|| false);
    let mut input_focused = use_signal(|| false);

    // Clone props for use in closures
    let tags_clone = props.tags.clone();
    let suggestions_clone = props.suggestions.clone();
    let _on_tags_change = props.on_tags_change.clone(); // Prefix with underscore to indicate intentionally unused
    let max_tags = props.max_tags;

    // Filter suggestions based on input value
    use_effect(move || {
        if input_value().is_empty() {
            filtered_suggestions.set(Vec::new());
            return;
        }

        let current_tags: HashSet<String> = tags_clone.iter().cloned().collect();
        let matching_suggestions: Vec<String> = suggestions_clone
            .iter()
            .filter(|s| {
                s.to_lowercase().contains(&input_value().to_lowercase()) && !current_tags.contains(*s)
            })
            .cloned()
            .collect();

        filtered_suggestions.set(matching_suggestions);
    });

    // Handle adding a tag
    let add_tag = {
        let tags_clone = props.tags.clone();
        let on_tags_change = props.on_tags_change.clone();
        let mut input_value = input_value.clone();
        move |tag: String| {
            let tag = tag.trim().to_string();
            if !tag.is_empty() && !tags_clone.contains(&tag) && tags_clone.len() < max_tags {
                let mut new_tags = tags_clone.clone();
                new_tags.push(tag);
                on_tags_change.call(new_tags);
                input_value.set(String::new());
            }
        }
    };

    // Handle removing a tag
    let remove_tag = {
        let tags_clone = props.tags.clone();
        let on_tags_change = props.on_tags_change.clone();
        move |index: usize| {
            let mut new_tags = tags_clone.clone();
            if index < new_tags.len() {
                new_tags.remove(index);
                on_tags_change.call(new_tags);
            }
        }
    };

    // Handle key press events
    let handle_key_press = {
        let mut add_tag = add_tag.clone();
        let input_value = input_value.clone();
        move |event: KeyboardEvent| {
            let key = event.key();
            match key {
                dioxus::events::Key::Enter => {
                    event.prevent_default();
                    add_tag(input_value().clone());
                }
                dioxus::events::Key::Character(c) if c == "," => {
                    event.prevent_default();
                    add_tag(input_value().clone());
                }
                dioxus::events::Key::Escape => {
                    show_suggestions.set(false);
                }
                _ => {}
            }
        }
    };

    // Handle input focus
    let handle_focus = move |_| {
        show_suggestions.set(true);
        input_focused.set(true);
    };

    // Handle input blur
    let handle_blur = move |_| {
        // Delay hiding suggestions to allow clicking on them
        input_focused.set(false);
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            show_suggestions.set(false);
        });
    };

    // Handle suggestion click
    let handle_suggestion_click = {
        let mut add_tag = add_tag.clone();
        move |suggestion: String| {
            add_tag(suggestion);
            show_suggestions.set(false);
        }
    };

    // Handle add button click
    let handle_add_click = {
        let mut add_tag = add_tag.clone();
        let input_value = input_value.clone();
        move |_| {
            add_tag(input_value().clone());
        }
    };

    // Animation for tags
    let tag_animation = AnimationConfig::new(AnimationMode::Spring(Spring {
        stiffness: 300.0,
        damping: 25.0,
        ..Spring::default()
    }));

    // Store filtered suggestions in a variable to avoid temporary value issues
    let current_filtered_suggestions = filtered_suggestions();
    let current_tags = props.tags.clone();

    rsx! {
        div {
            class: "w-full {props.class.clone().unwrap_or_default()}",
            // Tags display
            div {
                class: "flex flex-wrap gap-2 mb-2",
                {current_tags.iter().enumerate().map(|(index, tag)| {
                    let mut tag_opacity = use_motion(0.0f32);
                    let mut tag_scale = use_motion(0.8f32);
                    let animation = tag_animation.clone();
                    let remove_tag = remove_tag.clone();
                    
                    use_effect(move || {
                        tag_opacity.animate_to(1.0, animation.clone());
                        tag_scale.animate_to(1.0, animation.clone());
                    });
                    
                    let tag_style = use_memo(move || {
                        format!(
                            "opacity: {}; transform: scale({});",
                            tag_opacity.get_value(),
                            tag_scale.get_value()
                        )
                    });
                    
                    rsx! {
                        div {
                            key: "{tag}-{index}",
                            class: "badge badge-accent badge-outline gap-1 animate-in fade-in",
                            style: "{tag_style}",
                            span { "#{tag}" }
                            button {
                                class: "btn btn-ghost btn-xs btn-circle",
                                onclick: move |_| remove_tag(index),
                                Icon { icon: FaTimes, class: "w-3 h-3" }
                            }
                        }
                    }
                })}
            }
            
            // Input with autocomplete
            div {
                class: "relative",
                div {
                    class: "join w-full",
                    input {
                        class: "input input-bordered join-item w-full",
                        placeholder: "{props.placeholder}",
                        value: "{input_value}",
                        oninput: move |e| input_value.set(e.value().clone()),
                        onkeydown: handle_key_press,
                        onfocus: handle_focus,
                        onblur: handle_blur,
                    }
                    button {
                        class: "btn join-item",
                        disabled: input_value().is_empty() || current_tags.len() >= props.max_tags,
                        onclick: handle_add_click,
                        Icon { icon: FaPlus, class: "w-4 h-4" }
                    }
                }
                
                // Suggestions dropdown
                if show_suggestions() && !current_filtered_suggestions.is_empty() {
                    div {
                        class: "absolute z-10 mt-1 w-full bg-base-200 shadow-lg rounded-md max-h-48 overflow-y-auto",
                        {current_filtered_suggestions.iter().map(|suggestion| {
                            let suggestion_clone = suggestion.clone();
                            let mut handle_suggestion_click = handle_suggestion_click.clone();
                            rsx! {
                                div {
                                    key: "{suggestion}",
                                    class: "px-4 py-2 hover:bg-base-300 cursor-pointer",
                                    onclick: move |_| handle_suggestion_click(suggestion_clone.clone()),
                                    "{suggestion}"
                                }
                            }
                        })}
                    }
                }
            }
            
            // Helper text
            div {
                class: "text-xs text-base-content/60 mt-1",
                "Press Enter or comma to add a tag ({current_tags.len()}/{props.max_tags})"
            }
        }
    }
}