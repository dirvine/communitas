//! Dashboard view -- landing page showing system stats and discovered agents.

use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;

use crate::tokens::{colors, radius, spacing, typography};

/// How often to refresh dashboard data (seconds).
const REFRESH_INTERVAL_SECS: u64 = 8;

/// Truncate an ID for display.
fn short_id(id: &str) -> String {
    if id.len() <= 16 {
        id.to_owned()
    } else {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    }
}

/// Format uptime seconds.
fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        format!("{h}h {m}m")
    }
}

#[derive(Clone, Default)]
struct DashboardData {
    status: String,
    version: String,
    peers: u32,
    uptime_secs: u64,
    agent_id: String,
    machine_id: String,
    discovered: Vec<DiscoveredRow>,
    groups_count: u32,
    contacts_count: u32,
}

#[derive(Clone, PartialEq)]
struct DiscoveredRow {
    agent_id: String,
    addresses: Vec<String>,
    last_seen: Option<u64>,
}

/// Copy text to clipboard via JS.
fn copy_text(value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("navigator.clipboard.writeText(\"{escaped}\").catch(()=>{{}});",);
    spawn(async move {
        let _ = dioxus::document::eval(&script);
    });
}

/// Dashboard landing page.
#[component]
pub fn Dashboard() -> Element {
    let mut data = use_signal(DashboardData::default);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();

        loop {
            let mut d = DashboardData::default();
            let mut had_error = false;

            match client.health().await {
                Ok(health) => {
                    d.status = health.status;
                    d.version = health.version;
                    d.peers = health.peers;
                    d.uptime_secs = health.uptime_secs;
                }
                Err(e) => {
                    error.set(Some(format!("Daemon unreachable: {e}")));
                    had_error = true;
                }
            }

            if !had_error {
                if let Ok(agent) = client.agent().await {
                    d.agent_id = agent.agent_id;
                    d.machine_id = agent.machine_id;
                }

                if let Ok(agents) = client.discovered_agents().await {
                    d.discovered = agents
                        .into_iter()
                        .map(|a| DiscoveredRow {
                            agent_id: a.agent_id,
                            addresses: a.addresses,
                            last_seen: a.last_seen,
                        })
                        .collect();
                }

                if let Ok(groups) = client.list_groups().await {
                    d.groups_count = groups.len() as u32;
                }

                if let Ok(contacts) = client.list_contacts().await {
                    d.contacts_count = contacts.len() as u32;
                }

                error.set(None);
            }

            data.set(d);
            loading.set(false);

            tokio::time::sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
        }
    });

    let d = data.read().clone();
    let is_loading = *loading.read();
    let current_error = error.read().clone();

    // Styles
    let page_style = format!(
        "padding: {}; display: flex; flex-direction: column; gap: {}; \
         overflow-y: auto; height: 100%;",
        spacing::LG,
        spacing::LG,
    );

    let heading_style = format!(
        "font-size: {}; font-weight: 700; color: {}; margin: 0; \
         letter-spacing: -0.01em;",
        typography::TEXT_XL,
        colors::TEXT_PRIMARY,
    );

    let card_style = format!(
        "background-color: {}; border: 1px solid {}; border-radius: {}; \
         padding: {};",
        colors::SURFACE_ELEVATED,
        colors::BORDER_DEFAULT,
        radius::LG,
        spacing::MD,
    );

    let stat_value_style = format!(
        "font-size: {}; font-weight: 700; color: {};",
        typography::TEXT_2XL,
        colors::TEXT_PRIMARY,
    );

    let stat_label_style = format!(
        "font-size: {}; color: {}; text-transform: uppercase; letter-spacing: 0.06em; \
         margin-top: {};",
        typography::TEXT_XS,
        colors::TEXT_MUTED,
        spacing::XS,
    );

    let mono_style = format!(
        "font-family: {}; font-size: {}; color: {}; \
         word-break: break-all; cursor: pointer;",
        typography::FONT_MONO,
        typography::TEXT_XS,
        colors::PRIMARY,
    );

    let table_header_style = format!(
        "font-size: {}; color: {}; text-transform: uppercase; letter-spacing: 0.06em; \
         padding: {} 0; border-bottom: 1px solid {};",
        typography::TEXT_XS,
        colors::TEXT_MUTED,
        spacing::XS,
        colors::BORDER_DEFAULT,
    );

    let table_row_style = format!(
        "padding: {} 0; border-bottom: 1px solid {};",
        spacing::SM,
        colors::BORDER_DEFAULT,
    );

    if is_loading {
        return rsx! {
            div {
                style: "{page_style}",
                div {
                    style: format!(
                        "display: flex; align-items: center; justify-content: center; \
                         height: 200px; color: {};",
                        colors::TEXT_MUTED,
                    ),
                    "Loading dashboard..."
                }
            }
        };
    }

    rsx! {
        div {
            style: "{page_style}",

            h1 { style: "{heading_style}", "Dashboard" }

            // Error banner
            if let Some(ref err) = current_error {
                div {
                    style: format!(
                        "background-color: rgba(255, 68, 102, 0.1); border: 1px solid {}; \
                         border-radius: {}; padding: {}; color: {}; font-size: {};",
                        colors::DANGER,
                        radius::MD,
                        spacing::MD,
                        colors::DANGER,
                        typography::TEXT_SM,
                    ),
                    "{err}"
                }
            }

            // Stats grid (4 columns)
            div {
                style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;",

                // Status
                div {
                    style: "{card_style}",
                    div {
                        style: "{stat_value_style}",
                        {if d.status == "healthy" || d.status == "running" { "Online" } else if d.status.is_empty() { "Offline" } else { &d.status }}
                    }
                    div { style: "{stat_label_style}", "Status" }
                }

                // Version
                div {
                    style: "{card_style}",
                    div {
                        style: "{stat_value_style}",
                        {if d.version.is_empty() { "-" } else { &d.version }}
                    }
                    div { style: "{stat_label_style}", "Version" }
                }

                // Peers
                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{d.peers}" }
                    div { style: "{stat_label_style}", "Peers" }
                }

                // Uptime
                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{format_uptime(d.uptime_secs)}" }
                    div { style: "{stat_label_style}", "Uptime" }
                }
            }

            // Identity section
            div {
                style: "{card_style}",

                div {
                    style: format!(
                        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM,
                        colors::TEXT_PRIMARY,
                        spacing::SM,
                    ),
                    "Identity"
                }

                div {
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",

                    div {
                        div {
                            style: format!("font-size: {}; color: {}; margin-bottom: {};",
                                typography::TEXT_XS, colors::TEXT_MUTED, spacing::XS),
                            "Agent ID"
                        }
                        div {
                            style: "{mono_style}",
                            title: "Click to copy",
                            onclick: {
                                let id = d.agent_id.clone();
                                move |_| copy_text(&id)
                            },
                            {if d.agent_id.is_empty() { "-".to_string() } else { d.agent_id.clone() }}
                        }
                    }

                    div {
                        div {
                            style: format!("font-size: {}; color: {}; margin-bottom: {};",
                                typography::TEXT_XS, colors::TEXT_MUTED, spacing::XS),
                            "Machine ID"
                        }
                        div {
                            style: "{mono_style}",
                            title: "Click to copy",
                            onclick: {
                                let id = d.machine_id.clone();
                                move |_| copy_text(&id)
                            },
                            {if d.machine_id.is_empty() { "-".to_string() } else { d.machine_id.clone() }}
                        }
                    }
                }
            }

            // Quick stats row
            div {
                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",

                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{d.groups_count}" }
                    div { style: "{stat_label_style}", "Spaces" }
                }
                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{d.contacts_count}" }
                    div { style: "{stat_label_style}", "Contacts" }
                }
            }

            // Discovered agents table
            div {
                style: "{card_style}",

                div {
                    style: format!(
                        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM,
                        colors::TEXT_PRIMARY,
                        spacing::SM,
                    ),
                    "Discovered Agents"
                }

                if d.discovered.is_empty() {
                    div {
                        style: format!(
                            "font-size: {}; color: {}; padding: {};",
                            typography::TEXT_SM,
                            colors::TEXT_MUTED,
                            spacing::MD,
                        ),
                        "No agents discovered yet. Agents on the network will appear here."
                    }
                } else {
                    // Table header
                    div {
                        style: format!("display: grid; grid-template-columns: 2fr 2fr 1fr; {table_header_style}"),
                        span { "Agent ID" }
                        span { "Addresses" }
                        span { "Last Seen" }
                    }

                    // Table rows
                    for row in &d.discovered {
                        {
                            let agent_id = row.agent_id.clone();
                            let addresses = row.addresses.join(", ");
                            let last_seen = row.last_seen.map(|ts| {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|dur| dur.as_secs())
                                    .unwrap_or(0);
                                let diff = now.saturating_sub(ts);
                                if diff < 60 { "just now".to_string() }
                                else if diff < 3600 { format!("{}m ago", diff / 60) }
                                else { format!("{}h ago", diff / 3600) }
                            }).unwrap_or_else(|| "-".to_string());

                            rsx! {
                                div {
                                    key: "{agent_id}",
                                    style: format!("display: grid; grid-template-columns: 2fr 2fr 1fr; align-items: center; {table_row_style}"),

                                    span {
                                        style: format!(
                                            "font-family: {}; font-size: {}; color: {};",
                                            typography::FONT_MONO,
                                            typography::TEXT_XS,
                                            colors::PRIMARY,
                                        ),
                                        "{short_id(&agent_id)}"
                                    }
                                    span {
                                        style: format!(
                                            "font-family: {}; font-size: {}; color: {}; \
                                             overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                            typography::FONT_MONO,
                                            typography::TEXT_XS,
                                            colors::TEXT_SECONDARY,
                                        ),
                                        "{addresses}"
                                    }
                                    span {
                                        style: format!(
                                            "font-size: {}; color: {};",
                                            typography::TEXT_XS,
                                            colors::TEXT_MUTED,
                                        ),
                                        "{last_seen}"
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
