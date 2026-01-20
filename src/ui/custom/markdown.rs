//! Safe Markdown renderer for Dioxus UI components.

use std::sync::atomic::{AtomicUsize, Ordering};

use ammonia::Builder;
use dioxus::prelude::*;
use pulldown_cmark::{Options, Parser, html};

static MARKDOWN_RENDERER_ID: AtomicUsize = AtomicUsize::new(1);

fn next_markdown_id() -> usize {
    MARKDOWN_RENDERER_ID.fetch_add(1, Ordering::Relaxed)
}

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
    let element_id = use_signal(next_markdown_id);
    let element_id_str = format!("markdown-render-{}", *element_id.read());
    let src = props.src.clone();

    {
        let element_id_str = element_id_str.clone();
        use_effect(move || {
            let element_id_str = element_id_str.clone();
            let _src = src.clone();
            spawn(async move {
                let eval_script = format!(
                    r#"
                    (function renderWithRetry(attempts) {{
                        const el = document.getElementById("{id}");
                        if (!el) {{
                            if (attempts > 0) {{
                                setTimeout(function() {{ renderWithRetry(attempts - 1); }}, 100);
                            }}
                            return;
                        }}

                        function render() {{
                            if (typeof renderMathInElement === "function") {{
                                renderMathInElement(el, {{
                                    delimiters: [
                                        {{left: "$$", right: "$$", display: true}},
                                        {{left: "$", right: "$", display: false}},
                                        {{left: "\\(", right: "\\)", display: false}},
                                        {{left: "\\[", right: "\\]", display: true}}
                                    ],
                                    throwOnError: false
                                }});
                            }}
                        }}

                        function ensureAssets() {{
                            const head = document.head || document.getElementsByTagName("head")[0];

                            if (!document.getElementById("katex-css")) {{
                                const link = document.createElement("link");
                                link.id = "katex-css";
                                link.rel = "stylesheet";
                                link.href = "https://cdn.jsdelivr.net/npm/katex@0.16.25/dist/katex.min.css";
                                head.appendChild(link);
                            }}

                            if (!document.getElementById("katex-js")) {{
                                const script = document.createElement("script");
                                script.id = "katex-js";
                                script.defer = true;
                                script.src = "https://cdn.jsdelivr.net/npm/katex@0.16.25/dist/katex.min.js";
                                head.appendChild(script);
                            }}

                            if (!document.getElementById("katex-auto-render")) {{
                                const script = document.createElement("script");
                                script.id = "katex-auto-render";
                                script.defer = true;
                                script.src = "https://cdn.jsdelivr.net/npm/katex@0.16.25/dist/contrib/auto-render.min.js";
                                script.onload = render;
                                head.appendChild(script);
                            }}
                        }}

                        if (typeof renderMathInElement !== "function") {{
                            ensureAssets();
                            if (attempts > 0) {{
                                setTimeout(function() {{ renderWithRetry(attempts - 1); }}, 150);
                            }}
                            return;
                        }}

                        render();
                    }})(20);
                    "#,
                    id = element_id_str
                );

                let _ = document::eval(&eval_script).await;
            });
        });
    }

    rsx! {
        div {
            id: "{element_id_str}",
            class: "{class}",
            dangerous_inner_html: "{html}",
        }
    }
}
