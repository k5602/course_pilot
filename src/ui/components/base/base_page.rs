use dioxus::prelude::*;

/// Props for the BasePage component
#[derive(Props, PartialEq, Clone)]
pub struct BasePageProps {
    /// Page title
    #[props(optional)]
    pub title: Option<String>,

    /// Page subtitle or description
    #[props(optional)]
    pub subtitle: Option<String>,

    /// Page content
    pub children: Element,

    /// Header actions (e.g., buttons, dropdowns)
    #[props(optional)]
    pub header_actions: Option<Element>,

    /// Breadcrumbs component
    #[props(optional)]
    pub breadcrumbs: Option<Element>,

    /// Additional CSS classes for the container
    #[props(default = "")]
    pub class: &'static str,

    /// Container max width
    #[props(default = "max-w-7xl")]
    pub max_width: &'static str,

    /// Enable padding
    #[props(default = true)]
    pub padded: bool,

    /// Background variant
    #[props(default = "bg-base-100")]
    pub background: &'static str,
}

/// Generic BasePage component using DaisyUI styling
/// Provides consistent page layout with header, breadcrumbs, and content
#[component]
pub fn BasePage(props: BasePageProps) -> Element {
    let container_classes =
        format!("min-h-screen {} {}", props.background, if props.padded { "p-6" } else { "" });

    let content_classes = format!("mx-auto {} {}", props.max_width, props.class);

    rsx! {
        div {
            class: "{container_classes}",

            div {
                class: "{content_classes}",

                // Breadcrumbs
                if let Some(breadcrumbs) = &props.breadcrumbs {
                    div {
                        class: "mb-4",
                        {breadcrumbs.clone()}
                    }
                }

                // Page header
                if props.title.is_some() || props.header_actions.is_some() {
                    header {
                        class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6",

                        // Title section
                        if props.title.is_some() {
                            div {
                                class: "flex-1",

                                h1 {
                                    class: "text-3xl font-bold text-base-content",
                                    "{props.title.as_ref().unwrap()}"
                                }

                                if let Some(subtitle) = &props.subtitle {
                                    p {
                                        class: "text-base-content/70 mt-2",
                                        "{subtitle}"
                                    }
                                }
                            }
                        }

                        // Header actions
                        if let Some(header_actions) = &props.header_actions {
                            div {
                                class: "flex items-center gap-2 flex-shrink-0",
                                {header_actions.clone()}
                            }
                        }
                    }
                }

                // Main content
                main {
                    class: "w-full",
                    {props.children}
                }
            }
        }
    }
}
