//! Integration tests for passkey authentication in the UI service layer.
//!
//! These tests verify the passkey registration and authentication flows
//! through the AuthService trait implementation.
//!
//! Note: Tests requiring platform keyring are marked as `#[ignore]` for CI compatibility.

use communitas_ui_service::auth::{AuthController, AuthError, AuthService, RecentIdentity};
use communitas_ui_service::storage::UiStorage;
use tempfile::TempDir;

/// Create a test auth controller with temporary storage.
fn make_controller(temp: &TempDir) -> AuthController {
    let storage = UiStorage::from_path(temp.path()).unwrap();
    AuthController::new(storage).unwrap()
}

/// Test that has_passkey returns error when not logged in.
#[tokio::test]
async fn has_passkey_requires_session() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    let result = controller.has_passkey("alpha-beta-gamma-delta").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthError::State(_)));
}

/// Test that has_passkey validates input.
#[tokio::test]
async fn has_passkey_validates_input() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    // Empty four_words should fail with InvalidInput
    let result = controller.has_passkey("").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthError::InvalidInput(_)));
}

/// Test that register_passkey requires an active session.
#[tokio::test]
async fn register_passkey_requires_session() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    let result = controller.register_passkey().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthError::State(_)));
}

/// Test that delete_passkey requires an active session.
#[tokio::test]
async fn delete_passkey_requires_session() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    let result = controller.delete_passkey("alpha-beta-gamma-delta").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthError::State(_)));
}

/// Test that delete_passkey validates input.
#[tokio::test]
async fn delete_passkey_validates_input() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    // Empty four_words should fail with InvalidInput
    let result = controller.delete_passkey("").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthError::InvalidInput(_)));
}

/// Test that list_recent_identities returns empty when no identities.
#[tokio::test]
async fn list_recent_identities_empty() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    // Should return empty list or error depending on implementation
    let result = controller.list_recent_identities().await;
    // Either empty list or State error is acceptable for no-session case
    match result {
        Ok(list) => assert!(list.is_empty()),
        Err(e) => assert!(matches!(e, AuthError::State(_))),
    }
}

/// Test that try_auto_login returns None when no passkey available.
#[tokio::test]
async fn try_auto_login_no_passkey() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    let result = controller.try_auto_login().await;
    // Should return None (no auto-login available) or an error
    // Error is acceptable for no passkeys case
    if let Ok(session) = result {
        assert!(session.is_none(), "Should not auto-login without passkey");
    }
}

/// Test that remove_recent_identity validates input.
#[tokio::test]
async fn remove_recent_identity_validates_input() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    // Empty four_words should fail
    let result = controller.remove_recent_identity("").await;
    assert!(result.is_err());
}

/// Test RecentIdentity struct.
#[test]
fn recent_identity_struct() {
    let identity = RecentIdentity {
        four_words: "ocean-forest-moon-star".to_string(),
        display_name: "Test User".to_string(),
        last_used: 1234567890,
        has_passkey: true,
    };

    assert_eq!(identity.four_words, "ocean-forest-moon-star");
    assert_eq!(identity.display_name, "Test User");
    assert_eq!(identity.last_used, 1234567890);
    assert!(identity.has_passkey);
}

/// Test that switch_identity requires a session.
#[tokio::test]
async fn switch_identity_requires_session() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    let result = controller.switch_identity("alpha-beta-gamma-delta").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthError::State(_)));
}

/// Test that switch_identity validates input.
#[tokio::test]
async fn switch_identity_validates_input() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    // Empty four_words should fail with InvalidInput before checking session
    let result = controller.switch_identity("").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AuthError::InvalidInput(_)));
}

/// Test auth state subscription.
#[test]
fn auth_state_subscription() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    let receiver = controller.subscribe();
    // Should start in LoggedOut state
    let snapshot = receiver.borrow();
    assert!(
        matches!(
            *snapshot,
            communitas_ui_service::auth::AuthStateSnapshot::LoggedOut
        ),
        "Initial state should be LoggedOut"
    );
}

/// Full passkey flow test (requires platform keyring and valid identity).
#[tokio::test]
#[ignore = "Requires platform keyring and valid identity - run manually with --ignored"]
async fn full_passkey_flow() {
    let temp = TempDir::new().unwrap();
    let controller = make_controller(&temp);

    // This would require:
    // 1. Creating an identity
    // 2. Logging in with password
    // 3. Registering a passkey
    // 4. Logging out
    // 5. Auto-login with passkey
    // 6. Verifying session was restored

    // For now, just verify controller creation works
    let receiver = controller.subscribe();
    let snapshot = receiver.borrow();
    assert!(matches!(
        *snapshot,
        communitas_ui_service::auth::AuthStateSnapshot::LoggedOut
    ));
}
