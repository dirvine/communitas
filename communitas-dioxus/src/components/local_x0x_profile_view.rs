//! Local x0x identity/details view for the desktop-first shell.
//!
//! This is the owned-machine profile surface for the active Dioxus lane.
//! It intentionally emphasizes local identity, share details, daemon status,
//! and network diagnostics instead of legacy login/four-word onboarding.

use std::collections::BTreeSet;

use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;
use tracing::warn;

use crate::tokens::{colors, radius, spacing, typography};
use crate::x0x_contract;

#[derive(Clone, Default)]
struct LocalX0xProfile {
    display_name: String,
    agent_id: String,
    machine_id: String,
    user_id: Option<String>,
    share_link: Option<String>,
    addresses: Vec<String>,
    status: String,
    version: String,
    api_address: Option<String>,
    uptime_secs: Option<u64>,
    warnings: Vec<String>,
    peers: Vec<String>,
    connected_peers: u32,
    discovered_agents: u32,
    direct_connections: u32,
    ws_sessions: u32,
    nat_type: Option<String>,
    can_receive_direct: bool,
    has_public_ip: bool,
    bootstrap_peers: Vec<String>,
}

fn short_id(id: &str) -> String {
    if id.len() <= 18 {
        id.to_string()
    } else {
        format!("{}...{}", &id[..10], &id[id.len() - 6..])
    }
}

