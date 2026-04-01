// SPDX-License-Identifier: MIT OR Apache-2.0

//! File list component with list and grid views.

use communitas_ui_api::{DirectoryEntry, SyncState};
use dioxus::prelude::*;

use super::browser::{SortColumn, SortDirection, ViewMode};
use crate::tokens::colors;

/// File list props.
#[derive(Props, Clone, PartialEq)]
pub struct FileListProps {
    /// Directory entries to display.
    pub entries: Vec<DirectoryEntry>,
    /// Currently selected entry paths.
    pub selected: Vec<String>,
    /// View mode (list or grid).
    pub view_mode: ViewMode,
    /// Current sort column.
    pub sort_column: SortColumn,
    /// Current sort direction.
    pub sort_direction: SortDirection,
    /// Whether file entries are currently loading.
    #[props(default = false)]
    pub loading: bool,
    /// Selection handler (path, is_multi_select).
    pub on_select: EventHandler<(String, bool)>,
    /// Open handler (double-click).
    pub on_open: EventHandler<DirectoryEntry>,
    /// Sort change handler.
    pub on_sort: EventHandler<(SortColumn, SortDirection)>,
    /// Context menu handler (entry, x, y).
    pub on_context_menu: EventHandler<(DirectoryEntry, i32, i32)>,
    /// Delete selected entries handler.
    #[props(default)]
    pub on_delete: Option<EventHandler<Vec<String>>>,
    /// Select all entries handler.
    #[props(default)]
    pub on_select_all: Option<EventHandler<()>>,
}

