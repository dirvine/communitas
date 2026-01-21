//! Integration tests for Phase 6.1 Auth Hardening features.
//!
//! Tests cover:
//! - Session expiration detection and refresh
//! - Multi-identity quick switch API
//! - Audit log integration with auth events
//! - Device fingerprinting stability and tracking
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use communitas_core::app::CommunitasApp;
use communitas_core::security::{AuditEventType, DeviceFingerprint, KnownDevices};
use communitas_ui_service::UiServices;
use communitas_ui_service::audit::{AuditService, parse_event_types};
use communitas_ui_service::auth::{AuthService, AuthStateSnapshot, RecentIdentity};
use communitas_ui_service::storage::UiStorage;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Stack size for test threads (8MB) to handle large async state machines.
const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Run a test with a larger stack size to avoid overflow.
fn run_with_large_stack<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(TEST_STACK_SIZE)
        .spawn(test_fn)
        .expect("Failed to spawn test thread")
        .join()
        .expect("Test thread panicked");
}

/// Helper to create UiServices with a dummy app for testing.
async fn make_services(temp: &TempDir) -> UiServices {
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let app = Arc::new(
        CommunitasApp::new(
            "test-word-word-word".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .unwrap(),
    );
    UiServices::new(storage, app).unwrap()
}

// =============================================================================
// Device Fingerprinting Tests
// =============================================================================

mod device_fingerprinting {
    use super::*;

    /// Test that device fingerprint generation produces stable, valid output.
    #[test]
    fn test_fingerprint_generation_stability() {
        let fp1 = DeviceFingerprint::current().unwrap();
        let fp2 = DeviceFingerprint::current().unwrap();

        // Fingerprints should be identical for the same machine
        assert_eq!(
            fp1.fingerprint, fp2.fingerprint,
            "Fingerprints should be stable across calls"
        );

        // Fingerprint should be 64 hex chars (Blake3 output)
        assert_eq!(
            fp1.fingerprint.len(),
            64,
            "Fingerprint should be 64 hex chars"
        );

        // All chars should be valid hex
        assert!(
            fp1.fingerprint.chars().all(|c| c.is_ascii_hexdigit()),
            "Fingerprint should contain only hex characters"
        );

        // Device name should be populated
        assert!(
            !fp1.device_name.is_empty(),
            "Device name should not be empty"
        );
    }

    /// Test fingerprint short_id returns first 16 chars.
    #[test]
    fn test_fingerprint_short_id() {
        let fp = DeviceFingerprint::current().unwrap();
        let short = fp.short_id();

        assert_eq!(short.len(), 16, "Short ID should be 16 chars");
        assert_eq!(
            short,
            &fp.fingerprint[..16],
            "Short ID should match first 16 chars of fingerprint"
        );
    }

    /// Test that fingerprint touch() updates last_seen timestamp.
    #[test]
    fn test_fingerprint_touch_updates_last_seen() {
        let mut fp = DeviceFingerprint::current().unwrap();
        let original_last_seen = fp.last_seen;

        // Small delay to ensure timestamp changes
        std::thread::sleep(Duration::from_millis(10));

        fp.touch();

        assert!(
            fp.last_seen > original_last_seen,
            "last_seen should be updated after touch()"
        );
    }

    /// Test KnownDevices tracking new vs existing devices.
    #[test]
    fn test_known_devices_tracking() {
        let mut known = KnownDevices::new();
        let fp = DeviceFingerprint::current().unwrap();

        // First add should indicate new device
        assert!(!known.is_known(&fp.fingerprint));
        let is_new = known.add_or_update(fp.clone());
        assert!(is_new, "First add should return true (new device)");
        assert!(known.is_known(&fp.fingerprint));

        // Second add should indicate existing device
        let is_new = known.add_or_update(fp.clone());
        assert!(!is_new, "Second add should return false (existing device)");
        assert_eq!(known.count(), 1, "Should still have only 1 device");
    }

    /// Test KnownDevices max limit eviction.
    #[test]
    fn test_known_devices_max_limit_eviction() {
        use chrono::Utc;

        let mut known = KnownDevices::with_max(2);

        // Create three devices with different timestamps
        let mut fp1 = DeviceFingerprint::current().unwrap();
        fp1.fingerprint = "a".repeat(64);
        fp1.last_seen = Utc::now() - chrono::Duration::hours(3); // Oldest

        let mut fp2 = DeviceFingerprint::current().unwrap();
        fp2.fingerprint = "b".repeat(64);
        fp2.last_seen = Utc::now() - chrono::Duration::hours(1);

        let mut fp3 = DeviceFingerprint::current().unwrap();
        fp3.fingerprint = "c".repeat(64);
        fp3.last_seen = Utc::now(); // Newest

        // Add first two devices
        known.add_or_update(fp1.clone());
        known.add_or_update(fp2.clone());
        assert_eq!(known.count(), 2);

        // Adding third should evict oldest (fp1)
        known.add_or_update(fp3.clone());
        assert_eq!(known.count(), 2, "Count should remain at max (2)");
        assert!(
            !known.is_known(&fp1.fingerprint),
            "Oldest device should be evicted"
        );
        assert!(known.is_known(&fp2.fingerprint), "fp2 should remain");
        assert!(known.is_known(&fp3.fingerprint), "fp3 should be added");
    }

    /// Test device removal from known devices.
    #[test]
    fn test_known_devices_removal() {
        let mut known = KnownDevices::new();
        let fp = DeviceFingerprint::current().unwrap();

        known.add_or_update(fp.clone());
        assert!(known.is_known(&fp.fingerprint));

        let removed = known.remove(&fp.fingerprint);
        assert!(removed.is_some(), "Remove should return the device");
        assert!(!known.is_known(&fp.fingerprint), "Device should be gone");
        assert_eq!(known.count(), 0);
    }
}

// =============================================================================
// Session Expiration Tests
// =============================================================================

mod session_expiration {
    use super::*;
    use communitas_ui_service::auth::AuthSession;

    /// Test AuthSession expires_soon() detection.
    #[test]
    fn test_session_expires_soon_detection() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Session expiring in 2 minutes (within 5 minute threshold)
        let session = AuthSession {
            pubkey_hex: "abcd1234".to_string(),
            four_words: "test-word-word-word".to_string(),
            display_name: "Test User".to_string(),
            device_name: "TestDevice".to_string(),
            expires_at: now + 2 * 60, // 2 minutes from now
        };

        assert!(
            session.expires_soon(),
            "Session expiring in 2 minutes should return expires_soon() = true"
        );
    }

    /// Test AuthSession not expiring soon.
    #[test]
    fn test_session_not_expiring_soon() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Session expiring in 8 hours (way beyond 5 minute threshold)
        let session = AuthSession {
            pubkey_hex: "abcd1234".to_string(),
            four_words: "test-word-word-word".to_string(),
            display_name: "Test User".to_string(),
            device_name: "TestDevice".to_string(),
            expires_at: now + 8 * 60 * 60, // 8 hours from now
        };

        assert!(
            !session.expires_soon(),
            "Session expiring in 8 hours should return expires_soon() = false"
        );
    }

    /// Test time_remaining() calculation.
    #[test]
    fn test_session_time_remaining() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = AuthSession {
            pubkey_hex: "abcd1234".to_string(),
            four_words: "test-word-word-word".to_string(),
            display_name: "Test User".to_string(),
            device_name: "TestDevice".to_string(),
            expires_at: now + 300, // 5 minutes from now
        };

        let remaining = session.time_remaining();
        // Allow some slack for test execution time
        assert!(
            remaining.as_secs() <= 300 && remaining.as_secs() >= 298,
            "time_remaining should be ~300 seconds, got {}",
            remaining.as_secs()
        );
    }

    /// Test expired session has zero time_remaining.
    #[test]
    fn test_expired_session_time_remaining() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = AuthSession {
            pubkey_hex: "abcd1234".to_string(),
            four_words: "test-word-word-word".to_string(),
            display_name: "Test User".to_string(),
            device_name: "TestDevice".to_string(),
            expires_at: now - 60, // Expired 1 minute ago
        };

        let remaining = session.time_remaining();
        assert_eq!(
            remaining.as_secs(),
            0,
            "Expired session should have 0 time_remaining"
        );
        assert!(
            session.expires_soon(),
            "Expired session should return expires_soon() = true"
        );
    }

    /// Test AuthController session_expires_at returns None when not authenticated.
    #[test]
    fn test_session_expires_at_none_when_logged_out() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            // Create services in async context, but call blocking methods outside block_on
            let auth = rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                services.auth()
            });

            // Call synchronous method outside of async context (blocking_read works here)
            let expires_at = auth.session_expires_at();
            assert!(
                expires_at.is_none(),
                "session_expires_at should be None when not authenticated"
            );
        });
    }

    /// Test AuthController session_expires_soon returns false when not authenticated.
    #[test]
    fn test_session_expires_soon_false_when_logged_out() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            // Create services in async context, but call blocking methods outside block_on
            let auth = rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                services.auth()
            });

            // Call synchronous method outside of async context (blocking_read works here)
            let expires_soon = auth.session_expires_soon();
            assert!(
                !expires_soon,
                "session_expires_soon should be false when not authenticated"
            );
        });
    }

    /// Test refresh_session fails when not authenticated.
    #[test]
    fn test_refresh_session_fails_when_logged_out() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                let result = auth.refresh_session().await;
                assert!(
                    result.is_err(),
                    "refresh_session should fail when not authenticated"
                );
            });
        });
    }
}

