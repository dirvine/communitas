//! Platform-specific screen source enumeration using scap.
//!
//! This module implements the [`ScreenSourceEnumerator`] trait from `communitas-ui-service`
//! using the `scap` crate for cross-platform screen and window discovery.
//!
//! ## Platform Support
//!
//! - **macOS**: Uses ScreenCaptureKit (macOS 12.3+)
//! - **Windows**: Uses Windows Graphics Capture
//! - **Linux**: Uses PipeWire
//!
//! ## Permissions
//!
//! Screen capture requires user permission on all platforms. The enumerator checks
//! permission status and returns an empty list gracefully if denied.

use async_trait::async_trait;
use communitas_ui_api::call::ScreenShareSource;
use communitas_ui_service::call::{CallError, ScreenSourceEnumerator};
use std::sync::Arc;
use tracing::{debug, warn};

/// Screen source enumerator using scap for cross-platform screen capture discovery.
///
/// This implementation uses the scap library to enumerate available monitors and
/// windows for screen sharing. It handles permission checks and graceful fallback.
///
/// ## Thread Safety
///
/// This enumerator is thread-safe and can be shared across async tasks via `Arc`.
///
/// ## Permission Handling
///
/// The enumerator checks for screen recording permission before enumerating.
/// If permission is not granted, it returns an empty list rather than an error.
/// Use `scap::request_permission()` to prompt the user if needed.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // Used by create_screen_source_enumerator when screen share picker is shown
pub struct ScapScreenSourceEnumerator;

impl ScapScreenSourceEnumerator {
    /// Create a new scap-based screen source enumerator.
    pub fn new() -> Self {
        Self
    }

    /// Check if screen capture is supported on this platform.
    #[allow(dead_code)]
    pub fn is_supported() -> bool {
        scap::is_supported()
    }

    /// Check if we have screen recording permission.
    #[allow(dead_code)]
    pub fn has_permission() -> bool {
        scap::has_permission()
    }

    /// Request screen recording permission from the user.
    ///
    /// On macOS, this opens System Preferences to the Privacy settings.
    /// On other platforms, this may show a system dialog.
    #[allow(dead_code)]
    pub fn request_permission() {
        scap::request_permission();
    }

    /// Enumerate screen sources synchronously.
    fn enumerate_sources_sync() -> Result<Vec<ScreenShareSource>, CallError> {
        // Check if platform is supported
        if !scap::is_supported() {
            debug!("Screen capture not supported on this platform");
            return Ok(Vec::new());
        }

        // Check permission - return empty list if denied (not an error)
        if !scap::has_permission() {
            debug!("Screen capture permission not granted, returning empty list");
            return Ok(Vec::new());
        }

        // Get all targets (monitors and windows)
        let targets = scap::get_all_targets();

        let mut sources = Vec::new();
        let mut monitor_count = 0;

        for target in targets {
            match target {
                scap::Target::Display(display) => {
                    monitor_count += 1;
                    let id = format!("monitor-{}", display.id);
                    let name = if display.title.is_empty() {
                        format!("Display {monitor_count}")
                    } else {
                        display.title
                    };
                    let is_primary = monitor_count == 1; // First display is primary

                    sources.push(ScreenShareSource::monitor(id, name, is_primary));
                }
                scap::Target::Window(window) => {
                    let id = format!("window-{}", window.id);
                    let name = if window.title.is_empty() {
                        "Untitled Window".to_string()
                    } else {
                        window.title
                    };
                    // scap doesn't provide app_name, use title as fallback
                    let app_name = name.clone();

                    sources.push(ScreenShareSource::window(id, name, app_name));
                }
            }
        }

        debug!(
            monitors = monitor_count,
            windows = sources.len() - monitor_count,
            "Enumerated screen sources"
        );
        Ok(sources)
    }
}

#[async_trait]
impl ScreenSourceEnumerator for ScapScreenSourceEnumerator {
    async fn enumerate_sources(&self) -> Result<Vec<ScreenShareSource>, CallError> {
        // Run enumeration in a blocking task since scap may block
        let result = tokio::task::spawn_blocking(Self::enumerate_sources_sync).await;

        match result {
            Ok(sources) => sources,
            Err(e) => {
                warn!(error = %e, "Screen source enumeration task panicked");
                Err(CallError::DeviceEnumerationFailed(format!(
                    "Screen source enumeration failed: {e}"
                )))
            }
        }
    }
}

/// Create a shared screen source enumerator instance.
///
/// This is the recommended way to create a screen source enumerator for use with
/// `CallService::with_enumerators` from `communitas_ui_service`.
///
/// # Example
///
/// ```ignore
/// use communitas_dioxus::platform::{create_device_enumerator, create_screen_source_enumerator};
/// use communitas_ui_service::call::CallService;
///
/// let device_enum = create_device_enumerator();
/// let screen_enum = create_screen_source_enumerator();
/// let call_service = CallService::with_enumerators(auth, app, device_enum, screen_enum, None);
/// ```
#[allow(dead_code)] // Used by CallLobby when screen share picker is shown
pub fn create_screen_source_enumerator() -> Arc<dyn ScreenSourceEnumerator> {
    Arc::new(ScapScreenSourceEnumerator::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scap_enumerator_creation() {
        let _enumerator = ScapScreenSourceEnumerator::new();
        // Just verify it creates without panicking
    }

    #[test]
    fn test_create_screen_source_enumerator() {
        let _enumerator = create_screen_source_enumerator();
        // Just verify it creates without panicking
    }

    #[test]
    fn test_is_supported_does_not_panic() {
        // This should not panic regardless of platform
        let _supported = ScapScreenSourceEnumerator::is_supported();
    }

    #[test]
    fn test_has_permission_does_not_panic() {
        // This should not panic regardless of permission state
        let _has_perm = ScapScreenSourceEnumerator::has_permission();
    }

    #[tokio::test]
    async fn test_enumerate_sources_does_not_panic() {
        let enumerator = ScapScreenSourceEnumerator::new();
        // This test verifies enumeration doesn't panic, even if permission denied
        let result = enumerator.enumerate_sources().await;
        // We accept either success (with any number of sources) or meaningful error
        match result {
            Ok(sources) => {
                // Verify source structure if any returned
                for source in &sources {
                    assert!(!source.id.is_empty());
                    assert!(!source.name.is_empty());
                }
            }
            Err(e) => {
                // Error message should be meaningful
                let msg = format!("{e}");
                assert!(!msg.is_empty());
            }
        }
    }
}
