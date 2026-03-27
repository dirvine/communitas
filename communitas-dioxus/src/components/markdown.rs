//! Lightweight markdown renderer for message bubbles.
//!
//! Supports a safe, minimal subset of CommonMark:
//! - Fenced code blocks (triple-backtick, with optional language tag)
//! - Block quotes (lines starting with `> `)
//! - **Bold** (`**text**` or `__text__`)
//! - *Italic* (`*text*` or `_text_`)
//! - `Inline code` (single-backtick)
//! - Plain text (everything else)
//!
//! The composer always stores raw text; rendering happens display-only.

use crate::design_tokens::{palette, radius, semantic, spacing, typography};
use dioxus::prelude::*;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A block-level element in the parsed document.
#[derive(Clone, PartialEq, Debug)]
enum Block {
    /// A fenced code block. `lang` is the optional language tag.
    Code { lang: Option<String>, text: String },
    /// A block-quote (one or more `> ` prefixed lines).
    Quote(Vec<Inline>),
    /// A normal paragraph (may contain inline markup).
    Para(Vec<Inline>),
}

/// An inline-level span.
#[derive(Clone, PartialEq, Debug)]
enum Inline {
    Plain(String),
    Bold(String),
    Italic(String),
    Code(String),
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse raw message text into a list of [`Block`]s.
fn parse_blocks(text: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Fenced code block: starts with ```
        if line.trim_start().starts_with("```") {
            let fence = line.trim_start();
            let lang_tag = fence.trim_start_matches('`').trim();
            let lang = if lang_tag.is_empty() {
                None
            } else {
                Some(lang_tag.to_string())
            };
            i += 1;
            let mut code_lines: Vec<&str> = Vec::new();
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            // Skip closing ```
            if i < lines.len() {
                i += 1;
            }
            blocks.push(Block::Code {
                lang,
                text: code_lines.join("\n"),
            });
            continue;
        }

        // Block quote: starts with `> `
        if line.starts_with("> ") || line == ">" {
            let mut quote_lines: Vec<&str> = Vec::new();
            while i < lines.len()
                && (lines[i].starts_with("> ") || lines[i] == ">")
            {
                let stripped = lines[i].strip_prefix("> ").unwrap_or(
                    lines[i].strip_prefix('>').unwrap_or(lines[i]),
                );
                quote_lines.push(stripped);
                i += 1;
            }
            let quote_text = quote_lines.join(" ");
            blocks.push(Block::Quote(parse_inlines(&quote_text)));
            continue;
        }

        // Normal paragraph line — collect consecutive non-special lines
        let mut para_lines: Vec<&str> = Vec::new();
        while i < lines.len()
            && !lines[i].trim_start().starts_with("```")
            && !lines[i].starts_with("> ")
            && lines[i] != ">"
        {
            para_lines.push(lines[i]);
            i += 1;
        }
        let para_text = para_lines.join("\n");
        if !para_text.trim().is_empty() {
            blocks.push(Block::Para(parse_inlines(&para_text)));
        }
    }

    blocks
}

/// Parse a string into a sequence of [`Inline`] spans.
fn parse_inlines(text: &str) -> Vec<Inline> {
    let mut spans: Vec<Inline> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut plain_buf = String::new();

    /// Flush `plain_buf` into spans if non-empty.
    fn flush(buf: &mut String, spans: &mut Vec<Inline>) {
        if !buf.is_empty() {
            spans.push(Inline::Plain(std::mem::take(buf)));
        }
    }

    while i < chars.len() {
        let c = chars[i];

        // Inline code: backtick
        if c == '`' {
            flush(&mut plain_buf, &mut spans);
            i += 1;
            let mut code_buf = String::new();
            while i < chars.len() && chars[i] != '`' {
                code_buf.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1; // skip closing backtick
            }
            if !code_buf.is_empty() {
                spans.push(Inline::Code(code_buf));
            }
            continue;
        }

        // Bold: **text** or __text__
        if (c == '*' && chars.get(i + 1) == Some(&'*'))
            || (c == '_' && chars.get(i + 1) == Some(&'_'))
        {
            let delim = c;
            // Look ahead for a closing pair
            let close_pos = chars[i + 2..]
                .windows(2)
                .position(|w| w == [delim, delim])
                .map(|p| p + i + 2);
            if let Some(end) = close_pos {
                flush(&mut plain_buf, &mut spans);
                let inner: String = chars[i + 2..end].iter().collect();
                if !inner.is_empty() {
                    spans.push(Inline::Bold(inner));
                }
                i = end + 2;
                continue;
            }
        }

        // Italic: *text* or _text_ (single delimiter, not followed by same delimiter)
        if (c == '*' && chars.get(i + 1) != Some(&'*'))
            || (c == '_' && chars.get(i + 1) != Some(&'_'))
        {
            let delim = c;
            // Look ahead for single closing delimiter
            let close_pos = chars[i + 1..]
                .iter()
                .position(|&ch| ch == delim)
                .map(|p| p + i + 1);
            if let Some(end) = close_pos {
                let inner: String = chars[i + 1..end].iter().collect();
                // Only treat as italic if inner is non-empty and doesn't contain newlines
                if !inner.is_empty() && !inner.contains('\n') {
                    flush(&mut plain_buf, &mut spans);
                    spans.push(Inline::Italic(inner));
                    i = end + 1;
                    continue;
                }
            }
        }

        plain_buf.push(c);
        i += 1;
    }

    flush(&mut plain_buf, &mut spans);
    spans
}

