//! File picker component using native platform dialogs.
//!
//! Provides cross-platform file selection dialogs with:
//! - Single and multi-file selection
//! - Directory selection for bulk uploads
//! - File type filters
//! - Last used directory memory

use dioxus::prelude::*;
use native_dialog::FileDialog;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// File type filter for the picker dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    /// Display name (e.g., "Images")
    pub name: &'static str,
    /// File extensions (e.g., ["png", "jpg", "gif"])
    pub extensions: &'static [&'static str],
}

impl FileFilter {
    /// Common filter for all files.
    pub const ALL: Self = Self {
        name: "All Files",
        extensions: &["*"],
    };

    // Test-only constants
    #[cfg(test)]
    pub const IMAGES: Self = Self {
        name: "Images",
        extensions: &["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "svg"],
    };

    #[cfg(test)]
    pub const DOCUMENTS: Self = Self {
        name: "Documents",
        extensions: &["pdf", "doc", "docx", "txt", "rtf", "odt", "md"],
    };

    #[cfg(test)]
    pub const VIDEOS: Self = Self {
        name: "Videos",
        extensions: &["mp4", "mkv", "avi", "mov", "webm", "m4v"],
    };

    #[cfg(test)]
    pub const AUDIO: Self = Self {
        name: "Audio",
        extensions: &["mp3", "wav", "flac", "ogg", "m4a", "aac"],
    };

    #[cfg(test)]
    pub const ARCHIVES: Self = Self {
        name: "Archives",
        extensions: &["zip", "tar", "gz", "7z", "rar", "xz"],
    };
}

/// Result of a file picker operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerResult {
    /// User selected one or more files.
    Selected(Vec<PathBuf>),
    /// User cancelled the dialog.
    Cancelled,
    /// An error occurred.
    Error(String),
}


// Test-only methods for PickerResult
#[cfg(test)]
impl PickerResult {
    pub fn has_selection(&self) -> bool {
        matches!(self, Self::Selected(files) if !files.is_empty())
    }

    pub fn files(&self) -> Vec<PathBuf> {
        match self {
            Self::Selected(files) => files.clone(),
            _ => Vec::new(),
        }
    }
}

/// Configuration for the file picker.
#[derive(Debug, Clone, PartialEq)]
pub struct PickerConfig {
    /// Dialog title.
    pub title: String,
    /// Whether to allow multiple file selection.
    pub multiple: bool,
    /// Whether to select directories instead of files.
    pub directory: bool,
    /// File type filters (ignored for directory selection).
    pub filters: Vec<FileFilter>,
    /// Starting directory (None = last used or home).
    pub start_directory: Option<PathBuf>,
}

impl Default for PickerConfig {
    fn default() -> Self {
        Self {
            title: "Select Files".to_string(),
            multiple: false,
            directory: false,
            filters: vec![FileFilter::ALL],
            start_directory: None,
        }
    }
}

impl PickerConfig {
    /// Create a config for multi-file selection.
    pub fn multiple_files() -> Self {
        Self {
            title: "Select Files".to_string(),
            multiple: true,
            directory: false,
            filters: vec![FileFilter::ALL],
            start_directory: None,
        }
    }

    /// Builder: set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    // Test-only builder methods
    #[cfg(test)]
    pub fn single_file() -> Self {
        Self {
            title: "Select File".to_string(),
            multiple: false,
            directory: false,
            filters: vec![FileFilter::ALL],
            start_directory: None,
        }
    }

    #[cfg(test)]
    pub fn directory() -> Self {
        Self {
            title: "Select Folder".to_string(),
            multiple: false,
            directory: true,
            filters: vec![],
            start_directory: None,
        }
    }

    #[cfg(test)]
    pub fn with_multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    #[cfg(test)]
    pub fn with_start_directory(mut self, dir: PathBuf) -> Self {
        self.start_directory = Some(dir);
        self
    }

    #[cfg(test)]
    pub fn with_filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }
}

/// Global state for remembering last used directory.
static LAST_DIRECTORY: std::sync::OnceLock<std::sync::Mutex<Option<PathBuf>>> =
    std::sync::OnceLock::new();

fn get_last_directory() -> Option<PathBuf> {
    LAST_DIRECTORY
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

fn set_last_directory(path: &std::path::Path) {
    let dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().map(|p| p.to_path_buf()).unwrap_or_default()
    };

    if let Ok(mut guard) = LAST_DIRECTORY
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
    {
        *guard = Some(dir);
    }
}

