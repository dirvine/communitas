// SPDX-License-Identifier: MIT OR Apache-2.0

//! People/contacts management view.
//!
//! Shows contacts with trust levels, agent card import, and search.
//! Also displays a "Discovered Agents" section populated from `GET /agents/discovered`.

use communitas_x0x_client::{Contact, DiscoveredAgent, TrustLevel, X0xClient};
use dioxus::prelude::*;
use tracing::{info, warn};

use crate::tokens::{colors, radius, spacing, typography};

/// How often to refresh the contacts list.
const REFRESH_INTERVAL_SECS: u64 = 10;

/// How often to refresh the discovered agents list.
const AGENT_REFRESH_SECS: u64 = 15;

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

/// Relative time display from a unix-epoch seconds timestamp.
fn relative_time(last_seen_secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let diff = now.saturating_sub(last_seen_secs);
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

fn same_contacts(left: &[Contact], right: &[Contact]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.agent_id == right.agent_id
                && left.trust_level == right.trust_level
                && left.label == right.label
                && left.added_at == right.added_at
                && left.last_seen == right.last_seen
                && left.identity_type == right.identity_type
        })
}

fn same_discovered_agents(left: &[DiscoveredAgent], right: &[DiscoveredAgent]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.agent_id == right.agent_id
                && left.machine_id == right.machine_id
                && left.user_id == right.user_id
                && left.addresses == right.addresses
                && left.announced_at == right.announced_at
                && left.last_seen == right.last_seen
        })
}