// =============================================================================
// Multi-Identity Quick Switch Tests
// =============================================================================

mod multi_identity {
    use super::*;

    /// Test list_recent_identities requires active session.
    #[test]
    fn test_list_recent_identities_requires_session() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                let result = auth.list_recent_identities().await;
                assert!(
                    result.is_err(),
                    "list_recent_identities should fail without session"
                );
            });
        });
    }

    /// Test switch_identity validates input before checking session.
    #[test]
    fn test_switch_identity_validates_input() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                // Empty four_words should fail with InvalidInput
                let result = auth.switch_identity("").await;
                assert!(result.is_err());
                let err_msg = format!("{:?}", result.unwrap_err());
                assert!(
                    err_msg.contains("InvalidInput"),
                    "Empty four_words should return InvalidInput error"
                );

                // Whitespace-only should also fail
                let result = auth.switch_identity("   ").await;
                assert!(result.is_err());
            });
        });
    }

    /// Test switch_identity requires session for valid input.
    #[test]
    fn test_switch_identity_requires_session() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                let result = auth.switch_identity("alpha-beta-gamma-delta").await;
                assert!(result.is_err());
                let err_msg = format!("{:?}", result.unwrap_err());
                assert!(
                    err_msg.contains("State"),
                    "Should fail with State error when not authenticated"
                );
            });
        });
    }

    /// Test RecentIdentity struct conversion.
    #[test]
    fn test_recent_identity_conversion() {
        use communitas_core::ui_core::UiRecentIdentity;

        let ui_recent = UiRecentIdentity {
            four_words: "alpha-beta-gamma-delta".to_string(),
            display_name: "Alice".to_string(),
            last_used: 1700000000,
            has_passkey: true,
        };

        let recent = RecentIdentity::from(ui_recent);
        assert_eq!(recent.four_words, "alpha-beta-gamma-delta");
        assert_eq!(recent.display_name, "Alice");
        assert_eq!(recent.last_used, 1700000000);
        assert!(recent.has_passkey);
    }

    /// Test try_auto_login returns None when no identity available.
    #[test]
    fn test_try_auto_login_returns_none_when_no_identity() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                // On fresh install, auto-login should return None
                let result = auth.try_auto_login().await;
                // This may succeed with None or fail due to no vault
                // Both are acceptable for this test
                match result {
                    Ok(None) => {} // Expected
                    Ok(Some(_)) => panic!("Should not have identity on fresh install"),
                    Err(_) => {} // Also acceptable - no vault exists
                }
            });
        });
    }

    /// Test remove_recent_identity validates input.
    #[test]
    fn test_remove_recent_identity_validates_input() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                // Empty four_words should fail with InvalidInput
                let result = auth.remove_recent_identity("").await;
                assert!(result.is_err());
                let err_msg = format!("{:?}", result.unwrap_err());
                assert!(
                    err_msg.contains("InvalidInput"),
                    "Empty four_words should return InvalidInput error"
                );
            });
        });
    }

    /// Test passkey operations validate input.
    #[test]
    fn test_passkey_operations_validate_input() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                // has_passkey with empty input
                let result = auth.has_passkey("").await;
                assert!(result.is_err());

                // delete_passkey with empty input
                let result = auth.delete_passkey("").await;
                assert!(result.is_err());

                // register_passkey requires session
                let result = auth.register_passkey().await;
                assert!(result.is_err());
            });
        });
    }
}

