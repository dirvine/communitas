//! Preview panel for showing file details and content preview.

use base64::Engine;
use communitas_ui_api::{DirectoryEntry, DiskType, FilePreview};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Preview panel props.
#[derive(Props, Clone, PartialEq)]
pub struct PreviewPanelProps {
    /// Selected entry to preview.
    pub entry: Option<DirectoryEntry>,
    /// Number of selected entries (for multi-select info).
    #[props(default = 0)]
    pub selected_count: usize,
    /// Entity ID that owns the drive.
    pub entity_id: String,
    /// Current disk type.
    pub disk_type: DiskType,
    /// Close handler.
    pub on_close: EventHandler<()>,
    /// Download handler (path).
    pub on_download: EventHandler<String>,
    /// Delete handler (path).
    pub on_delete: EventHandler<String>,
}

/// Preview panel showing file details and content preview.
#[component]
pub fn PreviewPanel(props: PreviewPanelProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let drive = services.drive();

    // State for async preview loading
    let mut preview = use_signal(|| Option::<FilePreview>::None);
    let mut is_loading = use_signal(|| false);

    // Load preview when entry changes
    let entity_id = props.entity_id.clone();
    let disk_type = props.disk_type;
    let entry_path = props.entry.as_ref().map(|e| e.path.clone());

    use_effect(move || {
        if let Some(path) = entry_path.clone() {
            let drive = drive.clone();
            let entity_id = entity_id.clone();
            spawn(async move {
                is_loading.set(true);
                if let Ok(p) = drive.get_file_preview(&entity_id, disk_type, &path).await {
                    preview.set(Some(p));
                }
                is_loading.set(false);
            });
        } else {
            preview.set(None);
        }
    });

    // Handle multi-select
    if props.selected_count > 1 {
        return rsx! {
            aside {
                class: "preview-panel flex flex-col bg-slate-900/50",
                role: "complementary",
                aria_label: "Selection info",
                header {
                    class: "flex items-center justify-between p-4 border-b border-slate-800",
                    h2 {
                        class: "text-sm font-semibold text-slate-200",
                        "{props.selected_count} items selected"
                    }
                    button {
                        class: "p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-slate-200",
                        onclick: move |_| props.on_close.call(()),
                        "✕"
                    }
                }
                div {
                    class: "flex-1 flex items-center justify-center text-slate-500",
                    "Select a single item to preview"
                }
            }
        };
    }

    let Some(entry) = props.entry.as_ref() else {
        return rsx! {
            EmptyPreview {}
        };
    };

    let entry_path_for_download = entry.path.clone();
    let entry_path_for_delete = entry.path.clone();

    rsx! {
        aside {
            class: "preview-panel flex flex-col bg-slate-900/50",
            role: "complementary",
            aria_label: "File preview",
            // Header
            header {
                class: "flex items-center justify-between p-4 border-b border-slate-800",
                h2 {
                    class: "text-sm font-semibold text-slate-200 truncate",
                    "{entry.name}"
                }
                button {
                    class: "p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-slate-200",
                    onclick: move |_| props.on_close.call(()),
                    "✕"
                }
            }
            // Preview content
            div {
                class: "flex-1 overflow-auto p-4",
                if is_loading() {
                    PreviewSkeleton {}
                } else if let Some(p) = preview() {
                    PreviewContent { entry: entry.clone(), preview: p }
                } else {
                    FileInfo { entry: entry.clone() }
                }
            }
            // Actions footer
            footer {
                class: "p-4 border-t border-slate-800 flex gap-2",
                button {
                    class: "flex-1 px-3 py-2 rounded-lg bg-emerald-500 text-slate-900 font-semibold hover:bg-emerald-400 transition text-sm",
                    onclick: move |_| props.on_download.call(entry_path_for_download.clone()),
                    "Download"
                }
                button {
                    class: "px-3 py-2 rounded-lg border border-slate-700 text-slate-300 hover:border-red-500 hover:text-red-400 transition text-sm",
                    onclick: move |_| props.on_delete.call(entry_path_for_delete.clone()),
                    "Delete"
                }
            }
        }
    }
}

/// Empty preview state.
#[component]
fn EmptyPreview() -> Element {
    rsx! {
        aside {
            class: "preview-panel w-80 border-l border-slate-800 bg-slate-900/50 flex flex-col items-center justify-center text-center p-8",
            role: "complementary",
            aria_label: "File preview",
            div { class: "text-4xl mb-4 opacity-50", "📄" }
            p { class: "text-sm text-slate-500", "Select a file to preview" }
        }
    }
}

/// Preview loading skeleton.
#[component]
fn PreviewSkeleton() -> Element {
    rsx! {
        div {
            class: "animate-pulse space-y-4",
            // Thumbnail placeholder
            div { class: "aspect-video bg-slate-800 rounded-lg" }
            // Info placeholders
            div { class: "h-4 bg-slate-800 rounded w-3/4" }
            div { class: "h-4 bg-slate-800 rounded w-1/2" }
            div { class: "h-4 bg-slate-800 rounded w-2/3" }
        }
    }
}

