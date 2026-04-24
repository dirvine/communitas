// SPDX-License-Identifier: MIT OR Apache-2.0

//! Network diagnostics view.
//!
//! Shows network status, peer list, NAT info, external addresses.

use communitas_x0x_client::{ConnectivityDiagnostics, DiscoveredMachine, GossipStats, X0xClient};
use dioxus::prelude::*;

use crate::tokens::{colors, radius, spacing, typography};

/// How often to poll network status.
const POLL_INTERVAL_SECS: u64 = 5;

/// Truncate a peer/agent ID.
fn short_id(id: &str) -> String {
    if id.len() <= 16 {
        id.to_owned()
    } else {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    }
}

/// Copy to clipboard.
fn copy_text(value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("navigator.clipboard.writeText(\"{escaped}\").catch(()=>{{}});",);
    spawn(async move {
        let _ = dioxus::document::eval(&script);
    });
}

#[derive(Clone, Default)]
struct NetworkData {
    connected_peers: u32,
    direct_connections: u32,
    relayed_connections: u32,
    avg_rtt_ms: Option<u64>,
    nat_type: String,
    hole_punch_rate: Option<f64>,
    external_addrs: Vec<String>,
    agent_id: String,
    machine_id: String,
    gossip_stats: Option<GossipStats>,
    connectivity: Option<ConnectivityDiagnostics>,
    discovered_machines: Vec<DiscoveredMachine>,
    agent_machine: Option<DiscoveredMachine>,
    user_machine_count: Option<usize>,
    peers: Vec<PeerRow>,
}

#[derive(Clone, PartialEq)]
struct PeerRow {
    peer_id: String,
    health: String,
    probe: String,
}