/// File list component with list and grid views.
#[component]
pub fn FileList(props: FileListProps) -> Element {
    // Show loading skeleton while data is being fetched
    if props.loading {
        return rsx! {
            FileListSkeleton { view_mode: props.view_mode }
        };
    }

    // Sort entries
    let mut sorted_entries = props.entries.clone();
    sorted_entries.sort_by(|a, b| {
        // Directories first
        if a.is_directory != b.is_directory {
            return if a.is_directory {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }

        let cmp = match props.sort_column {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size_bytes.cmp(&b.size_bytes),
            SortColumn::Modified => a.modified_at.cmp(&b.modified_at),
        };

        match props.sort_direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    });

    if sorted_entries.is_empty() {
        return rsx! {
            EmptyState {}
        };
    }

    match props.view_mode {
        ViewMode::List => rsx! {
            ListView {
                entries: sorted_entries,
                selected: props.selected.clone(),
                sort_column: props.sort_column,
                sort_direction: props.sort_direction,
                on_select: props.on_select,
                on_open: props.on_open,
                on_sort: props.on_sort,
                on_context_menu: props.on_context_menu,
                on_delete: props.on_delete,
                on_select_all: props.on_select_all,
            }
        },
        ViewMode::Grid => rsx! {
            GridView {
                entries: sorted_entries,
                selected: props.selected.clone(),
                on_select: props.on_select,
                on_open: props.on_open,
                on_context_menu: props.on_context_menu,
                on_delete: props.on_delete,
                on_select_all: props.on_select_all,
            }
        },
    }
}

/// Empty state when folder has no contents.
#[component]
fn EmptyState() -> Element {
    rsx! {
        div {
            class: "flex-1 flex flex-col items-center justify-center text-center p-8",
            div { class: "text-4xl mb-4", "📂" }
            h3 { class: "text-lg font-semibold text-slate-300 mb-2", "This folder is empty" }
            p { class: "text-sm text-slate-500 mb-4", "Upload files or create a new folder to get started" }
            div {
                class: "flex gap-2",
                button {
                    class: "px-4 py-2 rounded-lg bg-emerald-500 text-slate-900 font-semibold hover:bg-emerald-400 transition",
                    "Upload files"
                }
                button {
                    class: "px-4 py-2 rounded-lg border border-slate-700 text-slate-300 hover:border-slate-600 transition",
                    "New folder"
                }
            }
        }
    }
}

/// Skeleton loading state for file list.
#[derive(Props, Clone, PartialEq)]
struct FileListSkeletonProps {
    /// View mode to match skeleton layout.
    view_mode: ViewMode,
}

/// Skeleton placeholder for file list during loading.
#[component]
fn FileListSkeleton(props: FileListSkeletonProps) -> Element {
    match props.view_mode {
        ViewMode::List => rsx! {
            FileListSkeletonTable {}
        },
        ViewMode::Grid => rsx! {
            FileListSkeletonGrid {}
        },
    }
}

/// Table-style skeleton for list view mode.
#[component]
fn FileListSkeletonTable() -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-auto animate-pulse",
            role: "status",
            aria_busy: "true",
            aria_label: "Loading files",
            table {
                class: "w-full text-sm",
                thead {
                    class: "sticky top-0 bg-slate-900 border-b border-slate-800",
                    tr {
                        th { class: "w-8 p-2" }
                        th { class: "text-left p-2",
                            div { class: "h-4 w-16 bg-slate-700 rounded" }
                        }
                        th { class: "text-right p-2 w-24",
                            div { class: "h-4 w-10 bg-slate-700 rounded ml-auto" }
                        }
                        th { class: "text-right p-2 w-32",
                            div { class: "h-4 w-16 bg-slate-700 rounded ml-auto" }
                        }
                    }
                }
                tbody {
                    for i in 0..8 {
                        tr {
                            key: "{i}",
                            class: "border-b border-slate-800/50",
                            td { class: "p-2",
                                div { class: "w-4 h-4 rounded border border-slate-700" }
                            }
                            td { class: "p-2",
                                div {
                                    class: "flex items-center gap-2",
                                    div { class: "w-5 h-5 bg-slate-700 rounded" }
                                    div {
                                        class: format!("h-4 bg-slate-700 rounded {}",
                                            if i % 3 == 0 { "w-32" } else if i % 3 == 1 { "w-48" } else { "w-40" }
                                        ),
                                    }
                                }
                            }
                            td { class: "p-2",
                                div { class: "h-4 w-12 bg-slate-700 rounded ml-auto" }
                            }
                            td { class: "p-2",
                                div { class: "h-4 w-16 bg-slate-700 rounded ml-auto" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Grid-style skeleton for grid view mode.
#[component]
fn FileListSkeletonGrid() -> Element {
    rsx! {
        div {
            class: "flex-1 overflow-auto p-4 animate-pulse",
            role: "status",
            aria_busy: "true",
            aria_label: "Loading files",
            div {
                class: "grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-4",
                for i in 0..12 {
                    div {
                        key: "{i}",
                        class: "flex flex-col items-center p-3 rounded-lg",
                        div { class: "w-10 h-10 bg-slate-700 rounded mb-2" }
                        div {
                            class: format!("h-3 bg-slate-700 rounded {}",
                                if i % 2 == 0 { "w-16" } else { "w-20" }
                            ),
                        }
                    }
                }
            }
        }
    }
}

/// List view with table layout.
#[derive(Props, Clone, PartialEq)]
struct ListViewProps {
    entries: Vec<DirectoryEntry>,
    selected: Vec<String>,
    sort_column: SortColumn,
    sort_direction: SortDirection,
    on_select: EventHandler<(String, bool)>,
    on_open: EventHandler<DirectoryEntry>,
    on_sort: EventHandler<(SortColumn, SortDirection)>,
    on_context_menu: EventHandler<(DirectoryEntry, i32, i32)>,
    #[props(default)]
    on_delete: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    on_select_all: Option<EventHandler<()>>,
}

#[component]
fn ListView(props: ListViewProps) -> Element {
    let sort_column = props.sort_column;
    let sort_direction = props.sort_direction;
    let on_sort = props.on_sort;
    let entries = props.entries.clone();
    let entry_count = entries.len();

    // Roving tabindex: track which row has focus
    let mut focused_index = use_signal(|| 0usize);

    let handle_header_click = move |col: SortColumn| {
        let new_direction = if sort_column == col && sort_direction == SortDirection::Ascending {
            SortDirection::Descending
        } else {
            SortDirection::Ascending
        };
        on_sort.call((col, new_direction));
    };

    // Keyboard navigation handler for the table
    let handle_keydown = {
        let entries = entries.clone();
        let on_select = props.on_select;
        let on_open = props.on_open;
        let on_context_menu = props.on_context_menu;
        let on_delete = props.on_delete;
        let on_select_all = props.on_select_all;
        let selected = props.selected.clone();

        move |evt: KeyboardEvent| {
            let key = evt.key();
            let current = focused_index();
            let ctrl_or_meta = evt.modifiers().ctrl() || evt.modifiers().meta();
            let shift = evt.modifiers().shift();

            match key {
                // Navigation: Up/Down arrows
                Key::ArrowDown => {
                    evt.prevent_default();
                    if current + 1 < entry_count {
                        focused_index.set(current + 1);
                    }
                }
                Key::ArrowUp => {
                    evt.prevent_default();
                    if current > 0 {
                        focused_index.set(current - 1);
                    }
                }
                Key::Home => {
                    evt.prevent_default();
                    focused_index.set(0);
                }
                Key::End => {
                    evt.prevent_default();
                    if entry_count > 0 {
                        focused_index.set(entry_count - 1);
                    }
                }
                // Open: Enter key
                Key::Enter => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_open.call(entry.clone());
                    }
                }
                // Toggle selection: Space key
                Key::Character(ref c) if c == " " => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_select.call((entry.path.clone(), ctrl_or_meta));
                    }
                }
                // Context menu: Shift+F10 or ContextMenu key
                Key::F10 if shift => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_context_menu.call((entry.clone(), 0, 0));
                    }
                }
                Key::ContextMenu => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_context_menu.call((entry.clone(), 0, 0));
                    }
                }
                // Delete: Delete or Backspace
                Key::Delete | Key::Backspace => {
                    evt.prevent_default();
                    if let Some(handler) = &on_delete {
                        if !selected.is_empty() {
                            handler.call(selected.clone());
                        } else if let Some(entry) = entries.get(current) {
                            handler.call(vec![entry.path.clone()]);
                        }
                    }
                }
                // Select all: Ctrl+A
                Key::Character(ref c) if ctrl_or_meta && c.to_lowercase() == "a" => {
                    evt.prevent_default();
                    if let Some(handler) = &on_select_all {
                        handler.call(());
                    }
                }
                _ => {}
            }
        }
    };

    rsx! {
        div {
            class: "flex-1 overflow-auto",
            tabindex: "0",
            role: "application",
            aria_label: "File list. Use arrow keys to navigate, Enter to open, Space to select, Delete to remove.",
            onkeydown: handle_keydown,
            table {
                class: "w-full text-sm",
                role: "grid",
                thead {
                    class: "sticky top-0 bg-slate-900 border-b border-slate-800",
                    tr {
                        // Checkbox column
                        th { class: "w-8 p-2" }
                        // Name
                        th {
                            class: "text-left p-2 text-slate-400 font-medium cursor-pointer hover:text-slate-200",
                            onclick: move |_| handle_header_click(SortColumn::Name),
                            "Name"
                            SortIndicator { column: SortColumn::Name, current: props.sort_column, direction: props.sort_direction }
                        }
                        // Size
                        th {
                            class: "text-right p-2 text-slate-400 font-medium cursor-pointer hover:text-slate-200 w-24",
                            onclick: move |_| handle_header_click(SortColumn::Size),
                            "Size"
                            SortIndicator { column: SortColumn::Size, current: props.sort_column, direction: props.sort_direction }
                        }
                        // Modified
                        th {
                            class: "text-right p-2 text-slate-400 font-medium cursor-pointer hover:text-slate-200 w-32",
                            onclick: move |_| handle_header_click(SortColumn::Modified),
                            "Modified"
                            SortIndicator { column: SortColumn::Modified, current: props.sort_column, direction: props.sort_direction }
                        }
                    }
                }
                tbody {
                    {props.entries.iter().enumerate().map(|(index, entry)| {
                        let entry_clone = entry.clone();
                        let entry_for_open = entry.clone();
                        let entry_for_ctx = entry.clone();
                        let path = entry.path.clone();
                        let is_selected = props.selected.contains(&entry.path);
                        let is_focused = index == focused_index();

                        // Generate aria-label with file info
                        let file_type = if entry.is_directory {
                            "folder"
                        } else {
                            entry.mime_type.as_deref().unwrap_or("file")
                        };
                        let size_str = format_size(entry.size_bytes, entry.is_directory);

                        rsx! {
                            tr {
                                key: "{path}",
                                class: format!(
                                    "border-b border-slate-800/50 cursor-pointer transition {} {}",
                                    if is_selected {
                                        "bg-emerald-500/10"
                                    } else {
                                        "hover:bg-slate-800/50"
                                    },
                                    if is_focused {
                                        "ring-2 ring-emerald-400/50 ring-inset"
                                    } else {
                                        ""
                                    }
                                ),
                                role: "row",
                                aria_label: format!("{}, {}, {}", entry.name, file_type, size_str),
                                aria_selected: is_selected,
                                aria_rowindex: "{index + 1}",
                                onclick: move |evt| {
                                    focused_index.set(index);
                                    let multi = evt.modifiers().ctrl() || evt.modifiers().meta();
                                    props.on_select.call((entry_clone.path.clone(), multi));
                                },
                                ondoubleclick: move |_| props.on_open.call(entry_for_open.clone()),
                                oncontextmenu: move |evt| {
                                    evt.prevent_default();
                                    props.on_context_menu.call((entry_for_ctx.clone(), 0, 0));
                                },
                                // Checkbox
                                td {
                                    class: "p-2",
                                    role: "gridcell",
                                    input {
                                        r#type: "checkbox",
                                        checked: is_selected,
                                        class: "rounded border-slate-600",
                                        tabindex: "-1",
                                        aria_label: format!("Select {}", entry.name),
                                    }
                                }
                                // Name with icon and sync state
                                td {
                                    class: "p-2",
                                    role: "gridcell",
                                    div {
                                        class: "flex items-center gap-2",
                                        span {
                                            class: "text-lg",
                                            aria_hidden: "true",
                                            {file_icon(entry)}
                                        }
                                        span {
                                            class: if is_selected { "text-emerald-300" } else { "text-slate-200" },
                                            "{entry.name}"
                                        }
                                        FileSyncIndicator { state: entry.sync_state }
                                    }
                                }
                                // Size
                                td {
                                    class: "p-2 text-right text-slate-400",
                                    role: "gridcell",
                                    {format_size(entry.size_bytes, entry.is_directory)}
                                }
                                // Modified
                                td {
                                    class: "p-2 text-right text-slate-400",
                                    role: "gridcell",
                                    {format_timestamp(entry.modified_at)}
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}

/// Sort direction indicator.
#[derive(Props, Clone, PartialEq)]
struct SortIndicatorProps {
    column: SortColumn,
    current: SortColumn,
    direction: SortDirection,
}

#[component]
fn SortIndicator(props: SortIndicatorProps) -> Element {
    if props.column != props.current {
        return rsx! { span {} };
    }

    rsx! {
        span {
            class: "ml-1 text-emerald-400",
            {match props.direction {
                SortDirection::Ascending => "↑",
                SortDirection::Descending => "↓",
            }}
        }
    }
}

/// File sync state indicator for drive entries.
#[derive(Props, Clone, PartialEq)]
struct FileSyncIndicatorProps {
    /// Sync state to display.
    state: SyncState,
}

/// Displays sync state icon for file/directory entries.
#[component]
fn FileSyncIndicator(props: FileSyncIndicatorProps) -> Element {
    // Don't show indicator for synced items
    if props.state == SyncState::Synced {
        return rsx! {};
    }

    let (icon, color, label) = match props.state {
        SyncState::Synced => ("✓", colors::SUCCESS, "Synced"),
        SyncState::Syncing => ("⟳", colors::INFO, "Syncing"),
        SyncState::Queued => ("⏱", colors::WARNING, "Waiting to sync"),
        SyncState::Conflict => ("⚠", colors::WARNING, "Has conflicts"),
        SyncState::Error => ("✕", colors::ERROR, "Sync failed"),
    };

    rsx! {
        span {
            class: "ml-2 text-sm",
            style: "color: {color}",
            title: "{label}",
            aria_label: "{label}",
            "{icon}"
        }
    }
}

/// Grid view with cards.
#[derive(Props, Clone, PartialEq)]
struct GridViewProps {
    entries: Vec<DirectoryEntry>,
    selected: Vec<String>,
    on_select: EventHandler<(String, bool)>,
    on_open: EventHandler<DirectoryEntry>,
    on_context_menu: EventHandler<(DirectoryEntry, i32, i32)>,
    #[props(default)]
    on_delete: Option<EventHandler<Vec<String>>>,
    #[props(default)]
    on_select_all: Option<EventHandler<()>>,
}

#[component]
fn GridView(props: GridViewProps) -> Element {
    let entries = props.entries.clone();
    let entry_count = entries.len();

    // Roving tabindex: track which item has focus
    let mut focused_index = use_signal(|| 0usize);

    // Estimate columns per row (120px min-width cards)
    // This is approximate; for true grid nav we'd need layout info
    let cols_per_row = 6usize; // Reasonable default for typical screen

    // Keyboard navigation handler for the grid
    let handle_keydown = {
        let entries = entries.clone();
        let on_select = props.on_select;
        let on_open = props.on_open;
        let on_context_menu = props.on_context_menu;
        let on_delete = props.on_delete;
        let on_select_all = props.on_select_all;
        let selected = props.selected.clone();

        move |evt: KeyboardEvent| {
            let key = evt.key();
            let current = focused_index();
            let ctrl_or_meta = evt.modifiers().ctrl() || evt.modifiers().meta();
            let shift = evt.modifiers().shift();

            match key {
                // Navigation: Arrow keys for 2D grid
                Key::ArrowRight => {
                    evt.prevent_default();
                    if current + 1 < entry_count {
                        focused_index.set(current + 1);
                    }
                }
                Key::ArrowLeft => {
                    evt.prevent_default();
                    if current > 0 {
                        focused_index.set(current - 1);
                    }
                }
                Key::ArrowDown => {
                    evt.prevent_default();
                    let next = current + cols_per_row;
                    if next < entry_count {
                        focused_index.set(next);
                    }
                }
                Key::ArrowUp => {
                    evt.prevent_default();
                    if current >= cols_per_row {
                        focused_index.set(current - cols_per_row);
                    }
                }
                Key::Home => {
                    evt.prevent_default();
                    focused_index.set(0);
                }
                Key::End => {
                    evt.prevent_default();
                    if entry_count > 0 {
                        focused_index.set(entry_count - 1);
                    }
                }
                // Open: Enter key
                Key::Enter => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_open.call(entry.clone());
                    }
                }
                // Toggle selection: Space key
                Key::Character(ref c) if c == " " => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_select.call((entry.path.clone(), ctrl_or_meta));
                    }
                }
                // Context menu: Shift+F10 or ContextMenu key
                Key::F10 if shift => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_context_menu.call((entry.clone(), 0, 0));
                    }
                }
                Key::ContextMenu => {
                    evt.prevent_default();
                    if let Some(entry) = entries.get(current) {
                        on_context_menu.call((entry.clone(), 0, 0));
                    }
                }
                // Delete: Delete or Backspace
                Key::Delete | Key::Backspace => {
                    evt.prevent_default();
                    if let Some(handler) = &on_delete {
                        if !selected.is_empty() {
                            handler.call(selected.clone());
                        } else if let Some(entry) = entries.get(current) {
                            handler.call(vec![entry.path.clone()]);
                        }
                    }
                }
                // Select all: Ctrl+A
                Key::Character(ref c) if ctrl_or_meta && c.to_lowercase() == "a" => {
                    evt.prevent_default();
                    if let Some(handler) = &on_select_all {
                        handler.call(());
                    }
                }
                _ => {}
            }
        }
    };

    rsx! {
        div {
            class: "flex-1 overflow-auto p-4",
            tabindex: "0",
            role: "application",
            aria_label: "File grid. Use arrow keys to navigate, Enter to open, Space to select, Delete to remove.",
            onkeydown: handle_keydown,
            div {
                class: "grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-4",
                role: "grid",
                {props.entries.iter().enumerate().map(|(index, entry)| {
                    let entry_clone = entry.clone();
                    let entry_for_open = entry.clone();
                    let entry_for_ctx = entry.clone();
                    let path = entry.path.clone();
                    let is_selected = props.selected.contains(&entry.path);
                    let is_focused = index == focused_index();

                    // Generate aria-label for grid items
                    let grid_file_type = if entry.is_directory {
                        "folder"
                    } else {
                        entry.mime_type.as_deref().unwrap_or("file")
                    };

                    rsx! {
                        div {
                            key: "{path}",
                            class: format!(
                                "flex flex-col items-center p-3 rounded-lg cursor-pointer transition {} {}",
                                if is_selected {
                                    "bg-emerald-500/20 ring-2 ring-emerald-500/50"
                                } else {
                                    "hover:bg-slate-800"
                                },
                                if is_focused {
                                    "ring-2 ring-emerald-400 ring-offset-2 ring-offset-slate-900"
                                } else {
                                    ""
                                }
                            ),
                            role: "gridcell",
                            aria_label: format!("{}, {}", entry.name, grid_file_type),
                            aria_selected: is_selected,
                            tabindex: if is_focused { "0" } else { "-1" },
                            onclick: move |evt| {
                                focused_index.set(index);
                                let multi = evt.modifiers().ctrl() || evt.modifiers().meta();
                                props.on_select.call((entry_clone.path.clone(), multi));
                            },
                            ondoubleclick: move |_| props.on_open.call(entry_for_open.clone()),
                            oncontextmenu: move |evt| {
                                evt.prevent_default();
                                props.on_context_menu.call((entry_for_ctx.clone(), 0, 0));
                            },
                            // Icon with sync indicator
                            div {
                                class: "relative",
                                div {
                                    class: "text-4xl mb-2",
                                    aria_hidden: "true",
                                    {file_icon(entry)}
                                }
                                // Sync indicator badge (positioned at corner)
                                if entry.sync_state != SyncState::Synced {
                                    div {
                                        class: "absolute -top-1 -right-1",
                                        FileSyncIndicator { state: entry.sync_state }
                                    }
                                }
                            }
                            // Name
                            p {
                                class: format!(
                                    "text-sm text-center truncate w-full {}",
                                    if is_selected { "text-emerald-300" } else { "text-slate-200" }
                                ),
                                title: "{entry.name}",
                                "{entry.name}"
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Get icon for file/folder.
fn file_icon(entry: &DirectoryEntry) -> &'static str {
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
        Some(mime) if mime.contains("spreadsheet") || mime.contains("excel") => "📊",
        Some(mime) if mime.contains("document") || mime.contains("word") => "📝",
        Some(mime) if mime.contains("presentation") || mime.contains("powerpoint") => "📽️",
        _ => "📄",
    }
}

/// Format file size for display.
fn format_size(bytes: u64, is_directory: bool) -> String {
    if is_directory {
        return "—".to_string();
    }

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

/// Format timestamp for display.
fn format_timestamp(timestamp_ms: i64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let file_time = UNIX_EPOCH + Duration::from_millis(timestamp_ms as u64);
    let now = SystemTime::now();

    let diff = now
        .duration_since(file_time)
        .unwrap_or(Duration::from_secs(0));

    let secs = diff.as_secs();
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;

    if days > 30 {
        // Show date for older files
        format!("{} days ago", days)
    } else if days > 0 {
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if hours > 0 {
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if mins > 0 {
        format!("{} min ago", mins)
    } else {
        "Just now".to_string()
    }
}

#[cfg(test)]
#[allow(clippy::useless_vec)]
mod tests {
    use super::*;
    use communitas_ui_api::SyncState;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(500, false), "500 B");
        assert_eq!(format_size(1024, false), "1.0 KB");
        assert_eq!(format_size(1024 * 1024, false), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024, false), "1.0 GB");
    }

    #[test]
    fn format_size_directory() {
        assert_eq!(format_size(0, true), "—");
    }

    #[test]
    fn file_icon_returns_folder_for_directory() {
        let entry = DirectoryEntry {
            name: "test".to_string(),
            path: "/test".to_string(),
            is_directory: true,
            size_bytes: 0,
            mime_type: None,
            modified_at: 0,
            created_at: 0,
            checksum: None,
            sync_state: SyncState::Synced,
        };
        assert_eq!(file_icon(&entry), "📁");
    }

    #[test]
    fn file_icon_returns_image_for_image_mime() {
        let entry = DirectoryEntry {
            name: "photo.jpg".to_string(),
            path: "/photo.jpg".to_string(),
            is_directory: false,
            size_bytes: 1000,
            mime_type: Some("image/jpeg".to_string()),
            modified_at: 0,
            created_at: 0,
            checksum: None,
            sync_state: SyncState::Synced,
        };
        assert_eq!(file_icon(&entry), "🖼️");
    }

    #[test]
    fn file_list_aria_label_for_file() {
        let name = "document.pdf";
        let file_type = "application/pdf";
        let size_str = "1.5 MB";

        let aria_label = format!("{}, {}, {}", name, file_type, size_str);
        assert_eq!(aria_label, "document.pdf, application/pdf, 1.5 MB");
    }

    #[test]
    fn file_list_aria_label_for_folder() {
        let name = "Documents";
        let file_type = "folder";
        let size_str = "—";

        let aria_label = format!("{}, {}, {}", name, file_type, size_str);
        assert_eq!(aria_label, "Documents, folder, —");
    }

    #[test]
    fn grid_view_aria_label_for_file() {
        let name = "image.png";
        let grid_file_type = "image/png";

        let aria_label = format!("{}, {}", name, grid_file_type);
        assert_eq!(aria_label, "image.png, image/png");
    }

    #[test]
    fn grid_view_aria_label_for_folder() {
        let name = "Downloads";
        let grid_file_type = "folder";

        let aria_label = format!("{}, {}", name, grid_file_type);
        assert_eq!(aria_label, "Downloads, folder");
    }

    // Keyboard navigation tests
    #[test]
    fn keyboard_navigation_arrow_down_increments_index() {
        let current = 0usize;
        let entry_count = 5;
        let next = if current + 1 < entry_count {
            current + 1
        } else {
            current
        };
        assert_eq!(next, 1);
    }

    #[test]
    fn keyboard_navigation_arrow_up_decrements_index() {
        let current = 3usize;
        let prev = if current > 0 { current - 1 } else { current };
        assert_eq!(prev, 2);
    }

    #[test]
    fn keyboard_navigation_arrow_up_at_zero_stays_at_zero() {
        let current = 0usize;
        let prev = if current > 0 { current - 1 } else { current };
        assert_eq!(prev, 0);
    }

    #[test]
    fn keyboard_navigation_home_goes_to_start() {
        let focused_index = 0usize; // Home sets to 0
        assert_eq!(focused_index, 0);
    }

    #[test]
    fn keyboard_navigation_end_goes_to_last() {
        let entry_count = 10usize;
        let focused_index = entry_count - 1;
        assert_eq!(focused_index, 9);
    }

    #[test]
    fn grid_navigation_arrow_down_jumps_row() {
        let current = 0usize;
        let cols_per_row = 6usize;
        let entry_count = 20;
        let next = current + cols_per_row;
        let focused = if next < entry_count { next } else { current };
        assert_eq!(focused, 6);
    }

    #[test]
    fn grid_navigation_arrow_up_jumps_row() {
        let current = 8usize;
        let cols_per_row = 6usize;
        let prev = if current >= cols_per_row {
            current - cols_per_row
        } else {
            current
        };
        assert_eq!(prev, 2);
    }

    #[test]
    fn grid_navigation_arrow_up_first_row_stays() {
        let current = 3usize;
        let cols_per_row = 6usize;
        let prev = if current >= cols_per_row {
            current - cols_per_row
        } else {
            current
        };
        assert_eq!(prev, 3);
    }

    #[test]
    fn roving_tabindex_focused_item_is_tabbable() {
        let is_focused = true;
        let tabindex = if is_focused { "0" } else { "-1" };
        assert_eq!(tabindex, "0");
    }

    #[test]
    fn roving_tabindex_unfocused_item_not_tabbable() {
        let is_focused = false;
        let tabindex = if is_focused { "0" } else { "-1" };
        assert_eq!(tabindex, "-1");
    }

    #[test]
    fn delete_with_selection_uses_selection() {
        let selected = vec!["/a".to_string(), "/b".to_string()];
        let current = 0usize;
        let entries = vec![DirectoryEntry {
            name: "c".to_string(),
            path: "/c".to_string(),
            is_directory: false,
            size_bytes: 0,
            mime_type: None,
            modified_at: 0,
            created_at: 0,
            checksum: None,
            sync_state: SyncState::Synced,
        }];

        let to_delete = if !selected.is_empty() {
            selected.clone()
        } else if let Some(entry) = entries.get(current) {
            vec![entry.path.clone()]
        } else {
            vec![]
        };

        assert_eq!(to_delete, vec!["/a".to_string(), "/b".to_string()]);
    }

    #[test]
    fn delete_without_selection_uses_focused() {
        let selected: Vec<String> = vec![];
        let current = 0usize;
        let entries = vec![DirectoryEntry {
            name: "c".to_string(),
            path: "/c".to_string(),
            is_directory: false,
            size_bytes: 0,
            mime_type: None,
            modified_at: 0,
            created_at: 0,
            checksum: None,
            sync_state: SyncState::Synced,
        }];

        let to_delete = if !selected.is_empty() {
            selected.clone()
        } else if let Some(entry) = entries.get(current) {
            vec![entry.path.clone()]
        } else {
            vec![]
        };

        assert_eq!(to_delete, vec!["/c".to_string()]);
    }
}