async fn refresh_contacts(
    client: &X0xClient,
    mut contacts: Signal<Vec<Contact>>,
    mut error: Signal<Option<String>>,
    mut loading: Signal<bool>,
) {
    match client.list_contacts().await {
        Ok(list) => {
            contacts.set(list);
            if error.peek().is_some() {
                error.set(None);
            }
        }
        Err(e) => {
            warn!(target: "ui.people", "failed to list contacts: {e}");
            error.set(Some(format!("Failed to load contacts: {e}")));
        }
    }
    if *loading.peek() {
        loading.set(false);
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
    let mut add_agent_id = use_signal(String::new);
    let mut add_label = use_signal(String::new);
    let mut add_busy = use_signal(|| false);
    let mut add_error = use_signal(|| None::<String>);
    let mut lookup_agent_id = use_signal(String::new);
    let mut lookup_busy = use_signal(|| false);
    let mut lookup_result = use_signal(|| None::<String>);
    let mut lookup_error = use_signal(|| None::<String>);
    let mut refresh_key = use_signal(|| 0u64);

    // Discovered agents state
    let mut discovered_agents = use_signal(Vec::<DiscoveredAgent>::new);
    let mut agents_loading = use_signal(|| true);

    // Fetch contacts
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();

        loop {
            let _key = *refresh_key.peek();
            match client.list_contacts().await {
                Ok(list) => {
                    if !same_contacts(contacts.peek().as_slice(), list.as_slice()) {
                        contacts.set(list);
                    }
                    if error.peek().is_some() {
                        error.set(None);
                    }
                }
                Err(e) => {
                    warn!(target: "ui.people", "failed to list contacts: {e}");
                    let next_error = Some(format!("Failed to load contacts: {e}"));
                    if error.peek().as_ref() != next_error.as_ref() {
                        error.set(next_error);
                    }
                }
            }
            if *loading.peek() {
                loading.set(false);
            }

            crate::poll_sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
        }
    });

    // Fetch discovered agents
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();
        loop {
            match client.discovered_agents().await {
                Ok(agents) => {
                    if !same_discovered_agents(
                        discovered_agents.peek().as_slice(),
                        agents.as_slice(),
                    ) {
                        discovered_agents.set(agents);
                    }
                }
                Err(e) => {
                    warn!(target: "ui.people", "failed to list discovered agents: {e}");
                }
            }
            if *agents_loading.peek() {
                agents_loading.set(false);
            }
            crate::poll_sleep(tokio::time::Duration::from_secs(AGENT_REFRESH_SECS)).await;
        }
    });

    let current_contacts = contacts.read().clone();
    let current_agents = discovered_agents.read().clone();
    let is_loading = *loading.read();
    let is_agents_loading = *agents_loading.read();
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
                                        refresh_contacts(&client, contacts, error, loading).await;
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

            // Add contact by agent ID
            div {
                style: "{card_style}",
                div {
                    style: format!(
                        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM,
                    ),
                    "Add Contact by Agent ID"
                }
                div {
                    style: format!("display: flex; gap: {};", spacing::SM),
                    input {
                        style: "{input_style}",
                        r#type: "text",
                        placeholder: "Agent ID (64 hex chars)",
                        value: "{add_agent_id}",
                        oninput: move |evt: FormEvent| add_agent_id.set(evt.value()),
                    }
                    input {
                        style: "{input_style}",
                        r#type: "text",
                        placeholder: "Label (optional)",
                        value: "{add_label}",
                        oninput: move |evt: FormEvent| add_label.set(evt.value()),
                    }
                    button {
                        style: "{btn_style}",
                        disabled: add_busy() || add_agent_id().trim().is_empty(),
                        onclick: move |_| {
                            let agent_id = add_agent_id().trim().to_string();
                            if agent_id.is_empty() { return; }
                            let label = add_label().trim().to_string();
                            let label_opt = if label.is_empty() { None } else { Some(label) };
                            add_busy.set(true);
                            add_error.set(None);

                            spawn(async move {
                                let client = X0xClient::new();
                                match client.add_contact(&agent_id, TrustLevel::Known, label_opt.as_deref()).await {
                                    Ok(_) => {
                                        add_agent_id.set(String::new());
                                        add_label.set(String::new());
                                        refresh_contacts(&client, contacts, error, loading).await;
                                        refresh_key.set(refresh_key() + 1);
                                    }
                                    Err(e) => add_error.set(Some(format!("{e}"))),
                                }
                                add_busy.set(false);
                            });
                        },
                        if add_busy() { "Adding..." } else { "Add" }
                    }
                }
                if let Some(ref err) = *add_error.read() {
                    div {
                        style: format!(
                            "margin-top: {}; font-size: {}; color: {};",
                            spacing::SM, typography::TEXT_XS, colors::DANGER,
                        ),
                        "{err}"
                    }
                }
            }

            // Discovery tools
            div {
                style: "{card_style}",
                div {
                    style: format!(
                        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM,
                    ),
                    "Discovery Tools"
                }
                div {
                    style: format!("display: flex; gap: {};", spacing::SM),
                    input {
                        style: "{input_style}",
                        r#type: "text",
                        placeholder: "Find agent ID / check reachability",
                        value: "{lookup_agent_id}",
                        oninput: move |evt: FormEvent| lookup_agent_id.set(evt.value()),
                    }
                    button {
                        style: "{btn_style}",
                        disabled: lookup_busy() || lookup_agent_id().trim().is_empty(),
                        onclick: move |_| {
                            let agent_id = lookup_agent_id().trim().to_string();
                            if agent_id.is_empty() { return; }
                            lookup_busy.set(true);
                            lookup_error.set(None);
                            lookup_result.set(None);
                            spawn(async move {
                                let client = X0xClient::new();
                                let mut lines = Vec::new();

                                match client.find_agent(&agent_id).await {
                                    Ok(found) => {
                                        lines.push(if found.found {
                                            "Found on network".to_string()
                                        } else {
                                            "Not found yet".to_string()
                                        });
                                        if !found.addresses.is_empty() {
                                            lines.push(format!("Search addrs: {}", found.addresses.join(", ")));
                                        }
                                    }
                                    Err(err) => {
                                        lookup_error.set(Some(format!("Find failed: {err}")));
                                        lookup_busy.set(false);
                                        return;
                                    }
                                }

                                if let Ok(status) = client.presence_status(&agent_id).await {
                                    lines.push(if status.online {
                                        "Presence: online".to_string()
                                    } else {
                                        "Presence: offline / unknown".to_string()
                                    });
                                }

                                if let Ok(reachability) = client.agent_reachability(&agent_id).await {
                                    let path = if reachability.likely_direct {
                                        "likely direct"
                                    } else if reachability.needs_coordination {
                                        "needs coordination"
                                    } else {
                                        "unknown path"
                                    };
                                    lines.push(format!("Reachability: {path}"));
                                    if !reachability.addresses.is_empty() {
                                        lines.push(format!("Known addrs: {}", reachability.addresses.join(", ")));
                                    }
                                }

                                lookup_result.set(Some(lines.join(" • ")));
                                lookup_busy.set(false);
                            });
                        },
                        if lookup_busy() { "Checking..." } else { "Inspect" }
                    }
                }
                if let Some(ref err) = *lookup_error.read() {
                    div {
                        style: format!(
                            "margin-top: {}; font-size: {}; color: {};",
                            spacing::SM, typography::TEXT_XS, colors::DANGER,
                        ),
                        "{err}"
                    }
                }
                if let Some(ref result) = *lookup_result.read() {
                    div {
                        style: format!(
                            "margin-top: {}; font-size: {}; color: {};",
                            spacing::SM, typography::TEXT_XS, colors::TEXT_SECONDARY,
                        ),
                        "{result}"
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

            // Contacts section header
            div {
                style: format!(
                    "font-size: {}; font-weight: 700; color: {}; \
                     text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: -{};",
                    typography::TEXT_XS,
                    colors::TEXT_MUTED,
                    spacing::SM,
                ),
                "Contacts"
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
                                                        } else {
                                                            refresh_contacts(&client, contacts, error, loading).await;
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

            // ── Discovered Agents section ──────────────────────────────
            div {
                style: format!(
                    "font-size: {}; font-weight: 700; color: {}; \
                     text-transform: uppercase; letter-spacing: 0.06em; margin-top: {};",
                    typography::TEXT_XS,
                    colors::TEXT_MUTED,
                    spacing::SM,
                ),
                "\u{1F916} Discovered Agents"
            }

            if is_agents_loading {
                div {
                    style: format!("color: {}; font-size: {};", colors::TEXT_MUTED, typography::TEXT_SM),
                    "Scanning network for agents..."
                }
            } else if current_agents.is_empty() {
                div {
                    style: format!(
                        "{card_style} display: flex; align-items: center; gap: {}; color: {};",
                        spacing::MD,
                        colors::TEXT_MUTED,
                    ),
                    span {
                        style: format!("font-size: {};", typography::TEXT_XL),
                        "\u{1F4E1}"
                    }
                    span {
                        style: format!("font-size: {};", typography::TEXT_SM),
                        "No agents discovered on the network"
                    }
                }
            } else {
                for agent in &current_agents {
                    {
                        let agent_id = agent.agent_id.clone();
                        let display = agent.user_id.clone()
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| short_id(&agent_id));
                        let last_seen_text = agent.last_seen
                            .map(relative_time)
                            .unwrap_or_else(|| "never".to_string());

                        rsx! {
                            div {
                                key: "{agent_id}",
                                style: format!(
                                    "{card_style} display: flex; align-items: center; gap: {};",
                                    spacing::MD,
                                ),

                                // Robot icon
                                span {
                                    style: format!(
                                        "font-size: {}; flex-shrink: 0;",
                                        typography::TEXT_LG,
                                    ),
                                    "\u{1F916}"
                                }

                                // Agent info
                                div {
                                    style: "flex: 1; min-width: 0;",

                                    div {
                                        style: format!(
                                            "display: flex; align-items: center; gap: {}; \
                                             margin-bottom: {};",
                                            spacing::XS,
                                            "2px",
                                        ),
                                        span {
                                            style: format!(
                                                "font-size: {}; font-weight: 600; color: {}; \
                                                 overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                typography::TEXT_SM,
                                                colors::TEXT_PRIMARY,
                                            ),
                                            "{display}"
                                        }
                                        // Agent badge
                                        span {
                                            style: format!(
                                                "font-size: {}; font-weight: 500; \
                                                 color: {}; background: rgba(0,150,255,0.12); \
                                                 padding: 1px {}; border-radius: 20px; \
                                                 white-space: nowrap; flex-shrink: 0;",
                                                typography::TEXT_XS,
                                                colors::PRIMARY,
                                                spacing::XS,
                                            ),
                                            "Agent"
                                        }
                                    }

                                    div {
                                        style: format!(
                                            "font-family: {}; font-size: {}; color: {}; cursor: pointer; \
                                             overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                            typography::FONT_MONO,
                                            typography::TEXT_XS,
                                            colors::TEXT_MUTED,
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

                                button {
                                    style: format!(
                                        "background-color: {}; color: {}; border: none; border-radius: {}; \
                                         padding: 4px {}; font-size: {}; cursor: pointer; flex-shrink: 0;",
                                        colors::PRIMARY,
                                        colors::TEXT_INVERSE,
                                        radius::SM,
                                        spacing::SM,
                                        typography::TEXT_XS,
                                    ),
                                    onclick: {
                                        let aid = agent_id.clone();
                                        move |_| {
                                            let aid = aid.clone();
                                            spawn(async move {
                                                let client = X0xClient::new();
                                                if let Err(e) = client.add_contact(&aid, TrustLevel::Known, None).await {
                                                    warn!(target: "ui.people", "failed to add discovered agent as contact: {e}");
                                                } else {
                                                    refresh_contacts(&client, contacts, error, loading).await;
                                                }
                                                refresh_key.set(refresh_key() + 1);
                                            });
                                        }
                                    },
                                    "Add"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
