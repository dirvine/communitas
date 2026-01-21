//! Drive browser component with three-panel layout.

use communitas_ui_api::{DirectoryEntry, DiskType, QuotaInfo};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

use super::file_list::FileList;
use super::file_picker::{open_file_picker, PickerConfig, PickerResult};
use super::preview_panel::PreviewPanel;
use super::quota_meter::QuotaMeter;
use super::tree_view::TreeView;
use super::upload_progress::UploadProgressPanel;

/// View mode for the file list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    List,
    Grid,
}

/// Sort column for file list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    #[default]
    Name,
    Size,
    Modified,
    #[allow(dead_code)]
    Type,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

/// Drive browser props.
#[derive(Props, Clone, PartialEq)]
pub struct DriveBrowserProps {
    /// Entity ID that owns the drive.
    pub entity_id: String,
    /// Initial disk type to display.
    #[props(default)]
    pub initial_disk_type: Option<String>,
    /// Initial path to navigate to.
    #[props(default)]
    pub initial_path: Option<String>,
}

/// Main drive browser component with three-panel layout.
#[component]
pub fn DriveBrowser(props: DriveBrowserProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let drive = services.drive();

    // State
    let mut current_disk = use_signal(|| {
        props
            .initial_disk_type
            .as_ref()
            .map(|s| match s.to_lowercase().as_str() {
                "public" => DiskType::Public,
                "shared" => DiskType::Shared,
                _ => DiskType::Private,
            })
            .unwrap_or(DiskType::Private)
    });
    let mut current_path = use_signal(|| {
        props
            .initial_path
            .clone()
            .unwrap_or_else(|| "/".to_string())
    });
    let mut entries = use_signal(Vec::<DirectoryEntry>::new);
    let mut selected_entries = use_signal(Vec::<String>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);

    // UI state
    let mut view_mode = use_signal(ViewMode::default);
    let mut sort_column = use_signal(SortColumn::default);
    let mut sort_direction = use_signal(SortDirection::default);
    let mut tree_collapsed = use_signal(|| false);
    let mut preview_collapsed = use_signal(|| true);
    let mut show_uploads = use_signal(|| false);
    let mut drag_over = use_signal(|| false);

    // Quota state
    let mut quota = use_signal(|| Option::<QuotaInfo>::None);

    // Fetch directory contents
    let entity_id = props.entity_id.clone();
    let entity_id_for_fetch = entity_id.clone();
    use_effect(move || {
        let drive = drive.clone();
        let entity_id = entity_id_for_fetch.clone();
        let disk = current_disk();
        let path = current_path();

        spawn(async move {
            loading.set(true);
            error.set(None);

            match drive.list_directory(&entity_id, disk, &path).await {
                Ok(result) => {
                    entries.set(result);
                    loading.set(false);
                }
                Err(err) => {
                    error.set(Some(err.to_string()));
                    loading.set(false);
                }
            }

            // Fetch quota
            if let Ok(q) = drive.get_quota(&entity_id, disk).await {
                quota.set(Some(q));
            }
        });
    });

    // Breadcrumb segments - owned to avoid temporary lifetime issues
    let path_str = current_path();
    let path_segments: Vec<String> = path_str
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // Handle folder navigation
    let mut navigate_to = move |path: String| {
        current_path.set(path);
        selected_entries.set(Vec::new());
        preview_collapsed.set(true);
    };

    // Handle disk tab change
    let change_disk = move |disk: DiskType| {
        current_disk.set(disk);
        current_path.set("/".to_string());
        selected_entries.set(Vec::new());
        preview_collapsed.set(true);
    };

    // Handle file/folder selection
    let mut handle_select = move |entry_path: String, multi: bool| {
        let mut current = selected_entries();
        if multi {
            if current.contains(&entry_path) {
                current.retain(|p| p != &entry_path);
            } else {
                current.push(entry_path);
            }
        } else {
            current = vec![entry_path];
        }
        selected_entries.set(current.clone());
        preview_collapsed.set(current.is_empty());
    };

    // Handle double-click to open
    let mut handle_open = move |entry: DirectoryEntry| {
        if entry.is_directory {
            navigate_to(entry.path.clone());
        } else {
            info!(target = "ui.drive", event = "file_opened", path = %entry.path);
            // TODO: Open file preview or download
        }
    };

    // Get selected entry for preview
    let selected_entry: Option<DirectoryEntry> = if selected_entries().len() == 1 {
        entries()
            .iter()
            .find(|e| selected_entries().contains(&e.path))
            .cloned()
    } else {
        None
    };

    let entity_id_for_tree = props.entity_id.clone();
    let entity_id_for_upload = props.entity_id.clone();
    let entity_id_for_picker = props.entity_id.clone();

    // Handle file upload via picker
    let drive_for_upload = services.drive();
    let handle_upload_files = move |files: Vec<PathBuf>| {
        if files.is_empty() {
            return;
        }

        let drive = drive_for_upload.clone();
        let entity_id = entity_id_for_picker.clone();
        let disk = current_disk();
        let dest_path = current_path();

        info!(
            target: "ui.drive",
            event = "upload_started",
            count = files.len(),
            disk = ?disk,
            path = %dest_path
        );

        show_uploads.set(true);

        spawn(async move {
            for file_path in files {
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let dest = if dest_path == "/" {
                    format!("/{file_name}")
                } else {
                    format!("{dest_path}/{file_name}")
                };

                info!(
                    target: "ui.drive",
                    event = "upload_file",
                    source = ?file_path,
                    dest = %dest
                );

                match drive
                    .start_streaming_upload(&entity_id, disk, &dest, &file_path)
                    .await
                {
                    Ok(_) => {
                        info!(
                            target: "ui.drive",
                            event = "upload_queued",
                            file = %file_name
                        );
                    }
                    Err(e) => {
                        warn!(
                            target: "ui.drive",
                            event = "upload_failed",
                            file = %file_name,
                            error = %e
                        );
                    }
                }
            }
        });
    };

    // Open file picker for uploads
    let open_upload_picker = move |_| {
        let mut on_select = handle_upload_files.clone();
        spawn(async move {
            let config = PickerConfig::multiple_files().with_title("Select Files to Upload");

            let result = tokio::task::spawn_blocking(move || open_file_picker(&config))
                .await
                .unwrap_or_else(|e| PickerResult::Error(e.to_string()));

            match result {
                PickerResult::Selected(files) => {
                    on_select(files);
                }
                PickerResult::Cancelled => {
                    info!(target: "ui.drive", event = "upload_picker_cancelled");
                }
                PickerResult::Error(err) => {
                    warn!(target: "ui.drive", event = "upload_picker_error", error = %err);
                }
            }
        });
    };

    rsx! {
        div {
            class: "drive-browser flex flex-col h-full",
            role: "main",
            aria_label: "Drive file browser",
            ondragover: move |evt| {
                evt.prevent_default();
                drag_over.set(true);
            },
            ondragleave: move |_| {
                drag_over.set(false);
            },
            ondrop: move |evt| {
                evt.prevent_default();
                drag_over.set(false);
                info!(target = "ui.drive", event = "drop_detected");
                // TODO: Handle file drop
            },
            // Drag overlay
            if drag_over() {
                DragOverlay {}
            }
            // Toolbar
            DriveToolbar {
                current_disk: current_disk(),
                view_mode: view_mode(),
                on_disk_change: change_disk,
                on_view_change: move |mode| view_mode.set(mode),
                on_upload: open_upload_picker,
                on_new_folder: move |_| {
                    info!(target = "ui.drive", event = "new_folder_clicked");
                    // TODO: Show new folder dialog
                },
                on_tree_toggle: move |_| tree_collapsed.set(!tree_collapsed()),
                tree_collapsed: tree_collapsed(),
                on_show_uploads: move |_| show_uploads.set(true),
            }
            // Breadcrumb
            Breadcrumb {
                segments: path_segments.iter().map(|s| s.to_string()).collect(),
                on_navigate: move |idx: usize| {
                    let path = if idx == 0 {
                        "/".to_string()
                    } else {
                        format!("/{}", path_segments[..=idx].join("/"))
                    };
                    navigate_to(path);
                },
            }
            // Main content area
            div {
                class: "flex-1 flex overflow-hidden",
                // Tree view sidebar
                if !tree_collapsed() {
                    div {
                        class: "w-64 flex-shrink-0 border-r border-slate-800 overflow-y-auto",
                        TreeView {
                            entity_id: entity_id_for_tree.clone(),
                            current_disk: current_disk(),
                            current_path: current_path(),
                            on_navigate: move |path: String| navigate_to(path),
                        }
                    }
                }
                // File list (center)
                div {
                    class: "flex-1 flex flex-col overflow-hidden",
                    if loading() {
                        FileListSkeleton {}
                    } else if let Some(err) = error() {
                        ErrorPanel {
                            message: err,
                            on_retry: move |_| {
                                loading.set(true);
                                error.set(None);
                            },
                        }
                    } else {
                        FileList {
                            entries: entries(),
                            selected: selected_entries(),
                            view_mode: view_mode(),
                            sort_column: sort_column(),
                            sort_direction: sort_direction(),
                            on_select: move |(path, multi): (String, bool)| {
                                handle_select(path, multi);
                            },
                            on_open: move |entry: DirectoryEntry| {
                                handle_open(entry);
                            },
                            on_sort: move |(col, dir): (SortColumn, SortDirection)| {
                                sort_column.set(col);
                                sort_direction.set(dir);
                            },
                            on_context_menu: move |(entry, _x, _y): (DirectoryEntry, i32, i32)| {
                                info!(target = "ui.drive", event = "context_menu", path = %entry.path);
                                // TODO: Show context menu
                            },
                        }
                    }
                    // Quota meter at bottom
                    if let Some(q) = quota() {
                        div {
                            class: "border-t border-slate-800 p-2",
                            QuotaMeter {
                                quota: q,
                            }
                        }
                    }
                }
                // Preview panel (right)
                if !preview_collapsed() {
                    div {
                        class: "w-80 flex-shrink-0 border-l border-slate-800 overflow-y-auto",
                        PreviewPanel {
                            entry: selected_entry,
                            selected_count: selected_entries().len(),
                            entity_id: entity_id.clone(),
                            disk_type: current_disk(),
                            on_download: move |path: String| {
                                info!(target = "ui.drive", event = "download_requested", path = %path);
                                // TODO: Start download
                            },
                            on_delete: move |path: String| {
                                info!(target = "ui.drive", event = "delete_requested", path = %path);
                                // TODO: Confirm and delete
                            },
                            on_close: move |_| preview_collapsed.set(true),
                        }
                    }
                }
            }
            // Upload progress panel (modal/drawer)
            if show_uploads() {
                UploadProgressPanel {
                    entity_id: entity_id_for_upload.clone(),
                    disk_type: current_disk(),
                    on_close: move |_| show_uploads.set(false),
                }
            }
        }
    }
}