// =============================================================================
// Audit Service Tests
// =============================================================================

mod audit_service {
    use super::*;

    /// Test AuditService lazy initialization.
    #[tokio::test]
    async fn test_audit_service_lazy_init() {
        let temp = TempDir::new().unwrap();
        let audit = AuditService::new(temp.path().join("audit_logs"));

        // First read should trigger initialization
        let events = audit.read_recent(10, None).await.unwrap();
        assert!(events.is_empty(), "Fresh audit log should be empty");

        // Second read should use cached instance
        let events = audit.read_recent(10, None).await.unwrap();
        assert!(events.is_empty());
    }

    /// Test parse_event_types for valid event types.
    #[test]
    fn test_parse_event_types_valid() {
        let types = vec![
            "login".to_string(),
            "logout".to_string(),
            "failed_login".to_string(),
            "identity_switch".to_string(),
            "device_change".to_string(),
            "recovery".to_string(),
            "passkey_register".to_string(),
            "passkey_auth".to_string(),
            "session_refresh".to_string(),
            "session_expired".to_string(),
        ];

        let parsed = parse_event_types(&types).unwrap();
        assert_eq!(parsed.len(), 10);
        assert_eq!(parsed[0], AuditEventType::Login);
        assert_eq!(parsed[1], AuditEventType::Logout);
        assert_eq!(parsed[2], AuditEventType::FailedLogin);
        assert_eq!(parsed[3], AuditEventType::IdentitySwitch);
        assert_eq!(parsed[4], AuditEventType::DeviceChange);
        assert_eq!(parsed[5], AuditEventType::Recovery);
        assert_eq!(parsed[6], AuditEventType::PasskeyRegister);
        assert_eq!(parsed[7], AuditEventType::PasskeyAuth);
        assert_eq!(parsed[8], AuditEventType::SessionRefresh);
        assert_eq!(parsed[9], AuditEventType::SessionExpired);
    }

