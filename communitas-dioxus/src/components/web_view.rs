//! Website publishing view per space using x0x KvStore.
//!
//! Each space gets a dedicated KvStore (`x0x-web-{group_prefix}`) holding:
//! - `web:index` -- a JSON array of page paths
//! - `web:{path}` -- the HTML content of a page (base64 encoded, text/html)

use crate::design_tokens::{motion, radius, semantic, spacing, typography};
use crate::x0x_contract;
use base64::Engine as _;
use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;
use tracing::warn;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// KvStore ID for web pages in a space.
fn web_store_id(group_id: &str) -> String {
    format!("x0x-web-{}", x0x_contract::group_prefix(group_id))
}

/// Normalise a path: lowercase, alphanumeric + hyphens + slashes, trimmed.
fn normalize_path(raw: &str) -> String {
    let mut path = String::with_capacity(raw.len());
    let mut last_was_sep = false;

    for ch in raw.trim().chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '.' {
            path.push(lower);
            last_was_sep = false;
        } else if lower == '/' {
            if !last_was_sep {
                path.push('/');
                last_was_sep = true;
            }
        } else if !last_was_sep {
            path.push('-');
            last_was_sep = false;
        }
    }

    let trimmed = path.trim_matches(|c: char| c == '-' || c == '/');
    trimmed.to_string()
}