/// Drag overlay when files are being dragged over.
#[component]
fn DragOverlay() -> Element {
    rsx! {
        div {
            class: "absolute inset-0 z-50 bg-emerald-500/20 border-2 border-dashed border-emerald-400 flex items-center justify-center pointer-events-none",
            div {
                class: "text-center",
                div { class: "text-4xl mb-2", "📁" }
                p { class: "text-emerald-300 font-semibold", "Drop files to upload" }
            }
        }
    }
}

/// Drive toolbar with disk tabs and actions.
#[derive(Props, Clone, PartialEq)]
struct DriveToolbarProps {
    current_disk: DiskType,
    view_mode: ViewMode,
    on_disk_change: EventHandler<DiskType>,
    on_view_change: EventHandler<ViewMode>,
    on_upload: EventHandler<()>,
    on_new_folder: EventHandler<()>,
    on_tree_toggle: EventHandler<()>,
    tree_collapsed: bool,
    on_show_uploads: EventHandler<()>,
}

#[component]
fn DriveToolbar(props: DriveToolbarProps) -> Element {
    let disks = [DiskType::Private, DiskType::Public, DiskType::Shared];

    rsx! {
        div {
            class: "flex items-center justify-between p-3 border-b border-slate-800",
            // Disk tabs
            div {
                class: "flex gap-1",
                role: "tablist",
                {disks.iter().map(|disk| {
                    let disk = *disk;
                    let active = props.current_disk == disk;
                    rsx! {
                        button {
                            key: "{disk:?}",
                            role: "tab",
                            aria_selected: "{active}",
                            class: format!(
                                "px-4 py-2 text-sm font-medium rounded-lg transition {}",
                                if active {
                                    "bg-emerald-500/20 text-emerald-400 border border-emerald-500/50"
                                } else {
                                    "text-slate-400 hover:text-slate-200 hover:bg-slate-800"
                                }
                            ),
                            onclick: move |_| props.on_disk_change.call(disk),
                            "{disk.label()}"
                        }
                    }
                })}
            }
            // Actions
            div {
                class: "flex items-center gap-2",
                // Tree toggle
                button {
                    class: format!(
                        "p-2 rounded-lg transition {}",
                        if props.tree_collapsed {
                            "text-slate-500 hover:text-slate-300"
                        } else {
                            "text-emerald-400 bg-emerald-500/10"
                        }
                    ),
                    title: if props.tree_collapsed { "Show tree" } else { "Hide tree" },
                    onclick: move |_| props.on_tree_toggle.call(()),
                    "🌲"
                }
                // View mode toggle
                div {
                    class: "flex rounded-lg border border-slate-700 overflow-hidden",
                    button {
                        class: format!(
                            "px-3 py-1.5 text-sm transition {}",
                            if props.view_mode == ViewMode::List {
                                "bg-slate-700 text-slate-200"
                            } else {
                                "text-slate-400 hover:text-slate-200"
                            }
                        ),
                        onclick: move |_| props.on_view_change.call(ViewMode::List),
                        "☰"
                    }
                    button {
                        class: format!(
                            "px-3 py-1.5 text-sm transition {}",
                            if props.view_mode == ViewMode::Grid {
                                "bg-slate-700 text-slate-200"
                            } else {
                                "text-slate-400 hover:text-slate-200"
                            }
                        ),
                        onclick: move |_| props.on_view_change.call(ViewMode::Grid),
                        "⊞"
                    }
                }
                // New folder
                button {
                    class: "px-3 py-1.5 rounded-lg text-sm text-slate-300 border border-slate-700 hover:border-slate-600 transition",
                    onclick: move |_| props.on_new_folder.call(()),
                    "New Folder"
                }
                // View uploads button
                button {
                    class: "px-3 py-1.5 rounded-lg text-sm text-slate-300 border border-slate-700 hover:border-slate-600 transition",
                    title: "View active uploads",
                    onclick: move |_| props.on_show_uploads.call(()),
                    "Uploads"
                }
                // Upload button (opens file picker)
                button {
                    class: "px-4 py-1.5 rounded-lg text-sm font-semibold bg-emerald-500 text-slate-900 hover:bg-emerald-400 transition",
                    title: "Select files to upload",
                    onclick: move |_| props.on_upload.call(()),
                    "Upload Files"
                }
            }
        }
    }
}

