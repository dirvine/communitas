// SPDX-License-Identifier: MIT OR Apache-2.0

//! Download manager component with multi-file support and checksum verification.

use communitas_ui_api::{DownloadProgress as DownloadProgressData, DownloadState};
use dioxus::prelude::*;

/// Download manager props.
#[derive(Props, Clone, PartialEq)]
pub struct DownloadManagerProps {
    /// Active downloads.
    pub downloads: Vec<DownloadProgressData>,
    /// Cancel handler.
    pub on_cancel: EventHandler<String>,
    /// Dismiss completed handler.
    pub on_dismiss: EventHandler<String>,
    /// Retry failed handler.
    pub on_retry: EventHandler<String>,
}

/// Download manager panel showing active downloads.
#[component]
pub fn DownloadManager(props: DownloadManagerProps) -> Element {
    if props.downloads.is_empty() {
        return rsx! {};
    }

    let active_count = props
        .downloads
        .iter()
        .filter(|d| !d.state.is_terminal())
        .count();
    let completed_count = props
        .downloads
        .iter()
        .filter(|d| d.state.is_terminal())
        .count();

    rsx! {
        div {
            class: "download-manager fixed bottom-4 left-4 w-80 bg-slate-900 border border-slate-700 rounded-lg shadow-xl overflow-hidden z-50",
            role: "region",
            aria_label: "Download progress",
            // Header
            header {
                class: "flex items-center justify-between p-3 bg-slate-800",
                h3 {
                    class: "text-sm font-semibold text-slate-200",
                    if active_count > 0 {
                        "Downloading {active_count} file(s)..."
                    } else {
                        "Downloads complete"
                    }
                }
                if completed_count > 0 {
                    button {
                        class: "text-xs text-slate-400 hover:text-slate-200",
                        onclick: {
                            let downloads_for_clear = props.downloads.clone();
                            let on_dismiss = props.on_dismiss;
                            move |_| {
                                for download in &downloads_for_clear {
                                    if download.state.is_terminal() {
                                        on_dismiss.call(download.id.clone());
                                    }
                                }
                            }
                        },
                        "Clear completed"
                    }
                }
            }
            // Download list
            ul {
                class: "max-h-64 overflow-auto divide-y divide-slate-800",
                {props.downloads.iter().map(|download| {
                    let download_id = download.id.clone();
                    let download_id_cancel = download_id.clone();
                    let download_id_dismiss = download_id.clone();
                    let download_id_retry = download_id.clone();

                    rsx! {
                        li {
                            key: "{download_id}",
                            class: "p-3",
                            DownloadItem {
                                download: download.clone(),
                                on_cancel: move |_| props.on_cancel.call(download_id_cancel.clone()),
                                on_dismiss: move |_| props.on_dismiss.call(download_id_dismiss.clone()),
                                on_retry: move |_| props.on_retry.call(download_id_retry.clone()),
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Single download item.
#[derive(Props, Clone, PartialEq)]
struct DownloadItemProps {
    download: DownloadProgressData,
    on_cancel: EventHandler<()>,
    on_dismiss: EventHandler<()>,
    on_retry: EventHandler<()>,
}

#[component]
fn DownloadItem(props: DownloadItemProps) -> Element {
    let percent = props.download.percent_complete();
    let is_terminal = props.download.state.is_terminal();

    rsx! {
        div {
            class: "space-y-2",
            // File name and status
            div {
                class: "flex items-center justify-between gap-2",
                span {
                    class: "text-sm text-slate-200 truncate flex-1",
                    title: "{props.download.file_name}",
                    "{props.download.file_name}"
                }
                match &props.download.state {
                    DownloadState::Pending => rsx! {
                        span { class: "text-xs text-slate-500", "Pending" }
                    },
                    DownloadState::Downloading => rsx! {
                        span { class: "text-xs text-blue-400", "{percent:.0}%" }
                    },
                    DownloadState::Verifying => rsx! {
                        span { class: "text-xs text-amber-400", "Verifying..." }
                    },
                    DownloadState::Complete => rsx! {
                        span { class: "text-xs text-emerald-400", "✓ Verified" }
                    },
                    DownloadState::Failed(err) => rsx! {
                        span {
                            class: "text-xs text-red-400",
                            title: "{err}",
                            "Failed"
                        }
                    },
                    DownloadState::Cancelled => rsx! {
                        span { class: "text-xs text-slate-500", "Cancelled" }
                    },
                }
            }
            // Progress bar
            if !is_terminal {
                div {
                    class: "h-1.5 bg-slate-800 rounded-full overflow-hidden",
                    div {
                        class: format!(
                            "h-full transition-all duration-300 {}",
                            match props.download.state {
                                DownloadState::Verifying => "bg-amber-500",
                                _ => "bg-blue-500",
                            }
                        ),
                        style: "width: {percent}%",
                    }
                }
            }
            // Actions
            div {
                class: "flex justify-end gap-2",
                match &props.download.state {
                    DownloadState::Pending | DownloadState::Downloading | DownloadState::Verifying => rsx! {
                        button {
                            class: "text-xs text-slate-400 hover:text-red-400",
                            onclick: move |_| props.on_cancel.call(()),
                            "Cancel"
                        }
                    },
                    DownloadState::Failed(_) => rsx! {
                        button {
                            class: "text-xs text-slate-400 hover:text-blue-400",
                            onclick: move |_| props.on_retry.call(()),
                            "Retry"
                        }
                        button {
                            class: "text-xs text-slate-400 hover:text-slate-200",
                            onclick: move |_| props.on_dismiss.call(()),
                            "Dismiss"
                        }
                    },
                    _ => rsx! {
                        button {
                            class: "text-xs text-slate-400 hover:text-slate-200",
                            onclick: move |_| props.on_dismiss.call(()),
                            "Dismiss"
                        }
                    },
                }
            }
        }
    }
}

/// Compact download indicator for toolbar.
#[derive(Props, Clone, PartialEq)]
pub struct DownloadIndicatorProps {
    /// Number of active downloads.
    pub active_count: usize,
    /// Click handler to show full panel.
    pub on_click: EventHandler<()>,
}

#[component]
pub fn DownloadIndicator(props: DownloadIndicatorProps) -> Element {
    if props.active_count == 0 {
        return rsx! {};
    }

    rsx! {
        button {
            class: "flex items-center gap-2 px-3 py-1.5 rounded-lg bg-blue-500/20 text-blue-400 text-sm hover:bg-blue-500/30 transition",
            onclick: move |_| props.on_click.call(()),
            // Download arrow
            svg {
                class: "w-4 h-4 animate-bounce",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                path {
                    d: "M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3",
                }
            }
            span { "Downloading {props.active_count}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_state_is_terminal() {
        assert!(!DownloadState::Pending.is_terminal());
        assert!(!DownloadState::Downloading.is_terminal());
        assert!(!DownloadState::Verifying.is_terminal());
        assert!(DownloadState::Complete.is_terminal());
        assert!(DownloadState::Failed("error".to_string()).is_terminal());
        assert!(DownloadState::Cancelled.is_terminal());
    }

    #[test]
    fn download_progress_percent_complete() {
        let progress = DownloadProgressData {
            id: "test".to_string(),
            file_name: "file.txt".to_string(),
            destination_path: "/tmp/file.txt".to_string(),
            total_bytes: 100,
            bytes_downloaded: 75,
            state: DownloadState::Downloading,
            checksum_verified: false,
        };
        assert_eq!(progress.percent_complete(), 75);
    }
}