/// Decode a base64 string to a UTF-8 string, returning an empty string on failure.
fn b64_decode_string(b64: &str) -> String {
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Props for [`WebView`].
#[derive(Props, Clone, PartialEq)]
pub struct WebViewProps {
    /// The group/space ID this web publisher belongs to.
    pub group_id: String,
}

/// Web publishing view -- sidebar page list + preview/editor.
#[component]
pub fn WebView(props: WebViewProps) -> Element {
    let group_id = props.group_id.clone();
    let store_id = web_store_id(&group_id);

    // State
    let mut paths: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_path: Signal<Option<String>> = use_signal(|| None);
    let mut page_content: Signal<String> = use_signal(String::new);
    let mut editing = use_signal(|| false);
    let mut editor_text = use_signal(String::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut new_path_input = use_signal(String::new);
    let mut saving = use_signal(|| false);

    // Ensure store exists and load index on mount
    let init_store_id = store_id.clone();
    use_future(move || {
        let store_id = init_store_id.clone();
        async move {
            let client = X0xClient::new();

            // Ensure the store exists
            if let Err(e) = client.create_store(&store_id, &store_id).await {
                let msg = format!("{e}");
                if !msg.contains("409") && !msg.contains("already") && !msg.contains("exists") {
                    warn!(target: "ui.web", "failed to create web store {store_id}: {e}");
                }
            }

            // Load index
            match client.get(&store_id, "web:index").await {
                Ok(value) => {
                    let json_str = b64_decode_string(&value.value);
                    match serde_json::from_str::<Vec<String>>(&json_str) {
                        Ok(page_paths) => paths.set(page_paths),
                        Err(e) => {
                            warn!(target: "ui.web", "failed to parse web index: {e}");
                        }
                    }
                }
                Err(_) => {
                    // No index yet -- empty website
                }
            }

            loading.set(false);
        }
    });

    // Load a page's content
    let load_page = {
        let store_id = store_id.clone();
        move |path: String| {
            let store_id = store_id.clone();
            spawn(async move {
                selected_path.set(Some(path.clone()));
                editing.set(false);
                page_content.set(String::new());

                let client = X0xClient::new();
                let key = format!("web:{path}");
                match client.get(&store_id, &key).await {
                    Ok(value) => {
                        let text = b64_decode_string(&value.value);
                        page_content.set(text);
                    }
                    Err(e) => {
                        warn!(target: "ui.web", "failed to load web page {path}: {e}");
                        page_content.set(String::new());
                    }
                }
            });
        }
    };

    // Create / add a page path
    let add_page = {
        let store_id = store_id.clone();
        move || {
            let raw = new_path_input();
            let path = normalize_path(&raw);
            if path.is_empty() {
                return;
            }

            let store_id = store_id.clone();
            spawn(async move {
                let client = X0xClient::new();

                let mut current_paths = paths().clone();
                if !current_paths.contains(&path) {
                    current_paths.push(path.clone());
                }

                match serde_json::to_vec(&current_paths) {
                    Ok(index_bytes) => {
                        if let Err(e) = client
                            .put(&store_id, "web:index", &index_bytes, Some("application/json"))
                            .await
                        {
                            warn!(target: "ui.web", "failed to update web index: {e}");
                            error.set(Some(format!("Failed to create page: {e}")));
                            return;
                        }
                    }
                    Err(e) => {
                        warn!(target: "ui.web", "failed to serialize web index: {e}");
                        return;
                    }
                }

                // Create empty page
                let key = format!("web:{path}");
                if let Err(e) = client
                    .put(&store_id, &key, b"", Some("text/html"))
                    .await
                {
                    warn!(target: "ui.web", "failed to create web page {path}: {e}");
                }

                paths.set(current_paths);
                new_path_input.set(String::new());
                selected_path.set(Some(path));
                page_content.set(String::new());
                editing.set(true);
                editor_text.set(String::new());
            });
        }
    };

    // Publish (save) page
    let publish_page = {
        let store_id = store_id.clone();
        move || {
            let path = match selected_path() {
                Some(p) => p,
                None => return,
            };
            let text = editor_text();
            let store_id = store_id.clone();

            saving.set(true);
            spawn(async move {
                let client = X0xClient::new();
                let key = format!("web:{path}");
                if let Err(e) = client
                    .put(&store_id, &key, text.as_bytes(), Some("text/html"))
                    .await
                {
                    warn!(target: "ui.web", "failed to publish web page {path}: {e}");
                    error.set(Some(format!("Failed to publish: {e}")));
                } else {
                    page_content.set(text);
                    editing.set(false);
                }
                saving.set(false);
            });
        }
    };

    // --- Styles ---

    let container_style = format!(
        "display: flex; flex: 1; height: 100%; overflow: hidden; background: {};",
        semantic::BG_PRIMARY
    );

    let sidebar_style = format!(
        "width: 200px; flex-shrink: 0; border-right: 1px solid {}; \
         display: flex; flex-direction: column; overflow-y: auto; \
         background: {}; padding: {};",
        semantic::BORDER_SUBTLE,
        semantic::BG_SECONDARY,
        spacing::SM
    );

    let sidebar_title_style = format!(
        "font-size: {}; font-weight: {}; color: {}; margin-bottom: {};",
        typography::SIZE_XS,
        typography::WEIGHT_SEMIBOLD,
        semantic::TEXT_MUTED,
        spacing::SM
    );

    let content_style = format!(
        "flex: 1; overflow-y: auto; padding: {}; display: flex; flex-direction: column; gap: {};",
        spacing::XL,
        spacing::BASE
    );

    rsx! {
        div {
            style: "{container_style}",

            // Sidebar
            div {
                style: "{sidebar_style}",

                div {
                    style: "{sidebar_title_style}",
                    "WEB PAGES"
                }

                // New path input + add button
                div {
                    style: format!(
                        "display: flex; gap: {}; margin-bottom: {};",
                        spacing::XS,
                        spacing::SM
                    ),

                    input {
                        r#type: "text",
                        placeholder: "page/path",
                        value: "{new_path_input}",
                        style: format!(
                            "flex: 1; min-width: 0; padding: {} {}; background: {}; \
                             border: 1px solid {}; border-radius: {}; color: {}; \
                             font-size: {}; outline: none;",
                            spacing::XS,
                            spacing::SM,
                            semantic::BG_TERTIARY,
                            semantic::BORDER_SUBTLE,
                            radius::SM,
                            semantic::TEXT_PRIMARY,
                            typography::SIZE_XS
                        ),
                        oninput: move |evt: Event<FormData>| {
                            new_path_input.set(evt.value().to_string());
                        },
                        onkeydown: {
                            let add = add_page.clone();
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter {
                                    add();
                                }
                            }
                        },
                    }

                    button {
                        style: format!(
                            "padding: {} {}; background: {}; color: {}; border: none; \
                             border-radius: {}; font-size: {}; cursor: pointer; flex-shrink: 0;",
                            spacing::XS,
                            spacing::SM,
                            semantic::PRIMARY,
                            semantic::TEXT_INVERSE,
                            radius::SM,
                            typography::SIZE_XS
                        ),
                        onclick: {
                            let add = add_page.clone();
                            move |_| add()
                        },
                        "+"
                    }
                }

                // Page list
                if loading() {
                    div {
                        style: format!("color: {}; font-size: {};", semantic::TEXT_MUTED, typography::SIZE_XS),
                        "Loading..."
                    }
                } else if paths().is_empty() {
                    div {
                        style: format!("color: {}; font-size: {};", semantic::TEXT_MUTED, typography::SIZE_XS),
                        "No pages yet."
                    }
                } else {
                    for path in paths() {
                        {
                            let is_active = selected_path().as_deref() == Some(&path);
                            let path_click = path.clone();
                            let load = load_page.clone();
                            rsx! {
                                button {
                                    key: "{path}",
                                    style: format!(
                                        "display: block; width: 100%; text-align: left; \
                                         padding: {} {}; background: {}; color: {}; \
                                         border: none; border-radius: {}; font-size: {}; \
                                         cursor: pointer; transition: {}; margin-bottom: {};",
                                        spacing::XS,
                                        spacing::SM,
                                        if is_active { semantic::BG_ELEVATED } else { "transparent" },
                                        if is_active { semantic::TEXT_PRIMARY } else { semantic::TEXT_SECONDARY },
                                        radius::SM,
                                        typography::SIZE_SM,
                                        motion::transition("background, color"),
                                        spacing::XXS
                                    ),
                                    onclick: move |_| {
                                        let p = path_click.clone();
                                        load(p);
                                    },
                                    "/{path}"
                                }
                            }
                        }
                    }
                }

                // Edit button for selected page
                if selected_path().is_some() && !editing() {
                    button {
                        style: format!(
                            "margin-top: auto; padding: {} {}; background: {}; color: {}; \
                             border: none; border-radius: {}; font-size: {}; \
                             font-weight: {}; cursor: pointer;",
                            spacing::SM,
                            spacing::BASE,
                            semantic::BG_ELEVATED,
                            semantic::TEXT_SECONDARY,
                            radius::MD,
                            typography::SIZE_SM,
                            typography::WEIGHT_MEDIUM
                        ),
                        onclick: move |_| {
                            editor_text.set(page_content().clone());
                            editing.set(true);
                        },
                        "Edit"
                    }
                }
            }

            // Content area
            div {
                style: "{content_style}",

                // Error banner
                if let Some(ref err_msg) = error() {
                    div {
                        style: format!(
                            "padding: {}; background: rgba(239, 68, 68, 0.1); \
                             border: 1px solid {}; border-radius: {}; color: {}; font-size: {};",
                            spacing::SM,
                            semantic::ERROR,
                            radius::MD,
                            semantic::ERROR,
                            typography::SIZE_SM
                        ),
                        "{err_msg}"
                    }
                }

                match selected_path() {
                    None => {
                        rsx! {
                            div {
                                style: format!(
                                    "flex: 1; display: flex; align-items: center; \
                                     justify-content: center; color: {}; font-size: {};",
                                    semantic::TEXT_MUTED,
                                    typography::SIZE_SM
                                ),
                                "Select a page or add a new path."
                            }
                        }
                    }
                    Some(ref path) => {
                        if editing() {
                            // Editor mode
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; flex: 1; gap: 0;",

                                    div {
                                        style: format!(
                                            "display: flex; align-items: center; gap: {}; margin-bottom: {};",
                                            spacing::SM,
                                            spacing::SM
                                        ),
                                        span {
                                            style: format!(
                                                "font-size: {}; font-weight: {}; color: {};",
                                                typography::SIZE_LG,
                                                typography::WEIGHT_SEMIBOLD,
                                                semantic::TEXT_PRIMARY
                                            ),
                                            "Editing: /{path}"
                                        }
                                    }

                                    textarea {
                                        value: "{editor_text}",
                                        style: format!(
                                            "flex: 1; min-height: 300px; padding: {}; \
                                             background: {}; border: 1px solid {}; \
                                             border-radius: {}; color: {}; font-family: {}; \
                                             font-size: {}; line-height: {}; resize: vertical; \
                                             outline: none;",
                                            spacing::BASE,
                                            semantic::BG_TERTIARY,
                                            semantic::BORDER_DEFAULT,
                                            radius::MD,
                                            semantic::TEXT_PRIMARY,
                                            typography::FONT_MONO,
                                            typography::SIZE_SM,
                                            typography::LEADING_NORMAL
                                        ),
                                        oninput: move |evt: Event<FormData>| {
                                            editor_text.set(evt.value().to_string());
                                        },
                                    }

                                    // Buttons
                                    div {
                                        style: format!(
                                            "display: flex; gap: {}; margin-top: {};",
                                            spacing::SM,
                                            spacing::SM
                                        ),

                                        button {
                                            style: format!(
                                                "padding: {} {}; background: {}; color: {}; \
                                                 border: none; border-radius: {}; font-size: {}; \
                                                 font-weight: {}; cursor: {}; opacity: {};",
                                                spacing::SM,
                                                spacing::BASE,
                                                semantic::PRIMARY,
                                                semantic::TEXT_INVERSE,
                                                radius::MD,
                                                typography::SIZE_SM,
                                                typography::WEIGHT_SEMIBOLD,
                                                if saving() { "not-allowed" } else { "pointer" },
                                                if saving() { "0.5" } else { "1" }
                                            ),
                                            disabled: saving(),
                                            onclick: {
                                                let mut publish = publish_page.clone();
                                                move |_| publish()
                                            },
                                            if saving() { "Publishing..." } else { "Publish" }
                                        }

                                        button {
                                            style: format!(
                                                "padding: {} {}; background: transparent; \
                                                 color: {}; border: 1px solid {}; \
                                                 border-radius: {}; font-size: {}; cursor: pointer;",
                                                spacing::SM,
                                                spacing::BASE,
                                                semantic::TEXT_SECONDARY,
                                                semantic::BORDER_DEFAULT,
                                                radius::MD,
                                                typography::SIZE_SM
                                            ),
                                            onclick: move |_| editing.set(false),
                                            "Cancel"
                                        }
                                    }
                                }
                            }
                        } else {
                            // Preview mode
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; flex: 1;",

                                    // Title
                                    div {
                                        style: format!(
                                            "display: flex; align-items: center; gap: {}; margin-bottom: {};",
                                            spacing::SM,
                                            spacing::BASE
                                        ),
                                        span {
                                            style: format!(
                                                "font-size: {}; font-weight: {}; color: {};",
                                                typography::SIZE_XL,
                                                typography::WEIGHT_BOLD,
                                                semantic::TEXT_PRIMARY
                                            ),
                                            "/{path}"
                                        }
                                        span {
                                            style: format!(
                                                "font-size: {}; color: {}; margin-left: auto;",
                                                typography::SIZE_XS,
                                                semantic::TEXT_MUTED
                                            ),
                                            "text/html"
                                        }
                                    }

                                    // Content preview in <pre> code block (no raw HTML rendering)
                                    if page_content().is_empty() {
                                        div {
                                            style: format!(
                                                "color: {}; font-size: {}; font-style: italic;",
                                                semantic::TEXT_MUTED,
                                                typography::SIZE_SM
                                            ),
                                            "This page is empty. Click Edit to add content."
                                        }
                                    } else {
                                        pre {
                                            style: format!(
                                                "white-space: pre-wrap; word-wrap: break-word; \
                                                 font-family: {}; font-size: {}; color: {}; \
                                                 line-height: {}; margin: 0; padding: {}; \
                                                 background: {}; border: 1px solid {}; \
                                                 border-radius: {}; overflow-x: auto;",
                                                typography::FONT_MONO,
                                                typography::SIZE_SM,
                                                semantic::TEXT_PRIMARY,
                                                typography::LEADING_NORMAL,
                                                spacing::BASE,
                                                semantic::BG_TERTIARY,
                                                semantic::BORDER_SUBTLE,
                                                radius::MD
                                            ),
                                            code {
                                                "{page_content}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