// ---------------------------------------------------------------------------
// Dioxus component
// ---------------------------------------------------------------------------

/// Renders message content with lightweight markdown formatting.
///
/// Supports: fenced code blocks, block quotes, **bold**, *italic*, `inline code`.
/// Plain text is preserved exactly. This is display-only — the composer stores raw text.
#[component]
pub fn MarkdownContent(
    /// Raw message text to render.
    content: String,
    /// Text color (defaults to semantic primary text).
    #[props(default)]
    text_color: Option<String>,
    /// Whether this message is from the current user (affects code block styling).
    #[props(default = false)]
    is_own: bool,
) -> Element {
    let blocks = parse_blocks(&content);
    let color = text_color
        .as_deref()
        .unwrap_or(if is_own { "white" } else { semantic::TEXT_PRIMARY });

    rsx! {
        div {
            style: format!(
                "font-size: {}; \
                 line-height: {}; \
                 color: {}; \
                 word-wrap: break-word; \
                 display: flex; \
                 flex-direction: column; \
                 gap: {};",
                typography::SIZE_BASE,
                typography::LEADING_RELAXED,
                color,
                spacing::XS
            ),

            for block in blocks.iter() {
                {render_block(block, is_own)}
            }
        }
    }
}

/// Render a single [`Block`] as a Dioxus element.
fn render_block(block: &Block, is_own: bool) -> Element {
    match block {
        Block::Code { lang, text } => {
            let lang_label = lang.as_deref().unwrap_or("").to_string();
            let text = text.clone();
            rsx! {
                div {
                    style: format!(
                        "background: {}; \
                         border-radius: {}; \
                         overflow: hidden; \
                         border: 1px solid {};",
                        if is_own { "rgba(0,0,0,0.25)" } else { semantic::BG_ELEVATED },
                        radius::MD,
                        if is_own { "rgba(255,255,255,0.15)" } else { semantic::BORDER_DEFAULT }
                    ),

                    // Language label bar
                    if !lang_label.is_empty() {
                        div {
                            style: format!(
                                "padding: {} {}; \
                                 font-size: {}; \
                                 font-family: {}; \
                                 color: {}; \
                                 border-bottom: 1px solid {};",
                                spacing::XXS,
                                spacing::SM,
                                typography::SIZE_XS,
                                typography::FONT_MONO,
                                if is_own { "rgba(255,255,255,0.6)" } else { semantic::TEXT_MUTED },
                                if is_own { "rgba(255,255,255,0.15)" } else { semantic::BORDER_DEFAULT }
                            ),
                            "{lang_label}"
                        }
                    }

                    pre {
                        style: format!(
                            "margin: 0; \
                             padding: {}; \
                             font-family: {}; \
                             font-size: {}; \
                             overflow-x: auto; \
                             white-space: pre; \
                             color: {};",
                            spacing::SM,
                            typography::FONT_MONO,
                            typography::SIZE_SM,
                            if is_own { "rgba(255,255,255,0.9)" } else { semantic::TEXT_PRIMARY }
                        ),
                        "{text}"
                    }
                }
            }
        }

        Block::Quote(inlines) => {
            rsx! {
                div {
                    style: format!(
                        "padding: {} {}; \
                         border-left: 3px solid {}; \
                         background: {}; \
                         border-radius: 0 {} {} 0; \
                         font-style: italic;",
                        spacing::XS,
                        spacing::SM,
                        if is_own { "rgba(255,255,255,0.5)" } else { palette::JADE_500 },
                        if is_own { "rgba(0,0,0,0.15)" } else { semantic::BG_SECONDARY },
                        radius::SM,
                        radius::SM
                    ),
                    {render_inlines(inlines, is_own)}
                }
            }
        }

        Block::Para(inlines) => {
            rsx! {
                p {
                    style: "margin: 0;",
                    {render_inlines(inlines, is_own)}
                }
            }
        }
    }
}

