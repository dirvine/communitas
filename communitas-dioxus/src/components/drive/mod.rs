//! Drive file browser UI components for Communitas.
//!
//! This module provides a complete drive file browser implementation with:
//! - Three-panel layout (tree view, file list, preview)
//! - Disk type tabs (Private/Public/Shared)
//! - Upload with progress and drag-drop
//! - Download with checksum verification
//! - Quota meter with usage warnings

mod browser;
pub(crate) mod download_manager;
pub(crate) mod file_list;
pub(crate) mod preview_panel;
pub(crate) mod quota_meter;
pub(crate) mod tree_view;
pub(crate) mod upload_progress;

// Public exports for use in routes
pub use browser::DriveBrowser;