/// Open a file picker dialog with the given configuration.
///
/// This function blocks until the user makes a selection or cancels.
/// It should be called from a spawned async task, not the main thread.
pub fn open_file_picker(config: &PickerConfig) -> PickerResult {
    // Determine starting directory
    let start_dir = config
        .start_directory
        .clone()
        .or_else(get_last_directory)
        .or_else(dirs::home_dir);

    debug!(
        target: "ui.drive.picker",
        title = %config.title,
        multiple = %config.multiple,
        directory = %config.directory,
        start_dir = ?start_dir,
        "Opening file picker"
    );

    if config.directory {
        // Directory selection
        let mut dialog = FileDialog::new();

        if let Some(ref dir) = start_dir {
            dialog = dialog.set_location(dir);
        }

        match dialog.show_open_single_dir() {
            Ok(Some(path)) => {
                info!(target: "ui.drive.picker", path = ?path, "Directory selected");
                set_last_directory(&path);
                PickerResult::Selected(vec![path])
            }
            Ok(None) => {
                debug!(target: "ui.drive.picker", "Directory selection cancelled");
                PickerResult::Cancelled
            }
            Err(e) => {
                warn!(target: "ui.drive.picker", error = %e, "Directory picker error");
                PickerResult::Error(e.to_string())
            }
        }
    } else if config.multiple {
        // Multiple file selection
        let mut dialog = FileDialog::new();

        if let Some(ref dir) = start_dir {
            dialog = dialog.set_location(dir);
        }

        // Add filters
        for filter in &config.filters {
            dialog = dialog.add_filter(filter.name, filter.extensions);
        }

        match dialog.show_open_multiple_file() {
            Ok(paths) if paths.is_empty() => {
                debug!(target: "ui.drive.picker", "Multi-file selection cancelled");
                PickerResult::Cancelled
            }
            Ok(paths) => {
                info!(target: "ui.drive.picker", count = paths.len(), "Files selected");
                if let Some(first) = paths.first() {
                    set_last_directory(first);
                }
                PickerResult::Selected(paths)
            }
            Err(e) => {
                warn!(target: "ui.drive.picker", error = %e, "Multi-file picker error");
                PickerResult::Error(e.to_string())
            }
        }
    } else {
        // Single file selection
        let mut dialog = FileDialog::new();

        if let Some(ref dir) = start_dir {
            dialog = dialog.set_location(dir);
        }

        // Add filters
        for filter in &config.filters {
            dialog = dialog.add_filter(filter.name, filter.extensions);
        }

        match dialog.show_open_single_file() {
            Ok(Some(path)) => {
                info!(target: "ui.drive.picker", path = ?path, "File selected");
                set_last_directory(&path);
                PickerResult::Selected(vec![path])
            }
            Ok(None) => {
                debug!(target: "ui.drive.picker", "Single file selection cancelled");
                PickerResult::Cancelled
            }
            Err(e) => {
                warn!(target: "ui.drive.picker", error = %e, "Single file picker error");
                PickerResult::Error(e.to_string())
            }
        }
    }
}


/// File picker button component props.
#[derive(Props, Clone, PartialEq)]
pub struct FilePickerButtonProps {
    /// Button label.
    #[props(default = "Select Files".to_string())]
    pub label: String,
    /// Picker configuration.
    #[props(default)]
    pub config: PickerConfig,
    /// Callback when files are selected.
    pub on_select: EventHandler<Vec<PathBuf>>,
    /// Callback when an error occurs.
    #[props(default)]
    pub on_error: Option<EventHandler<String>>,
    /// Additional CSS classes for the button.
    #[props(default)]
    pub class: String,
    /// Whether the button is disabled.
    #[props(default)]
    pub disabled: bool,
}