fn format_uptime(secs: Option<u64>) -> String {
    let Some(secs) = secs else {
        return "-".to_string();
    };

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

fn copy_text(value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("navigator.clipboard.writeText(\"{escaped}\").catch(()=>{{}});",);
    spawn(async move {
        let _ = dioxus::document::eval(&script);
    });
}

async fn load_local_x0x_profile() -> LocalX0xProfile {
    let client = X0xClient::new();
    let mut profile = LocalX0xProfile::default();
    let mut addresses = BTreeSet::new();

    match client.agent().await {
        Ok(agent) => {
            profile.agent_id = agent.agent_id.clone();
            profile.machine_id = agent.machine_id.clone();
            profile.user_id = agent.user_id.filter(|value| !value.trim().is_empty());
        }
        Err(err) => warn!(target: "ui.local_x0x_profile", "failed to load /agent: {err}"),
    }

    match client.agent_card(None, Some(false)).await {
        Ok(card_resp) => {
            let card = card_resp.card;
            if !card.display_name.trim().is_empty() {
                profile.display_name = card.display_name.trim().to_string();
            }
            if profile.agent_id.is_empty() {
                profile.agent_id = card.agent_id;
            }
            if profile.machine_id.is_empty() {
                profile.machine_id = card.machine_id;
            }
            if profile.user_id.is_none() {
                profile.user_id = card.user_id.filter(|value| !value.trim().is_empty());
            }
            profile.share_link = Some(card_resp.link);
            for addr in card.addresses {
                if !addr.trim().is_empty() {
                    addresses.insert(addr);
                }
            }
        }
        Err(err) => warn!(target: "ui.local_x0x_profile", "failed to load /agent/card: {err}"),
    }

    match client.status().await {
        Ok(status) => {
            profile.status = status.status;
            profile.version = status.version;
            profile.api_address = Some(status.api_address);
            profile.uptime_secs = Some(status.uptime_secs);
            profile.connected_peers = status.peers;
            profile.warnings = status.warnings;
            for addr in status.external_addrs {
                if !addr.trim().is_empty() {
                    addresses.insert(addr);
                }
            }
        }
        Err(err) => {
            warn!(target: "ui.local_x0x_profile", "failed to load /status: {err}");
            match client.health().await {
                Ok(health) => {
                    profile.status = health.status;
                    profile.version = health.version;
                    profile.uptime_secs = Some(health.uptime_secs);
                    profile.connected_peers = health.peers;
                }
                Err(health_err) => {
                    warn!(target: "ui.local_x0x_profile", "failed to load /health after /status failure: {health_err}")
                }
            }
        }
    }

    match client.network_status().await {
        Ok(network) => {
            profile.nat_type = network.nat_type;
            profile.can_receive_direct = network.can_receive_direct;
            profile.has_public_ip = network.has_public_ip;
            if profile.connected_peers == 0 {
                profile.connected_peers = network.connected_peers;
            }
            for addr in network.external_addrs {
                if !addr.trim().is_empty() {
                    addresses.insert(addr);
                }
            }
            if let Some(local_addr) = network.local_addr.filter(|value| !value.trim().is_empty()) {
                addresses.insert(local_addr);
            }
        }
        Err(err) => warn!(target: "ui.local_x0x_profile", "failed to load /network/status: {err}"),
    }

    match client.bootstrap_cache().await {
        Ok(bootstrap) => {
            if profile.connected_peers == 0 {
                profile.connected_peers = bootstrap.connection_count;
            }
            profile.bootstrap_peers = bootstrap.connected_peers;
        }
        Err(err) => {
            warn!(target: "ui.local_x0x_profile", "failed to load /network/bootstrap-cache: {err}")
        }
    }

    match client.peers().await {
        Ok(peers) => {
            profile.peers = peers.into_iter().map(|peer| peer.id).collect();
            if profile.connected_peers == 0 {
                profile.connected_peers = profile.peers.len() as u32;
            }
        }
        Err(err) => warn!(target: "ui.local_x0x_profile", "failed to load /peers: {err}"),
    }

    match client.discovered_agents().await {
        Ok(agents) => {
            profile.discovered_agents = agents.len() as u32;
        }
        Err(err) => {
            warn!(target: "ui.local_x0x_profile", "failed to load /agents/discovered: {err}")
        }
    }

    match client.direct_connections().await {
        Ok(connections) => {
            profile.direct_connections = connections.len() as u32;
        }
        Err(err) => {
            warn!(target: "ui.local_x0x_profile", "failed to load /direct/connections: {err}")
        }
    }

    match client.ws_sessions().await {
        Ok(sessions) => {
            profile.ws_sessions = sessions.sessions.len() as u32;
        }
        Err(err) => warn!(target: "ui.local_x0x_profile", "failed to load /ws/sessions: {err}"),
    }

    profile.addresses = addresses.into_iter().collect();

    if profile.display_name.is_empty() {
        profile.display_name = if let Some(user_id) = profile.user_id.clone() {
            user_id
        } else if !profile.agent_id.is_empty() {
            format!(
                "agent:{}",
                x0x_contract::fallback_sender_name(&profile.agent_id)
            )
        } else {
            "Local x0x".to_string()
        };
    }

    profile
}

#[component]
pub fn LocalX0xProfileView() -> Element {
    let mut profile = use_signal(LocalX0xProfile::default);
    let mut loading = use_signal(|| true);

    use_future(move || async move {
        profile.set(load_local_x0x_profile().await);
        loading.set(false);
    });

    let data = profile.read().clone();

    let heading_style = format!(
        "font-size: {}; font-weight: 700; color: {}; letter-spacing: -0.01em;",
        typography::TEXT_XL,
        colors::TEXT_PRIMARY,
    );
    let subheading_style = format!(
        "font-size: {}; color: {}; max-width: 840px; line-height: 1.5;",
        typography::TEXT_SM,
        colors::TEXT_MUTED,
    );
    let card_style = format!(
        "background-color: {}; border: 1px solid {}; border-radius: {}; padding: {};",
        colors::SURFACE_ELEVATED,
        colors::BORDER_DEFAULT,
        radius::LG,
        spacing::MD,
    );
    let section_title_style = format!(
        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
        typography::TEXT_SM,
        colors::TEXT_PRIMARY,
        spacing::SM,
    );
    let label_style = format!(
        "font-size: {}; color: {}; text-transform: uppercase; letter-spacing: 0.06em;",
        typography::TEXT_XS,
        colors::TEXT_MUTED,
    );
    let value_style = format!(
        "font-size: {}; color: {}; margin-top: {};",
        typography::TEXT_SM,
        colors::TEXT_PRIMARY,
        spacing::XS,
    );
    let mono_style = format!(
        "font-family: {}; font-size: {}; color: {}; word-break: break-all;",
        typography::FONT_MONO,
        typography::TEXT_XS,
        colors::PRIMARY,
    );
    let stat_value_style = format!(
        "font-size: {}; font-weight: 700; color: {};",
        typography::TEXT_2XL,
        colors::TEXT_PRIMARY,
    );
    let stat_label_style = format!(
        "font-size: {}; color: {}; text-transform: uppercase; letter-spacing: 0.06em; margin-top: {};",
        typography::TEXT_XS,
        colors::TEXT_MUTED,
        spacing::XS,
    );
    let secondary_btn_style = format!(
        "background-color: transparent; color: {}; border: 1px solid {}; border-radius: {}; padding: {} {}; font-size: {}; font-weight: 500; cursor: pointer;",
        colors::TEXT_PRIMARY,
        colors::BORDER_DEFAULT,
        radius::MD,
        spacing::SM,
        spacing::MD,
        typography::TEXT_SM,
    );

    if loading() {
        return rsx! {
            div {
                style: "padding: 24px; color: #98a2b3;",
                "Loading local x0x identity…"
            }
        };
    }

    rsx! {
        div {
            style: "height: 100%; overflow: hidden;",
            div {
                style: "max-width: 1040px; margin: 0 auto; height: 100%;",
                div {
                    style: "height: 100%; overflow-y: auto;",
                    div {
                        style: "max-width: 960px; margin: 0 auto;",
                        div {
                            style: "display: flex; flex-direction: column; gap: 24px;",

                            div {
                                style: "display: flex; flex-direction: column; gap: 8px;",
                                h1 { style: "{heading_style}", "Local identity" }
                                p {
                                    style: "{subheading_style}",
                                    "This desktop lane is local-first and owned-machine-first: no login wall, no recovery wizard, and no four-word startup detour before you can use x0x."
                                }
                            }

                            div {
                                style: "display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px;",

                                div {
                                    style: "{card_style}",
                                    div { style: "{stat_value_style}", "{data.connected_peers}" }
                                    div { style: "{stat_label_style}", "Connected peers" }
                                }
                                div {
                                    style: "{card_style}",
                                    div { style: "{stat_value_style}", "{data.discovered_agents}" }
                                    div { style: "{stat_label_style}", "Discovered agents" }
                                }
                                div {
                                    style: "{card_style}",
                                    div { style: "{stat_value_style}", "{data.ws_sessions}" }
                                    div { style: "{stat_label_style}", "WebSocket sessions" }
                                }
                                div {
                                    style: "{card_style}",
                                    div { style: "{stat_value_style}", "{data.direct_connections}" }
                                    div { style: "{stat_label_style}", "Direct links" }
                                }
                            }

                            div {
                                style: "display: grid; grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.8fr); gap: 16px; align-items: start;",

                                div {
                                    style: "{card_style}",
                                    div { style: "{section_title_style}", "Identity" }

                                    div {
                                        style: "display: grid; gap: 14px;",

                                        div {
                                            div { style: "{label_style}", "Display name" }
                                            div { style: "{value_style}", "{data.display_name}" }
                                        }

                                        div {
                                            div { style: "{label_style}", "Agent ID" }
                                            div { style: "{mono_style}", "{data.agent_id}" }
                                        }

                                        div {
                                            div { style: "{label_style}", "Machine ID" }
                                            div { style: "{mono_style}", "{data.machine_id}" }
                                        }

                                        if let Some(user_id) = data.user_id.clone() {
                                            div {
                                                div { style: "{label_style}", "User ID" }
                                                div { style: "{value_style}", "{user_id}" }
                                            }
                                        }
                                    }

                                    div {
                                        style: format!("display: flex; flex-wrap: wrap; gap: {}; margin-top: {};", spacing::MD, spacing::MD),
                                        button {
                                            style: "{secondary_btn_style}",
                                            onclick: {
                                                let value = data.agent_id.clone();
                                                move |_| copy_text(&value)
                                            },
                                            "Copy agent ID"
                                        }
                                        button {
                                            style: "{secondary_btn_style}",
                                            onclick: {
                                                let value = data.machine_id.clone();
                                                move |_| copy_text(&value)
                                            },
                                            "Copy machine ID"
                                        }
                                        if let Some(link) = data.share_link.clone() {
                                            button {
                                                style: "{secondary_btn_style}",
                                                onclick: move |_| copy_text(&link),
                                                "Copy share link"
                                            }
                                        }
                                    }
                                }

                                div {
                                    style: "{card_style}",
                                    div { style: "{section_title_style}", "Local shell" }

                                    div {
                                        style: "display: grid; gap: 14px;",
                                        div {
                                            div { style: "{label_style}", "Status" }
                                            div { style: "{value_style}",
                                                {if data.status.is_empty() { "Unknown" } else { data.status.as_str() }}
                                            }
                                        }
                                        div {
                                            div { style: "{label_style}", "Version" }
                                            div { style: "{value_style}",
                                                {if data.version.is_empty() { "-" } else { data.version.as_str() }}
                                            }
                                        }
                                        div {
                                            div { style: "{label_style}", "Uptime" }
                                            div { style: "{value_style}", "{format_uptime(data.uptime_secs)}" }
                                        }
                                        div {
                                            div { style: "{label_style}", "Reachability" }
                                            div {
                                                style: "{value_style}",
                                                {if data.can_receive_direct { "Receives direct traffic" } else { "Indirect / relay path" }}
                                            }
                                        }
                                        div {
                                            div { style: "{label_style}", "Public IP" }
                                            div {
                                                style: "{value_style}",
                                                {if data.has_public_ip { "Detected" } else { "Not detected" }}
                                            }
                                        }
                                    }
                                }
                            }

                            div {
                                style: "display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; align-items: start;",

                                div {
                                    style: "{card_style}",
                                    div { style: "{section_title_style}", "Addresses & sharing" }

                                    if let Some(link) = data.share_link.clone() {
                                        div {
                                            style: format!("margin-bottom: {};", spacing::MD),
                                            div { style: "{label_style}", "Share link" }
                                            div { style: "{mono_style}", "{link}" }
                                        }
                                    }

                                    div { style: "{label_style}", "Addresses" }
                                    if data.addresses.is_empty() {
                                        div {
                                            style: format!("{} color: {};", value_style, colors::TEXT_MUTED),
                                            "No announced addresses yet."
                                        }
                                    } else {
                                        div {
                                            style: format!("display: flex; flex-direction: column; gap: {}; margin-top: {};", spacing::XS, spacing::XS),
                                            for addr in data.addresses.iter() {
                                                div {
                                                    key: "{addr}",
                                                    style: "{mono_style}",
                                                    "{addr}"
                                                }
                                            }
                                        }
                                    }
                                }

                                div {
                                    style: "{card_style}",
                                    div { style: "{section_title_style}", "Daemon & network details" }

                                    div {
                                        style: "display: grid; gap: 14px;",
                                        div {
                                            div { style: "{label_style}", "API address" }
                                            div {
                                                style: "{mono_style}",
                                                {data.api_address.clone().unwrap_or_else(|| "127.0.0.1:12700".to_string())}
                                            }
                                        }
                                        div {
                                            div { style: "{label_style}", "NAT type" }
                                            div {
                                                style: "{value_style}",
                                                {data.nat_type.clone().unwrap_or_else(|| "Unknown".to_string())}
                                            }
                                        }
                                        div {
                                            div { style: "{label_style}", "Bootstrap peers" }
                                            div { style: "{value_style}", "{data.bootstrap_peers.len()}" }
                                        }
                                    }

                                    if !data.warnings.is_empty() {
                                        div {
                                            style: format!("margin-top: {}; display: flex; flex-direction: column; gap: {};", spacing::MD, spacing::XS),
                                            div { style: "{label_style}", "Warnings" }
                                            for warning in data.warnings.iter() {
                                                div {
                                                    key: "{warning}",
                                                    style: format!("font-size: {}; color: {};", typography::TEXT_SM, colors::DANGER),
                                                    "{warning}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div {
                                style: "display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 16px; align-items: start;",

                                div {
                                    style: "{card_style}",
                                    div { style: "{section_title_style}", "Connected peers" }

                                    if data.peers.is_empty() {
                                        div {
                                            style: format!("font-size: {}; color: {};", typography::TEXT_SM, colors::TEXT_MUTED),
                                            "No active gossip peers right now."
                                        }
                                    } else {
                                        div {
                                            style: format!("display: flex; flex-direction: column; gap: {};", spacing::XS),
                                            for peer in data.peers.iter() {
                                                div {
                                                    key: "{peer}",
                                                    style: format!("display: flex; align-items: center; justify-content: space-between; gap: {}; padding: {} 0; border-bottom: 1px solid {};", spacing::SM, spacing::XS, colors::BORDER_DEFAULT),
                                                    span { style: "{mono_style}", "{peer}" }
                                                    button {
                                                        style: "{secondary_btn_style}",
                                                        onclick: {
                                                            let peer_id = peer.clone();
                                                            move |_| copy_text(&peer_id)
                                                        },
                                                        "Copy"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div {
                                    style: "{card_style}",
                                    div { style: "{section_title_style}", "Desktop direction" }
                                    ul {
                                        style: format!("padding-left: 18px; display: flex; flex-direction: column; gap: {}; color: {}; font-size: {}; line-height: 1.5;", spacing::XS, colors::TEXT_SECONDARY, typography::TEXT_SM),
                                        li { "Owned-machine first: launch into x0x without a login or recovery wall." }
                                        li { "Identity lives in the sidebar header and this page, not in startup onboarding." }
                                        li { "Messaging and spaces stay aligned with x0x topics, stores, and local history conventions." }
                                        li { "Legacy auth/four-word flows are kept out of the active desktop lane unless explicitly revisited later." }
                                    }
                                    div {
                                        style: format!("margin-top: {}; font-size: {}; color: {};", spacing::MD, typography::TEXT_XS, colors::TEXT_MUTED),
                                        "Build: {crate::version::CURRENT.version} ({crate::version::CURRENT.commit_hash})"
                                    }
                                    if !data.agent_id.is_empty() {
                                        div {
                                            style: format!("margin-top: {}; font-size: {}; color: {};", spacing::SM, typography::TEXT_XS, colors::TEXT_MUTED),
                                            "Visible shell identity: {data.display_name} · {short_id(&data.agent_id)}"
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
