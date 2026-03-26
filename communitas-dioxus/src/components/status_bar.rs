//! Bottom status bar showing daemon connectivity, peer count, and agent identity.

use communitas_x0x_client::{DaemonManager, DaemonState, X0xClient};
use dioxus::prelude::*;

use crate::tokens::{colors, radius, spacing, typography};

/// How often (in seconds) to poll the daemon health endpoint.
const POLL_INTERVAL_SECS: u64 = 5;

/// Truncate an agent ID for display.
fn truncate_id(id: &str) -> String {
    if id.len() <= 16 {
        id.to_owned()
    } else {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    }
}

/// Format an uptime duration in seconds into a human-readable string.
#[cfg(test)]
fn format_uptime(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{hours}h {mins}m")
    }
}

/// Compact bottom status bar (32px).
///
/// - Left: connection dot + "Connected"/"Disconnected"
/// - Center: peer count
/// - Right: truncated agent ID (click to copy) + version
#[component]
pub fn StatusBar() -> Element {
    let mut daemon_state = use_signal(|| DaemonState::NotRunning);
    let mut agent_id = use_signal(|| None::<String>);
    let mut peers = use_signal(|| 0u32);
    let mut version = use_signal(|| None::<String>);
    let mut copied = use_signal(|| false);

    // Poll daemon health every POLL_INTERVAL_SECS.
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();
        let manager = DaemonManager::new();

        loop {
            match client.health().await {
                Ok(health) => {
                    let state = if health.status == "healthy" || health.status == "running" {
                        DaemonState::Running
                    } else {
                        DaemonState::Degraded
                    };
                    daemon_state.set(state);
                    peers.set(health.peers);
                    version.set(Some(health.version.clone()));

                    match client.agent().await {
                        Ok(identity) => agent_id.set(Some(identity.agent_id)),
                        Err(e) => {
                            warn!(target: "ui.status_bar", "failed to fetch agent: {e}");
                            agent_id.set(None);
                        }
                    }
                }
                Err(_) => {
                    let state = manager.state().await;
                    daemon_state.set(state);
                    agent_id.set(None);
                    peers.set(0);
                    version.set(None);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    });

    let current_state = *daemon_state.read();
    let current_peers = *peers.read();
    let current_agent_id = agent_id.read().clone();
    let current_version = version.read().clone();
    let is_copied = *copied.read();

    let (dot_color, status_label) = match current_state {
        DaemonState::Running => (colors::SUCCESS, "Connected"),
        DaemonState::Degraded => (colors::WARNING, "Degraded"),
        DaemonState::NotRunning => (colors::DANGER, "Disconnected"),
        DaemonState::NotInstalled => (colors::TEXT_MUTED, "Not Installed"),
    };

    let bar_style = format!(
        "display: flex; align-items: center; gap: {}; \
         padding: 0 {}; \
         background-color: {}; \
         border-top: 1px solid {}; \
         font-family: {}; font-size: {}; \
         height: 32px; flex-shrink: 0; user-select: none;",
        spacing::MD,
        spacing::MD,
        colors::SURFACE_CARD,
        colors::BORDER_DEFAULT,
        typography::FONT_SANS,
        typography::TEXT_XS,
    );

    let dot_style = format!(
        "width: 8px; height: 8px; border-radius: 50%; background-color: {}; flex-shrink: 0;",
        dot_color,
    );

    let label_style = format!(
        "color: {}; font-weight: 500; white-space: nowrap;",
        colors::TEXT_SECONDARY,
    );

    let detail_style = format!("color: {}; white-space: nowrap;", colors::TEXT_MUTED);

    let separator_style = format!(
        "width: 1px; height: 14px; background-color: {}; flex-shrink: 0;",
        colors::BORDER_DEFAULT,
    );

    let agent_style = format!(
        "color: {}; font-family: {}; font-size: {}; white-space: nowrap; \
         cursor: pointer; padding: 2px {}; border-radius: {}; \
         transition: color 150ms ease;",
        if is_copied {
            colors::SUCCESS
        } else {
            colors::PRIMARY
        },
        typography::FONT_MONO,
        typography::TEXT_XS,
        spacing::XS,
        radius::SM,
    );

    let copy_agent_id = move |_: MouseEvent| {
        if let Some(ref id) = *agent_id.read() {
            let id_clone = id.clone();
            spawn(async move {
                let escaped = id_clone.replace('\\', "\\\\").replace('"', "\\\"");
                let script =
                    format!("navigator.clipboard.writeText(\"{escaped}\").catch(()=>{{}});",);
                let _ = dioxus::document::eval(&script);
                copied.set(true);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                copied.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "status-bar",
            style: "{bar_style}",
            role: "status",
            "aria-live": "polite",
            "aria-label": "Application status",

            // Status dot
            span {
                style: "{dot_style}",
                "aria-hidden": "true",
            }

            // Status label
            span {
                style: "{label_style}",
                "{status_label}"
            }

            if matches!(current_state, DaemonState::Running | DaemonState::Degraded) {
                span { style: "{separator_style}", "aria-hidden": "true" }
                span {
                    style: "{detail_style}",
                    {format!("{current_peers} peer{}", if current_peers != 1 { "s" } else { "" })}
                }
            }

            // Spacer
            span { style: "flex: 1;" }

            // Version
            if let Some(ref ver) = current_version {
                span {
                    style: "{detail_style}",
                    "v{ver}"
                }
                span { style: "{separator_style}", "aria-hidden": "true" }
            }

            // Agent ID (click to copy)
            if let Some(ref id) = current_agent_id {
                span {
                    style: "{agent_style}",
                    title: if is_copied { "Copied!" } else { "Click to copy agent ID" },
                    onclick: copy_agent_id,
                    if is_copied {
                        "Copied!"
                    } else {
                        "{truncate_id(id)}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_id() {
        assert_eq!(truncate_id("abc123"), "abc123");
    }

    #[test]
    fn truncate_long_id() {
        let long = "abcdefghijklmnopqrstuvwxyz012345";
        let t = truncate_id(long);
        assert!(t.contains("..."));
        assert!(t.starts_with("abcdefgh"));
    }

    #[test]
    fn format_uptime_seconds_only() {
        assert_eq!(format_uptime(45), "45s");
    }

    #[test]
    fn format_uptime_minutes_and_seconds() {
        assert_eq!(format_uptime(125), "2m 5s");
    }

    #[test]
    fn format_uptime_hours_and_minutes() {
        assert_eq!(format_uptime(7265), "2h 1m");
    }
}
