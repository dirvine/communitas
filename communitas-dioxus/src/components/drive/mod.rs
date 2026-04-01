// SPDX-License-Identifier: MIT OR Apache-2.0

//! Drive file browser UI components for Communitas.
//!
//! This module provides a complete drive file browser implementation with:
//! - Three-panel layout (tree view, file list, preview)
//! - Disk type tabs (Private/Public/Shared)
//! - Upload with progress and drag-drop
//! - Download with checksum verification
//! - Quota meter with usage warnings
//! - Native file picker dialogs
//! - Permission dialogs for security-sensitive operations

mod browser;
pub(crate) mod download_manager;
pub(crate) mod file_list;
pub(crate) mod file_picker;
pub(crate) mod permission_dialog;
pub(crate) mod preview_panel;
pub(crate) mod quota_meter;
pub(crate) mod tree_view;
pub(crate) mod upload_progress;

// Public exports for use in routes
pub use browser::DriveBrowser;

// File picker exports for external use (download dialogs, etc.)
#[allow(unused_imports)]
pub use file_picker::{
    DropZone, FileFilter, FilePickerButton, PickerConfig, PickerResult, open_file_picker,
};

// Permission dialog exports for security-sensitive operations
#[allow(unused_imports)]
pub use permission_dialog::{
    DangerousFileDialog, DangerousFileDialogProps, DangerousFileType, FileAccessDialog,
    FileAccessDialogProps, NetworkShareDialog, NetworkShareDialogProps, PermissionResult,
    PermissionType, QuotaWarningDialog, QuotaWarningDialogProps, is_dangerous_extension,
};
