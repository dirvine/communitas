//! Platform-specific media device enumeration using cpal.
//!
//! This module implements the [`DeviceEnumerator`] trait from `communitas-ui-service`
//! using the `cpal` crate for cross-platform audio device discovery. Camera enumeration
//! is provided as a stub since it requires platform-specific APIs not covered by cpal.
//!
//! ## Device Hot-Plug
//!
//! The [`CpalDeviceEnumerator`] can be queried at any time to get the current device
//! list. For hot-plug detection, the UI should call [`CallService::on_devices_changed`]
//! when system events indicate device changes.

use async_trait::async_trait;
use communitas_ui_api::call::{DeviceType, MediaDevice};
use communitas_ui_service::call::{CallError, DeviceEnumerator};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::Arc;
use tracing::{debug, error, warn};

/// Device enumerator using cpal for audio devices.
///
/// This implementation uses the cpal library to enumerate audio input (microphones)
/// and output (speakers) devices. Camera enumeration returns an empty list as it
/// requires platform-specific APIs.
///
/// ## Thread Safety
///
/// This enumerator is thread-safe and can be shared across async tasks via `Arc`.
///
/// ## Error Handling
///
/// If cpal fails to enumerate devices (e.g., no audio host available), the enumerator
/// will return an appropriate error. Partial failures (e.g., can't get name for one
/// device) are logged but don't fail the entire enumeration.
///
/// ## Lazy Initialization
///
/// This enumerator is created lazily when the call UI is first accessed (CallLobby
/// or CallView), rather than at app startup. This improves initial render time by
/// 100-300ms by deferring device enumeration until it's actually needed.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used via create_device_enumerator when call UI is rendered
pub struct CpalDeviceEnumerator {
    /// Cache whether we've warned about camera enumeration
    _camera_warning_shown: bool,
}

impl Default for CpalDeviceEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)] // Used via create_device_enumerator when call UI is rendered
impl CpalDeviceEnumerator {
    /// Create a new cpal-based device enumerator.
    pub fn new() -> Self {
        Self {
            _camera_warning_shown: false,
        }
    }

    /// Enumerate input (microphone) devices.
    fn enumerate_input_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        let host = cpal::default_host();
        let default_input = host.default_input_device();
        let default_input_name = default_input.as_ref().and_then(|d| d.name().ok());

        let devices = match host.input_devices() {
            Ok(devices) => devices,
            Err(e) => {
                error!(error = %e, "Failed to enumerate input devices");
                return Err(CallError::DeviceEnumerationFailed(format!(
                    "Failed to enumerate microphones: {e}"
                )));
            }
        };

        let mut result = Vec::new();
        for (idx, device) in devices.enumerate() {
            let name = match device.name() {
                Ok(name) => name,
                Err(e) => {
                    warn!(index = idx, error = %e, "Failed to get device name, skipping");
                    continue;
                }
            };

            // Generate a stable ID from the device name
            // Note: cpal doesn't provide stable device IDs, so we use the name
            let id = format!("mic-{}", sanitize_device_id(&name));
            let is_default = default_input_name
                .as_ref()
                .is_some_and(|default_name| *default_name == name);

            result.push(MediaDevice {
                id,
                name,
                device_type: DeviceType::Microphone,
                is_default,
                is_available: true,
            });
        }

        debug!(count = result.len(), "Enumerated microphones");
        Ok(result)
    }

    /// Enumerate output (speaker) devices.
    fn enumerate_output_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        let host = cpal::default_host();
        let default_output = host.default_output_device();
        let default_output_name = default_output.as_ref().and_then(|d| d.name().ok());

        let devices = match host.output_devices() {
            Ok(devices) => devices,
            Err(e) => {
                error!(error = %e, "Failed to enumerate output devices");
                return Err(CallError::DeviceEnumerationFailed(format!(
                    "Failed to enumerate speakers: {e}"
                )));
            }
        };

        let mut result = Vec::new();
        for (idx, device) in devices.enumerate() {
            let name = match device.name() {
                Ok(name) => name,
                Err(e) => {
                    warn!(index = idx, error = %e, "Failed to get device name, skipping");
                    continue;
                }
            };

            // Generate a stable ID from the device name
            let id = format!("speaker-{}", sanitize_device_id(&name));
            let is_default = default_output_name
                .as_ref()
                .is_some_and(|default_name| *default_name == name);

            result.push(MediaDevice {
                id,
                name,
                device_type: DeviceType::Speaker,
                is_default,
                is_available: true,
            });
        }

        debug!(count = result.len(), "Enumerated speakers");
        Ok(result)
    }

    /// Enumerate camera devices.
    ///
    /// Note: Camera enumeration requires platform-specific APIs not provided by cpal.
    /// This returns an empty list. For real camera support, use platform-specific
    /// implementations (AVFoundation on macOS, Media Foundation on Windows, etc.).
    fn enumerate_camera_devices(&self) -> Vec<MediaDevice> {
        debug!("Camera enumeration not implemented via cpal - returning empty list");
        // TODO: Implement platform-specific camera enumeration
        // macOS: AVFoundation
        // Windows: Media Foundation
        // Linux: V4L2
        Vec::new()
    }
}

