// SPDX-License-Identifier: MIT OR Apache-2.0

//! Preview panel for showing file details and content preview.
//!
//! Features:
//! - LRU cache for thumbnails
//! - Lazy loading with loading states
//! - Enhanced preview for images, PDFs, and videos
//! - Skeleton loading animations

use base64::Engine;
use communitas_ui_api::{DirectoryEntry, DiskType, FilePreview};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

// ============================================================================
// Preview Cache
// ============================================================================

/// Maximum number of cached previews.
const CACHE_MAX_SIZE: usize = 50;

/// Cached preview entry with access tracking.
#[derive(Clone)]
struct CacheEntry {
    preview: FilePreview,
    thumbnail_data_uri: Option<String>,
    access_count: u64,
    last_access: i64,
}

/// LRU cache for file previews.
#[derive(Default)]
pub struct PreviewCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    access_counter: RwLock<u64>,
}

impl PreviewCache {
    /// Create a new preview cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a cached preview by key (entity_id:disk_type:path).
    pub fn get(&self, key: &str) -> Option<(FilePreview, Option<String>)> {
        let mut entries = self.entries.write().ok()?;
        let entry = entries.get_mut(key)?;

        // Update access tracking
        if let Ok(mut counter) = self.access_counter.write() {
            *counter += 1;
            entry.access_count = *counter;
        }
        entry.last_access = current_timestamp_ms();

        Some((entry.preview.clone(), entry.thumbnail_data_uri.clone()))
    }

    /// Insert a preview into the cache.
    pub fn insert(&self, key: String, preview: FilePreview) {
        // Generate data URI for thumbnail if present
        let thumbnail_data_uri = preview.thumbnail.as_ref().map(|bytes| {
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            format!("data:{};base64,{}", preview.mime_type, encoded)
        });

        let entry = CacheEntry {
            preview,
            thumbnail_data_uri,
            access_count: self.access_counter.read().map(|c| *c).unwrap_or(0),
            last_access: current_timestamp_ms(),
        };

        if let Ok(mut entries) = self.entries.write() {
            // Evict oldest entries if at capacity
            if entries.len() >= CACHE_MAX_SIZE {
                self.evict_lru(&mut entries);
            }
            entries.insert(key, entry);
        }
    }

    /// Evict least recently used entries.
    fn evict_lru(&self, entries: &mut HashMap<String, CacheEntry>) {
        // Remove 10% of entries
        let to_remove = CACHE_MAX_SIZE / 10;

        // Collect keys sorted by access count
        let mut sorted: Vec<_> = entries
            .iter()
            .map(|(k, v)| (k.clone(), v.access_count))
            .collect();
        sorted.sort_by(|a, b| a.1.cmp(&b.1));

        for (key, _) in sorted.into_iter().take(to_remove) {
            debug!(target: "ui.drive.preview", key = %key, "Evicting preview from cache");
            entries.remove(&key);
        }
    }

    // Test-only methods
    #[cfg(test)]
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    #[cfg(test)]
    pub fn size(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }
}

/// Global preview cache.
static PREVIEW_CACHE: std::sync::OnceLock<PreviewCache> = std::sync::OnceLock::new();

fn get_preview_cache() -> &'static PreviewCache {
    PREVIEW_CACHE.get_or_init(PreviewCache::new)
}

fn current_timestamp_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ============================================================================
// Preview Loading State
// ============================================================================

/// State of preview loading.
#[derive(Debug, Clone, PartialEq)]
enum PreviewLoadState {
    /// No preview requested.
    Idle,
    /// Preview is loading.
    Loading,
    /// Preview loaded from cache.
    CacheHit(FilePreview, Option<String>),
    /// Preview loaded from service.
    Loaded(FilePreview, Option<String>),
    /// Preview failed to load.
    Error(String),
}

