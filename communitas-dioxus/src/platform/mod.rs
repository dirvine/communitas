//! Platform-specific functionality for Communitas.
//!
//! This module provides platform abstractions for:
//! - WebView availability detection

mod webview;

pub use webview::check_webview_available;
