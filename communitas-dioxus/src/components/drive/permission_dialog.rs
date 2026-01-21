//! Permission dialog components for drive operations.
//!
//! This module provides dialogs for:
//! - File access permission requests
//! - Storage quota warning dialogs
//! - Dangerous file type warnings
//! - Network permission for external shares

#![allow(dead_code)]

use dioxus::prelude::*;
use std::path::PathBuf;
use tracing::{debug, info};

// ============================================================================
// Permission Types
// ============================================================================

/// Result of a permission dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionResult {
    /// Permission was granted by the user.
    Granted,
    /// Permission was denied by the user.
    Denied,
    /// User dismissed the dialog without making a choice.
    Dismissed,
}

impl PermissionResult {
    /// Returns true if permission was granted.
    pub fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Type of permission being requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionType {
    /// Access to a local file or directory.
    FileAccess { path: PathBuf, read_only: bool },
    /// Network access for sharing files externally.
    NetworkShare { share_url: String },
    /// Storage operation that exceeds quota.
    StorageQuota { required_bytes: u64, available_bytes: u64 },
    /// Dangerous file type (executable, script).
    DangerousFile { file_name: String, file_type: DangerousFileType },
}

/// Categories of dangerous file types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerousFileType {
    /// Executable file (.exe, .app, .dmg, etc.)
    Executable,
    /// Script file (.sh, .bat, .ps1, .py, etc.)
    Script,
    /// Archive that may contain executables.
    ArchiveWithExecutables,
    /// System file or configuration.
    SystemFile,
    /// Unknown but potentially dangerous.
    Unknown,
}

impl DangerousFileType {
    /// Get a description of the file type.
    pub fn description(self) -> &'static str {
        match self {
            Self::Executable => "executable program",
            Self::Script => "script file",
            Self::ArchiveWithExecutables => "archive that may contain executables",
            Self::SystemFile => "system configuration file",
            Self::Unknown => "potentially unsafe file",
        }
    }

    /// Get the warning level (higher = more dangerous).
    pub fn warning_level(self) -> u8 {
        match self {
            Self::Executable => 3,
            Self::Script => 2,
            Self::ArchiveWithExecutables => 2,
            Self::SystemFile => 2,
            Self::Unknown => 1,
        }
    }
}

/// Check if a file extension is dangerous.
pub fn is_dangerous_extension(ext: &str) -> Option<DangerousFileType> {
    let ext_lower = ext.to_lowercase();
    match ext_lower.as_str() {
        // Executables
        "exe" | "msi" | "app" | "dmg" | "pkg" | "deb" | "rpm" | "appimage" | "com" | "scr" => {
            Some(DangerousFileType::Executable)
        }
        // Scripts
        "sh" | "bash" | "zsh" | "bat" | "cmd" | "ps1" | "psm1" | "vbs" | "vbe" | "js" | "jse"
        | "wsf" | "wsh" => Some(DangerousFileType::Script),
        // Archives (may contain executables)
        "zip" | "rar" | "7z" | "tar" | "gz" | "xz" | "bz2" => {
            Some(DangerousFileType::ArchiveWithExecutables)
        }
        // System files
        "dll" | "sys" | "drv" | "reg" | "inf" | "plist" => Some(DangerousFileType::SystemFile),
        // Not dangerous
        _ => None,
    }
}

// ============================================================================
// Modal Dialog Base
// ============================================================================

/// Props for the base modal dialog component.
#[derive(Props, Clone, PartialEq)]
struct ModalDialogProps {
    /// Whether the dialog is open.
    open: bool,
    /// Called when the dialog should be closed.
    on_close: EventHandler<()>,
    /// Dialog title.
    title: String,
    /// Optional icon element.
    #[props(default)]
    icon: Option<Element>,
    /// Dialog content.
    children: Element,
}