/// Network diagnostics page.
#[component]
pub fn NetworkView() -> Element {
    let mut data = use_signal(NetworkData::default);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let action_message = use_signal(|| None::<String>);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();

        loop {
            let mut d = NetworkData::default();
            let mut had_error = false;

            match client.network_status().await {
                Ok(ns) => {
                    d.connected_peers = ns.connected_peers;
                    d.direct_connections = ns.direct_connections;
                    d.relayed_connections = ns.relayed_connections;
                    d.avg_rtt_ms = ns.avg_rtt_ms;
                    d.nat_type = ns.nat_type.unwrap_or_else(|| "Unknown".to_string());
                    d.hole_punch_rate = ns.hole_punch_success_rate;
                    d.external_addrs = ns.external_addrs;
                }
                Err(e) => {
                    error.set(Some(format!("Failed to fetch network status: {e}")));
                    had_error = true;
                }
            }

            if !had_error {
                if let Ok(agent) = client.agent().await {
                    if let Ok(agent_machine) = client.machine_for_agent(&agent.agent_id).await {
                        d.agent_machine = Some(agent_machine.machine);
                    }
                    if let Some(user_id) = &agent.user_id
                        && let Ok(user_machines) = client.machines_by_user(user_id).await
                    {
                        d.user_machine_count = Some(user_machines.machines.len());
                    }
                    d.agent_id = agent.agent_id;
                    d.machine_id = agent.machine_id;
                }

                if let Ok(connectivity) = client.connectivity_diagnostics().await {
                    d.connectivity = Some(connectivity);
                }

                if let Ok(machines) = client.discovered_machines(false).await {
                    if d.agent_machine.is_none()
                        && let Some(first) = machines.first()
                        && let Ok(detail) =
                            client.discovered_machine(&first.machine_id, false).await
                    {
                        d.agent_machine = Some(detail);
                    }
                    d.discovered_machines = machines;
                }

                if let Ok(peers) = client.peers().await {
                    let mut rows = Vec::with_capacity(peers.len());
                    for peer in peers {
                        let peer_id = peer.id;
                        let health = match client.peer_health(&peer_id).await {
                            Ok(snapshot) => {
                                snapshot.health.or(snapshot.error).unwrap_or_else(|| {
                                    if snapshot.ok {
                                        "Healthy".to_string()
                                    } else {
                                        "Unknown".to_string()
                                    }
                                })
                            }
                            Err(err) => format!("Unavailable: {err}"),
                        };
                        let probe = match client.probe_peer(&peer_id, 1_000).await {
                            Ok(result) => {
                                if let Some(ms) = result.rtt_ms {
                                    format!("{ms} ms")
                                } else if let Some(us) = result.rtt_us {
                                    format!("{us} us")
                                } else if let Some(err) = result.error {
                                    err
                                } else if result.ok {
                                    "OK".to_string()
                                } else {
                                    "No RTT".to_string()
                                }
                            }
                            Err(err) => format!("Probe failed: {err}"),
                        };
                        rows.push(PeerRow {
                            peer_id,
                            health,
                            probe,
                        });
                    }
                    d.peers = rows;
                }

                if let Ok(stats) = client.gossip_stats().await {
                    d.gossip_stats = Some(stats);
                }

                error.set(None);
            }

            data.set(d);
            loading.set(false);

            tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    });

    let d = data.read().clone();
    let is_loading = *loading.read();
    let current_error = error.read().clone();
    let current_action_message = action_message.read().clone();

    let page_style = format!(
        "padding: {}; display: flex; flex-direction: column; gap: {}; \
         overflow-y: auto; height: 100%;",
        spacing::LG,
        spacing::LG,
    );

    let heading_style = format!(
        "font-size: {}; font-weight: 700; color: {};",
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

    let mono_style = format!(
        "font-family: {}; font-size: {}; color: {}; word-break: break-all; cursor: pointer;",
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
                    style: format!("color: {}; padding: {};", colors::TEXT_MUTED, spacing::LG),
                    "Loading network status..."
                }
            }
        };
    }

    rsx! {
        div {
            style: "{page_style}",

            h1 { style: "{heading_style}", "Network" }

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
            if let Some(msg) = current_action_message.as_ref() {
                div {
                    style: format!(
                        "background-color: rgba(51, 204, 153, 0.1); border: 1px solid {}; \
                         border-radius: {}; padding: {}; color: {}; font-size: {};",
                        colors::SUCCESS, radius::MD, spacing::MD, colors::SUCCESS, typography::TEXT_SM,
                    ),
                    "{msg}"
                }
            }

            // Stats grid
            div {
                style: "display: grid; grid-template-columns: repeat(5, 1fr); gap: 12px;",

                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{d.connected_peers}" }
                    div { style: "{stat_label_style}", "Connected" }
                }
                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{d.direct_connections}" }
                    div { style: "{stat_label_style}", "Direct" }
                }
                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{d.relayed_connections}" }
                    div { style: "{stat_label_style}", "Relayed" }
                }
                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}",
                        {d.avg_rtt_ms.map(|ms| format!("{ms}ms")).unwrap_or_else(|| "-".to_string())}
                    }
                    div { style: "{stat_label_style}", "Avg RTT" }
                }
                div {
                    style: "{card_style}",
                    div { style: "{stat_value_style}", "{d.nat_type}" }
                    div { style: "{stat_label_style}", "NAT Type" }
                }
            }

            // Hole-punch rate
            if let Some(rate) = d.hole_punch_rate {
                div {
                    style: "{card_style}",
                    div {
                        style: format!("font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                            typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM),
                        "Hole-Punch Success Rate"
                    }
                    div {
                        style: format!("font-size: {}; color: {};", typography::TEXT_2XL, colors::SUCCESS),
                        {format!("{:.1}%", rate * 100.0)}
                    }
                }
            }

            // Connectivity snapshot
            if let Some(snapshot) = &d.connectivity {
                div {
                    style: "{card_style}",
                    div {
                        style: format!("font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                            typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM),
                        "Connectivity Snapshot"
                    }
                    div {
                        style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;",
                        div {
                            div { style: "{stat_value_style}", "{snapshot.connections.connected_peers}" }
                            div { style: "{stat_label_style}", "Node Peers" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{snapshot.connections.direct}" }
                            div { style: "{stat_label_style}", "Direct Paths" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{snapshot.mdns.discovered_peers}" }
                            div { style: "{stat_label_style}", "mDNS Peers" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{snapshot.services.relay_enabled}" }
                            div { style: "{stat_label_style}", "Relay Service" }
                        }
                    }
                }
            }

            // Gossip diagnostics
            if let Some(stats) = &d.gossip_stats {
                div {
                    style: "{card_style}",
                    div {
                        style: format!("font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                            typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM),
                        "Gossip Pipeline"
                    }
                    div {
                        style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;",
                        div {
                            div { style: "{stat_value_style}", "{stats.publish_total}" }
                            div { style: "{stat_label_style}", "Published" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{stats.incoming_total}" }
                            div { style: "{stat_label_style}", "Incoming" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{stats.delivered_to_subscriber}" }
                            div { style: "{stat_label_style}", "Delivered" }
                        }
                        div {
                            div {
                                style: format!(
                                    "font-size: {}; font-weight: 700; color: {};",
                                    typography::TEXT_2XL,
                                    if stats.decode_to_delivery_drops == 0 { colors::SUCCESS } else { colors::DANGER },
                                ),
                                "{stats.decode_to_delivery_drops}"
                            }
                            div { style: "{stat_label_style}", "Drops" }
                        }
                    }
                    div {
                        style: format!(
                            "display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-top: {};",
                            spacing::SM,
                        ),
                        div {
                            div { style: "{stat_value_style}", "{stats.publish_failed}" }
                            div { style: "{stat_label_style}", "Publish Failed" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{stats.incoming_decoded}" }
                            div { style: "{stat_label_style}", "Decoded" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{stats.incoming_decode_failed}" }
                            div { style: "{stat_label_style}", "Decode Failed" }
                        }
                        div {
                            div { style: "{stat_value_style}", "{stats.in_flight_decode}" }
                            div { style: "{stat_label_style}", "In Flight" }
                        }
                    }
                }
            }

            // Identity card
            div {
                style: "{card_style}",
                div {
                    style: format!("font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM),
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

            // External addresses
            if !d.external_addrs.is_empty() {
                div {
                    style: "{card_style}",
                    div {
                        style: format!("font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                            typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM),
                        "External Addresses"
                    }
                    for addr in &d.external_addrs {
                        div {
                            key: "{addr}",
                            style: format!(
                                "font-family: {}; font-size: {}; color: {}; padding: 2px 0;",
                                typography::FONT_MONO, typography::TEXT_XS, colors::TEXT_SECONDARY,
                            ),
                            "{addr}"
                        }
                    }
                }
            }

            // Discovered machine endpoints
            div {
                style: "{card_style}",
                div {
                    style: format!("font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM),
                    "Discovered Machines"
                }

                if let Some(machine) = &d.agent_machine {
                    div {
                        style: format!("font-size: {}; color: {}; margin-bottom: {};",
                            typography::TEXT_XS, colors::TEXT_SECONDARY, spacing::SM),
                        "Current agent machine: {short_id(&machine.machine_id)}"
                    }
                }

                if let Some(count) = d.user_machine_count {
                    div {
                        style: format!("font-size: {}; color: {}; margin-bottom: {};",
                            typography::TEXT_XS, colors::TEXT_SECONDARY, spacing::SM),
                        "Machines for current user: {count}"
                    }
                }

                if d.discovered_machines.is_empty() {
                    div {
                        style: format!("color: {}; font-size: {};", colors::TEXT_MUTED, typography::TEXT_SM),
                        "No machine announcements discovered."
                    }
                } else {
                    div {
                        style: format!("display: grid; grid-template-columns: 1fr 1fr 96px; gap: 12px; {table_header_style}"),
                        span { "Machine ID" }
                        span { "Addresses" }
                        span { "Action" }
                    }
                    for machine in &d.discovered_machines {
                        {
                            let machine_id = machine.machine_id.clone();
                            let address_count = machine.addresses.len();
                            let mut action_message = action_message;
                            rsx! {
                                div {
                                    key: "{machine_id}",
                                    style: format!("display: grid; grid-template-columns: 1fr 1fr 96px; gap: 12px; align-items: center; {table_row_style}"),
                                    span {
                                        style: format!(
                                            "font-family: {}; font-size: {}; color: {};",
                                            typography::FONT_MONO, typography::TEXT_XS, colors::PRIMARY,
                                        ),
                                        "{short_id(&machine_id)}"
                                    }
                                    span {
                                        style: format!("font-size: {}; color: {};", typography::TEXT_XS, colors::TEXT_SECONDARY),
                                        "{address_count}"
                                    }
                                    button {
                                        style: format!(
                                            "border: 1px solid {}; border-radius: {}; background: {}; color: {}; padding: 4px 8px; cursor: pointer;",
                                            colors::BORDER_DEFAULT, radius::SM, colors::SURFACE_CARD, colors::TEXT_PRIMARY,
                                        ),
                                        onclick: move |_| {
                                            let machine_id = machine_id.clone();
                                            spawn(async move {
                                                let client = X0xClient::new();
                                                let msg = match client.connect_machine(&machine_id).await {
                                                    Ok(result) => format!("{}: {}", short_id(&machine_id), result.outcome),
                                                    Err(err) => format!("{}: {err}", short_id(&machine_id)),
                                                };
                                                action_message.set(Some(msg));
                                            });
                                        },
                                        "Connect"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Connected peers table
            div {
                style: "{card_style}",
                div {
                    style: format!("font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                        typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM),
                    "Connected Peers"
                }

                if d.peers.is_empty() {
                    div {
                        style: format!("color: {}; font-size: {};", colors::TEXT_MUTED, typography::TEXT_SM),
                        "No peers connected."
                    }
                } else {
                    div {
                        style: format!("display: grid; grid-template-columns: 1.3fr 1fr 1fr; gap: 12px; {table_header_style}"),
                        span { "Peer ID" }
                        span { "Health" }
                        span { "Probe RTT" }
                    }
                    for peer in &d.peers {
                        {
                            let pid = peer.peer_id.clone();
                            let health = peer.health.clone();
                            let probe = peer.probe.clone();
                            rsx! {
                                div {
                                    key: "{pid}",
                                    style: format!("display: grid; grid-template-columns: 1.3fr 1fr 1fr; gap: 12px; {table_row_style}"),
                                    span {
                                        style: format!(
                                            "font-family: {}; font-size: {}; color: {};",
                                            typography::FONT_MONO, typography::TEXT_XS, colors::PRIMARY,
                                        ),
                                        "{short_id(&pid)}"
                                    }
                                    span {
                                        style: format!(
                                            "font-size: {}; color: {}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                            typography::TEXT_XS, colors::TEXT_SECONDARY,
                                        ),
                                        title: "{health}",
                                        "{health}"
                                    }
                                    span {
                                        style: format!(
                                            "font-family: {}; font-size: {}; color: {};",
                                            typography::FONT_MONO, typography::TEXT_XS, colors::PRIMARY,
                                        ),
                                        "{probe}"
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
