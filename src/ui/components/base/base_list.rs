use dioxus::prelude::*;
use dioxus_motion::prelude::*;

/// Item data for BaseList
#[derive(Clone, PartialEq)]
pub struct BaseListItem<T: Clone + PartialEq> {
    pub id: String,
    pub data: T,
    pub disabled: bool,
}

impl<T: Clone + PartialEq> BaseListItem<T> {
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Self { id: id.into(), data, disabled: false }
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Props for the BaseList component
#[derive(Props)]
pub struct BaseListProps<T: Clone + PartialEq + 'static> {
    /// List items
    pub items: Vec<BaseListItem<T>>,

    /// Item renderer function
    pub render_item: Box<dyn Fn(&BaseListItem<T>, usize) -> Element>,

    /// List variant (menu, list, grid, etc.)
    #[props(default = "menu")]
    pub variant: &'static str,

    /// Additional CSS classes
    #[props(default = "")]
    pub class: &'static str,

    /// Enable animations
    #[props(default = true)]
    pub animated: bool,

    /// Empty state content
    #[props(optional)]
    pub empty_state: Option<Element>,

    /// Loading state
    #[props(default = false)]
    pub loading: bool,

    /// Grid columns (for grid variant)
    #[props(optional)]
    pub grid_cols: Option<String>,
}

impl<T: Clone + PartialEq + 'static> PartialEq for BaseListProps<T> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
            && self.variant == other.variant
            && self.class == other.class
            && self.animated == other.animated
            && self.loading == other.loading
            && self.grid_cols == other.grid_cols
    }
}

/// Generic BaseList component using DaisyUI styling
/// Provides consistent list structure with configurable item rendering
#[component]
pub fn BaseList<T: Clone + PartialEq + 'static>(props: BaseListProps<T>) -> Element {
    let list_classes = match props.variant {
        "grid" => {
            let cols =
                props.grid_cols.as_deref().unwrap_or("grid-cols-1 md:grid-cols-2 lg:grid-cols-3");
            format!("grid {} gap-4 {}", cols, props.class)
        },
        "menu" => format!("menu w-full {}", props.class),
        "list" => format!("space-y-2 {}", props.class),
        _ => format!("{} {}", props.variant, props.class),
    };

    // Container animation
    let mut container_opacity = use_motion(0.0f32);
    let mut container_y = use_motion(20.0f32);

    use_effect(move || {
        container_opacity
            .animate_to(1.0, AnimationConfig::new(AnimationMode::Tween(Tween::default())));
        container_y.animate_to(0.0, AnimationConfig::new(AnimationMode::Spring(Spring::default())));
    });

    let container_style = if props.animated {
        format!(
            "opacity: {}; transform: translateY({}px); transition: all 0.3s ease-out;",
            container_opacity.get_value(),
            container_y.get_value()
        )
    } else {
        String::new()
    };

    rsx! {
        div {
            class: "{list_classes}",
            style: "{container_style}",

            if props.loading {
                // Loading state
                div {
                    class: "flex justify-center items-center p-8",
                    span { class: "loading loading-spinner loading-lg" }
                }
            } else if props.items.is_empty() {
                // Empty state
                if let Some(empty_state) = &props.empty_state {
                    {empty_state.clone()}
                } else {
                    div {
                        class: "text-center text-base-content/60 p-8",
                        "No items to display"
                    }
                }
            } else {
                // Render items
                {props.items.iter().enumerate().map(|(index, item)| {
                    let item_element = (props.render_item)(item, index);

                    if props.animated {
                        rsx! {
                            AnimatedListItem {
                                key: "{item.id}",
                                index: index,
                                disabled: item.disabled,
                                {item_element}
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                key: "{item.id}",
                                class: if item.disabled { "opacity-50 pointer-events-none" } else { "" },
                                {item_element}
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Animated wrapper for list items
#[component]
fn AnimatedListItem(index: usize, disabled: bool, children: Element) -> Element {
    let mut item_opacity = use_motion(0.0f32);
    let mut item_x = use_motion(-20.0f32);

    use_effect({
        let index = index;
        move || {
            // Stagger animation based on index
            let delay = index as f32 * 0.1;

            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis((delay * 1000.0) as u64))
                    .await;

                item_opacity
                    .animate_to(1.0, AnimationConfig::new(AnimationMode::Tween(Tween::default())));
                item_x.animate_to(
                    0.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring::default())),
                );
            });
        }
    });

    let item_style = format!(
        "opacity: {}; transform: translateX({}px); transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);",
        item_opacity.get_value(),
        item_x.get_value()
    );

    rsx! {
        div {
            style: "{item_style}",
            class: if disabled { "opacity-50 pointer-events-none" } else { "" },
            {children}
        }
    }
}
