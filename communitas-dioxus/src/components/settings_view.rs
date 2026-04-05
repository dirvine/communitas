// SPDX-License-Identifier: MIT OR Apache-2.0

//! Settings view -- display name, agent card, daemon info.

use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;
use tracing::{info, warn};

use crate::tokens::{colors, radius, spacing, typography};

/// Copy to clipboard.
fn copy_text(value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("navigator.clipboard.writeText(\"{escaped}\").catch(()=>{{}});",);
    spawn(async move {
        let _ = dioxus::document::eval(&script);
    });
}

/// Format uptime.
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

/// Settings page component.
#[component]
pub fn SettingsView() -> Element {
    let mut display_name = use_signal(String::new);
    let mut display_name_original = use_signal(String::new);
    let mut saving_name = use_signal(|| false);
    let mut save_error = use_signal(|| None::<String>);
    let mut save_success = use_signal(|| false);

    let mut agent_card_link = use_signal(|| None::<String>);
    let mut generating_card = use_signal(|| false);
    let mut card_copied = use_signal(|| false);

    let mut daemon_version = use_signal(|| None::<String>);
    let mut daemon_uptime = use_signal(|| 0u64);
    let mut daemon_api_addr = use_signal(|| "127.0.0.1:12700".to_string());
    let mut loading = use_signal(|| true);

    // Load initial data
    use_future(move || async move {
        let client = X0xClient::new();

        if let Ok(card_resp) = client.agent_card(None, Some(false)).await {
            display_name.set(card_resp.card.display_name.clone());
            display_name_original.set(card_resp.card.display_name);
        }

        if let Ok(card_resp) = client.agent_card(None, Some(true)).await {
            agent_card_link.set(Some(card_resp.link));
        }

        if let Ok(status) = client.status().await {
            daemon_version.set(Some(status.version));
            daemon_uptime.set(status.uptime_secs);
            daemon_api_addr.set(status.api_address);
        } else if let Ok(health) = client.health().await {
            daemon_version.set(Some(health.version));
            daemon_uptime.set(health.uptime_secs);
        }

        loading.set(false);
    });

    let is_loading = *loading.read();
    let name_changed = display_name() != display_name_original();

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

    let section_label_style = format!(
        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
        typography::TEXT_SM,
        colors::TEXT_PRIMARY,
        spacing::SM,
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
        typography::FONT_SANS,
        typography::TEXT_SM,
    );

    let btn_primary = format!(
        "background-color: {}; color: {}; border: none; border-radius: {}; \
         padding: {} {}; font-size: {}; font-weight: 500; cursor: pointer;",
        colors::PRIMARY,
        colors::TEXT_INVERSE,
        radius::MD,
        spacing::SM,
        spacing::MD,
        typography::TEXT_SM,
    );

    let btn_secondary = format!(
        "background-color: transparent; color: {}; border: 1px solid {}; border-radius: {}; \
         padding: {} {}; font-size: {}; font-weight: 500; cursor: pointer;",
        colors::TEXT_PRIMARY,
        colors::BORDER_DEFAULT,
        radius::MD,
        spacing::SM,
        spacing::MD,
        typography::TEXT_SM,
    );

    let detail_style = format!(
        "font-size: {}; color: {};",
        typography::TEXT_SM,
        colors::TEXT_SECONDARY,
    );

    let mono_detail = format!(
        "font-family: {}; font-size: {}; color: {};",
        typography::FONT_MONO,
        typography::TEXT_XS,
        colors::PRIMARY,
    );

    if is_loading {
        return rsx! {
            div {
                style: "{page_style}",
                div {
                    style: format!("color: {};", colors::TEXT_MUTED),
                    "Loading settings..."
                }
            }
        };
    }

    rsx! {
        div {
            style: "{page_style}",

            h1 { style: "{heading_style}", "Settings" }

            // Display name
            div {
                style: "{card_style}",
                div { style: "{section_label_style}", "Display Name" }

                div {
                    style: "display: flex; gap: 8px; align-items: center;",

                    input {
                        style: "{input_style}",
                        r#type: "text",
                        placeholder: "Your display name",
                        value: "{display_name}",
                        oninput: move |evt: FormEvent| {
                            display_name.set(evt.value());
                            save_success.set(false);
                        },
                    }

                    button {
                        style: "{btn_primary}",
                        disabled: !name_changed || saving_name(),
                        onclick: move |_| {
                            let new_name = display_name().trim().to_string();
                            if new_name.is_empty() {
                                save_error.set(Some("Name cannot be empty".to_string()));
                                return;
                            }
                            saving_name.set(true);
                            save_error.set(None);

                            spawn(async move {
                                let client = X0xClient::new();
                                // Generate a new card with the updated name
                                match client.agent_card(Some(&new_name), Some(false)).await {
                                    Ok(_) => {
                                        info!(target: "ui.settings", "display name updated to: {new_name}");
                                        display_name_original.set(new_name);
                                        save_success.set(true);
                                    }
                                    Err(e) => {
                                        save_error.set(Some(format!("{e}")));
                                    }
                                }
                                saving_name.set(false);
                            });
                        },
                        if saving_name() { "Saving..." } else { "Save" }
                    }
                }

                if let Some(ref err) = *save_error.read() {
                    div {
                        style: format!("margin-top: {}; font-size: {}; color: {};",
                            spacing::SM, typography::TEXT_XS, colors::DANGER),
                        "{err}"
                    }
                }

                if save_success() {
                    div {
                        style: format!("margin-top: {}; font-size: {}; color: {};",
                            spacing::SM, typography::TEXT_XS, colors::SUCCESS),
                        "Saved!"
                    }
                }
            }

            // Agent card
            div {
                style: "{card_style}",
                div { style: "{section_label_style}", "Agent Card" }

                div {
                    style: format!("font-size: {}; color: {}; margin-bottom: {};",
                        typography::TEXT_XS, colors::TEXT_MUTED, spacing::SM),
                    "Share your agent card link so others can add you as a contact."
                }

                if let Some(ref link) = *agent_card_link.read() {
                    div {
                        style: format!(
                            "font-family: {}; font-size: {}; color: {}; \
                             word-break: break-all; margin-bottom: {};",
                            typography::FONT_MONO,
                            typography::TEXT_XS,
                            colors::TEXT_SECONDARY,
                            spacing::SM,
                        ),
                        "{link}"
                    }
                }

                div {
                    style: "display: flex; gap: 8px;",

                    button {
                        style: "{btn_secondary}",
                        disabled: generating_card(),
                        onclick: move |_| {
                            generating_card.set(true);
                            spawn(async move {
                                let client = X0xClient::new();
                                match client.agent_card(None, Some(true)).await {
                                    Ok(resp) => {
                                        agent_card_link.set(Some(resp.link));
                                    }
                                    Err(e) => {
                                        warn!(target: "ui.settings", "failed to generate card: {e}");
                                    }
                                }
                                generating_card.set(false);
                            });
                        },
                        if generating_card() { "Generating..." } else { "Generate New" }
                    }

                    if agent_card_link.read().is_some() {
                        button {
                            style: if card_copied() {
                                format!("background-color: {}; color: {}; border: none; border-radius: {}; \
                                         padding: {} {}; font-size: {}; font-weight: 500; cursor: pointer;",
                                    colors::SUCCESS, colors::TEXT_INVERSE, radius::MD,
                                    spacing::SM, spacing::MD, typography::TEXT_SM)
                            } else {
                                btn_primary.clone()
                            },
                            onclick: move |_| {
                                if let Some(ref link) = *agent_card_link.read() {
                                    copy_text(link);
                                    card_copied.set(true);
                                    spawn(async move {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                        card_copied.set(false);
                                    });
                                }
                            },
                            if card_copied() { "Copied!" } else { "Copy Link" }
                        }
                    }
                }
            }

            // Daemon info
            div {
                style: "{card_style}",
                div { style: "{section_label_style}", "Daemon Info" }

                div {
                    style: "display: grid; grid-template-columns: auto 1fr; gap: 8px 16px;",

                    span { style: "{detail_style}", "Version" }
                    span { style: "{mono_detail}",
                        {daemon_version.read().clone().unwrap_or_else(|| "-".to_string())}
                    }

                    span { style: "{detail_style}", "Uptime" }
                    span { style: "{detail_style}", "{format_uptime(*daemon_uptime.read())}" }

                    span { style: "{detail_style}", "API Address" }
                    span { style: "{mono_detail}", "{daemon_api_addr}" }
                }
            }

            // Software Updates
            {update_check_section()}

            // About (below update section)
            div {
                style: "{card_style}",
                div { style: "{section_label_style}", "About" }
                div {
                    style: format!("font-size: {}; color: {};", typography::TEXT_SM, colors::TEXT_MUTED),
                    "Communitas -- local-first, decentralized collaboration platform."
                }
                div {
                    style: format!("font-size: {}; color: {}; margin-top: {};",
                        typography::TEXT_XS, colors::TEXT_MUTED, spacing::SM),
                    "Build: {crate::version::CURRENT.version} ({crate::version::CURRENT.commit_hash})"
                }
            }
        }
    }
}

