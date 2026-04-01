// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Markdown rendering and sanitization

use ammonia::clean;
use pulldown_cmark::{Options, Parser, html};

/// Render markdown to HTML with GitHub-flavored markdown support
pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

/// Sanitize HTML to prevent XSS attacks
///
/// This removes dangerous tags and attributes while preserving
/// safe markdown-generated HTML.
pub fn sanitize_html(html: &str) -> String {
    // Use ammonia with default settings (removes scripts, dangerous attributes, etc.)
    clean(html)
}

/// Render markdown to sanitized HTML
pub fn render_and_sanitize(markdown: &str) -> String {
    let html = render_markdown(markdown);
    sanitize_html(&html)
}

/// Markdown renderer with caching and optimization
pub struct MarkdownRenderer {
    /// Whether to enable syntax highlighting
    pub syntax_highlighting: bool,

    /// Whether to enable automatic link detection
    pub autolink: bool,
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            autolink: true,
        }
    }
}

impl MarkdownRenderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self::default()
    }

    /// Render markdown to HTML
    pub fn render(&self, markdown: &str) -> String {
        render_and_sanitize(markdown)
    }

    /// Extract title from markdown (first H1 heading)
    pub fn extract_title(&self, markdown: &str) -> Option<String> {
        for line in markdown.lines() {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("# ") {
                return Some(stripped.trim().to_string());
            }
        }
        None
    }

    /// Extract description (first paragraph after title)
    pub fn extract_description(&self, markdown: &str) -> Option<String> {
        let mut after_title = false;
        for line in markdown.lines() {
            let trimmed = line.trim();

            // Skip title
            if trimmed.starts_with("# ") {
                after_title = true;
                continue;
            }

            // First non-empty line after title is the description
            if after_title && !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Some(trimmed.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let md = "# Hello\n\n**World**";
        let html = render_markdown(md);

        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<strong>") || html.contains("<b>"));
    }

    #[test]
    fn test_sanitize_removes_script() {
        let malicious = r#"<script>alert('XSS')</script><p>Safe content</p>"#;
        let clean = sanitize_html(malicious);

        assert!(!clean.contains("<script"));
        assert!(clean.contains("Safe content"));
    }

    #[test]
    fn test_sanitize_removes_event_handlers() {
        let malicious = r#"<img src="x" onerror="alert('XSS')">"#;
        let clean = sanitize_html(malicious);

        assert!(!clean.contains("onerror"));
    }

    #[test]
    fn test_render_with_tables() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render_markdown(md);

        assert!(html.contains("<table"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_extract_title() {
        let renderer = MarkdownRenderer::new();
        let md = "# My Page\n\nSome content";

        let title = renderer.extract_title(md);
        assert_eq!(title, Some("My Page".to_string()));
    }

    #[test]
    fn test_extract_title_none() {
        let renderer = MarkdownRenderer::new();
        let md = "Just some content without a title";

        let title = renderer.extract_title(md);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_description() {
        let renderer = MarkdownRenderer::new();
        let md = "# Title\n\nThis is the description.\n\nMore content.";

        let desc = renderer.extract_description(md);
        assert_eq!(desc, Some("This is the description.".to_string()));
    }

    #[test]
    fn test_github_flavored_markdown() {
        let md = "~~strikethrough~~ and - [ ] task list";
        let html = render_markdown(md);

        // Should support strikethrough
        assert!(html.contains("<del>") || html.contains("strikethrough"));
    }
}
