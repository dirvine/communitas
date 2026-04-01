// SPDX-License-Identifier: MIT OR Apache-2.0

//! Constitution view — displays the x0x constitution fetched from the daemon.
//!
//! The x0x constitution defines the foundational principles, rights,
//! responsibilities, and governance for all Intelligent Entities on the
//! network.  It is embedded in every x0x binary at compile time and served
//! via `GET /constitution/json` (auth-exempt).

use crate::tokens::colors;
use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;

/// Render the constitution inside a scrollable pane with proper markdown
/// formatting.  Fetches the constitution from the local x0xd daemon on mount.
#[component]
pub fn ConstitutionView() -> Element {
    let mut markdown_html = use_signal(|| None::<String>);
    let mut version = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);

    // Fetch on mount
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            error_msg.set(None);

            let client = X0xClient::new();
            match client.constitution_json().await {
                Ok(info) => {
                    version.set(info.version);
                    status.set(info.status);

                    // Convert markdown → HTML via pulldown-cmark
                    let parser = pulldown_cmark::Parser::new(&info.content);
                    let mut html_output = String::new();
                    pulldown_cmark::html::push_html(&mut html_output, parser);
                    markdown_html.set(Some(html_output));
                }
                Err(e) => {
                    error_msg.set(Some(format!("{e}")));
                }
            }
            loading.set(false);
        });
    });

    let container_style = format!(
        "display: flex; flex-direction: column; height: 100%; \
         background: {}; color: {};",
        colors::SURFACE_BG,
        colors::TEXT_PRIMARY,
    );

    let muted = colors::TEXT_MUTED;
    let primary_text = colors::TEXT_PRIMARY;

    rsx! {
        div {
            style: "{container_style}",

            if *loading.read() {
                div {
                    style: format!(
                        "display: flex; align-items: center; justify-content: center; \
                         height: 100%; color: {};",
                        muted,
                    ),
                    "Loading constitution…"
                }
            } else if let Some(err) = error_msg.read().as_ref() {
                div {
                    style: "display: flex; flex-direction: column; align-items: center; \
                            justify-content: center; height: 100%; gap: 1rem;",
                    div {
                        style: format!(
                            "font-size: 1.125rem; font-weight: 600; color: {};",
                            primary_text,
                        ),
                        "Could not load constitution"
                    }
                    div {
                        style: format!("font-size: 0.8125rem; color: {};", muted),
                        "{err}"
                    }
                    div {
                        style: format!("font-size: 0.75rem; color: {};", muted),
                        "Make sure x0xd is running."
                    }
                }
            } else if let Some(html) = markdown_html.read().as_ref() {
                // Version badge bar
                div {
                    style: format!(
                        "display: flex; align-items: center; gap: 0.5rem; \
                         padding: 0.75rem 2rem; border-bottom: 1px solid {}; \
                         flex-shrink: 0;",
                        colors::BORDER_DEFAULT,
                    ),
                    span {
                        style: format!("font-size: 0.75rem; color: {};", muted),
                        {format!("v{}", version.read())}
                    }
                    span {
                        style: format!("color: {};", muted),
                        "·"
                    }
                    span {
                        style: format!(
                            "font-size: 0.6875rem; color: {}; \
                             background: rgba(0,212,255,0.12); \
                             padding: 2px 8px; border-radius: 9999px;",
                            colors::PRIMARY,
                        ),
                        {status.read().clone()}
                    }
                }

                // Scrollable markdown body
                div {
                    style: "flex: 1; overflow-y: auto; padding: 2rem;",
                    div {
                        style: format!(
                            "max-width: 720px; margin: 0 auto; \
                             line-height: 1.7; font-size: 0.9375rem; \
                             color: {};",
                            colors::TEXT_SECONDARY,
                        ),
                        // Inject scoped styles for markdown elements
                        style { "{CONSTITUTION_CSS}" }
                        div {
                            class: "constitution-body",
                            dangerous_inner_html: "{html}",
                        }
                    }
                }
            }
        }
    }
}

/// Scoped CSS for the constitution markdown rendering.
const CONSTITUTION_CSS: &str = r#"
.constitution-body h1 {
    font-size: 1.75rem;
    font-weight: 700;
    color: #e4e6f0;
    margin: 2rem 0 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #1e2140;
}
.constitution-body h2 {
    font-size: 1.375rem;
    font-weight: 600;
    color: #e4e6f0;
    margin: 1.75rem 0 0.75rem;
    padding-bottom: 0.375rem;
    border-bottom: 1px solid #1e2140;
}
.constitution-body h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: #e4e6f0;
    margin: 1.5rem 0 0.5rem;
}
.constitution-body h4 {
    font-size: 1rem;
    font-weight: 600;
    color: #b0b3cc;
    margin: 1.25rem 0 0.5rem;
}
.constitution-body p {
    margin: 0.75rem 0;
}
.constitution-body em {
    color: #b0b3cc;
    font-style: italic;
}
.constitution-body strong {
    color: #e4e6f0;
    font-weight: 600;
}
.constitution-body hr {
    border: none;
    border-top: 1px solid #1e2140;
    margin: 1.5rem 0;
}
.constitution-body ul, .constitution-body ol {
    padding-left: 1.5rem;
    margin: 0.5rem 0;
}
.constitution-body li {
    margin: 0.25rem 0;
}
.constitution-body blockquote {
    border-left: 3px solid #00d4ff;
    padding: 0.5rem 1rem;
    margin: 0.75rem 0;
    background: rgba(0, 212, 255, 0.05);
    border-radius: 0 6px 6px 0;
}
.constitution-body a {
    color: #00d4ff;
    text-decoration: none;
}
.constitution-body a:hover {
    text-decoration: underline;
}
.constitution-body table {
    width: 100%;
    border-collapse: collapse;
    margin: 0.75rem 0;
    font-size: 0.875rem;
}
.constitution-body th {
    text-align: left;
    padding: 0.5rem 0.75rem;
    border-bottom: 2px solid #1e2140;
    color: #e4e6f0;
    font-weight: 600;
}
.constitution-body td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid #1e2140;
}
.constitution-body code {
    background: rgba(0, 212, 255, 0.08);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    font-size: 0.85em;
    color: #00d4ff;
}
"#;