/// Base modal dialog component.
#[component]
fn ModalDialog(props: ModalDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }

    // Handle escape key
    let on_close = props.on_close;

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(()),
            role: "presentation",
            // Dialog panel
            div {
                class: "bg-slate-900 border border-slate-700 rounded-xl shadow-2xl max-w-md w-full p-6 transform transition-all",
                onclick: move |evt| evt.stop_propagation(),
                role: "dialog",
                aria_modal: "true",
                aria_labelledby: "dialog-title",
                // Header with icon and title
                div {
                    class: "flex items-start gap-4 mb-4",
                    if let Some(icon) = props.icon {
                        div {
                            class: "flex-shrink-0",
                            {icon}
                        }
                    }
                    h2 {
                        id: "dialog-title",
                        class: "text-lg font-semibold text-slate-100",
                        "{props.title}"
                    }
                }
                // Content
                {&props.children}
            }
        }
    }
}

// ============================================================================
// File Access Permission Dialog
// ============================================================================

/// Props for the file access permission dialog.
#[derive(Props, Clone, PartialEq)]
pub struct FileAccessDialogProps {
    /// Whether the dialog is open.
    pub open: bool,
    /// The file path requesting access.
    pub path: PathBuf,
    /// Whether read-only access is requested (vs read-write).
    #[props(default = true)]
    pub read_only: bool,
    /// Callback when a decision is made.
    pub on_result: EventHandler<PermissionResult>,
}

/// File access permission dialog component.
#[component]
pub fn FileAccessDialog(props: FileAccessDialogProps) -> Element {
    let on_result = props.on_result;
    let path_display = props.path.display().to_string();
    let access_type = if props.read_only {
        "read"
    } else {
        "read and write"
    };

    let on_close = move |_| on_result.call(PermissionResult::Dismissed);

    let handle_allow = {
        let path = props.path.clone();
        move |_| {
            info!(target: "ui.drive.permission", path = ?path, "File access granted");
            on_result.call(PermissionResult::Granted);
        }
    };

    let handle_deny = {
        let path = props.path.clone();
        move |_| {
            debug!(target: "ui.drive.permission", path = ?path, "File access denied");
            on_result.call(PermissionResult::Denied);
        }
    };

    rsx! {
        ModalDialog {
            open: props.open,
            on_close: on_close,
            title: "File Access Request",
            icon: rsx! {
                div {
                    class: "w-12 h-12 rounded-full bg-blue-500/20 flex items-center justify-center",
                    svg {
                        class: "w-6 h-6 text-blue-400",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        stroke_width: "2",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                        }
                    }
                }
            },
            // Description
            p {
                class: "text-slate-400 text-sm mb-4",
                "This operation requires "
                span { class: "text-slate-200 font-medium", "{access_type}" }
                " access to:"
            }
            // File path
            div {
                class: "bg-slate-800 rounded-lg px-3 py-2 mb-6 font-mono text-xs text-slate-300 break-all",
                "{path_display}"
            }
            // Actions
            div {
                class: "flex justify-end gap-3",
                button {
                    r#type: "button",
                    class: "px-4 py-2 rounded-lg text-sm font-medium text-slate-300 hover:bg-slate-800 transition",
                    onclick: handle_deny,
                    "Deny"
                }
                button {
                    r#type: "button",
                    class: "px-4 py-2 rounded-lg text-sm font-medium bg-blue-500 text-white hover:bg-blue-400 transition",
                    onclick: handle_allow,
                    "Allow"
                }
            }
        }
    }
}

// ============================================================================
// Storage Quota Warning Dialog
// ============================================================================

/// Props for the storage quota warning dialog.
#[derive(Props, Clone, PartialEq)]
pub struct QuotaWarningDialogProps {
    /// Whether the dialog is open.
    pub open: bool,
    /// Bytes required for the operation.
    pub required_bytes: u64,
    /// Bytes currently available.
    pub available_bytes: u64,
    /// Callback when a decision is made.
    pub on_result: EventHandler<PermissionResult>,
}

