//! x0xd daemon status bar component.
//!
//! Periodically polls the x0xd daemon health endpoint and displays
//! connection status, peer count, uptime, and agent identity.
//! When the daemon is not installed or not running, offers actions
//! to install or start it.
//!
//! # Example
//!
//! ```ignore
//! use communitas_dioxus::components::daemon_status::DaemonStatusBar;
//!
//! rsx! {
//!     DaemonStatusBar {}
//!     // ... rest of your app
//! }
//! ```

use communitas_x0x_client::{DaemonManager, DaemonState, X0xClient};
use dioxus::prelude::*;
use tracing::warn;

use crate::tokens::{colors, radius, spacing, typography};

/// How often (in seconds) to poll the daemon health endpoint.
const POLL_INTERVAL_SECS: u64 = 5;

/// Format an uptime duration in seconds into a human-readable string.
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

/// Truncate an agent ID for display, showing the first and last few characters.
fn truncate_agent_id(id: &str) -> String {
    if id.len() <= 16 {
        id.to_owned()
    } else {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    }
}

/// Status bar showing the current state of the x0xd daemon.
///
/// Renders a compact bar at the top of the application with:
/// - A colored status dot (green = running, amber = degraded, red = not running, gray = not installed)
/// - Status label
/// - Peer count and uptime when connected
/// - Truncated agent ID when available
/// - "Install & Start" or "Start" button when daemon is not available
#[component]
pub fn DaemonStatusBar() -> Element {
    let mut state = use_signal(|| DaemonState::NotRunning);
    let mut agent_id = use_signal(|| None::<String>);
    let mut peers = use_signal(|| 0u32);
    let mut uptime = use_signal(|| 0u64);
    let mut version = use_signal(|| None::<String>);
    let mut action_in_progress = use_signal(|| false);
    let mut action_error = use_signal(|| None::<String>);

    // Poll daemon health every POLL_INTERVAL_SECS seconds.
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();
        let manager = DaemonManager::new();

        loop {
            match client.health().await {
                Ok(health) => {
                    let daemon_state = if health.status == "healthy" || health.status == "running" {
                        DaemonState::Running
                    } else {
                        DaemonState::Degraded
                    };
                    state.set(daemon_state);
                    peers.set(health.peers);
                    uptime.set(health.uptime_secs);
                    version.set(Some(health.version.clone()));

                    match client.agent().await {
                        Ok(identity) => agent_id.set(Some(identity.agent_id)),
                        Err(e) => {
                            warn!(target: "ui.daemon", "failed to fetch agent identity: {e}");
                            agent_id.set(None);
                        }
                    }
                }
                Err(_) => {
                    let current_state = manager.state().await;
                    state.set(current_state);
                    agent_id.set(None);
                    peers.set(0);
                    uptime.set(0);
                    version.set(None);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    });

    let current_state = *state.read();
    let current_peers = *peers.read();
    let current_uptime = *uptime.read();
    let current_agent_id = agent_id.read().clone();
    let current_version = version.read().clone();
    let is_busy = *action_in_progress.read();
    let current_error = action_error.read().clone();

    // Status dot color and label
    let (dot_color, status_label) = match current_state {
        DaemonState::Running => (colors::SUCCESS, "x0xd Running"),
        DaemonState::Degraded => (colors::WARNING, "x0xd Degraded"),
        DaemonState::NotRunning => (colors::DANGER, "x0xd Stopped"),
        DaemonState::NotInstalled => (colors::TEXT_MUTED, "x0xd Not Installed"),
    };

    // Whether to show the action button
    let show_action = matches!(
        current_state,
        DaemonState::NotRunning | DaemonState::NotInstalled
    );

    let action_label = match current_state {
        DaemonState::NotInstalled => "Install & Start",
        DaemonState::NotRunning => "Start",
        _ => "",
    };

    // Bar container style
    let bar_style = format!(
        "display: flex; align-items: center; gap: {}; \
         padding: {} {}; \
         background-color: {}; \
         border-bottom: 1px solid {}; \
         font-family: {}; font-size: {}; \
         min-height: 32px; flex-shrink: 0;",
        spacing::SM,
        spacing::XS,
        spacing::MD,
        colors::SURFACE_CARD,
        colors::BORDER_DEFAULT,
        typography::FONT_SANS,
        typography::TEXT_XS,
    );

    // Status dot style
    let dot_style = format!(
        "width: 8px; height: 8px; border-radius: 50%; background-color: {}; flex-shrink: 0;{}",
        dot_color,
        if matches!(current_state, DaemonState::Degraded) {
            " animation: pulse 1.5s ease-in-out infinite;"
        } else {
            ""
        }
    );

    // Label style
    let label_style = format!(
        "color: {}; font-weight: 600; white-space: nowrap;",
        colors::TEXT_PRIMARY,
    );

    // Detail text style
    let detail_style = format!("color: {}; white-space: nowrap;", colors::TEXT_SECONDARY,);

    // Separator style
    let separator_style = format!(
        "width: 1px; height: 14px; background-color: {}; flex-shrink: 0;",
        colors::BORDER_DEFAULT,
    );

    // Action button style
    let button_style = format!(
        "background-color: {}; color: {}; border: none; \
         padding: 0.125rem {}; font-size: {}; font-family: {}; \
         border-radius: {}; cursor: pointer; white-space: nowrap; \
         font-weight: 500; transition: background-color 150ms ease;",
        colors::PRIMARY,
        colors::TEXT_INVERSE,
        spacing::SM,
        typography::TEXT_XS,
        typography::FONT_SANS,
        radius::SM,
    );

    let disabled_button_style = format!(
        "background-color: {}; color: {}; border: none; \
         padding: 0.125rem {}; font-size: {}; font-family: {}; \
         border-radius: {}; cursor: not-allowed; white-space: nowrap; \
         font-weight: 500; opacity: 0.6;",
        colors::SURFACE_ELEVATED,
        colors::TEXT_MUTED,
        spacing::SM,
        typography::TEXT_XS,
        typography::FONT_SANS,
        radius::SM,
    );

    // Error style
    let error_style = format!(
        "color: {}; font-size: {}; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 300px;",
        colors::DANGER,
        typography::TEXT_XS,
    );

    // Spacer to push details to the right
    let spacer_style = "flex: 1;";

    // Agent ID style (monospace)
    let agent_style = format!(
        "color: {}; font-family: {}; font-size: {}; white-space: nowrap;",
        colors::TEXT_MUTED,
        typography::FONT_MONO,
        typography::TEXT_XS,
    );

    rsx! {
        div {
            class: "daemon-status-bar",
            style: "{bar_style}",
            role: "status",
            "aria-live": "polite",
            "aria-label": "x0x daemon status",

            // Status dot
            span {
                class: "daemon-status-dot",
                style: "{dot_style}",
                "aria-hidden": "true",
            }

            // Status label
            span {
                class: "daemon-status-label",
                style: "{label_style}",
                "{status_label}"
            }

            // Details when running or degraded
            if matches!(current_state, DaemonState::Running | DaemonState::Degraded) {
                span {
                    style: "{separator_style}",
                    "aria-hidden": "true",
                }
                span {
                    class: "daemon-peers",
                    style: "{detail_style}",
                    {format!("{current_peers} peer{}", if current_peers != 1 { "s" } else { "" })}
                }
                span {
                    style: "{separator_style}",
                    "aria-hidden": "true",
                }
                span {
                    class: "daemon-uptime",
                    style: "{detail_style}",
                    {format!("up {}", format_uptime(current_uptime))}
                }
                if let Some(ref ver) = current_version {
                    span {
                        style: "{separator_style}",
                        "aria-hidden": "true",
                    }
                    span {
                        class: "daemon-version",
                        style: "{detail_style}",
                        "v{ver}"
                    }
                }
            }

            // Spacer
            span { style: "{spacer_style}" }

            // Error message
            if let Some(ref err) = current_error {
                span {
                    class: "daemon-action-error",
                    style: "{error_style}",
                    title: "{err}",
                    "{err}"
                }
            }

            // Agent ID (right-aligned)
            if let Some(ref id) = current_agent_id {
                span {
                    class: "daemon-agent-id",
                    style: "{agent_style}",
                    title: "{id}",
                    "{truncate_agent_id(id)}"
                }
            }

            // Action button when not running
            if show_action {
                button {
                    r#type: "button",
                    class: "daemon-action-button",
                    style: if is_busy { "{disabled_button_style}" } else { "{button_style}" },
                    disabled: is_busy,
                    "aria-label": if is_busy { "Operation in progress" } else { "{action_label}" },
                    onclick: move |_| {
                        if is_busy {
                            return;
                        }
                        let target_state = current_state;
                        action_in_progress.set(true);
                        action_error.set(None);
                        spawn(async move {
                            let result = match target_state {
                                DaemonState::NotInstalled => {
                                    tracing::info!(target: "ui.daemon", "installing and starting x0xd");
                                    let dm = DaemonManager::new();
                                    dm.ensure_running().await
                                }
                                DaemonState::NotRunning => {
                                    tracing::info!(target: "ui.daemon", "starting x0xd");
                                    DaemonManager::start().await
                                }
                                _ => Ok(()),
                            };
                            match result {
                                Ok(()) => {
                                    tracing::info!(target: "ui.daemon", "daemon action completed successfully");
                                    action_error.set(None);
                                }
                                Err(e) => {
                                    tracing::error!(target: "ui.daemon", "daemon action failed: {e}");
                                    action_error.set(Some(format!("{e}")));
                                }
                            }
                            action_in_progress.set(false);
                        });
                    },
                    if is_busy { "Working..." } else { "{action_label}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_uptime_seconds() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(45), "45s");
    }

    #[test]
    fn format_uptime_minutes() {
        assert_eq!(format_uptime(60), "1m 0s");
        assert_eq!(format_uptime(125), "2m 5s");
    }

    #[test]
    fn format_uptime_hours() {
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(7265), "2h 1m");
    }

    #[test]
    fn truncate_short_agent_id() {
        assert_eq!(truncate_agent_id("abc123"), "abc123");
        assert_eq!(truncate_agent_id("0123456789abcdef"), "0123456789abcdef");
    }

    #[test]
    fn truncate_long_agent_id() {
        let long = "abcdefghijklmnopqrstuvwxyz012345";
        let truncated = truncate_agent_id(long);
        assert!(truncated.contains("..."));
        assert!(truncated.starts_with("abcdefgh"));
        assert!(truncated.ends_with("012345"));
    }
}