/// GitHub release update check URL.
const UPDATE_CHECK_URL: &str =
    "https://github.com/saorsa-labs/communitas/releases/latest/download/update.json";

/// Update status for the settings view.
#[derive(Clone, PartialEq)]
enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available(String),
    Error(String),
}

/// Software update check section.
fn update_check_section() -> Element {
    let mut status = use_signal(|| UpdateStatus::Idle);

    let card_style = format!(
        "background: {}; border: 1px solid {}; border-radius: {}; padding: {};",
        colors::SURFACE_ELEVATED,
        colors::BORDER_DEFAULT,
        radius::LG,
        spacing::MD,
    );
    let section_label = format!(
        "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
        typography::TEXT_SM,
        colors::TEXT_PRIMARY,
        spacing::SM,
    );

    let current = crate::version::CURRENT.version;
    let status_text = match &*status.read() {
        UpdateStatus::Idle => String::new(),
        UpdateStatus::Checking => "Checking for updates...".to_string(),
        UpdateStatus::UpToDate => format!("You're up to date (v{current})."),
        UpdateStatus::Available(v) => format!("Version {v} is available (you have {current})."),
        UpdateStatus::Error(e) => format!("Check failed: {e}"),
    };
    let status_color = match &*status.read() {
        UpdateStatus::Available(_) => colors::PRIMARY,
        UpdateStatus::Error(_) => colors::DANGER,
        UpdateStatus::UpToDate => colors::SUCCESS,
        _ => colors::TEXT_MUTED,
    };

    rsx! {
        div {
            style: "{card_style}",
            div { style: "{section_label}", "Software Updates" }

            div {
                style: format!("display: flex; align-items: center; gap: {};", spacing::MD),

                div {
                    style: format!("font-size: {}; color: {};", typography::TEXT_SM, colors::TEXT_SECONDARY),
                    "Current version: {current}"
                }

                button {
                    style: format!(
                        "padding: {} {}; background: {}; color: white; border: none; \
                         border-radius: {}; cursor: pointer; font-size: {};",
                        spacing::XS, spacing::MD, colors::PRIMARY, radius::MD, typography::TEXT_XS,
                    ),
                    disabled: matches!(*status.read(), UpdateStatus::Checking),
                    onclick: move |_| {
                        status.set(UpdateStatus::Checking);
                        spawn(async move {
                            match check_for_update().await {
                                Ok(Some(version)) => status.set(UpdateStatus::Available(version)),
                                Ok(None) => status.set(UpdateStatus::UpToDate),
                                Err(e) => status.set(UpdateStatus::Error(e)),
                            }
                        });
                    },
                    if matches!(*status.read(), UpdateStatus::Checking) {
                        "Checking..."
                    } else {
                        "Check for Updates"
                    }
                }
            }

            if !status_text.is_empty() {
                div {
                    style: format!(
                        "margin-top: {}; font-size: {}; color: {status_color};",
                        spacing::SM, typography::TEXT_XS,
                    ),
                    "{status_text}"
                }
            }
        }
    }
}

/// Check GitHub releases for a newer version.
///
/// Returns `Ok(Some(version))` if a newer version is available,
/// `Ok(None)` if up to date, or `Err(message)` on failure.
async fn check_for_update() -> std::result::Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("{e}"))?;

    let resp = client
        .get(UPDATE_CHECK_URL)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {e}"))?;

    let latest = body["version"]
        .as_str()
        .ok_or("Missing version field in update.json")?;

    let current = crate::version::CURRENT.version;
    if crate::version::CURRENT.is_newer_than(latest) || latest == current {
        Ok(None)
    } else {
        Ok(Some(latest.to_string()))
    }
}