    /// Test parse_event_types is case-insensitive.
    #[test]
    fn test_parse_event_types_case_insensitive() {
        let types = vec![
            "LOGIN".to_string(),
            "LogOut".to_string(),
            "FAILED_LOGIN".to_string(),
        ];

        let parsed = parse_event_types(&types).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], AuditEventType::Login);
        assert_eq!(parsed[1], AuditEventType::Logout);
        assert_eq!(parsed[2], AuditEventType::FailedLogin);
    }

    /// Test parse_event_types rejects unknown types.
    #[test]
    fn test_parse_event_types_invalid() {
        let types = vec!["invalid_type".to_string()];
        let result = parse_event_types(&types);
        assert!(result.is_err(), "Unknown event type should fail");
    }

    /// Test export_range with invalid date formats.
    #[tokio::test]
    async fn test_audit_export_range_invalid_dates() {
        let temp = TempDir::new().unwrap();
        let audit = AuditService::new(temp.path().join("audit_logs"));

        // Invalid start date
        let result = audit
            .export_range("not-a-date", "2024-01-02T00:00:00Z", None)
            .await;
        assert!(result.is_err(), "Invalid start date should fail");

        // Invalid end date
        let result = audit
            .export_range("2024-01-01T00:00:00Z", "not-a-date", None)
            .await;
        assert!(result.is_err(), "Invalid end date should fail");
    }

    /// Test export_range with valid date range.
    #[tokio::test]
    async fn test_audit_export_range_valid_dates() {
        let temp = TempDir::new().unwrap();
        let audit = AuditService::new(temp.path().join("audit_logs"));

        // Valid date range (should return empty for fresh log)
        let result = audit
            .export_range("2024-01-01T00:00:00Z", "2024-12-31T23:59:59Z", None)
            .await;
        assert!(result.is_ok(), "Valid date range should succeed");
        assert!(
            result.unwrap().is_empty(),
            "Fresh log should have no events"
        );
    }

    /// Test cleanup_old_logs on fresh audit service.
    #[tokio::test]
    async fn test_audit_cleanup_old_logs() {
        let temp = TempDir::new().unwrap();
        let audit = AuditService::new(temp.path().join("audit_logs"));

        // Cleanup should work on fresh log
        let result = audit.cleanup_old_logs().await;
        assert!(result.is_ok(), "Cleanup should succeed on fresh log");
        assert_eq!(result.unwrap(), 0, "No old logs to clean up");
    }
}

