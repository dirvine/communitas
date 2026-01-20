//! Shared utility functions for UI services.

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in milliseconds since Unix epoch.
///
/// Returns 0 if the system clock is before the Unix epoch (should never happen
/// in practice, but we handle it gracefully rather than panicking).
pub fn current_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_is_positive() {
        let ts = current_timestamp_millis();
        assert!(ts > 0);
    }

    #[test]
    fn timestamp_is_reasonable() {
        // Should be after 2020-01-01 (1577836800000 ms)
        let ts = current_timestamp_millis();
        assert!(ts > 1_577_836_800_000);
    }
}