/// Breadcrumb navigation.
#[derive(Props, Clone, PartialEq)]
struct BreadcrumbProps {
    segments: Vec<String>,
    on_navigate: EventHandler<usize>,
}

#[component]
fn Breadcrumb(props: BreadcrumbProps) -> Element {
    rsx! {
        nav {
            class: "flex items-center gap-1 px-4 py-2 text-sm border-b border-slate-800/50",
            aria_label: "Breadcrumb",
            // Root
            button {
                class: "text-slate-400 hover:text-emerald-400 transition",
                onclick: move |_| props.on_navigate.call(0),
                "/"
            }
            {props.segments.iter().enumerate().map(|(idx, segment)| {
                let segment = segment.clone();
                rsx! {
                    span {
                        key: "{idx}",
                        class: "flex items-center gap-1",
                        span { class: "text-slate-600", "/" }
                        button {
                            class: "text-slate-300 hover:text-emerald-400 transition",
                            onclick: move |_| props.on_navigate.call(idx + 1),
                            "{segment}"
                        }
                    }
                }
            })}
        }
    }
}

/// Error panel.
#[derive(Props, Clone, PartialEq)]
struct ErrorPanelProps {
    message: String,
    on_retry: EventHandler<()>,
}

#[component]
fn ErrorPanel(props: ErrorPanelProps) -> Element {
    rsx! {
        div {
            class: "flex-1 flex flex-col items-center justify-center",
            role: "alert",
            div { class: "text-4xl mb-4", "⚠️" }
            p { class: "text-red-400 mb-4", "{props.message}" }
            button {
                class: "px-4 py-2 rounded-lg bg-emerald-500 text-slate-900 font-semibold hover:bg-emerald-400 transition",
                onclick: move |_| props.on_retry.call(()),
                "Retry"
            }
        }
    }
}

/// Skeleton loader for file list.
#[component]
fn FileListSkeleton() -> Element {
    rsx! {
        div {
            class: "flex-1 p-4 animate-pulse",
            aria_busy: "true",
            aria_label: "Loading files",
            {(0..8).map(|i| rsx! {
                div {
                    key: "{i}",
                    class: "flex items-center gap-3 p-2 mb-2",
                    div { class: "w-8 h-8 bg-slate-700 rounded" }
                    div { class: "flex-1",
                        div { class: "h-4 w-48 bg-slate-700 rounded mb-1" }
                        div { class: "h-3 w-24 bg-slate-700 rounded" }
                    }
                }
            })}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_mode_default_is_list() {
        assert_eq!(ViewMode::default(), ViewMode::List);
    }

    #[test]
    fn sort_column_default_is_name() {
        assert_eq!(SortColumn::default(), SortColumn::Name);
    }

    #[test]
    fn sort_direction_default_is_ascending() {
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
    }

    #[test]
    fn disk_type_label_returns_correct_strings() {
        assert_eq!(DiskType::Private.label(), "Private");
        assert_eq!(DiskType::Public.label(), "Public");
        assert_eq!(DiskType::Shared.label(), "Shared");
    }
}
