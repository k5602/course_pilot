use crate::ui::components::badge::Badge;
use crate::ui::components::modal_confirmation::{ActionMenu, DropdownItem};
use crate::ui::components::progress_ring::ProgressRing;
use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// Card variant types for different use cases
#[derive(PartialEq, Clone)]
pub enum CardVariant {
    Course {
        video_count: usize,
        duration: String,
        progress: f32, // 0.0 - 1.0
    },
    Plan {
        completion: f32, // 0.0 - 1.0
        total_items: usize,
    },
    Note {
        timestamp: Option<String>,
        tags: Vec<String>,
    },
    Generic,
}

/// Action item for card interactions
#[derive(PartialEq, Clone)]
pub struct ActionItem {
    pub label: String,
    pub icon: Option<Element>,
    pub on_select: Option<EventHandler<()>>,
    pub disabled: bool,
}

/// Badge data for card display
#[derive(PartialEq, Clone)]
pub struct BadgeData {
    pub label: String,
    pub color: Option<String>,
}

/// Props for the unified Card component
#[derive(Props, PartialEq, Clone)]
pub struct CardProps {
    // Core properties
    pub variant: CardVariant,
    pub title: String,
    #[props(optional)]
    pub subtitle: Option<String>,

    // Content properties
    #[props(optional)]
    pub content: Option<Element>,
    #[props(optional)]
    pub metadata: Option<Vec<String>>,

    // Interactive properties
    #[props(optional)]
    pub actions: Option<Vec<ActionItem>>,
    #[props(optional)]
    pub badges: Option<Vec<BadgeData>>,

    // Styling properties
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub hover_effect: Option<bool>,

    // Event handlers
    #[props(optional)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(optional)]
    pub on_action: Option<EventHandler<String>>,
}

/// Unified DaisyUI Card component with multiple variants
/// Supports course, plan, note, and generic variants with consistent styling
#[component]
pub fn Card(props: CardProps) -> Element {
    let class = props.class.as_deref().unwrap_or("");
    let hover_effect = props.hover_effect.unwrap_or(true);

    // Animation setup
    let mut scale = use_motion(1.0f32);
    let mut y = use_motion(0.0f32);
    let mut opacity = use_motion(0.95f32);

    // Mount animation
    use_effect(move || {
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(Spring::default())),
        );
    });

    let card_style = use_memo(move || {
        format!(
            "transform: translateY({}px) scale({}); opacity: {};",
            y.get_value(),
            scale.get_value(),
            opacity.get_value()
        )
    });

    // Convert ActionItem to DropdownItem for ActionMenu
    let dropdown_actions = props.actions.as_ref().map(|actions| {
        actions
            .iter()
            .map(|action| DropdownItem {
                label: action.label.clone(),
                icon: action.icon.clone(),
                on_select: action.on_select,
                children: None,
                disabled: action.disabled,
            })
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            style: "{card_style}",
            class: "card w-full bg-base-100 shadow-xl border border-base-300 hover:border-primary transition-all duration-300 ease-in-out {class}",
            onclick: move |evt| {
                if let Some(handler) = &props.on_click {
                    handler.call(evt);
                }
            },
            onmouseenter: move |_| {
                if hover_effect {
                    scale.animate_to(1.02, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                    y.animate_to(-5.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                }
            },
            onmouseleave: move |_| {
                if hover_effect {
                    scale.animate_to(1.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                    y.animate_to(0.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
                }
            },

            div {
                class: "card-body p-4",

                // Header with title, badges, and actions
                div {
                    class: "flex justify-between items-start mb-2",
                    div {
                        class: "flex-1",
                        h2 {
                            class: "card-title text-lg flex items-center gap-2 flex-wrap",
                            "{props.title}"

                            // Render badges
                            if let Some(badges) = &props.badges {
                                for badge in badges {
                                    Badge {
                                        label: badge.label.clone(),
                                        color: badge.color.clone(),
                                        class: Some("ml-2".to_string())
                                    }
                                }
                            }
                        }

                        if let Some(subtitle) = &props.subtitle {
                            p { class: "text-base-content/70 text-sm mt-1", "{subtitle}" }
                        }
                    }

                    // Action menu
                    if let Some(actions) = dropdown_actions {
                        ActionMenu {
                            actions: actions,
                            class: Some("".to_string())
                        }
                    }
                }

                // Variant-specific content
                {render_variant_content(&props.variant)}

                // Custom content
                if let Some(content) = &props.content {
                    div { class: "mt-4", {content.clone()} }
                }

                // Metadata
                if let Some(metadata) = &props.metadata {
                    div {
                        class: "text-sm text-base-content/70 mt-2",
                        for (i, item) in metadata.iter().enumerate() {
                            if i > 0 { " • " }
                            "{item}"
                        }
                    }
                }
            }
        }
    }
}

/// Render variant-specific content
fn render_variant_content(variant: &CardVariant) -> Element {
    match variant {
        CardVariant::Course {
            video_count,
            duration,
            progress,
        } => {
            let progress_percent = (*progress * 100.0).round() as u32;
            let status = if progress_percent >= 100 {
                "Completed"
            } else if progress_percent > 0 {
                "In Progress"
            } else {
                "Not Started"
            };

            rsx! {
                div {
                    class: "space-y-3",

                    // Course metadata
                    p {
                        class: "text-sm text-base-content/70",
                        "{video_count} videos • {duration}"
                    }

                    // Progress section
                    div {
                        class: "flex items-center justify-between",
                        div {
                            class: "flex items-center gap-3",
                            ProgressRing {
                                value: progress_percent,
                                size: Some(36),
                                color: Some("accent".to_string()),
                            }
                            div {
                                class: "text-sm",
                                div { class: "font-medium", "{progress_percent}% complete" }
                                div { class: "text-base-content/60", "{status}" }
                            }
                        }
                    }
                }
            }
        }
        CardVariant::Plan {
            completion,
            total_items,
        } => {
            let completion_percent = (*completion * 100.0).round() as u32;
            let completed_items = (*completion * *total_items as f32).round() as usize;

            rsx! {
                div {
                    class: "space-y-3",

                    // Plan metadata
                    p {
                        class: "text-sm text-base-content/70",
                        "{completed_items} of {total_items} items completed"
                    }

                    // Progress section
                    div {
                        class: "flex items-center gap-3",
                        ProgressRing {
                            value: completion_percent,
                            size: Some(40),
                            color: Some("success".to_string()),
                        }
                        div {
                            class: "text-sm",
                            div { class: "font-medium", "{completion_percent}% complete" }
                            div { class: "text-base-content/60", "Study plan progress" }
                        }
                    }
                }
            }
        }
        CardVariant::Note { timestamp, tags } => {
            rsx! {
                div {
                    class: "space-y-3",

                    // Note metadata
                    div {
                        class: "flex items-center justify-between text-sm text-base-content/70",
                        if let Some(timestamp) = timestamp {
                            span { "{timestamp}" }
                        }
                    }

                    // Tags
                    if !tags.is_empty() {
                        div {
                            class: "flex flex-wrap gap-1",
                            for tag in tags {
                                Badge {
                                    label: tag.clone(),
                                    color: Some("outline".to_string()),
                                    class: Some("badge-sm".to_string())
                                }
                            }
                        }
                    }
                }
            }
        }
        CardVariant::Generic => {
            rsx! {
                div { class: "space-y-2" }
            }
        }
    }
}
