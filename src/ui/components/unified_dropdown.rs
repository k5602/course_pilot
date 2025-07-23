use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaChevronDown, FaEllipsisVertical};

/// Dropdown item for unified dropdown component
#[derive(Props, Clone, PartialEq)]
pub struct DropdownItem {
    pub label: String,
    #[props(optional)]
    pub icon: Option<String>, // Use string for icon classes instead of Element
    #[props(optional)]
    pub on_select: Option<EventHandler<()>>,
    #[props(optional)]
    pub disabled: bool,
    #[props(optional)]
    pub divider: bool, // Add divider before this item
}

/// Dropdown trigger types
#[derive(Debug, Clone, PartialEq)]
pub enum DropdownTrigger {
    /// Three dots menu (default for action menus)
    DotsMenu,
    /// Button with text and down arrow
    Button { label: String },
}

/// Unified DaisyUI Dropdown Component
/// Replaces ActionMenu, AppDropdown, and other dropdown variants
/// Uses pure DaisyUI classes for consistency and performance
#[component]
pub fn UnifiedDropdown(
    /// Dropdown items to display
    items: Vec<DropdownItem>,
    /// Trigger type and configuration
    #[props(default = DropdownTrigger::DotsMenu)]
    trigger: DropdownTrigger,
    /// Additional CSS classes
    #[props(optional)]
    class: Option<String>,
    /// Dropdown position relative to trigger
    #[props(default = "dropdown-end".to_string())]
    position: String,
) -> Element {
    let base_class = class.as_deref().unwrap_or("");

    rsx! {
        div {
            class: "dropdown {position} {base_class}",
            // Prevent click events from bubbling to parent elements
            onclick: move |evt| {
                evt.stop_propagation();
            },

            // Trigger element using DaisyUI classes
            {match &trigger {
                DropdownTrigger::DotsMenu => rsx! {
                    label {
                        tabindex: "0",
                        class: "btn btn-ghost btn-sm btn-circle",
                        role: "button",
                        onclick: move |evt| {
                            evt.stop_propagation();
                        },
                        Icon { icon: FaEllipsisVertical, class: "w-4 h-4" }
                    }
                },
                DropdownTrigger::Button { label } => rsx! {
                    label {
                        class: "btn btn-outline btn-sm",
                        role: "button",
                        onclick: move |evt| {
                            evt.stop_propagation();
                        },
                        "{label}"
                        Icon { icon: FaChevronDown, class: "w-3 h-3 ml-1" }
                    }
                },
            }}

            // Dropdown menu using DaisyUI classes
            ul {
                class: "dropdown-content menu bg-base-200 rounded-box z-[1] w-52 p-2 shadow-lg border border-base-300",
                tabindex: "0",

                // Render all items
                {items.into_iter().enumerate().map(|(index, item)| {
                    let divider = item.divider;
                    let disabled = item.disabled;
                    let label = item.label;
                    let icon = item.icon;
                    let on_select = item.on_select;

                    rsx! {
                        {if divider && index > 0 {
                            rsx! { li { hr { class: "my-1" } } }
                        } else {
                            rsx! { Fragment {} }
                        }}

                        li {
                            class: if disabled { "disabled" } else { "" },
                            a {
                                class: "flex items-center gap-2",
                                tabindex: if disabled { "-1" } else { "0" },
                                "aria-disabled": disabled.to_string(),
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    if !disabled {
                                        if let Some(handler) = &on_select {
                                            handler.call(());
                                        }
                                    }
                                },

                                {if let Some(icon_class) = &icon {
                                    rsx! { span { class: "text-base-content/70 {icon_class}" } }
                                } else {
                                    rsx! { Fragment {} }
                                }}

                                span { "{label}" }
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Create action menu items for course cards using DRY principle
pub fn create_course_actions(
    _course_id: uuid::Uuid,
    has_structure: bool,
    has_videos: bool,
    on_view: EventHandler<()>,
    on_create_plan: EventHandler<()>,
    on_edit: EventHandler<()>,
    on_structure: EventHandler<()>,
    on_export: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Vec<DropdownItem> {
    vec![
        DropdownItem {
            label: "View Plan".to_string(),
            icon: Some("📋".to_string()),
            on_select: Some(on_view),
            disabled: false,
            divider: false,
        },
        DropdownItem {
            label: "Create Study Plan".to_string(),
            icon: Some("➕".to_string()),
            on_select: Some(on_create_plan),
            disabled: !has_structure,
            divider: false,
        },
        DropdownItem {
            label: "Edit Course".to_string(),
            icon: Some("✏️".to_string()),
            on_select: Some(on_edit),
            disabled: false,
            divider: true,
        },
        DropdownItem {
            label: "Structure Course".to_string(),
            icon: Some("🏗️".to_string()),
            on_select: Some(on_structure),
            disabled: !has_videos || has_structure,
            divider: false,
        },
        DropdownItem {
            label: "Export".to_string(),
            icon: Some("📤".to_string()),
            on_select: Some(on_export),
            disabled: false,
            divider: true,
        },
        DropdownItem {
            label: "Delete".to_string(),
            icon: Some("🗑️".to_string()),
            on_select: Some(on_delete),
            disabled: false,
            divider: false,
        },
    ]
}

/// Create settings dropdown items - reusable pattern
pub fn create_settings_actions(
    on_preferences: EventHandler<()>,
    on_export_all: EventHandler<()>,
    on_import: EventHandler<()>,
    on_about: EventHandler<()>,
) -> Vec<DropdownItem> {
    vec![
        DropdownItem {
            label: "Preferences".to_string(),
            icon: Some("⚙️".to_string()),
            on_select: Some(on_preferences),
            disabled: false,
            divider: false,
        },
        DropdownItem {
            label: "Export All Data".to_string(),
            icon: Some("📦".to_string()),
            on_select: Some(on_export_all),
            disabled: false,
            divider: true,
        },
        DropdownItem {
            label: "Import Data".to_string(),
            icon: Some("📥".to_string()),
            on_select: Some(on_import),
            disabled: false,
            divider: false,
        },
        DropdownItem {
            label: "About".to_string(),
            icon: Some("ℹ️".to_string()),
            on_select: Some(on_about),
            disabled: false,
            divider: true,
        },
    ]
}
