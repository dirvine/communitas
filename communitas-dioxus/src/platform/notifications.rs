//! System notification support for Communitas.
//!
//! Provides cross-platform desktop notifications for:
//! - Incoming calls
//! - Missed calls
//! - Call endings
//!
//! On macOS, uses the native notification center via `notify-rust`.
//!
//! Note: These functions are currently exported but not yet integrated with
//! the incoming call detection flow. Will be used when WebRTC call receiving
//! is fully implemented.

use notify_rust::{Notification, Timeout};
use tracing::info;

/// Result type for notification operations.
pub type NotificationResult = Result<(), NotificationError>;

/// Errors that can occur when showing notifications.
#[derive(Debug)]
pub enum NotificationError {
    /// Failed to show the notification.
    ShowFailed(String),
}

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShowFailed(msg) => write!(f, "Failed to show notification: {msg}"),
        }
    }
}

impl std::error::Error for NotificationError {}

/// Show a system notification for an incoming call.
pub fn show_incoming_call_notification(caller_name: &str, call_type: &str) -> NotificationResult {
    info!(target: "platform.notifications", caller = %caller_name, call_type = %call_type, "showing incoming call notification");

    Notification::new()
        .summary(&format!("Incoming {call_type}"))
        .body(&format!("{caller_name} is calling"))
        .appname("Communitas")
        .icon("call-start")
        .timeout(Timeout::Never) // Keep visible until user acts
        .show()
        .map_err(|e: notify_rust::error::Error| NotificationError::ShowFailed(e.to_string()))?;

    Ok(())
}

/// Show a system notification for a missed call.
pub fn show_missed_call_notification(caller_name: &str, count: usize) -> NotificationResult {
    info!(target: "platform.notifications", caller = %caller_name, count = count, "showing missed call notification");

    let summary = if count == 1 {
        "Missed Call".to_string()
    } else {
        format!("{count} Missed Calls")
    };

    let body = if count == 1 {
        format!("You missed a call from {caller_name}")
    } else {
        format!("You missed {count} calls from {caller_name}")
    };

    Notification::new()
        .summary(&summary)
        .body(&body)
        .appname("Communitas")
        .icon("call-missed")
        .timeout(Timeout::Milliseconds(10000)) // 10 seconds
        .show()
        .map_err(|e: notify_rust::error::Error| NotificationError::ShowFailed(e.to_string()))?;

    Ok(())
}

/// Show a system notification that a call has ended.
pub fn show_call_ended_notification(peer_name: &str, duration_secs: u64) -> NotificationResult {
    info!(target: "platform.notifications", peer = %peer_name, duration = duration_secs, "showing call ended notification");

    let duration_str = format_duration(duration_secs);

    Notification::new()
        .summary("Call Ended")
        .body(&format!("Call with {peer_name} - {duration_str}"))
        .appname("Communitas")
        .icon("call-stop")
        .timeout(Timeout::Milliseconds(5000)) // 5 seconds
        .show()
        .map_err(|e: notify_rust::error::Error| NotificationError::ShowFailed(e.to_string()))?;

    Ok(())
}

/// Format a duration in seconds to a human-readable string.
fn format_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_seconds_only() {
        assert_eq!(format_duration(45), "45s");
    }

    #[test]
    fn format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(125), "2m 5s");
    }

    #[test]
    fn format_duration_hours_and_minutes() {
        assert_eq!(format_duration(3725), "1h 2m");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "0s");
    }
}