/// Preview content based on file type.
#[derive(Props, Clone, PartialEq)]
struct PreviewContentProps {
    entry: DirectoryEntry,
    preview: FilePreview,
}

#[component]
fn PreviewContent(props: PreviewContentProps) -> Element {
    // Convert thumbnail bytes to data URI if available
    let thumbnail_data_uri = props.preview.thumbnail.as_ref().map(|bytes| {
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        format!("data:{};base64,{}", props.preview.mime_type, encoded)
    });

    rsx! {
        div {
            class: "space-y-4",
            // Thumbnail if available
            if let Some(data_uri) = &thumbnail_data_uri {
                div {
                    class: "aspect-video bg-slate-800 rounded-lg overflow-hidden",
                    img {
                        src: "{data_uri}",
                        alt: "Preview thumbnail",
                        class: "w-full h-full object-contain",
                    }
                }
            } else {
                // Icon placeholder
                div {
                    class: "aspect-video bg-slate-800 rounded-lg flex items-center justify-center",
                    span {
                        class: "text-6xl opacity-50",
                        {file_type_icon(&props.entry)}
                    }
                }
            }
            // File info
            FileInfo { entry: props.entry.clone() }
            // Text preview if available
            if let Some(text_preview) = &props.preview.text_preview {
                div {
                    class: "p-3 bg-slate-800 rounded-lg text-xs text-slate-300 font-mono whitespace-pre-wrap max-h-48 overflow-auto",
                    "{text_preview}"
                }
            }
        }
    }
}

/// File info section.
#[derive(Props, Clone, PartialEq)]
struct FileInfoProps {
    entry: DirectoryEntry,
}

#[component]
fn FileInfo(props: FileInfoProps) -> Element {
    rsx! {
        dl {
            class: "space-y-2 text-sm",
            // Type
            div {
                class: "flex justify-between",
                dt { class: "text-slate-500", "Type" }
                dd {
                    class: "text-slate-300",
                    {if props.entry.is_directory { "Folder" } else { props.entry.mime_type.as_deref().unwrap_or("Unknown") }}
                }
            }
            // Size
            if !props.entry.is_directory {
                div {
                    class: "flex justify-between",
                    dt { class: "text-slate-500", "Size" }
                    dd { class: "text-slate-300", {format_size(props.entry.size_bytes)} }
                }
            }
            // Modified
            div {
                class: "flex justify-between",
                dt { class: "text-slate-500", "Modified" }
                dd { class: "text-slate-300", {format_date(props.entry.modified_at)} }
            }
            // Created
            div {
                class: "flex justify-between",
                dt { class: "text-slate-500", "Created" }
                dd { class: "text-slate-300", {format_date(props.entry.created_at)} }
            }
            // Checksum if available
            if let Some(checksum) = &props.entry.checksum {
                div {
                    class: "flex flex-col gap-1",
                    dt { class: "text-slate-500", "Checksum" }
                    dd {
                        class: "text-slate-400 text-xs font-mono truncate",
                        title: "{checksum}",
                        "{checksum}"
                    }
                }
            }
        }
    }
}

/// Get icon for file type.
fn file_type_icon(entry: &DirectoryEntry) -> &'static str {
    if entry.is_directory {
        return "📁";
    }

    match entry.mime_type.as_deref() {
        Some(mime) if mime.starts_with("image/") => "🖼️",
        Some(mime) if mime.starts_with("video/") => "🎬",
        Some(mime) if mime.starts_with("audio/") => "🎵",
        Some(mime) if mime.starts_with("text/") => "📄",
        Some("application/pdf") => "📕",
        Some("application/zip") | Some("application/x-tar") | Some("application/gzip") => "📦",
        _ => "📄",
    }
}

/// Format file size for display.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format timestamp as date string.
fn format_date(timestamp_ms: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let time = UNIX_EPOCH + Duration::from_millis(timestamp_ms as u64);
    let datetime = time
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Simple date formatting (year-month-day)
    let secs_per_day = 86400u64;
    let days_since_epoch = datetime / secs_per_day;
    let years = 1970 + (days_since_epoch / 365);
    let remaining_days = days_since_epoch % 365;
    let month = (remaining_days / 30) + 1;
    let day = (remaining_days % 30) + 1;

    format!("{}-{:02}-{:02}", years, month.min(12), day.min(31))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn file_type_icon_returns_folder_for_directory() {
        let entry = DirectoryEntry {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_directory: true,
            size_bytes: 0,
            mime_type: None,
            modified_at: 0,
            created_at: 0,
            checksum: None,
        };
        assert_eq!(file_type_icon(&entry), "📁");
    }
}
