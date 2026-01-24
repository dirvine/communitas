//! Platform-specific functionality for Communitas.
//!
//! This module provides platform abstractions for:
//! - WebView availability detection
//! - Media device enumeration (microphones, speakers, cameras)
//! - System notifications (calls, missed calls)
//!
//! ## Device Enumeration
//!
//! Use [`create_device_enumerator`] to get a platform-appropriate device enumerator
//! for use with `CallService::with_device_enumerator` from `communitas_ui_service`.
//!
//! ```ignore
//! use communitas_dioxus::platform::create_device_enumerator;
//! use communitas_ui_service::call::CallService;
//!
//! let enumerator = create_device_enumerator();
//! let call_service = CallService::with_device_enumerator(auth, app, enumerator);
//! ```
//!
//! ## System Notifications
//!
//! Use the notification functions to show system-level notifications:
//! - [`show_incoming_call_notification`] - For incoming calls
//! - [`show_missed_call_notification`] - For missed calls
//! - [`show_call_ended_notification`] - When calls end

mod device_enumerator;
#[allow(dead_code)]
mod notifications;
mod screen_source_enumerator;
mod webview;

pub use device_enumerator::create_device_enumerator;
#[allow(unused_imports)] // Used by CallLobby when screen share picker is shown
pub use screen_source_enumerator::create_screen_source_enumerator;
// Notification functions will be used when call UI is fully integrated
#[allow(unused_imports)]
pub use notifications::{
    show_call_ended_notification, show_incoming_call_notification, show_missed_call_notification,
};
pub use webview::check_webview_available;
