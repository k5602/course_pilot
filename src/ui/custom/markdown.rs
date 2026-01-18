//! Safe Markdown renderer for Dioxus UI components.

use ammonia::Builder;
use dioxus::prelude::*;
use pulldown_cmark::{Options, Parser, html};

/// Props for rendering Markdown safely.
#[derive(Props, PartialEq, Clone)]
pub struct MarkdownRendererProps {
    /// Markdown source text.
    pub src: String,
    /// Optional CSS classes for the wrapper element.
    #[props(optional)]
    pub class: Option<String>,
}

/// Render Markdown safely into sanitized HTML.
/// This strips any raw HTML and keeps only safe tags/attributes.
fn render_markdown_to_html(src: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(src, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Builder::default().clean(&html_output).to_string()
}

/// Safe Markdown renderer component.
///
/// This component:
/// - Converts Markdown to HTML (pulldown-cmark)
/// - Sanitizes HTML (ammonia)
/// - Renders it via `dangerous_inner_html`
#[component]
pub fn MarkdownRenderer(props: MarkdownRendererProps) -> Element {
    let html = render_markdown_to_html(&props.src);
    let class =
        props.class.clone().unwrap_or_else(|| "prose prose-base leading-7 max-w-none".to_string());

    rsx! {
        div {
            class: "{class}",
            dangerous_inner_html: "{html}",
        }
    }
}