/// Storage quota warning dialog component.
#[component]
pub fn QuotaWarningDialog(props: QuotaWarningDialogProps) -> Element {
    let on_result = props.on_result;
    let required = format_size(props.required_bytes);
    let available = format_size(props.available_bytes);
    let shortage = format_size(props.required_bytes.saturating_sub(props.available_bytes));

    let on_close = move |_| on_result.call(PermissionResult::Dismissed);

    let handle_continue = move |_| {
        info!(
            target: "ui.drive.permission",
            required = props.required_bytes,
            available = props.available_bytes,
            "User continued despite quota warning"
        );
        on_result.call(PermissionResult::Granted);
    };

    let handle_cancel = move |_| {
        debug!(target: "ui.drive.permission", "User cancelled due to quota warning");
        on_result.call(PermissionResult::Denied);
    };

    let is_exceeded = props.required_bytes > props.available_bytes;

    rsx! {
        ModalDialog {
            open: props.open,
            on_close: on_close,
            title: if is_exceeded { "Storage Quota Exceeded" } else { "Storage Warning" },
            icon: rsx! {
                div {
                    class: if is_exceeded { "w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center" } else { "w-12 h-12 rounded-full bg-amber-500/20 flex items-center justify-center" },
                    svg {
                        class: if is_exceeded { "w-6 h-6 text-red-400" } else { "w-6 h-6 text-amber-400" },
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        stroke_width: "2",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                        }
                    }
                }
            },
            // Description
            if is_exceeded {
                p {
                    class: "text-slate-400 text-sm mb-4",
                    "This operation requires more storage than is available. You need to free up space or upgrade your quota."
                }
            } else {
                p {
                    class: "text-slate-400 text-sm mb-4",
                    "This operation will use most of your remaining storage space."
                }
            }
            // Storage details
            div {
                class: "space-y-2 mb-6",
                div {
                    class: "flex justify-between text-sm",
                    span { class: "text-slate-500", "Required:" }
                    span { class: "text-slate-300", "{required}" }
                }
                div {
                    class: "flex justify-between text-sm",
                    span { class: "text-slate-500", "Available:" }
                    span { class: "text-slate-300", "{available}" }
                }
                if is_exceeded {
                    div {
                        class: "flex justify-between text-sm",
                        span { class: "text-slate-500", "Shortage:" }
                        span { class: "text-red-400 font-medium", "{shortage}" }
                    }
                }
            }
            // Actions
            div {
                class: "flex justify-end gap-3",
                button {
                    r#type: "button",
                    class: "px-4 py-2 rounded-lg text-sm font-medium text-slate-300 hover:bg-slate-800 transition",
                    onclick: handle_cancel,
                    "Cancel"
                }
                if !is_exceeded {
                    button {
                        r#type: "button",
                        class: "px-4 py-2 rounded-lg text-sm font-medium bg-amber-500 text-slate-900 hover:bg-amber-400 transition",
                        onclick: handle_continue,
                        "Continue Anyway"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Dangerous File Warning Dialog
// ============================================================================

/// Props for the dangerous file warning dialog.
#[derive(Props, Clone, PartialEq)]
pub struct DangerousFileDialogProps {
    /// Whether the dialog is open.
    pub open: bool,
    /// The file name.
    pub file_name: String,
    /// The type of dangerous file.
    pub file_type: DangerousFileType,
    /// Whether this is an upload (true) or download (false).
    #[props(default = false)]
    pub is_upload: bool,
    /// Callback when a decision is made.
    pub on_result: EventHandler<PermissionResult>,
}

/// Dangerous file warning dialog component.
#[component]
pub fn DangerousFileDialog(props: DangerousFileDialogProps) -> Element {
    let on_result = props.on_result;
    let warning_level = props.file_type.warning_level();
    let file_desc = props.file_type.description();
    let action = if props.is_upload {
        "upload"
    } else {
        "download"
    };

    let on_close = move |_| on_result.call(PermissionResult::Dismissed);

    let handle_continue = {
        let file_name = props.file_name.clone();
        move |_| {
            info!(
                target: "ui.drive.permission",
                file = %file_name,
                "User accepted dangerous file warning"
            );
            on_result.call(PermissionResult::Granted);
        }
    };

    let handle_cancel = {
        let file_name = props.file_name.clone();
        move |_| {
            debug!(
                target: "ui.drive.permission",
                file = %file_name,
                "User cancelled dangerous file"
            );
            on_result.call(PermissionResult::Denied);
        }
    };

    rsx! {
        ModalDialog {
            open: props.open,
            on_close: on_close,
            title: "Security Warning",
            icon: rsx! {
                div {
                    class: "w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center",
                    svg {
                        class: "w-6 h-6 text-red-400",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        stroke_width: "2",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                        }
                    }
                }
            },
            // Description
            p {
                class: "text-slate-400 text-sm mb-2",
                "You are about to {action} a "
                span { class: "text-red-400 font-medium", "{file_desc}" }
                ":"
            }
            // File name
            div {
                class: "bg-slate-800 rounded-lg px-3 py-2 mb-4 font-mono text-sm text-slate-300 break-all",
                "{props.file_name}"
            }
            // Warning message based on level
            div {
                class: "mb-6 p-3 rounded-lg bg-red-500/10 border border-red-500/30",
                if warning_level >= 3 {
                    p {
                        class: "text-red-400 text-sm",
                        "⚠️ This file type can execute code on your computer. Only proceed if you trust the source."
                    }
                } else if warning_level >= 2 {
                    p {
                        class: "text-amber-400 text-sm",
                        "⚠️ This file type may contain executable content. Exercise caution."
                    }
                } else {
                    p {
                        class: "text-amber-400 text-sm",
                        "⚠️ This file type has been flagged as potentially unsafe."
                    }
                }
            }
            // Actions
            div {
                class: "flex justify-end gap-3",
                button {
                    r#type: "button",
                    class: "px-4 py-2 rounded-lg text-sm font-medium bg-slate-700 text-slate-200 hover:bg-slate-600 transition",
                    onclick: handle_cancel,
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "px-4 py-2 rounded-lg text-sm font-medium bg-red-500 text-white hover:bg-red-400 transition",
                    onclick: handle_continue,
                    "I understand, proceed"
                }
            }
        }
    }
}

// ============================================================================
// Network Share Permission Dialog
// ============================================================================

/// Props for the network share permission dialog.
#[derive(Props, Clone, PartialEq)]
pub struct NetworkShareDialogProps {
    /// Whether the dialog is open.
    pub open: bool,
    /// The share URL being created.
    pub share_url: String,
    /// Whether the share is password protected.
    #[props(default = false)]
    pub has_password: bool,
    /// Optional expiry description.
    #[props(default)]
    pub expiry: Option<String>,
    /// Callback when a decision is made.
    pub on_result: EventHandler<PermissionResult>,
}

/// Network share permission dialog component.
#[component]
pub fn NetworkShareDialog(props: NetworkShareDialogProps) -> Element {
    let on_result = props.on_result;

    let on_close = move |_| on_result.call(PermissionResult::Dismissed);

    let handle_allow = {
        let url = props.share_url.clone();
        move |_| {
            info!(target: "ui.drive.permission", url = %url, "Network share allowed");
            on_result.call(PermissionResult::Granted);
        }
    };

    let handle_deny = move |_| {
        debug!(target: "ui.drive.permission", "Network share denied");
        on_result.call(PermissionResult::Denied);
    };

    rsx! {
        ModalDialog {
            open: props.open,
            on_close: on_close,
            title: "Share File Externally?",
            icon: rsx! {
                div {
                    class: "w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center",
                    svg {
                        class: "w-6 h-6 text-emerald-400",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke: "currentColor",
                        stroke_width: "2",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"
                        }
                    }
                }
            },
            // Description
            p {
                class: "text-slate-400 text-sm mb-4",
                "This will create a publicly accessible link. Anyone with this link can access the file."
            }
            // Share details
            div {
                class: "space-y-2 mb-6",
                // Protection status
                div {
                    class: "flex items-center gap-2 text-sm",
                    if props.has_password {
                        span { class: "text-emerald-400", "🔒" }
                        span { class: "text-slate-300", "Password protected" }
                    } else {
                        span { class: "text-amber-400", "🔓" }
                        span { class: "text-slate-300", "No password" }
                    }
                }
                // Expiry status
                if let Some(expiry) = &props.expiry {
                    div {
                        class: "flex items-center gap-2 text-sm",
                        span { class: "text-blue-400", "⏱️" }
                        span { class: "text-slate-300", "Expires: {expiry}" }
                    }
                } else {
                    div {
                        class: "flex items-center gap-2 text-sm",
                        span { class: "text-amber-400", "∞" }
                        span { class: "text-slate-300", "Never expires" }
                    }
                }
            }
            // Actions
            div {
                class: "flex justify-end gap-3",
                button {
                    r#type: "button",
                    class: "px-4 py-2 rounded-lg text-sm font-medium text-slate-300 hover:bg-slate-800 transition",
                    onclick: handle_deny,
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "px-4 py-2 rounded-lg text-sm font-medium bg-emerald-500 text-slate-900 hover:bg-emerald-400 transition",
                    onclick: handle_allow,
                    "Create Share Link"
                }
            }
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Format size for display.
fn format_size(bytes: u64) -> String {
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_result_is_granted() {
        assert!(PermissionResult::Granted.is_granted());
        assert!(!PermissionResult::Denied.is_granted());
        assert!(!PermissionResult::Dismissed.is_granted());
    }

    #[test]
    fn dangerous_file_type_description() {
        assert!(!DangerousFileType::Executable.description().is_empty());
        assert!(!DangerousFileType::Script.description().is_empty());
        assert!(!DangerousFileType::ArchiveWithExecutables.description().is_empty());
        assert!(!DangerousFileType::SystemFile.description().is_empty());
        assert!(!DangerousFileType::Unknown.description().is_empty());
    }

    #[test]
    fn dangerous_file_type_warning_levels() {
        assert!(DangerousFileType::Executable.warning_level() > DangerousFileType::Script.warning_level());
        assert!(DangerousFileType::Script.warning_level() >= DangerousFileType::Unknown.warning_level());
    }

    #[test]
    fn is_dangerous_extension_executables() {
        assert_eq!(
            is_dangerous_extension("exe"),
            Some(DangerousFileType::Executable)
        );
        assert_eq!(
            is_dangerous_extension("EXE"),
            Some(DangerousFileType::Executable)
        );
        assert_eq!(
            is_dangerous_extension("dmg"),
            Some(DangerousFileType::Executable)
        );
        assert_eq!(
            is_dangerous_extension("app"),
            Some(DangerousFileType::Executable)
        );
    }

    #[test]
    fn is_dangerous_extension_scripts() {
        assert_eq!(
            is_dangerous_extension("sh"),
            Some(DangerousFileType::Script)
        );
        assert_eq!(
            is_dangerous_extension("bat"),
            Some(DangerousFileType::Script)
        );
        assert_eq!(
            is_dangerous_extension("ps1"),
            Some(DangerousFileType::Script)
        );
    }

    #[test]
    fn is_dangerous_extension_archives() {
        assert_eq!(
            is_dangerous_extension("zip"),
            Some(DangerousFileType::ArchiveWithExecutables)
        );
        assert_eq!(
            is_dangerous_extension("tar"),
            Some(DangerousFileType::ArchiveWithExecutables)
        );
    }

    #[test]
    fn is_dangerous_extension_system() {
        assert_eq!(
            is_dangerous_extension("dll"),
            Some(DangerousFileType::SystemFile)
        );
        assert_eq!(
            is_dangerous_extension("plist"),
            Some(DangerousFileType::SystemFile)
        );
    }

    #[test]
    fn is_dangerous_extension_safe() {
        assert_eq!(is_dangerous_extension("txt"), None);
        assert_eq!(is_dangerous_extension("pdf"), None);
        assert_eq!(is_dangerous_extension("png"), None);
        assert_eq!(is_dangerous_extension("md"), None);
    }

    #[test]
    fn format_size_units() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn permission_type_variants() {
        let file_access = PermissionType::FileAccess {
            path: PathBuf::from("/test"),
            read_only: true,
        };
        assert!(matches!(file_access, PermissionType::FileAccess { .. }));

        let network = PermissionType::NetworkShare {
            share_url: "https://example.com".to_string(),
        };
        assert!(matches!(network, PermissionType::NetworkShare { .. }));

        let quota = PermissionType::StorageQuota {
            required_bytes: 100,
            available_bytes: 50,
        };
        assert!(matches!(quota, PermissionType::StorageQuota { .. }));

        let dangerous = PermissionType::DangerousFile {
            file_name: "test.exe".to_string(),
            file_type: DangerousFileType::Executable,
        };
        assert!(matches!(dangerous, PermissionType::DangerousFile { .. }));
    }
}
