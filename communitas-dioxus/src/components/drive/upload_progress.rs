// SPDX-License-Identifier: MIT OR Apache-2.0

//! Upload progress component with multi-file support.

use communitas_ui_api::{DiskType, UploadProgress as UploadProgressData, UploadState};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Upload progress panel props.
#[derive(Props, Clone, PartialEq)]
pub struct UploadProgressPanelProps {
    /// Entity ID that owns the drive.
    pub entity_id: String,
    /// Current disk type.
    pub disk_type: DiskType,
    /// Close handler.
    pub on_close: EventHandler<()>,
}

/// Upload progress panel showing active uploads.
#[component]
pub fn UploadProgressPanel(props: UploadProgressPanelProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let drive = services.drive();

    // State for uploads
    let mut uploads = use_signal(Vec::<UploadProgressData>::new);

    // Subscribe to upload progress via DriveSnapshot
    use_effect(move || {
        let drive = drive.clone();
        spawn(async move {
            let mut rx = drive.subscribe_uploads();
            while rx.changed().await.is_ok() {
                let snapshot = rx.borrow().clone();
                // Extract uploads from snapshot
                let upload_list: Vec<UploadProgressData> = snapshot.uploads.into_values().collect();
                uploads.set(upload_list);
            }
        });
    });

    // Cancel handler
    let drive_for_cancel = services.drive();
    let on_cancel = move |upload_id: String| {
        let drive = drive_for_cancel.clone();
        spawn(async move {
            let _ = drive.cancel_upload(&upload_id).await;
        });
    };

    // Dismiss handler (removes from list)
    let on_dismiss = move |_upload_id: String| {
        // For now, just let the service handle cleanup
        // In production, this would remove from the UI list
    };

    // Resume handler (resumes a single upload)
    let drive_for_resume = services.drive();
    let on_resume = move |upload_id: String, source_path: String| {
        let drive = drive_for_resume.clone();
        spawn(async move {
            let path = std::path::Path::new(&source_path);
            let _ = drive.resume_upload_by_id(&upload_id, path).await;
        });
    };

    // Resume all handler
    let drive_for_resume_all = services.drive();
    let on_resume_all = move |_| {
        let drive = drive_for_resume_all.clone();
        let current = uploads();
        spawn(async move {
            let source_paths: std::collections::HashMap<String, std::path::PathBuf> = current
                .iter()
                .filter(|u| u.state.is_resumable())
                .map(|u| {
                    // Use file_path as the source (this should be stored in a separate field in production)
                    (u.id.clone(), std::path::PathBuf::from(&u.file_path))
                })
                .collect();
            let _ = drive.resume_all_pending(&source_paths).await;
        });
    };

    let current_uploads = uploads();

    if current_uploads.is_empty() {
        return rsx! {
            div {
                class: "upload-progress fixed bottom-4 right-4 w-80 bg-slate-900 border border-slate-700 rounded-lg shadow-xl overflow-hidden z-50",
                role: "region",
                aria_label: "Upload progress",
                header {
                    class: "flex items-center justify-between p-3 bg-slate-800",
                    h3 {
                        class: "text-sm font-semibold text-slate-200",
                        "No active uploads"
                    }
                    button {
                        class: "p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200",
                        onclick: move |_| props.on_close.call(()),
                        "✕"
                    }
                }
                div {
                    class: "p-4 text-center text-slate-500 text-sm",
                    "Drag files here or click Upload to add files"
                }
            }
        };
    }

    let active_count = current_uploads
        .iter()
        .filter(|u| !u.state.is_terminal() && !matches!(u.state, UploadState::Resumable))
        .count();
    let completed_count = current_uploads
        .iter()
        .filter(|u| u.state.is_terminal())
        .count();
    let resumable_count = current_uploads
        .iter()
        .filter(|u| u.state.is_resumable())
        .count();

    rsx! {
        div {
            class: "upload-progress fixed bottom-4 right-4 w-80 bg-slate-900 border border-slate-700 rounded-lg shadow-xl overflow-hidden z-50",
            role: "region",
            aria_label: "Upload progress",
            // Header
            header {
                class: "flex items-center justify-between p-3 bg-slate-800",
                h3 {
                    class: "text-sm font-semibold text-slate-200",
                    if active_count > 0 {
                        "Uploading {active_count} file(s)..."
                    } else if resumable_count > 0 {
                        "{resumable_count} upload(s) can resume"
                    } else {
                        "Uploads complete"
                    }
                }
                div {
                    class: "flex items-center gap-2",
                    if resumable_count > 0 {
                        button {
                            class: "text-xs px-2 py-1 rounded bg-amber-500/20 text-amber-400 hover:bg-amber-500/30",
                            onclick: on_resume_all.clone(),
                            "Resume All"
                        }
                    }
                    if completed_count > 0 {
                        button {
                            class: "text-xs text-slate-400 hover:text-slate-200",
                            onclick: move |_| {
                                // Clear all completed - handled by dismiss
                            },
                            "Clear"
                        }
                    }
                    button {
                        class: "p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200",
                        onclick: move |_| props.on_close.call(()),
                        "✕"
                    }
                }
            }
            // Upload list
            ul {
                class: "max-h-64 overflow-auto divide-y divide-slate-800",
                {current_uploads.iter().map(|upload| {
                    let upload_id = upload.id.clone();
                    let upload_id_cancel = upload_id.clone();
                    let upload_id_dismiss = upload_id.clone();
                    let upload_id_resume = upload_id.clone();
                    let source_path_resume = upload.file_path.clone();
                    let on_cancel = on_cancel.clone();
                    let on_resume = on_resume.clone();

                    rsx! {
                        li {
                            key: "{upload_id}",
                            class: "p-3",
                            UploadItem {
                                upload: upload.clone(),
                                on_cancel: move |_| on_cancel(upload_id_cancel.clone()),
                                on_dismiss: move |_| on_dismiss(upload_id_dismiss.clone()),
                                on_resume: move |_| on_resume(upload_id_resume.clone(), source_path_resume.clone()),
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Single upload item.
#[derive(Props, Clone, PartialEq)]
struct UploadItemProps {
    upload: UploadProgressData,
    on_cancel: EventHandler<()>,
    on_dismiss: EventHandler<()>,
    on_resume: EventHandler<()>,
}

#[component]
fn UploadItem(props: UploadItemProps) -> Element {
    let percent = props.upload.percent_complete();
    let is_terminal = props.upload.state.is_terminal();
    let is_resumable = props.upload.state.is_resumable();

    // Show resume info if this upload was resumed from a previous session
    let resume_info = props
        .upload
        .resumed_from_bytes
        .map(|bytes| format!("Resumed from {}", format_bytes(bytes)));

    rsx! {
        div {
            class: "space-y-2",
            // File name and status
            div {
                class: "flex items-center justify-between gap-2",
                span {
                    class: "text-sm text-slate-200 truncate flex-1",
                    title: "{props.upload.file_name}",
                    "{props.upload.file_name}"
                }
                match &props.upload.state {
                    UploadState::Pending => rsx! {
                        span { class: "text-xs text-slate-500", "Pending" }
                    },
                    UploadState::Uploading => rsx! {
                        span { class: "text-xs text-emerald-400", "{percent:.0}%" }
                    },
                    UploadState::Verifying => rsx! {
                        span { class: "text-xs text-amber-400", "Verifying..." }
                    },
                    UploadState::Complete => rsx! {
                        span { class: "text-xs text-emerald-400", "✓" }
                    },
                    UploadState::Failed(err) => rsx! {
                        span {
                            class: "text-xs text-red-400",
                            title: "{err}",
                            "Failed"
                        }
                    },
                    UploadState::Cancelled => rsx! {
                        span { class: "text-xs text-slate-500", "Cancelled" }
                    },
                    UploadState::Resumable => rsx! {
                        span { class: "text-xs text-amber-400", "Resumable" }
                    },
                }
            }
            // Resume info if applicable
            if let Some(info) = resume_info.as_ref() {
                div {
                    class: "text-xs text-slate-500",
                    "{info}"
                }
            }
            // Progress bar - show for active uploads and resumable (to show resume point)
            if !is_terminal || is_resumable {
                div {
                    class: "h-1.5 bg-slate-800 rounded-full overflow-hidden",
                    div {
                        class: if is_resumable { "h-full bg-amber-500 transition-all duration-300" } else { "h-full bg-emerald-500 transition-all duration-300" },
                        style: "width: {percent}%",
                    }
                }
            }
            // Actions
            div {
                class: "flex justify-end gap-2",
                if is_resumable {
                    button {
                        class: "text-xs px-2 py-0.5 rounded bg-amber-500/20 text-amber-400 hover:bg-amber-500/30",
                        onclick: move |_| props.on_resume.call(()),
                        "Resume"
                    }
                    button {
                        class: "text-xs text-slate-400 hover:text-slate-200",
                        onclick: move |_| props.on_dismiss.call(()),
                        "Dismiss"
                    }
                } else if !is_terminal {
                    button {
                        class: "text-xs text-slate-400 hover:text-red-400",
                        onclick: move |_| props.on_cancel.call(()),
                        "Cancel"
                    }
                } else {
                    button {
                        class: "text-xs text-slate-400 hover:text-slate-200",
                        onclick: move |_| props.on_dismiss.call(()),
                        "Dismiss"
                    }
                }
            }
        }
    }
}

/// Formats bytes into a human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Compact upload indicator for toolbar.
#[derive(Props, Clone, PartialEq)]
pub struct UploadIndicatorProps {
    /// Number of active uploads.
    pub active_count: usize,
    /// Click handler to show full panel.
    pub on_click: EventHandler<()>,
}

#[component]
pub fn UploadIndicator(props: UploadIndicatorProps) -> Element {
    if props.active_count == 0 {
        return rsx! {};
    }

    rsx! {
        button {
            class: "flex items-center gap-2 px-3 py-1.5 rounded-lg bg-emerald-500/20 text-emerald-400 text-sm hover:bg-emerald-500/30 transition",
            onclick: move |_| props.on_click.call(()),
            // Spinner
            svg {
                class: "w-4 h-4 animate-spin",
                view_box: "0 0 24 24",
                fill: "none",
                circle {
                    class: "opacity-25",
                    cx: "12",
                    cy: "12",
                    r: "10",
                    stroke: "currentColor",
                    stroke_width: "4",
                }
                path {
                    class: "opacity-75",
                    fill: "currentColor",
                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                }
            }
            span { "Uploading {props.active_count}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_state_is_terminal() {
        assert!(!UploadState::Pending.is_terminal());
        assert!(!UploadState::Uploading.is_terminal());
        assert!(UploadState::Complete.is_terminal());
        assert!(UploadState::Failed("error".to_string()).is_terminal());
        assert!(UploadState::Cancelled.is_terminal());
        // Resumable is NOT terminal - it can transition to Uploading
        assert!(!UploadState::Resumable.is_terminal());
    }

    #[test]
    fn upload_state_is_resumable() {
        assert!(!UploadState::Pending.is_resumable());
        assert!(!UploadState::Uploading.is_resumable());
        assert!(!UploadState::Complete.is_resumable());
        assert!(UploadState::Failed("error".to_string()).is_resumable());
        assert!(!UploadState::Cancelled.is_resumable());
        assert!(UploadState::Resumable.is_resumable());
    }

    #[test]
    fn upload_progress_percent_complete() {
        let progress = UploadProgressData {
            id: "test".to_string(),
            file_name: "file.txt".to_string(),
            file_path: "/file.txt".to_string(),
            bytes_uploaded: 50,
            total_bytes: 100,
            state: UploadState::Uploading,
            started_at: 0,
            checksum_verified: false,
            transfer_id: None,
            resumed_from_bytes: None,
        };
        assert_eq!(progress.percent_complete(), 50);
    }

    #[test]
    fn format_bytes_helper() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }
}
