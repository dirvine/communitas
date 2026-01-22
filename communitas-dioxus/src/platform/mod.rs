//! Platform-specific functionality for Communitas.
//!
//! This module provides platform abstractions for:
//! - WebView availability detection
//! - Media device enumeration (microphones, speakers, cameras)
//!
//! ## Device Enumeration
//!
//! Use [`create_device_enumerator`] to get a platform-appropriate device enumerator
//! for use with [`CallService::with_device_enumerator`].
//!
//! ```ignore
//! use communitas_dioxus::platform::create_device_enumerator;
//! use communitas_ui_service::call::CallService;
//!
//! let enumerator = create_device_enumerator();
//! let call_service = CallService::with_device_enumerator(auth, app, enumerator);
//! ```

mod device_enumerator;
mod webview;

pub use device_enumerator::create_device_enumerator;
pub use webview::check_webview_available;
