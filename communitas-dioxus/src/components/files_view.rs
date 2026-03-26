//! File sharing view -- drop zone, send/receive, transfer list.

use communitas_x0x_client::{FileTransfer, TransferDirection, TransferStatus, X0xClient};
use dioxus::prelude::*;
use tracing::warn;

use crate::tokens::{colors, radius, spacing, typography};

/// How often to poll file transfers.
const POLL_INTERVAL_SECS: u64 = 5;

/// Truncate an ID.
fn short_id(id: &str) -> String {
    if id.len() <= 16 {
        id.to_owned()
    } else {
        format!("{}...{}", &id[..8], &id[id.len() - 6..])
    }
}

/// Props for the files view.
#[derive(Props, Clone, PartialEq)]
pub struct FilesViewProps {
    /// The space (group) ID context.
    pub space_id: String,
}

/// Files tab component.
#[component]
pub fn FilesView(props: FilesViewProps) -> Element {
    let _space_id = &props.space_id;
    let mut transfers = use_signal(Vec::<FileTransfer>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut refresh_key = use_signal(|| 0u64);

    // Poll transfers
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();

        loop {
            let _key = *refresh_key.read();
            match client.transfers().await {
                Ok(list) => {
                    transfers.set(list);
                    error.set(None);
                }
                Err(e) => {
                    warn!(target: "ui.files", "failed to list transfers: {e}");
                    error.set(Some(format!("{e}")));
                }
            }
            loading.set(false);

            tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    });

    let current_transfers = transfers.read().clone();
    let is_loading = *loading.read();
    let current_error = error.read().clone();

    let page_style = format!(
        "padding: {}; display: flex; flex-direction: column; gap: {}; \
         overflow-y: auto; height: 100%;",
        spacing::LG,
        spacing::LG,
    );

    let card_style = format!(
        "background-color: {}; border: 1px solid {}; border-radius: {}; padding: {};",
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

    // Incoming pending transfers
    let incoming_pending: Vec<&FileTransfer> = current_transfers
        .iter()
        .filter(|t| {
            matches!(t.direction, TransferDirection::Receiving)
                && matches!(t.status, TransferStatus::Pending)
        })
        .collect();

    rsx! {
        div {
            style: "{page_style}",

            // Error
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

            // Drop zone placeholder
            div {
                style: format!(
                    "border: 2px dashed {}; border-radius: {}; padding: {}; \
                     text-align: center; color: {};",
                    colors::BORDER_DEFAULT,
                    radius::LG,
                    spacing::LG,
                    colors::TEXT_MUTED,
                ),
                div {
                    style: format!("font-size: {}; margin-bottom: {};", typography::TEXT_SM, spacing::SM),
                    "File sending is available via the x0x CLI"
                }
                div {
                    style: format!("font-size: {};", typography::TEXT_XS),
                    "Use: x0x send-file <agent-id> <file-path>"
                }
            }

            // Incoming files requiring action
            if !incoming_pending.is_empty() {
                div {
                    style: "{card_style}",
                    div { style: "{section_label}", "Incoming Files" }

                    for transfer in &incoming_pending {
                        {
                            let tid = transfer.transfer_id.clone();
                            let filename = transfer.filename.clone();
                            let sender = transfer.remote_agent_id.clone();
                            let size_kb = transfer.total_size / 1024;

                            rsx! {
                                div {
                                    key: "{tid}",
                                    style: format!(
                                        "display: flex; align-items: center; gap: {}; \
                                         padding: {}; border-bottom: 1px solid {};",
                                        spacing::MD,
                                        spacing::SM,
                                        colors::BORDER_DEFAULT,
                                    ),

                                    div {
                                        style: "flex: 1; min-width: 0;",
                                        div {
                                            style: format!(
                                                "font-size: {}; color: {}; font-weight: 500;",
                                                typography::TEXT_SM, colors::TEXT_PRIMARY,
                                            ),
                                            "{filename}"
                                        }
                                        div {
                                            style: format!(
                                                "font-size: {}; color: {};",
                                                typography::TEXT_XS, colors::TEXT_MUTED,
                                            ),
                                            "From: {short_id(&sender)} -- {size_kb} KB"
                                        }
                                    }

                                    button {
                                        style: format!(
                                            "background-color: {}; color: {}; border: none; \
                                             border-radius: {}; padding: 4px {}; font-size: {}; cursor: pointer;",
                                            colors::SUCCESS, colors::TEXT_INVERSE,
                                            radius::SM, spacing::SM, typography::TEXT_XS,
                                        ),
                                        onclick: {
                                            let tid = tid.clone();
                                            move |_| {
                                                let tid = tid.clone();
                                                spawn(async move {
                                                    let client = X0xClient::new();
                                                    if let Err(e) = client.accept_file(&tid).await {
                                                        warn!(target: "ui.files", "failed to accept: {e}");
                                                    }
                                                    refresh_key.set(refresh_key() + 1);
                                                });
                                            }
                                        },
                                        "Accept"
                                    }

                                    button {
                                        style: format!(
                                            "background-color: transparent; color: {}; border: 1px solid {}; \
                                             border-radius: {}; padding: 4px {}; font-size: {}; cursor: pointer;",
                                            colors::DANGER, colors::DANGER,
                                            radius::SM, spacing::SM, typography::TEXT_XS,
                                        ),
                                        onclick: {
                                            let tid = tid.clone();
                                            move |_| {
                                                let tid = tid.clone();
                                                spawn(async move {
                                                    let client = X0xClient::new();
                                                    if let Err(e) = client.reject_file(&tid, None).await {
                                                        warn!(target: "ui.files", "failed to reject: {e}");
                                                    }
                                                    refresh_key.set(refresh_key() + 1);
                                                });
                                            }
                                        },
                                        "Reject"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // All transfers table
            div {
                style: "{card_style}",
                div { style: "{section_label}", "Transfers" }

                if is_loading {
                    div {
                        style: format!("color: {};", colors::TEXT_MUTED),
                        "Loading transfers..."
                    }
                } else if current_transfers.is_empty() {
                    div {
                        style: format!("color: {}; font-size: {};", colors::TEXT_MUTED, typography::TEXT_SM),
                        "No file transfers."
                    }
                } else {
                    // Table header
                    div {
                        style: format!(
                            "display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr; {}",
                            table_header_style,
                        ),
                        span { "File" }
                        span { "Direction" }
                        span { "Peer" }
                        span { "Status" }
                        span { "Progress" }
                    }

                    for transfer in &current_transfers {
                        {
                            let tid = transfer.transfer_id.clone();
                            let filename = transfer.filename.clone();
                            let direction = match transfer.direction {
                                TransferDirection::Sending => "Sending",
                                TransferDirection::Receiving => "Receiving",
                            };
                            let peer = short_id(&transfer.remote_agent_id);
                            let status_label = match transfer.status {
                                TransferStatus::Pending => "Pending",
                                TransferStatus::InProgress => "In Progress",
                                TransferStatus::Complete => "Complete",
                                TransferStatus::Failed => "Failed",
                                TransferStatus::Rejected => "Rejected",
                            };
                            let status_color = match transfer.status {
                                TransferStatus::Complete => colors::SUCCESS,
                                TransferStatus::Failed | TransferStatus::Rejected => colors::DANGER,
                                TransferStatus::InProgress => colors::PRIMARY,
                                TransferStatus::Pending => colors::WARNING,
                            };
                            let progress = if transfer.total_size > 0 {
                                format!(
                                    "{:.0}%",
                                    (transfer.bytes_transferred as f64 / transfer.total_size as f64) * 100.0
                                )
                            } else {
                                "-".to_string()
                            };

                            rsx! {
                                div {
                                    key: "{tid}",
                                    style: format!(
                                        "display: grid; grid-template-columns: 2fr 1fr 1fr 1fr 1fr; \
                                         align-items: center; {}",
                                        table_row_style,
                                    ),
                                    span {
                                        style: format!(
                                            "font-size: {}; color: {}; overflow: hidden; \
                                             text-overflow: ellipsis; white-space: nowrap;",
                                            typography::TEXT_SM, colors::TEXT_PRIMARY,
                                        ),
                                        "{filename}"
                                    }
                                    span {
                                        style: format!("font-size: {}; color: {};",
                                            typography::TEXT_XS, colors::TEXT_SECONDARY),
                                        "{direction}"
                                    }
                                    span {
                                        style: format!(
                                            "font-family: {}; font-size: {}; color: {};",
                                            typography::FONT_MONO, typography::TEXT_XS, colors::TEXT_MUTED,
                                        ),
                                        "{peer}"
                                    }
                                    span {
                                        style: format!("font-size: {}; color: {};",
                                            typography::TEXT_XS, status_color),
                                        "{status_label}"
                                    }
                                    span {
                                        style: format!("font-size: {}; color: {};",
                                            typography::TEXT_XS, colors::TEXT_SECONDARY),
                                        "{progress}"
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