// Test-only methods
#[cfg(test)]
impl PreviewLoadState {
    fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    fn preview(&self) -> Option<(&FilePreview, &Option<String>)> {
        match self {
            Self::CacheHit(p, uri) | Self::Loaded(p, uri) => Some((p, uri)),
            _ => None,
        }
    }
}

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

    // State for preview loading with cache support
    let mut load_state = use_signal(|| PreviewLoadState::Idle);

    // Load preview when entry changes, checking cache first
    let entity_id = props.entity_id.clone();
    let disk_type = props.disk_type;
    let entry_path = props.entry.as_ref().map(|e| e.path.clone());

    use_effect(move || {
        if let Some(path) = entry_path.clone() {
            let drive = drive.clone();
            let entity_id = entity_id.clone();
            let cache_key = format!("{}:{}:{}", entity_id, disk_type as u8, path);

            spawn(async move {
                // Check cache first
                let cache = get_preview_cache();
                if let Some((preview, data_uri)) = cache.get(&cache_key) {
                    debug!(target: "ui.drive.preview", path = %path, "Preview cache hit");
                    load_state.set(PreviewLoadState::CacheHit(preview, data_uri));
                    return;
                }

                // Load from service
                load_state.set(PreviewLoadState::Loading);
                match drive.get_file_preview(&entity_id, disk_type, &path).await {
                    Ok(preview) => {
                        // Generate data URI
                        let data_uri = preview.thumbnail.as_ref().map(|bytes| {
                            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
                            format!("data:{};base64,{}", preview.mime_type, encoded)
                        });

                        // Cache it
                        cache.insert(cache_key, preview.clone());
                        debug!(target: "ui.drive.preview", path = %path, "Preview loaded and cached");
                        load_state.set(PreviewLoadState::Loaded(preview, data_uri));
                    }
                    Err(e) => {
                        debug!(target: "ui.drive.preview", path = %path, error = %e, "Preview load failed");
                        load_state.set(PreviewLoadState::Error(e.to_string()));
                    }
                }
            });
        } else {
            load_state.set(PreviewLoadState::Idle);
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
                match load_state() {
                    PreviewLoadState::Loading => rsx! { PreviewSkeleton { file_type: get_file_category(entry) } },
                    PreviewLoadState::CacheHit(p, uri) | PreviewLoadState::Loaded(p, uri) => {
                        rsx! { PreviewContent { entry: entry.clone(), preview: p, thumbnail_data_uri: uri } }
                    }
                    PreviewLoadState::Error(_) => rsx! { FileInfo { entry: entry.clone() } },
                    PreviewLoadState::Idle => rsx! { FileInfo { entry: entry.clone() } },
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

/// File category for skeleton loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileCategory {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Unknown,
}

fn get_file_category(entry: &DirectoryEntry) -> FileCategory {
    if entry.is_directory {
        return FileCategory::Unknown;
    }
    match entry.mime_type.as_deref() {
        Some(mime) if mime.starts_with("image/") => FileCategory::Image,
        Some(mime) if mime.starts_with("video/") => FileCategory::Video,
        Some(mime) if mime.starts_with("audio/") => FileCategory::Audio,
        Some("application/pdf") => FileCategory::Document,
        Some(mime) if mime.starts_with("text/") => FileCategory::Document,
        Some("application/zip") | Some("application/x-tar") | Some("application/gzip") => {
            FileCategory::Archive
        }
        _ => FileCategory::Unknown,
    }
}

/// Props for preview skeleton.
#[derive(Props, Clone, PartialEq)]
struct PreviewSkeletonProps {
    file_type: FileCategory,
}

/// Preview loading skeleton with file-type-specific hints.
#[component]
fn PreviewSkeleton(props: PreviewSkeletonProps) -> Element {
    let (icon, hint) = match props.file_type {
        FileCategory::Image => ("🖼️", "Loading image preview..."),
        FileCategory::Video => ("🎬", "Loading video thumbnail..."),
        FileCategory::Audio => ("🎵", "Loading audio info..."),
        FileCategory::Document => ("📄", "Loading document preview..."),
        FileCategory::Archive => ("📦", "Loading archive info..."),
        FileCategory::Unknown => ("📁", "Loading preview..."),
    };

    rsx! {
        div {
            class: "animate-pulse space-y-4",
            role: "status",
            aria_label: "Loading preview",
            // Thumbnail placeholder with icon
            div {
                class: "aspect-video bg-slate-800 rounded-lg flex flex-col items-center justify-center gap-2",
                span { class: "text-4xl opacity-30 animate-pulse", "{icon}" }
                span { class: "text-xs text-slate-600", "{hint}" }
            }
            // Info placeholders
            div { class: "h-4 bg-slate-800 rounded w-3/4" }
            div { class: "h-4 bg-slate-800 rounded w-1/2" }
            div { class: "h-4 bg-slate-800 rounded w-2/3" }
            // Screen reader hint
            span { class: "sr-only", "Loading file preview, please wait..." }
        }
    }
}

/// Preview content based on file type.
#[derive(Props, Clone, PartialEq)]
struct PreviewContentProps {
    entry: DirectoryEntry,
    preview: FilePreview,
    thumbnail_data_uri: Option<String>,
}

#[component]
fn PreviewContent(props: PreviewContentProps) -> Element {
    let file_category = get_file_category(&props.entry);

    rsx! {
        div {
            class: "space-y-4",
            // Thumbnail/preview based on file type
            match file_category {
                FileCategory::Image => rsx! {
                    ImagePreview {
                        data_uri: props.thumbnail_data_uri.clone(),
                        entry: props.entry.clone(),
                    }
                },
                FileCategory::Video => rsx! {
                    VideoPreview {
                        data_uri: props.thumbnail_data_uri.clone(),
                        entry: props.entry.clone(),
                    }
                },
                FileCategory::Document => rsx! {
                    DocumentPreview {
                        data_uri: props.thumbnail_data_uri.clone(),
                        text_preview: props.preview.text_preview.clone(),
                        entry: props.entry.clone(),
                    }
                },
                _ => rsx! {
                    GenericPreview {
                        data_uri: props.thumbnail_data_uri.clone(),
                        entry: props.entry.clone(),
                    }
                },
            }
            // File info
            FileInfo { entry: props.entry.clone() }
            // Text preview if available (for non-document types)
            if !matches!(file_category, FileCategory::Document) {
                if let Some(text_preview) = &props.preview.text_preview {
                    TextPreviewSection { text: text_preview.clone() }
                }
            }
        }
    }
}

/// Image preview component with zoom capability.
#[derive(Props, Clone, PartialEq)]
struct ImagePreviewProps {
    data_uri: Option<String>,
    entry: DirectoryEntry,
}

#[component]
fn ImagePreview(props: ImagePreviewProps) -> Element {
    let mut is_zoomed = use_signal(|| false);

    if let Some(ref data_uri) = props.data_uri {
        rsx! {
            div {
                class: if is_zoomed() {
                    "fixed inset-0 bg-black/90 z-50 flex items-center justify-center cursor-zoom-out"
                } else {
                    "aspect-video bg-slate-800 rounded-lg overflow-hidden cursor-zoom-in"
                },
                onclick: move |_| is_zoomed.set(!is_zoomed()),
                img {
                    src: "{data_uri}",
                    alt: "Image preview: {props.entry.name}",
                    class: if is_zoomed() {
                        "max-w-full max-h-full object-contain"
                    } else {
                        "w-full h-full object-contain"
                    },
                }
                if is_zoomed() {
                    div {
                        class: "absolute top-4 right-4 text-white/60 text-sm",
                        "Click to close"
                    }
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "aspect-video bg-slate-800 rounded-lg flex items-center justify-center",
                span { class: "text-6xl opacity-50", "🖼️" }
            }
        }
    }
}

/// Video preview component with thumbnail.
#[derive(Props, Clone, PartialEq)]
struct VideoPreviewProps {
    data_uri: Option<String>,
    entry: DirectoryEntry,
}

#[component]
fn VideoPreview(props: VideoPreviewProps) -> Element {
    rsx! {
        div {
            class: "aspect-video bg-slate-800 rounded-lg overflow-hidden relative group",
            if let Some(ref data_uri) = props.data_uri {
                img {
                    src: "{data_uri}",
                    alt: "Video thumbnail: {props.entry.name}",
                    class: "w-full h-full object-cover",
                }
            }
            // Play button overlay
            div {
                class: "absolute inset-0 flex items-center justify-center bg-black/30 group-hover:bg-black/50 transition",
                div {
                    class: "w-16 h-16 rounded-full bg-white/90 flex items-center justify-center shadow-lg",
                    svg {
                        class: "w-8 h-8 text-slate-900 ml-1",
                        fill: "currentColor",
                        view_box: "0 0 24 24",
                        path { d: "M8 5v14l11-7z" }
                    }
                }
            }
            // Duration badge (placeholder - would need actual duration)
            div {
                class: "absolute bottom-2 right-2 px-2 py-1 bg-black/70 rounded text-xs text-white",
                "Video"
            }
        }
    }
}

/// Document preview component (PDF, text).
#[derive(Props, Clone, PartialEq)]
struct DocumentPreviewProps {
    data_uri: Option<String>,
    text_preview: Option<String>,
    entry: DirectoryEntry,
}

#[component]
fn DocumentPreview(props: DocumentPreviewProps) -> Element {
    let is_pdf = props
        .entry
        .mime_type
        .as_ref()
        .is_some_and(|m| m == "application/pdf");

    rsx! {
        div {
            class: "space-y-3",
            // Thumbnail or PDF icon
            if let Some(ref data_uri) = props.data_uri {
                div {
                    class: "aspect-[3/4] bg-slate-800 rounded-lg overflow-hidden shadow-lg",
                    img {
                        src: "{data_uri}",
                        alt: "Document preview: {props.entry.name}",
                        class: "w-full h-full object-contain",
                    }
                }
            } else {
                div {
                    class: "aspect-[3/4] bg-slate-800 rounded-lg flex flex-col items-center justify-center",
                    span { class: "text-6xl mb-2", if is_pdf { "📕" } else { "📄" } }
                    span { class: "text-xs text-slate-500", if is_pdf { "PDF Document" } else { "Text Document" } }
                }
            }
            // Text preview
            if let Some(ref text) = props.text_preview {
                TextPreviewSection { text: text.clone() }
            }
        }
    }
}

/// Generic file preview.
#[derive(Props, Clone, PartialEq)]
struct GenericPreviewProps {
    data_uri: Option<String>,
    entry: DirectoryEntry,
}

#[component]
fn GenericPreview(props: GenericPreviewProps) -> Element {
    if let Some(ref data_uri) = props.data_uri {
        rsx! {
            div {
                class: "aspect-video bg-slate-800 rounded-lg overflow-hidden",
                img {
                    src: "{data_uri}",
                    alt: "Preview thumbnail",
                    class: "w-full h-full object-contain",
                }
            }
        }
    } else {
        rsx! {
            div {
                class: "aspect-video bg-slate-800 rounded-lg flex items-center justify-center",
                span {
                    class: "text-6xl opacity-50",
                    {file_type_icon(&props.entry)}
                }
            }
        }
    }
}

/// Text preview section.
#[derive(Props, Clone, PartialEq)]
struct TextPreviewSectionProps {
    text: String,
}

#[component]
fn TextPreviewSection(props: TextPreviewSectionProps) -> Element {
    let is_truncated = props.text.len() > 500;

    rsx! {
        div {
            class: "relative",
            div {
                class: "p-3 bg-slate-800 rounded-lg text-xs text-slate-300 font-mono whitespace-pre-wrap max-h-48 overflow-auto",
                "{props.text}"
            }
            if is_truncated {
                div {
                    class: "absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-slate-800 to-transparent rounded-b-lg",
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
    use communitas_ui_api::{FileMetadata, SyncState};

    fn make_entry(name: &str, is_dir: bool, mime: Option<&str>) -> DirectoryEntry {
        DirectoryEntry {
            name: name.to_string(),
            path: format!("/{}", name),
            is_directory: is_dir,
            size_bytes: 1024,
            mime_type: mime.map(|s| s.to_string()),
            modified_at: 0,
            created_at: 0,
            checksum: None,
            sync_state: SyncState::Synced,
        }
    }

    fn make_metadata() -> FileMetadata {
        FileMetadata {
            created_at: 0,
            modified_at: 0,
            checksum: "abc123".to_string(),
            block_count: 1,
            encryption: None,
        }
    }

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn file_type_icon_returns_folder_for_directory() {
        let entry = make_entry("test", true, None);
        assert_eq!(file_type_icon(&entry), "📁");
    }

    #[test]
    fn file_type_icon_returns_image_icon() {
        let entry = make_entry("photo.png", false, Some("image/png"));
        assert_eq!(file_type_icon(&entry), "🖼️");
    }

    #[test]
    fn file_type_icon_returns_video_icon() {
        let entry = make_entry("video.mp4", false, Some("video/mp4"));
        assert_eq!(file_type_icon(&entry), "🎬");
    }

    #[test]
    fn file_type_icon_returns_audio_icon() {
        let entry = make_entry("song.mp3", false, Some("audio/mpeg"));
        assert_eq!(file_type_icon(&entry), "🎵");
    }

    #[test]
    fn file_type_icon_returns_pdf_icon() {
        let entry = make_entry("doc.pdf", false, Some("application/pdf"));
        assert_eq!(file_type_icon(&entry), "📕");
    }

    #[test]
    fn get_file_category_image() {
        let entry = make_entry("photo.jpg", false, Some("image/jpeg"));
        assert_eq!(get_file_category(&entry), FileCategory::Image);
    }

    #[test]
    fn get_file_category_video() {
        let entry = make_entry("movie.mp4", false, Some("video/mp4"));
        assert_eq!(get_file_category(&entry), FileCategory::Video);
    }

    #[test]
    fn get_file_category_audio() {
        let entry = make_entry("song.mp3", false, Some("audio/mpeg"));
        assert_eq!(get_file_category(&entry), FileCategory::Audio);
    }

    #[test]
    fn get_file_category_document_pdf() {
        let entry = make_entry("doc.pdf", false, Some("application/pdf"));
        assert_eq!(get_file_category(&entry), FileCategory::Document);
    }

    #[test]
    fn get_file_category_document_text() {
        let entry = make_entry("readme.txt", false, Some("text/plain"));
        assert_eq!(get_file_category(&entry), FileCategory::Document);
    }

    #[test]
    fn get_file_category_archive() {
        let entry = make_entry("archive.zip", false, Some("application/zip"));
        assert_eq!(get_file_category(&entry), FileCategory::Archive);
    }

    #[test]
    fn get_file_category_unknown() {
        let entry = make_entry("file.bin", false, Some("application/octet-stream"));
        assert_eq!(get_file_category(&entry), FileCategory::Unknown);
    }

    #[test]
    fn get_file_category_directory() {
        let entry = make_entry("folder", true, None);
        assert_eq!(get_file_category(&entry), FileCategory::Unknown);
    }

    #[test]
    fn preview_load_state_is_loading() {
        assert!(PreviewLoadState::Loading.is_loading());
        assert!(!PreviewLoadState::Idle.is_loading());
    }

    #[test]
    fn preview_load_state_preview() {
        let preview = FilePreview {
            path: "/test".to_string(),
            mime_type: "text/plain".to_string(),
            size_bytes: 100,
            thumbnail: None,
            text_preview: None,
            metadata: make_metadata(),
        };

        let state = PreviewLoadState::Loaded(preview.clone(), None);
        assert!(state.preview().is_some());

        let state_idle = PreviewLoadState::Idle;
        assert!(state_idle.preview().is_none());
    }

    #[test]
    fn preview_cache_insert_and_get() {
        let cache = PreviewCache::new();
        let preview = FilePreview {
            path: "/test.txt".to_string(),
            mime_type: "text/plain".to_string(),
            size_bytes: 100,
            thumbnail: None,
            text_preview: Some("Hello".to_string()),
            metadata: make_metadata(),
        };

        cache.insert("key1".to_string(), preview.clone());
        let result = cache.get("key1");

        assert!(result.is_some());
        let (cached_preview, _uri) = result.unwrap();
        assert_eq!(cached_preview.path, "/test.txt");
    }

    #[test]
    fn preview_cache_miss() {
        let cache = PreviewCache::new();
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn preview_cache_size() {
        let cache = PreviewCache::new();
        assert_eq!(cache.size(), 0);

        let preview = FilePreview {
            path: "/test".to_string(),
            mime_type: "text/plain".to_string(),
            size_bytes: 100,
            thumbnail: None,
            text_preview: None,
            metadata: make_metadata(),
        };

        cache.insert("key1".to_string(), preview.clone());
        assert_eq!(cache.size(), 1);

        cache.insert("key2".to_string(), preview);
        assert_eq!(cache.size(), 2);
    }

    #[test]
    fn preview_cache_clear() {
        let cache = PreviewCache::new();
        let preview = FilePreview {
            path: "/test".to_string(),
            mime_type: "text/plain".to_string(),
            size_bytes: 100,
            thumbnail: None,
            text_preview: None,
            metadata: make_metadata(),
        };

        cache.insert("key1".to_string(), preview);
        assert_eq!(cache.size(), 1);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn preview_cache_generates_data_uri_for_thumbnail() {
        let cache = PreviewCache::new();
        let preview = FilePreview {
            path: "/test.png".to_string(),
            mime_type: "image/png".to_string(),
            size_bytes: 100,
            thumbnail: Some(vec![0x89, 0x50, 0x4E, 0x47]), // PNG magic bytes
            text_preview: None,
            metadata: make_metadata(),
        };

        cache.insert("key1".to_string(), preview);
        let result = cache.get("key1");

        assert!(result.is_some());
        let (_, uri) = result.unwrap();
        assert!(uri.is_some());
        let data_uri = uri.unwrap();
        assert!(data_uri.starts_with("data:image/png;base64,"));
    }
}
