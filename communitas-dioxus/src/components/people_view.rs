//! People/contacts management view.
//!
//! Shows contacts with trust levels, agent card import, and search.

use communitas_x0x_client::{Contact, TrustLevel, X0xClient};
use dioxus::prelude::*;
use tracing::{info, warn};

use crate::tokens::{colors, radius, spacing, typography};

/// How often to refresh the contacts list.
const REFRESH_INTERVAL_SECS: u64 = 10;

/// Truncate an agent ID for display.
fn short_id(id: &str) -> String {
    if id.len() <= 16 {
        id.to_owned()
    } else {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    }
}

/// Copy text to clipboard.
fn copy_to_clipboard(value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("navigator.clipboard.writeText(\"{escaped}\").catch(()=>{{}});",);
    spawn(async move {
        let _ = dioxus::document::eval(&script);
    });
}

/// Trust level color.
fn trust_color(level: TrustLevel) -> &'static str {
    match level {
        TrustLevel::Blocked => colors::DANGER,
        TrustLevel::Unknown => colors::TEXT_MUTED,
        TrustLevel::Known => colors::WARNING,
        TrustLevel::Trusted => colors::SUCCESS,
    }
}

/// Trust level display label.
fn trust_label(level: TrustLevel) -> &'static str {
    match level {
        TrustLevel::Blocked => "Blocked",
        TrustLevel::Unknown => "Unknown",
        TrustLevel::Known => "Known",
        TrustLevel::Trusted => "Trusted",
    }
}

