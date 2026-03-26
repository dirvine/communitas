//! Collaborative wiki pages per space using x0x KvStore.
//!
//! Each space gets a dedicated KvStore (`x0x-wiki-{group_prefix}`) holding:
//! - `wiki:index` -- a JSON array of page slugs
//! - `wiki:{slug}` -- the text content of a page (base64 encoded, text/plain)

use crate::design_tokens::{motion, radius, semantic, spacing, typography};
use crate::x0x_contract;
use base64::Engine as _;
use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;
use tracing::warn;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// KvStore ID for wiki pages in a space.
fn wiki_store_id(group_id: &str) -> String {
    format!("x0x-wiki-{}", x0x_contract::group_prefix(group_id))
}

/// Normalise a slug: lowercase, alphanumeric + hyphens, trimmed.
fn normalize_slug(raw: &str) -> String {
    let mut slug = String::with_capacity(raw.len());
    let mut last_was_dash = false;

    for ch in raw.trim().chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
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

/// Props for [`WikiView`].
#[derive(Props, Clone, PartialEq)]
pub struct WikiViewProps {
    /// The group/space ID this wiki belongs to.
    pub group_id: String,
}

/// Collaborative wiki view -- sidebar page list + reader/editor.
#[component]
pub fn WikiView(props: WikiViewProps) -> Element {
    let group_id = props.group_id.clone();
    let store_id = wiki_store_id(&group_id);

    // State
    let mut pages: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_slug: Signal<Option<String>> = use_signal(|| None);
    let mut page_content: Signal<String> = use_signal(String::new);
    let mut editing = use_signal(|| false);
    let mut editor_text = use_signal(String::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut new_slug_input = use_signal(String::new);
    let mut saving = use_signal(|| false);

    // Ensure store exists and load index on mount
    let init_store_id = store_id.clone();
    use_future(move || {
        let store_id = init_store_id.clone();
        async move {
            let client = X0xClient::new();

            // Ensure the store exists (create is idempotent / will 409 if exists)
            if let Err(e) = client.create_store(&store_id, &store_id).await {
                let msg = format!("{e}");
                // Ignore "already exists" style errors
                if !msg.contains("409") && !msg.contains("already") && !msg.contains("exists") {
                    warn!(target: "ui.wiki", "failed to create wiki store {store_id}: {e}");
                }
            }

            // Load index
            match client.get(&store_id, "wiki:index").await {
                Ok(value) => {
                    let json_str = b64_decode_string(&value.value);
                    match serde_json::from_str::<Vec<String>>(&json_str) {
                        Ok(slugs) => pages.set(slugs),
                        Err(e) => {
                            warn!(target: "ui.wiki", "failed to parse wiki index: {e}");
                        }
                    }
                }
                Err(_) => {
                    // No index yet -- empty wiki
                }
            }

            loading.set(false);
        }
    });

    // Load a page's content
    let load_page = {
        let store_id = store_id.clone();
        move |slug: String| {
            let store_id = store_id.clone();
            spawn(async move {
                selected_slug.set(Some(slug.clone()));
                editing.set(false);
                page_content.set(String::new());

                let client = X0xClient::new();
                let key = format!("wiki:{slug}");
                match client.get(&store_id, &key).await {
                    Ok(value) => {
                        let text = b64_decode_string(&value.value);
                        page_content.set(text);
                    }
                    Err(e) => {
                        warn!(target: "ui.wiki", "failed to load wiki page {slug}: {e}");
                        page_content.set(String::new());
                    }
                }
            });
        }
    };

    // Create a new page
    let create_page = {
        let store_id = store_id.clone();
        move || {
            let raw = new_slug_input();
            let slug = normalize_slug(&raw);
            if slug.is_empty() {
                return;
            }

            let store_id = store_id.clone();
            spawn(async move {
                let client = X0xClient::new();

                // Add slug to index
                let mut current_pages = pages().clone();
                if !current_pages.contains(&slug) {
                    current_pages.push(slug.clone());
                }

                match serde_json::to_vec(&current_pages) {
                    Ok(index_bytes) => {
                        if let Err(e) = client
                            .put(
                                &store_id,
                                "wiki:index",
                                &index_bytes,
                                Some("application/json"),
                            )
                            .await
                        {
                            warn!(target: "ui.wiki", "failed to update wiki index: {e}");
                            error.set(Some(format!("Failed to create page: {e}")));
                            return;
                        }
                    }
                    Err(e) => {
                        warn!(target: "ui.wiki", "failed to serialize wiki index: {e}");
                        return;
                    }
                }

                // Create empty page content
                let key = format!("wiki:{slug}");
                if let Err(e) = client.put(&store_id, &key, b"", Some("text/plain")).await {
                    warn!(target: "ui.wiki", "failed to create wiki page {slug}: {e}");
                }

                pages.set(current_pages);
                new_slug_input.set(String::new());
                selected_slug.set(Some(slug));
                page_content.set(String::new());
                editing.set(true);
                editor_text.set(String::new());
            });
        }
    };

    // Save page
    let save_page = {
        let store_id = store_id.clone();
        move || {
            let slug = match selected_slug() {
                Some(s) => s,
                None => return,
            };
            let text = editor_text();
            let store_id = store_id.clone();

            saving.set(true);
            spawn(async move {
                let client = X0xClient::new();
                let key = format!("wiki:{slug}");
                if let Err(e) = client
                    .put(&store_id, &key, text.as_bytes(), Some("text/plain"))
                    .await
                {
                    warn!(target: "ui.wiki", "failed to save wiki page {slug}: {e}");
                    error.set(Some(format!("Failed to save: {e}")));
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
                    "WIKI PAGES"
                }

                // New page input
                div {
                    style: format!(
                        "display: flex; gap: {}; margin-bottom: {};",
                        spacing::XS,
                        spacing::SM
                    ),

                    input {
                        r#type: "text",
                        placeholder: "new-page-slug",
                        value: "{new_slug_input}",
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
                            new_slug_input.set(normalize_slug(&evt.value()));
                        },
                        onkeydown: {
                            let create = create_page.clone();
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter {
                                    create();
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
                            let create = create_page.clone();
                            move |_| create()
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
                } else if pages().is_empty() {
                    div {
                        style: format!("color: {}; font-size: {};", semantic::TEXT_MUTED, typography::SIZE_XS),
                        "No pages yet."
                    }
                } else {
                    for slug in pages() {
                        {
                            let is_active = selected_slug().as_deref() == Some(&slug);
                            let slug_click = slug.clone();
                            let load = load_page.clone();
                            rsx! {
                                button {
                                    key: "{slug}",
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
                                        let s = slug_click.clone();
                                        load(s);
                                    },
                                    "{slug}"
                                }
                            }
                        }
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

                match selected_slug() {
                    None => {
                        rsx! {
                            div {
                                style: format!(
                                    "flex: 1; display: flex; align-items: center; \
                                     justify-content: center; color: {}; font-size: {};",
                                    semantic::TEXT_MUTED,
                                    typography::SIZE_SM
                                ),
                                "Select a page or create a new one."
                            }
                        }
                    }
                    Some(ref slug) => {
                        if editing() {
                            // Editor mode
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; flex: 1; gap: 0;",

                                    // Title bar
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
                                            "Editing: {slug}"
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
                                                let mut save = save_page.clone();
                                                move |_| save()
                                            },
                                            if saving() { "Saving..." } else { "Save" }
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
                            // Reader mode
                            rsx! {
                                div {
                                    style: "display: flex; flex-direction: column; flex: 1;",

                                    // Title + Edit button
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
                                            "{slug}"
                                        }
                                        button {
                                            style: format!(
                                                "padding: {} {}; background: {}; color: {}; \
                                                 border: none; border-radius: {}; font-size: {}; \
                                                 cursor: pointer; margin-left: auto;",
                                                spacing::XS,
                                                spacing::SM,
                                                semantic::BG_ELEVATED,
                                                semantic::TEXT_SECONDARY,
                                                radius::SM,
                                                typography::SIZE_XS
                                            ),
                                            onclick: move |_| {
                                                editor_text.set(page_content().clone());
                                                editing.set(true);
                                            },
                                            "Edit"
                                        }
                                    }

                                    // Content
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
                                                 line-height: {}; margin: 0;",
                                                typography::FONT_BODY,
                                                typography::SIZE_BASE,
                                                semantic::TEXT_PRIMARY,
                                                typography::LEADING_RELAXED
                                            ),
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