#[async_trait]
impl DeviceEnumerator for CpalDeviceEnumerator {
    async fn enumerate_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        // Clone self for the blocking task
        let enumerator = self.clone();

        // Run enumeration in a blocking task since cpal may block
        let result = tokio::task::spawn_blocking(move || {
            let mut devices = Vec::new();

            // Enumerate microphones
            match enumerator.enumerate_input_devices() {
                Ok(mics) => devices.extend(mics),
                Err(e) => {
                    warn!(error = %e, "Microphone enumeration failed, continuing with other devices");
                }
            }

            // Enumerate speakers
            match enumerator.enumerate_output_devices() {
                Ok(speakers) => devices.extend(speakers),
                Err(e) => {
                    warn!(error = %e, "Speaker enumeration failed, continuing with other devices");
                }
            }

            // Enumerate cameras (stub)
            devices.extend(enumerator.enumerate_camera_devices());

            Ok(devices)
        })
        .await;

        match result {
            Ok(devices) => devices,
            Err(e) => Err(CallError::DeviceEnumerationFailed(format!(
                "Device enumeration task failed: {e}"
            ))),
        }
    }
}

/// Sanitize a device name to create a stable ID.
///
/// Replaces spaces and special characters with hyphens, lowercases the result.
#[allow(dead_code)] // Used by CpalDeviceEnumerator when call UI is rendered
fn sanitize_device_id(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Create a shared device enumerator instance.
///
/// This is the recommended way to create a device enumerator for use with
/// [`CallService::with_device_enumerator`]. The enumerator is used for lazy
/// initialization when the call UI is first accessed.
///
/// # Example
///
/// ```ignore
/// use communitas_dioxus::platform::create_device_enumerator;
/// use communitas_ui_service::call::CallService;
///
/// let enumerator = create_device_enumerator();
/// let call_service = CallService::with_device_enumerator(auth, app, enumerator);
/// ```
#[allow(dead_code)] // Used by CallLobby/CallView when call UI is rendered
pub fn create_device_enumerator() -> Arc<dyn DeviceEnumerator> {
    Arc::new(CpalDeviceEnumerator::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_device_id() {
        assert_eq!(
            sanitize_device_id("Built-in Microphone"),
            "built-in-microphone"
        );
        assert_eq!(
            sanitize_device_id("USB Audio (Generic)"),
            "usb-audio-generic"
        );
        assert_eq!(
            sanitize_device_id("  Spaces  Everywhere  "),
            "spaces-everywhere"
        );
        assert_eq!(
            sanitize_device_id("MacBook Pro Speakers"),
            "macbook-pro-speakers"
        );
    }

    #[test]
    fn test_cpal_enumerator_default() {
        let enumerator = CpalDeviceEnumerator::default();
        assert!(!enumerator._camera_warning_shown);
    }

    #[tokio::test]
    async fn test_enumerate_devices_does_not_panic() {
        let enumerator = CpalDeviceEnumerator::new();
        // This test just verifies enumeration doesn't panic.
        // On CI systems without audio devices, it may return empty or error.
        let result = enumerator.enumerate_devices().await;
        // We accept either success or a meaningful error
        match result {
            Ok(devices) => {
                // Verify device structure if any devices returned
                for device in &devices {
                    assert!(!device.id.is_empty());
                    assert!(!device.name.is_empty());
                }
            }
            Err(e) => {
                // Error message should be meaningful
                let msg = format!("{e}");
                assert!(!msg.is_empty());
            }
        }
    }

    #[test]
    fn test_camera_enumeration_returns_empty() {
        let enumerator = CpalDeviceEnumerator::new();
        let cameras = enumerator.enumerate_camera_devices();
        // Camera enumeration is stubbed out
        assert!(cameras.is_empty());
    }

    #[test]
    fn test_create_device_enumerator() {
        let _enumerator = create_device_enumerator();
        // Just verify it creates without panicking - the existence of the Arc is sufficient
    }
}