/// Render a slice of [`Inline`] spans as a Dioxus element (a fragment of spans).
fn render_inlines(inlines: &[Inline], is_own: bool) -> Element {
    rsx! {
        for inline in inlines.iter() {
            {render_inline(inline, is_own)}
        }
    }
}

/// Render a single [`Inline`] span.
fn render_inline(inline: &Inline, is_own: bool) -> Element {
    match inline {
        Inline::Plain(text) => {
            let text = text.clone();
            rsx! {
                span { "{text}" }
            }
        }
        Inline::Bold(text) => {
            let text = text.clone();
            rsx! {
                strong {
                    style: format!("font-weight: {};", typography::WEIGHT_BOLD),
                    "{text}"
                }
            }
        }
        Inline::Italic(text) => {
            let text = text.clone();
            rsx! {
                em {
                    style: "font-style: italic;",
                    "{text}"
                }
            }
        }
        Inline::Code(text) => {
            let text = text.clone();
            rsx! {
                code {
                    style: format!(
                        "font-family: {}; \
                         font-size: 0.875em; \
                         padding: 1px 4px; \
                         border-radius: 3px; \
                         background: {}; \
                         color: {};",
                        typography::FONT_MONO,
                        if is_own { "rgba(0,0,0,0.25)" } else { semantic::BG_TERTIARY },
                        if is_own { "rgba(255,255,255,0.9)" } else { palette::JADE_400 }
                    ),
                    "{text}"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_text() {
        let blocks = parse_blocks("Hello world");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0], Block::Para(_)));
        if let Block::Para(ref inlines) = blocks[0] {
            assert_eq!(inlines, &[Inline::Plain("Hello world".into())]);
        }
    }

    #[test]
    fn parse_bold() {
        let inlines = parse_inlines("Hello **world**!");
        assert_eq!(
            inlines,
            vec![
                Inline::Plain("Hello ".into()),
                Inline::Bold("world".into()),
                Inline::Plain("!".into()),
            ]
        );
    }

    #[test]
    fn parse_italic() {
        let inlines = parse_inlines("Say *hi* there");
        assert_eq!(
            inlines,
            vec![
                Inline::Plain("Say ".into()),
                Inline::Italic("hi".into()),
                Inline::Plain(" there".into()),
            ]
        );
    }

    #[test]
    fn parse_inline_code() {
        let inlines = parse_inlines("Run `cargo test` now");
        assert_eq!(
            inlines,
            vec![
                Inline::Plain("Run ".into()),
                Inline::Code("cargo test".into()),
                Inline::Plain(" now".into()),
            ]
        );
    }

    #[test]
    fn parse_fenced_code_block() {
        let text = "```rust\nfn main() {}\n```";
        let blocks = parse_blocks(text);
        assert_eq!(blocks.len(), 1);
        if let Block::Code { ref lang, ref text } = blocks[0] {
            assert_eq!(lang.as_deref(), Some("rust"));
            assert_eq!(text, "fn main() {}");
        } else {
            panic!("expected Code block");
        }
    }

    #[test]
    fn parse_blockquote() {
        let text = "> Be the change";
        let blocks = parse_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0], Block::Quote(_)));
    }

    #[test]
    fn parse_mixed_blocks() {
        let text = "Before\n```\ncode\n```\nAfter";
        let blocks = parse_blocks(text);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(blocks[0], Block::Para(_)));
        assert!(matches!(blocks[1], Block::Code { .. }));
        assert!(matches!(blocks[2], Block::Para(_)));
    }

    #[test]
    fn bold_double_underscore() {
        let inlines = parse_inlines("__bold__");
        assert_eq!(inlines, vec![Inline::Bold("bold".into())]);
    }
}