/// People view component.
#[component]
pub fn PeopleView() -> Element {
    let mut contacts = use_signal(Vec::<Contact>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut import_input = use_signal(String::new);
    let mut import_busy = use_signal(|| false);
    let mut import_error = use_signal(|| None::<String>);
    let mut refresh_key = use_signal(|| 0u64);

    // Fetch contacts
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();

        loop {
            let _key = *refresh_key.read(); // reactive dependency
            match client.list_contacts().await {
                Ok(list) => {
                    contacts.set(list);
                    error.set(None);
                }
                Err(e) => {
                    warn!(target: "ui.people", "failed to list contacts: {e}");
                    error.set(Some(format!("Failed to load contacts: {e}")));
                }
            }
            loading.set(false);

            tokio::time::sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
        }
    });

    let current_contacts = contacts.read().clone();
    let is_loading = *loading.read();
    let current_error = error.read().clone();

    let page_style = format!(
        "padding: {}; display: flex; flex-direction: column; gap: {}; \
         overflow-y: auto; height: 100%;",
        spacing::LG,
        spacing::LG,
    );

    let heading_style = format!(
        "font-size: {}; font-weight: 700; color: {}; margin: 0;",
        typography::TEXT_XL,
        colors::TEXT_PRIMARY,
    );

    let card_style = format!(
        "background-color: {}; border: 1px solid {}; border-radius: {}; padding: {};",
        colors::SURFACE_ELEVATED,
        colors::BORDER_DEFAULT,
        radius::LG,
        spacing::MD,
    );

    let input_style = format!(
        "flex: 1; background-color: {}; border: 1px solid {}; border-radius: {}; \
         padding: {} {}; color: {}; font-family: {}; font-size: {}; outline: none;",
        colors::SURFACE_BG,
        colors::BORDER_DEFAULT,
        radius::MD,
        spacing::SM,
        spacing::SM,
        colors::TEXT_PRIMARY,
        typography::FONT_MONO,
        typography::TEXT_SM,
    );

    let btn_style = format!(
        "background-color: {}; color: {}; border: none; border-radius: {}; \
         padding: {} {}; font-size: {}; font-weight: 500; cursor: pointer;",
        colors::PRIMARY,
        colors::TEXT_INVERSE,
        radius::MD,
        spacing::SM,
        spacing::MD,
        typography::TEXT_SM,
    );

    let trust_btn_style = |active: bool, level: TrustLevel| -> String {
        let c = trust_color(level);
        if active {
            format!(
                "background-color: {c}; color: {}; border: 1px solid {c}; \
                 border-radius: {}; padding: 2px {}; font-size: {}; cursor: pointer; font-weight: 500;",
                colors::TEXT_INVERSE,
                radius::SM,
                spacing::SM,
                typography::TEXT_XS,
            )
        } else {
            format!(
                "background-color: transparent; color: {c}; border: 1px solid {c}; \
                 border-radius: {}; padding: 2px {}; font-size: {}; cursor: pointer; opacity: 0.6;",
                radius::SM,
                spacing::SM,
                typography::TEXT_XS,
            )
        }
    };

    rsx! {
        div {
            style: "{page_style}",

            h1 { style: "{heading_style}", "People" }

            // Import agent card
            div {
                style: "{card_style}",

                div {
                    style: format!(
                        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM,
                        colors::TEXT_PRIMARY,
                        spacing::SM,
                    ),
                    "Import Agent Card"
                }

                div {
                    style: "display: flex; gap: 8px;",

                    input {
                        style: "{input_style}",
                        r#type: "text",
                        placeholder: "Paste agent card link or JSON...",
                        value: "{import_input}",
                        oninput: move |evt: FormEvent| import_input.set(evt.value()),
                    }

                    button {
                        style: "{btn_style}",
                        disabled: import_busy() || import_input().trim().is_empty(),
                        onclick: move |_| {
                            let raw = import_input().trim().to_string();
                            if raw.is_empty() {
                                return;
                            }
                            import_busy.set(true);
                            import_error.set(None);

                            spawn(async move {
                                let client = X0xClient::new();
                                match client.import_agent_card(&raw, None).await {
                                    Ok(resp) => {
                                        info!(target: "ui.people", "imported agent card: {}", resp.agent_id);
                                        import_input.set(String::new());
                                        refresh_key.set(refresh_key() + 1);
                                    }
                                    Err(e) => {
                                        import_error.set(Some(format!("{e}")));
                                    }
                                }
                                import_busy.set(false);
                            });
                        },
                        if import_busy() { "Importing..." } else { "Import" }
                    }
                }

                if let Some(ref err) = *import_error.read() {
                    div {
                        style: format!(
                            "margin-top: {}; font-size: {}; color: {};",
                            spacing::SM,
                            typography::TEXT_XS,
                            colors::DANGER,
                        ),
                        "{err}"
                    }
                }
            }

            // Error banner
            if let Some(ref err) = current_error {
                div {
                    style: format!(
                        "background-color: rgba(255, 68, 102, 0.1); border: 1px solid {}; \
                         border-radius: {}; padding: {}; color: {}; font-size: {};",
                        colors::DANGER, radius::MD, spacing::MD, colors::DANGER, typography::TEXT_SM,
                    ),
                    "{err}"
                }
            }

            // Loading
            if is_loading {
                div {
                    style: format!("color: {}; font-size: {};", colors::TEXT_MUTED, typography::TEXT_SM),
                    "Loading contacts..."
                }
            }

            // Contacts list
            if !is_loading && current_contacts.is_empty() {
                div {
                    style: format!(
                        "{card_style} text-align: center; color: {};",
                        colors::TEXT_MUTED,
                    ),
                    "No contacts yet. Import an agent card above to add your first contact."
                }
            }

            for contact in &current_contacts {
                {
                    let agent_id = contact.agent_id.clone();
                    let label = contact.label.clone().unwrap_or_default();
                    let trust = contact.trust_level;
                    let last_seen_text = contact.last_seen.map(|ts| {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let diff = now.saturating_sub(ts);
                        if diff < 60 { "just now".to_string() }
                        else if diff < 3600 { format!("{}m ago", diff / 60) }
                        else if diff < 86400 { format!("{}h ago", diff / 3600) }
                        else { format!("{}d ago", diff / 86400) }
                    }).unwrap_or_else(|| "never".to_string());

                    rsx! {
                        div {
                            key: "{agent_id}",
                            style: format!(
                                "{card_style} display: flex; align-items: center; gap: {};",
                                spacing::MD,
                            ),

                            // Trust dot
                            span {
                                style: format!(
                                    "width: 10px; height: 10px; border-radius: 50%; \
                                     background-color: {}; flex-shrink: 0;",
                                    trust_color(trust),
                                ),
                                title: "{trust_label(trust)}",
                            }

                            // Info
                            div {
                                style: "flex: 1; min-width: 0;",

                                if !label.is_empty() {
                                    div {
                                        style: format!(
                                            "font-size: {}; font-weight: 600; color: {}; \
                                             overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                            typography::TEXT_SM,
                                            colors::TEXT_PRIMARY,
                                        ),
                                        "{label}"
                                    }
                                }

                                div {
                                    style: format!(
                                        "font-family: {}; font-size: {}; color: {}; cursor: pointer; \
                                         overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                        typography::FONT_MONO,
                                        typography::TEXT_XS,
                                        colors::PRIMARY,
                                    ),
                                    title: "Click to copy",
                                    onclick: {
                                        let id = agent_id.clone();
                                        move |_| copy_to_clipboard(&id)
                                    },
                                    "{short_id(&agent_id)}"
                                }
                            }

                            // Last seen
                            div {
                                style: format!(
                                    "font-size: {}; color: {}; white-space: nowrap;",
                                    typography::TEXT_XS,
                                    colors::TEXT_MUTED,
                                ),
                                "{last_seen_text}"
                            }

                            // Trust level buttons
                            div {
                                style: "display: flex; gap: 4px; flex-shrink: 0;",

                                for level in [TrustLevel::Blocked, TrustLevel::Unknown, TrustLevel::Known, TrustLevel::Trusted] {
                                    {
                                        let is_active = trust == level;
                                        let aid = agent_id.clone();
                                        rsx! {
                                            button {
                                                key: "{trust_label(level)}",
                                                style: trust_btn_style(is_active, level),
                                                disabled: is_active,
                                                onclick: move |_| {
                                                    let aid = aid.clone();
                                                    spawn(async move {
                                                        let client = X0xClient::new();
                                                        if let Err(e) = client.set_trust(&aid, level).await {
                                                            warn!(target: "ui.people", "failed to set trust: {e}");
                                                        }
                                                        refresh_key.set(refresh_key() + 1);
                                                    });
                                                },
                                                "{trust_label(level)}"
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