/// File picker button component.
///
/// Renders a button that opens a native file picker dialog when clicked.
#[component]
pub fn FilePickerButton(props: FilePickerButtonProps) -> Element {
    let mut loading = use_signal(|| false);

    let handle_click = move |_| {
        if loading() || props.disabled {
            return;
        }

        loading.set(true);
        let config = props.config.clone();
        let on_select = props.on_select;
        let on_error = props.on_error;

        spawn(async move {
            // Run file picker in a blocking task to avoid blocking the UI
            let result = tokio::task::spawn_blocking(move || open_file_picker(&config))
                .await
                .unwrap_or_else(|e| PickerResult::Error(e.to_string()));

            loading.set(false);

            match result {
                PickerResult::Selected(files) => {
                    on_select.call(files);
                }
                PickerResult::Cancelled => {
                    // Nothing to do
                }
                PickerResult::Error(err) => {
                    if let Some(handler) = on_error {
                        handler.call(err);
                    }
                }
            }
        });
    };

    let base_class = if props.class.is_empty() {
        "px-4 py-2 rounded-lg font-semibold transition"
    } else {
        &props.class
    };

    let button_class = if props.disabled || loading() {
        format!("{base_class} bg-slate-700 text-slate-500 cursor-not-allowed")
    } else {
        format!("{base_class} bg-emerald-500 text-slate-900 hover:bg-emerald-400")
    };

    rsx! {
        button {
            r#type: "button",
            class: "{button_class}",
            disabled: props.disabled || loading(),
            onclick: handle_click,
            if loading() {
                span {
                    class: "inline-flex items-center gap-2",
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
                    "Opening..."
                }
            } else {
                "{props.label}"
            }
        }
    }
}

/// Drop zone component props.
#[derive(Props, Clone, PartialEq)]
pub struct DropZoneProps {
    /// Callback when files are dropped.
    pub on_drop: EventHandler<Vec<PathBuf>>,
    /// Accepted file extensions (empty = all).
    #[props(default)]
    pub accept: Vec<String>,
    /// Children to render inside the drop zone.
    pub children: Element,
    /// Additional CSS classes.
    #[props(default)]
    pub class: String,
}

/// Drop zone component for drag-and-drop file uploads.
#[component]
pub fn DropZone(props: DropZoneProps) -> Element {
    let mut drag_over = use_signal(|| false);

    let zone_class = if drag_over() {
        format!(
            "{} border-2 border-dashed border-emerald-400 bg-emerald-500/10",
            props.class
        )
    } else {
        format!("{} border-2 border-dashed border-slate-700", props.class)
    };

    rsx! {
        div {
            class: "{zone_class}",
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

                // Note: Dioxus/web drag-and-drop file access is limited
                // This would need platform-specific handling for actual files
                info!(target: "ui.drive.picker", "Drop event received");
                // props.on_drop.call(vec![]); // Would need actual file paths
            },
            {&props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_config_defaults() {
        let config = PickerConfig::default();
        assert!(!config.multiple);
        assert!(!config.directory);
        assert_eq!(config.title, "Select Files");
    }

    #[test]
    fn picker_config_single_file() {
        let config = PickerConfig::single_file();
        assert!(!config.multiple);
        assert!(!config.directory);
    }

    #[test]
    fn picker_config_multiple_files() {
        let config = PickerConfig::multiple_files();
        assert!(config.multiple);
        assert!(!config.directory);
    }

    #[test]
    fn picker_config_directory() {
        let config = PickerConfig::directory();
        assert!(!config.multiple);
        assert!(config.directory);
    }

    #[test]
    fn picker_config_builder() {
        let config = PickerConfig::default()
            .with_title("Custom Title")
            .with_multiple(true)
            .with_filter(FileFilter::IMAGES)
            .with_start_directory(PathBuf::from("/tmp"));

        assert_eq!(config.title, "Custom Title");
        assert!(config.multiple);
        assert_eq!(config.start_directory, Some(PathBuf::from("/tmp")));
        assert!(config.filters.iter().any(|f| f.name == "Images"));
    }

    #[test]
    fn picker_result_has_selection() {
        let selected = PickerResult::Selected(vec![PathBuf::from("/test.txt")]);
        assert!(selected.has_selection());

        let empty = PickerResult::Selected(vec![]);
        assert!(!empty.has_selection());

        let cancelled = PickerResult::Cancelled;
        assert!(!cancelled.has_selection());

        let error = PickerResult::Error("test".to_string());
        assert!(!error.has_selection());
    }

    #[test]
    fn picker_result_files() {
        let path = PathBuf::from("/test.txt");
        let selected = PickerResult::Selected(vec![path.clone()]);
        assert_eq!(selected.files(), vec![path]);

        let cancelled = PickerResult::Cancelled;
        assert!(cancelled.files().is_empty());
    }

    #[test]
    fn file_filter_constants() {
        assert!(!FileFilter::IMAGES.extensions.is_empty());
        assert!(!FileFilter::DOCUMENTS.extensions.is_empty());
        assert!(!FileFilter::VIDEOS.extensions.is_empty());
        assert!(!FileFilter::AUDIO.extensions.is_empty());
        assert!(!FileFilter::ARCHIVES.extensions.is_empty());
        assert_eq!(FileFilter::ALL.extensions, &["*"]);
    }
}