// =============================================================================
// Auth State Machine Tests
// =============================================================================

mod auth_state_machine {
    use super::*;

    /// Test AuthStateSnapshot variants.
    #[test]
    fn test_auth_state_snapshot_variants() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                // Initial state should be LoggedOut
                let rx = auth.subscribe();
                match &*rx.borrow() {
                    AuthStateSnapshot::LoggedOut => {}
                    other => panic!("Expected LoggedOut, got {:?}", other),
                }
            });
        });
    }

    /// Test state broadcast to subscribers.
    #[test]
    fn test_auth_state_broadcast() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                let auth = services.auth();

                let rx = auth.subscribe();

                // Verify initial state
                assert!(matches!(&*rx.borrow(), AuthStateSnapshot::LoggedOut));
            });
        });
    }

    /// Test current_session returns None when logged out.
    #[test]
    fn test_current_session_none_when_logged_out() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            // Create services in async context, but call blocking methods outside block_on
            let auth = rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;
                services.auth()
            });

            // Call synchronous method outside of async context (blocking_read works here)
            let session = auth.current_session();
            assert!(
                session.is_none(),
                "current_session should be None when logged out"
            );
        });
    }
}

// =============================================================================
// UiServices Integration Tests
// =============================================================================

mod ui_services_integration {
    use super::*;

    /// Test UiServices.audit() returns working audit service.
    #[test]
    fn test_ui_services_audit_accessor() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_services(&temp).await;

                let audit = services.audit();
                // Should be able to read events (lazy init)
                let events = audit.read_recent(10, None).await.unwrap();
                assert!(events.is_empty());
            });
        });
    }

    /// Test audit service is shared across clones.
    #[test]
    fn test_audit_service_shared_across_clones() {
        run_with_large_stack(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services1 = make_services(&temp).await;
                let services2 = services1.clone();

                // Both should point to same Arc
                assert!(
                    Arc::ptr_eq(&services1.audit(), &services2.audit()),
                    "Audit service should be shared"
                );
            });
        });
    }
}
