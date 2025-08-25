use dioxus::prelude::*;

#[component]
pub fn MarkdownRenderer(content: String) -> Element {
    let html_content = use_memo(move || markdown::to_html(&content));

    rsx! {
        div {
            class: "prose prose-sm max-w-none prose-slate dark:prose-invert",
            dangerous_inner_html: "{html_content}",
        }
    }
}

/// Render markdown content with custom styling for chat messages
#[component]
pub fn ChatMarkdownRenderer(content: String) -> Element {
    let html_content = use_memo(move || {
        // Convert markdown to HTML
        let mut html = markdown::to_html(&content);

        // Apply custom styling for chat context
        html = html
            // Style code blocks
            .replace("<pre><code>", "<pre class=\"bg-base-300 p-2 rounded text-sm overflow-x-auto\"><code class=\"text-base-content\">")
            .replace("</code></pre>", "</code></pre>")
            // Style inline code
            .replace("<code>", "<code class=\"bg-base-300 px-1 py-0.5 rounded text-sm text-base-content\">")
            // Style blockquotes
            .replace("<blockquote>", "<blockquote class=\"border-l-4 border-primary pl-4 italic text-base-content/80\">")
            // Style headers
            .replace("<h1>", "<h1 class=\"text-lg font-bold text-base-content mb-2\">")
            .replace("<h2>", "<h2 class=\"text-base font-bold text-base-content mb-2\">")
            .replace("<h3>", "<h3 class=\"text-sm font-bold text-base-content mb-1\">")
            // Style lists
            .replace("<ul>", "<ul class=\"list-disc list-inside space-y-1 text-base-content\">")
            .replace("<ol>", "<ol class=\"list-decimal list-inside space-y-1 text-base-content\">")
            .replace("<li>", "<li class=\"text-base-content\">")
            // Style paragraphs
            .replace("<p>", "<p class=\"text-base-content mb-2\">")
            // Style links
            .replace("<a ", "<a class=\"text-primary hover:text-primary-focus underline\" ");

        html
    });

    rsx! {
        div {
            class: "markdown-content",
            dangerous_inner_html: "{html_content}",
        }
    }
}
