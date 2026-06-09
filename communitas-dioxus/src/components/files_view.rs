// SPDX-License-Identifier: MIT OR Apache-2.0

//! File sharing view -- drop zone, send/receive, transfer list.

use communitas_x0x_client::{FileTransfer, TransferDirection, TransferStatus, X0xClient};
use dioxus::prelude::*;
use sha2::{Digest, Sha256};
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

fn same_transfers(left: &[FileTransfer], right: &[FileTransfer]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.transfer_id == right.transfer_id
                && left.direction == right.direction
                && left.remote_agent_id == right.remote_agent_id
                && left.filename == right.filename
                && left.total_size == right.total_size
                && left.bytes_transferred == right.bytes_transferred
                && left.status == right.status
                && left.sha256 == right.sha256
                && left.error == right.error
                && left.started_at == right.started_at
        })
}

async fn refresh_transfers(
    mut transfers: Signal<Vec<FileTransfer>>,
    mut error: Signal<Option<String>>,
    mut loading: Signal<bool>,
) {
    let client = X0xClient::new();
    match client.transfers().await {
        Ok(list) => {
            transfers.set(list);
            if error.peek().is_some() {
                error.set(None);
            }
        }
        Err(e) => {
            warn!(target: "ui.files", "failed to list transfers: {e}");
            error.set(Some(format!("{e}")));
        }
    }
    if *loading.peek() {
        loading.set(false);
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
            let _key = *refresh_key.peek();
            match client.transfers().await {
                Ok(list) => {
                    if !same_transfers(transfers.peek().as_slice(), list.as_slice()) {
                        transfers.set(list);
                    }
                    if error.peek().is_some() {
                        error.set(None);
                    }
                }
                Err(e) => {
                    warn!(target: "ui.files", "failed to list transfers: {e}");
                    let next_error = Some(format!("{e}"));
                    if error.peek().as_ref() != next_error.as_ref() {
                        error.set(next_error);
                    }
                }
            }
            if *loading.peek() {
                loading.set(false);
            }

            crate::poll_sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
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

            // Send file form
            {send_file_form(transfers, error, loading, refresh_key)}

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
                                                    refresh_transfers(transfers, error, loading).await;
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
                                                    refresh_transfers(transfers, error, loading).await;
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

/// Send file form — agent ID + file path inputs.
fn send_file_form(
    transfers: Signal<Vec<FileTransfer>>,
    error: Signal<Option<String>>,
    loading: Signal<bool>,
    mut refresh_key: Signal<u64>,
) -> Element {
    let mut agent_id_input = use_signal(String::new);
    let mut file_path_input = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut send_error = use_signal(|| None::<String>);
    let mut send_success = use_signal(|| None::<String>);

    let card_style = format!(
        "background-color: {}; border: 1px solid {}; border-radius: {}; padding: {};",
        colors::SURFACE_ELEVATED,
        colors::BORDER_DEFAULT,
        radius::LG,
        spacing::MD,
    );

    let input_style = format!(
        "flex: 1; padding: {} {}; border: 1px solid {}; border-radius: {}; \
         background: {}; color: {}; font-size: {};",
        spacing::SM,
        spacing::MD,
        colors::BORDER_DEFAULT,
        radius::MD,
        colors::SURFACE_CARD,
        colors::TEXT_PRIMARY,
        typography::TEXT_SM,
    );

    rsx! {
        div {
            style: "{card_style}",
            div {
                style: format!(
                    "font-size: {}; font-weight: 600; color: {}; margin-bottom: {};",
                    typography::TEXT_SM, colors::TEXT_PRIMARY, spacing::SM,
                ),
                "Send File"
            }

            div {
                style: format!("display: flex; flex-direction: column; gap: {};", spacing::SM),

                input {
                    style: "{input_style}",
                    r#type: "text",
                    placeholder: "Recipient agent ID (hex)",
                    value: "{agent_id_input}",
                    oninput: move |evt: FormEvent| agent_id_input.set(evt.value()),
                }
                input {
                    style: "{input_style}",
                    r#type: "text",
                    placeholder: "File path (e.g. /Users/me/file.pdf)",
                    value: "{file_path_input}",
                    oninput: move |evt: FormEvent| file_path_input.set(evt.value()),
                }

                if let Some(ref err) = *send_error.read() {
                    div {
                        style: format!("color: {}; font-size: {};", colors::DANGER, typography::TEXT_XS),
                        "{err}"
                    }
                }
                if let Some(ref msg) = *send_success.read() {
                    div {
                        style: format!("color: {}; font-size: {};", colors::SUCCESS, typography::TEXT_XS),
                        "{msg}"
                    }
                }

                button {
                    style: format!(
                        "padding: {} {}; background: {}; color: white; border: none; \
                         border-radius: {}; cursor: pointer; font-size: {}; align-self: flex-start;",
                        spacing::SM, spacing::LG, colors::PRIMARY, radius::MD, typography::TEXT_SM,
                    ),
                    disabled: sending(),
                    onclick: move |_| {
                        let agent_id = agent_id_input().trim().to_string();
                        let file_path = file_path_input().trim().to_string();
                        if agent_id.is_empty() || file_path.is_empty() {
                            send_error.set(Some("Both agent ID and file path are required".into()));
                            return;
                        }
                        sending.set(true);
                        send_error.set(None);
                        send_success.set(None);

                        spawn(async move {
                            let path = std::path::Path::new(&file_path);
                            let canonical_path = match tokio::fs::canonicalize(path).await {
                                Ok(path) => path,
                                Err(e) => {
                                    send_error.set(Some(format!("Cannot read file: {e}")));
                                    sending.set(false);
                                    return;
                                }
                            };
                            let source_path = match canonical_path.to_str() {
                                Some(path) => path.to_owned(),
                                None => {
                                    send_error.set(Some("File path must be valid UTF-8".into()));
                                    sending.set(false);
                                    return;
                                }
                            };
                            let meta = match tokio::fs::metadata(&canonical_path).await {
                                Ok(m) => m,
                                Err(e) => {
                                    send_error.set(Some(format!("Cannot read file: {e}")));
                                    sending.set(false);
                                    return;
                                }
                            };
                            let size = meta.len();
                            let filename = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "file".to_string());

                            // Compute SHA-256
                            let data = match tokio::fs::read(&canonical_path).await {
                                Ok(d) => d,
                                Err(e) => {
                                    send_error.set(Some(format!("Failed to read file: {e}")));
                                    sending.set(false);
                                    return;
                                }
                            };
                            let hash = Sha256::digest(&data);
                            let sha256 = format!("{hash:x}");

                            let client = X0xClient::new();
                            match client
                                .send_file(&agent_id, &filename, size, &sha256, Some(&source_path))
                                .await
                            {
                                Ok(transfer_id) => {
                                    let short = if transfer_id.len() > 8 {
                                        &transfer_id[..8]
                                    } else {
                                        &transfer_id
                                    };
                                    send_success.set(Some(format!(
                                        "Transfer started: {short}"
                                    )));
                                    agent_id_input.set(String::new());
                                    file_path_input.set(String::new());
                                    refresh_transfers(transfers, error, loading).await;
                                    refresh_key.set(refresh_key() + 1);
                                }
                                Err(e) => send_error.set(Some(format!("Send failed: {e}"))),
                            }
                            sending.set(false);
                        });
                    },
                    if sending() { "Sending..." } else { "Send File" }
                }
            }
        }
    }
}
