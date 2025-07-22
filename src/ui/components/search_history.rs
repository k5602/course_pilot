use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaClock, FaStar, FaTrash};
use dioxus_free_icons::icons::fa_solid_icons::FaMagnifyingGlass;
use dioxus_free_icons::Icon;
use dioxus_motion::prelude::*;

/// SearchHistory: A component for managing and displaying search history
#[derive(Props,Clone ,PartialEq)]
pub struct SearchHistoryProps {
    /// Recent searches
    #[props(default = Vec::new())]
    pub recent_searches: Vec<String>,
    /// Saved searches
    #[props(default = Vec::new())]
    pub saved_searches: Vec<String>,
    /// Callback when a search is selected
    pub on_select: EventHandler<String>,
    /// Callback when a search is saved
    #[props(default)]
    pub on_save: Option<EventHandler<String>>,
    /// Callback when a search is deleted
    #[props(default)]
    pub on_delete: Option<EventHandler<String>>,
    /// Callback when all history is cleared
    #[props(default)]
    pub on_clear_all: Option<EventHandler<()>>,
    /// Optional class for styling
    #[props(default)]
    pub class: Option<String>,
}
#[component]
pub fn SearchHistory(props: SearchHistoryProps) -> Element {
    // Animation for search items
    let item_animation = AnimationConfig::new(AnimationMode::Spring(Spring {
        stiffness: 300.0,
        damping: 25.0,
        ..Spring::default()
    }));

    // Handle search selection
    let handle_select = move |search: String| {
        props.on_select.call(search);
    };

    // Handle search save
    let handle_save = move |search: String| {
        if let Some(on_save) = &props.on_save {
            on_save.call(search);
        }
    };

    // Handle search delete
    let handle_delete = move |search: String| {
        if let Some(on_delete) = &props.on_delete {
            on_delete.call(search);
        }
    };

    // Handle clear all
    let handle_clear_all = move |_| {
        if let Some(on_clear_all) = &props.on_clear_all {
            on_clear_all.call(());
        }
    };

    // Check if a search is saved
    let saved_searches_clone = props.saved_searches.clone();
    let is_saved = move |search: &str| -> bool {
        saved_searches_clone.contains(&search.to_string())
    };

    rsx! {
        div {
            class: "w-full {props.class.clone().unwrap_or_default()}",
            
            // Header with clear button
            div {
                class: "flex justify-between items-center mb-2",
                h3 { class: "text-sm font-semibold", "Search History" }
                if !props.recent_searches.is_empty() && props.on_clear_all.is_some() {
                    button {
                        class: "btn btn-ghost btn-xs",
                        onclick: handle_clear_all,
                        "Clear All"
                    }
                }
            }
            
            // Saved searches
            if !props.saved_searches.is_empty() {
                div {
                    class: "mb-3",
                    h4 { class: "text-xs text-base-content/60 mb-1", "Saved Searches" }
                    div {
                        class: "space-y-1",
                        {props.saved_searches.iter().map(|search| {
                            let mut item_opacity = use_motion(0.0f32);
                            let mut item_y = use_motion(5.0f32);
                            let animation = item_animation.clone();
                            
                            use_effect(move || {
                                item_opacity.animate_to(1.0, animation.clone());
                                item_y.animate_to(0.0, animation.clone());
                            });
                            
                            let item_style = use_memo(move || {
                                format!(
                                    "opacity: {}; transform: translateY({}px);",
                                    item_opacity.get_value(),
                                    item_y.get_value()
                                )
                            });
                            
                            let search_clone = search.clone();
                            let search_clone2 = search.clone();
                            
                            rsx! {
                                div {
                                    key: "{search}-saved",
                                    class: "flex items-center justify-between bg-base-200 hover:bg-base-300 rounded-md px-3 py-2",
                                    style: "{item_style}",
                                    div {
                                        class: "flex items-center gap-2 cursor-pointer",
                                        onclick: move |_| handle_select(search_clone.clone()),
                                        Icon { icon: FaStar, class: "w-3 h-3 text-warning" }
                                        span { class: "text-sm", "{search}" }
                                    }
                                    button {
                                        class: "btn btn-ghost btn-xs btn-circle",
                                        onclick: move |_| handle_delete(search_clone2.clone()),
                                        Icon { icon: FaTrash, class: "w-3 h-3" }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
            
            // Recent searches
            if !props.recent_searches.is_empty() {
                div {
                    h4 { class: "text-xs text-base-content/60 mb-1", "Recent Searches" }
                    div {
                        class: "space-y-1",
                        {props.recent_searches.iter().map(|search| {
                            let mut item_opacity = use_motion(0.0f32);
                            let mut item_y = use_motion(5.0f32);
                            let animation = item_animation.clone();
                            
                            use_effect(move || {
                                item_opacity.animate_to(1.0, animation.clone());
                                item_y.animate_to(0.0, animation.clone());
                            });
                            
                            let item_style = use_memo(move || {
                                format!(
                                    "opacity: {}; transform: translateY({}px);",
                                    item_opacity.get_value(),
                                    item_y.get_value()
                                )
                            });
                            
                            let search_clone = search.clone();
                            let search_clone2 = search.clone();
                            let search_clone3 = search.clone();
                            let saved = is_saved(search);
                            
                            rsx! {
                                div {
                                    key: "{search}-recent",
                                    class: "flex items-center justify-between bg-base-200 hover:bg-base-300 rounded-md px-3 py-2",
                                    style: "{item_style}",
                                    div {
                                        class: "flex items-center gap-2 cursor-pointer",
                                        onclick: move |_| handle_select(search_clone.clone()),
                                        Icon { icon: FaClock, class: "w-3 h-3 text-base-content/60" }
                                        span { class: "text-sm", "{search}" }
                                    }
                                    div {
                                        class: "flex items-center gap-1",
                                        if !saved && props.on_save.is_some() {
                                            button {
                                                class: "btn btn-ghost btn-xs btn-circle",
                                                onclick: move |_| handle_save(search_clone2.clone()),
                                                Icon { icon: FaStar, class: "w-3 h-3" }
                                            }
                                        }
                                        if props.on_delete.is_some() {
                                            button {
                                                class: "btn btn-ghost btn-xs btn-circle",
                                                onclick: move |_| handle_delete(search_clone3.clone()),
                                                Icon { icon: FaTrash, class: "w-3 h-3" }
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
            
            // Empty state
            if props.recent_searches.is_empty() && props.saved_searches.is_empty() {
                div {
                    class: "text-center py-4 text-base-content/60",
                    Icon { icon: FaMagnifyingGlass, class: "w-5 h-5 mx-auto mb-2 opacity-50" }
                    p { class: "text-sm", "No search history yet" }
                    p { class: "text-xs", "Your recent and saved searches will appear here" }
                }
            }
        }
    }
}